//! Setup mode: the open `BART-SIGN-XXXX` access point with a DHCP server, a
//! captive DNS responder and the config web page on port 80.
//!
//! All buffers here come from the heap. Setup mode has no TLS session, so
//! the heap has plenty of room, and `.bss` stays small for normal mode.

use alloc::boxed::Box;
use alloc::vec;
use core::cell::RefCell;
use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use edge_dhcp::io::DEFAULT_SERVER_PORT;
use edge_dhcp::server::{Server, ServerOptions};
use edge_nal::UdpBind;
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Ipv4Cidr, Stack, StaticConfigV4};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, with_timeout};
use embedded_io_async::Write as _;
use esp_radio::wifi::WifiController;
use esp_radio::wifi::ap::EventInfo;
use esp_radio::wifi::scan::{ScanConfig, ScanTypeConfig};
use log::{error, info, warn};

use crate::settings::Settings;
use crate::web::{self, Defaults, Form};
use crate::{dns, status, store};

pub const PORTAL_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 4, 1);
const PORTAL_IP_STR: &str = "192.168.4.1";

/// Concurrent HTTP connections. Phones open a few at once (the captive
/// check, the page, a favicon); with fewer listening sockets the extra
/// SYNs would be reset.
pub const HTTP_WORKERS: usize = 3;

pub fn ip_config() -> embassy_net::Config {
    embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(PORTAL_IP, 24),
        gateway: Some(PORTAL_IP),
        dns_servers: Default::default(),
    })
}

/// "BART-SIGN-1A2B" from the last two bytes of the access point's MAC.
pub fn ap_name(mac: [u8; 6]) -> heapless::String<16> {
    let mut s = heapless::String::new();
    let _ = core::fmt::write(&mut s, format_args!("BART-SIGN-{:02X}{:02X}", mac[4], mac[5]));
    s
}

const MAX_NETWORKS: usize = 16;
type Networks = heapless::Vec<heapless::String<32>, MAX_NETWORKS>;
static NETWORKS: Mutex<CriticalSectionRawMutex, RefCell<Networks>> = Mutex::new(RefCell::new(Networks::new()));

/// Scan for networks to offer in the page's dropdown. Needs the controller
/// in station mode (the ESP32 cannot scan in AP-only mode), so run it
/// before switching to the access point.
pub async fn scan(controller: &mut WifiController<'static>) {
    // The default dwell (10-20 ms per channel) misses networks: two scans
    // in a row saw different halves of the band. 120 ms is ESP-IDF's
    // default; 13 channels take about 2 s.
    let config = ScanConfig::default().with_max(30).with_scan_type(ScanTypeConfig::Active {
        min: esp_hal::time::Duration::from_millis(60),
        max: esp_hal::time::Duration::from_millis(120),
    });
    let mut found = match controller.scan_async(&config).await {
        Ok(found) => found,
        Err(e) => {
            warn!("Wi-Fi scan failed: {e:?}");
            return;
        }
    };
    found.sort_by_key(|ap| core::cmp::Reverse(ap.signal_strength));
    NETWORKS.lock(|n| {
        let mut n = n.borrow_mut();
        n.clear();
        for ap in &found {
            let ssid = ap.ssid.as_str();
            if ssid.is_empty() || n.iter().any(|s| s == ssid) {
                continue;
            }
            info!("  {:?} ch{} {}dBm", ssid, ap.channel, ap.signal_strength);
            if let Ok(s) = heapless::String::try_from(ssid)
                && n.push(s).is_err()
            {
                break;
            }
        }
        info!("Wi-Fi scan: {} networks, {} offered", found.len(), n.len());
    });
}

/// Keeps the Wi-Fi controller (and so the access point) alive, and logs
/// phones joining and leaving.
#[embassy_executor::task]
pub async fn ap_task(controller: WifiController<'static>) {
    loop {
        match controller.wait_for_access_point_connected_event_async().await {
            Ok(EventInfo::Connected(c)) => info!("AP: client {:02x?} joined (aid {})", c.mac, c.aid),
            Ok(EventInfo::Disconnected(d)) => info!("AP: client {:02x?} left ({:?})", d.mac, d.reason),
            Err(e) => {
                warn!("AP event error: {e:?}");
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) {
    // One socket; DHCP messages are well under 600 bytes.
    let buffers: &'static UdpBuffers<1, 1024, 1024, 4> = Box::leak(Box::new(UdpBuffers::new()));
    let udp = Udp::new(stack, buffers);
    let mut socket = match udp
        .bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DEFAULT_SERVER_PORT)))
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!("DHCP bind failed: {e:?}");
            return;
        }
    };
    let mut buf = vec![0u8; 1024];
    let mut gateway = [Ipv4Addr::UNSPECIFIED];
    let dns_servers = [PORTAL_IP];
    let mut options = ServerOptions::new(PORTAL_IP, Some(&mut gateway));
    options.dns = &dns_servers;
    options.lease_duration_secs = 3600;
    let mut server = Server::<_, 8>::new_with_et(PORTAL_IP);
    info!("DHCP server listening on :67");
    loop {
        if let Err(e) = edge_dhcp::io::server::run(&mut server, &options, &mut socket, &mut buf).await {
            warn!("DHCP server error: {e:?}");
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
pub async fn dns_task(stack: Stack<'static>) {
    let mut rx_meta = vec![PacketMetadata::EMPTY; 4];
    let mut tx_meta = vec![PacketMetadata::EMPTY; 4];
    let mut rx = vec![0u8; 1024];
    let mut tx = vec![0u8; 1024];
    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx, &mut tx_meta, &mut tx);
    if let Err(e) = socket.bind(53) {
        error!("DNS bind failed: {e:?}");
        return;
    }
    info!("captive DNS listening on :53");
    let mut query = vec![0u8; 512];
    let mut reply = vec![0u8; 512];
    loop {
        let (n, meta) = match socket.recv_from(&mut query).await {
            Ok(r) => r,
            Err(e) => {
                warn!("DNS recv: {e:?}");
                continue;
            }
        };
        if let Some(len) = dns::answer(&query[..n], PORTAL_IP.octets(), &mut reply)
            && let Err(e) = socket.send_to(&reply[..len], meta.endpoint).await
        {
            warn!("DNS send: {e:?}");
        }
    }
}

#[embassy_executor::task(pool_size = HTTP_WORKERS)]
pub async fn http_task(stack: Stack<'static>, id: usize) {
    let mut rx = vec![0u8; 1024];
    let mut tx = vec![0u8; 2048];
    // Whole request: headers plus the form body. Phones send ~500-900
    // bytes of headers; the form is ~150 bytes.
    let mut req = vec![0u8; 2048];
    if id == 0 {
        info!("HTTP server listening on :80 ({HTTP_WORKERS} sockets)");
    }
    loop {
        let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
        socket.set_timeout(Some(Duration::from_secs(10)));
        if let Err(e) = socket.accept(80).await {
            warn!("HTTP accept: {e:?}");
            continue;
        }
        let outcome = handle(&mut socket, &mut req).await;
        if let Err(e) = &outcome {
            warn!("HTTP[{id}]: {e:?}");
        }
        // Send everything, then close politely before the socket is dropped.
        let _ = with_timeout(Duration::from_secs(3), socket.flush()).await;
        socket.close();
        let mut sink = [0u8; 64];
        let _ = with_timeout(Duration::from_secs(1), async {
            while let Ok(n) = socket.read(&mut sink).await {
                if n == 0 {
                    break;
                }
            }
        })
        .await;
        socket.abort();
        drop(socket);

        if let Ok(Outcome::Saved) = outcome {
            status::set(&["SAVED", "RESTARTING"]);
            info!("settings saved, restarting in 2 s");
            Timer::after(Duration::from_secs(2)).await;
            esp_hal::system::software_reset();
        }
    }
}

#[derive(Debug)]
enum HttpError {
    Tcp(embassy_net::tcp::Error),
    Closed,
    TooLarge,
}

impl From<embassy_net::tcp::Error> for HttpError {
    fn from(e: embassy_net::tcp::Error) -> Self {
        Self::Tcp(e)
    }
}

enum Outcome {
    Served,
    Saved,
}

async fn handle(socket: &mut TcpSocket<'_>, buf: &mut [u8]) -> Result<Outcome, HttpError> {
    let mut len = 0;
    let (is_save, redirect, head_len, content_length) = loop {
        if len == buf.len() {
            send(socket, "413 Content Too Large", "text/plain", b"Request too large").await?;
            return Err(HttpError::TooLarge);
        }
        let n = socket.read(&mut buf[len..]).await?;
        if n == 0 {
            return Err(HttpError::Closed);
        }
        len += n;
        if let Some(r) = web::parse_request(&buf[..len]) {
            info!("HTTP {} {} (host {:?})", r.method, r.path, r.host);
            break (
                r.method == "POST" && r.path == "/save",
                web::should_redirect(&r, PORTAL_IP_STR),
                r.head_len,
                r.content_length,
            );
        }
    };

    if redirect {
        socket.write_all(web::REDIRECT.as_bytes()).await?;
        return Ok(Outcome::Served);
    }
    if !is_save {
        let page = render_page(&Defaults::default(), None);
        send(socket, "200 OK", "text/html; charset=utf-8", page.as_bytes()).await?;
        return Ok(Outcome::Served);
    }

    let end = head_len + content_length;
    if end > buf.len() {
        send(socket, "413 Content Too Large", "text/plain", b"Form too large").await?;
        return Err(HttpError::TooLarge);
    }
    while len < end {
        let n = socket.read(&mut buf[len..end]).await?;
        if n == 0 {
            return Err(HttpError::Closed);
        }
        len += n;
    }

    let body = core::str::from_utf8(&buf[head_len..end]).unwrap_or("");
    let form = Form::parse(body);
    let result = match &form {
        Ok(f) => Settings::new(f.network(), &f.password, &f.station, &f.line).map_err(|e| e.message()),
        Err(web::FormError::Field(field)) => Err(match *field {
            "password" => "The password could not be read. Please type it again.",
            "station" => "Pick a station from the list.",
            "line" => "Pick a line from the list.",
            _ => "The network name could not be read, or is longer than 32 bytes.",
        }),
    };
    let error = match result {
        Ok(settings) => match store::save(&settings) {
            Ok(()) => {
                info!(
                    "saved: ssid {:?}, station {}, line {:?}",
                    settings.ssid.as_str(),
                    settings.station,
                    settings.line
                );
                let page = web::saved_page(&settings.ssid);
                send(socket, "200 OK", "text/html; charset=utf-8", page.as_bytes()).await?;
                return Ok(Outcome::Saved);
            }
            Err(e) => {
                error!("config save failed: {e:?}");
                "The sign could not save the settings. Please try again."
            }
        },
        Err(msg) => msg,
    };
    warn!("rejected form: {error}");
    let defaults = match &form {
        Ok(f) => Defaults {
            ssid: f.network(),
            station: &f.station,
            line: &f.line,
        },
        Err(_) => Defaults::default(),
    };
    let page = render_page(&defaults, Some(error));
    send(socket, "200 OK", "text/html; charset=utf-8", page.as_bytes()).await?;
    Ok(Outcome::Served)
}

fn render_page(defaults: &Defaults<'_>, error: Option<&str>) -> alloc::string::String {
    NETWORKS.lock(|n| {
        let n = n.borrow();
        let names: heapless::Vec<&str, MAX_NETWORKS> = n.iter().map(|s| s.as_str()).collect();
        web::config_page(&names, defaults, error)
    })
}

async fn send(socket: &mut TcpSocket<'_>, status: &str, content_type: &str, body: &[u8]) -> Result<(), HttpError> {
    let head = web::response_head(status, content_type, body.len());
    socket.write_all(head.as_bytes()).await?;
    socket.write_all(body).await?;
    Ok(())
}
