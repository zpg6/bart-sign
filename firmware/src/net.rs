//! HTTPS fetch of the BART real-time departures (ETD) feed.
//!
//! TLS is MbedTLS (via `mbedtls-rs`) with full certificate verification:
//! chain, signature and hostname. Expiry dates are *not* checked because the
//! firmware has no wall clock (see `mbedtls-rs` feature `hook-wall-clock`).

use core::ffi::CStr;

use embassy_net::Stack;
use embassy_net::dns::DnsQueryType;
use embassy_net::tcp::TcpSocket;
use embassy_time::{Duration, with_timeout};
use mbedtls_rs::io::Write as _;
use mbedtls_rs::{Certificate, ClientSessionConfig, Session, SessionConfig, SessionError, TlsReference, X509};

pub const HOST: &str = "api.bart.gov";
const HOST_C: &CStr = c"api.bart.gov";
const PORT: u16 = 443;
/// The ETD path is `PATH_START` + station abbreviation + `PATH_END`.
const PATH_START: &str = "/api/etd.aspx?cmd=etd&orig=";
/// BART's public demo key, documented at api.bart.gov.
const PATH_END: &str = "&key=MW9S-E7SL-26DU-VV8V&json=y";
pub const USER_AGENT: &str = "bart-sign/0.1 (ESP32 hobby display)";

/// Trust anchors, PEM, NUL-terminated for MbedTLS.
///
/// api.bart.gov sits behind Cloudflare. Today its leaf is issued by Google
/// Trust Services WE1, which chains to GTS Root R4. Cloudflare Universal SSL
/// may rotate the issuing CA at renewal (GTS or Let's Encrypt), so the GTS and
/// ISRG roots are all trusted. Each parsed root costs about 1-2 KB of heap.
const CA_BUNDLE: &CStr = match CStr::from_bytes_with_nul(
    concat!(
        include_str!("../certs/gts-root-r4.pem"),
        include_str!("../certs/gts-root-r1.pem"),
        include_str!("../certs/isrg-root-x1.pem"),
        include_str!("../certs/isrg-root-x2.pem"),
        "\0"
    )
    .as_bytes(),
) {
    Ok(bundle) => bundle,
    Err(_) => panic!("CA bundle must be NUL-terminated text"),
};

/// Parse the trust anchors into heap memory. Do this once and reuse the
/// result for every request (cloning a `Certificate` is a refcount bump).
pub fn load_ca_chain() -> Result<Certificate<'static>, FetchError> {
    Certificate::new(X509::PEM(CA_BUNDLE)).map_err(|_| FetchError::CaParse)
}

/// Socket and response buffers. Keep this in a `static` (e.g. a
/// `StaticCell`), not on the stack.
///
/// The TLS record buffers (16 KB in + 16 KB out) live on the heap inside
/// MbedTLS, so the TCP buffers here only need to hold in-flight segments.
pub struct NetBuffers {
    tcp_rx: [u8; 4096],
    tcp_tx: [u8; 1024],
    /// Raw HTTP response (headers + body). The ETD JSON for one station is
    /// about 2 KB; 8 KB leaves room for busy hours and long headers.
    response: [u8; 8192],
}

impl NetBuffers {
    pub const fn new() -> Self {
        Self {
            tcp_rx: [0; 4096],
            tcp_tx: [0; 1024],
            response: [0; 8192],
        }
    }
}

impl Default for NetBuffers {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum FetchError {
    CaParse,
    /// The station abbreviation is not plain letters and digits.
    BadStation,
    Dns(embassy_net::dns::Error),
    NoAddress,
    Connect(embassy_net::tcp::ConnectError),
    Tls(SessionError),
    Timeout,
    /// The response did not fit in `NetBuffers::response`.
    ResponseTooLarge,
    /// Connection closed before the headers or the full body arrived.
    Truncated,
    BadResponse,
    HttpStatus(u16),
}

impl From<SessionError> for FetchError {
    fn from(e: SessionError) -> Self {
        Self::Tls(e)
    }
}

/// Fetch the ETD JSON for `station` (a BART abbreviation such as `"LAFY"`)
/// over HTTPS and return the response body.
///
/// The returned slice borrows from `bufs.response`.
pub async fn fetch_etd<'b>(
    stack: Stack<'_>,
    tls: TlsReference<'_>,
    ca_chain: &Certificate<'static>,
    station: &str,
    bufs: &'b mut NetBuffers,
) -> Result<&'b [u8], FetchError> {
    // The abbreviation goes into the request line: never let anything but
    // letters and digits through.
    if station.is_empty() || !station.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(FetchError::BadStation);
    }
    match with_timeout(Duration::from_secs(45), fetch_inner(stack, tls, ca_chain, station, bufs)).await {
        Ok(result) => result,
        Err(_) => Err(FetchError::Timeout),
    }
}

async fn fetch_inner<'b>(
    stack: Stack<'_>,
    tls: TlsReference<'_>,
    ca_chain: &Certificate<'static>,
    station: &str,
    bufs: &'b mut NetBuffers,
) -> Result<&'b [u8], FetchError> {
    let NetBuffers {
        tcp_rx,
        tcp_tx,
        response,
    } = bufs;

    let ip = *stack
        .dns_query(HOST, DnsQueryType::A)
        .await
        .map_err(FetchError::Dns)?
        .first()
        .ok_or(FetchError::NoAddress)?;
    log::info!("{HOST} -> {ip}");

    let mut socket = TcpSocket::new(stack, tcp_rx, tcp_tx);
    socket.set_timeout(Some(Duration::from_secs(20)));
    socket.connect((ip, PORT)).await.map_err(FetchError::Connect)?;

    let config = SessionConfig::Client(ClientSessionConfig {
        ca_chain: Some(ca_chain.clone()),
        server_name: Some(HOST_C),
        ..ClientSessionConfig::new()
    });
    let mut session = Session::new(tls, &mut socket, &config)?;
    session.connect().await?;
    log::info!("TLS up: {:?}", session.tls_version());

    // HTTP/1.1 is required: Cloudflare answers HTTP/1.0 requests with 403.
    for part in [
        "GET ",
        PATH_START,
        station,
        PATH_END,
        " HTTP/1.1\r\nHost: ",
        HOST,
        "\r\nUser-Agent: ",
        USER_AGENT,
        "\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
    ] {
        session.write_all(part.as_bytes()).await?;
    }
    session.flush().await?;

    let mut len = 0;
    let head = loop {
        if len == response.len() {
            return Err(FetchError::ResponseTooLarge);
        }
        let n = match session.read(&mut response[len..]).await {
            Ok(n) => n,
            // Some servers drop TCP without close_notify. If the body is
            // already complete that is fine.
            Err(e) => match parse_head(&response[..len]) {
                Some(h) if h.complete(len) => break h,
                _ => return Err(e.into()),
            },
        };
        len += n;
        let parsed = parse_head(&response[..len]);
        if n == 0 {
            break parsed.ok_or(FetchError::Truncated)?;
        }
        if let Some(h) = parsed
            && h.complete(len)
        {
            break h;
        }
    };
    // Best effort: the server closes the connection anyway.
    let _ = session.close().await;
    drop(session);
    socket.abort();

    if head.status != 200 {
        return Err(FetchError::HttpStatus(head.status));
    }

    let body_end = match head.content_length {
        Some(cl) => head.body_start + cl,
        None => len,
    };
    if body_end > len {
        return Err(FetchError::Truncated);
    }
    let body = &mut response[head.body_start..body_end];
    if head.chunked {
        let n = dechunk(body).ok_or(FetchError::BadResponse)?;
        Ok(&body[..n])
    } else {
        Ok(body)
    }
}

#[derive(Clone, Copy)]
struct Head {
    status: u16,
    body_start: usize,
    content_length: Option<usize>,
    chunked: bool,
}

impl Head {
    fn complete(&self, received: usize) -> bool {
        match self.content_length {
            Some(cl) => received >= self.body_start + cl,
            // Chunked or close-delimited: wait for EOF.
            None => false,
        }
    }
}

/// Parse the status line and the two headers we care about. Returns `None`
/// until the full header block (`\r\n\r\n`) has arrived.
fn parse_head(buf: &[u8]) -> Option<Head> {
    let end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
    let text = core::str::from_utf8(&buf[..end]).ok()?;
    let mut lines = text.split("\r\n");

    // "HTTP/1.1 200 OK"
    let status = lines.next()?.split(' ').nth(1)?.parse().ok()?;

    let mut content_length = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value.parse().ok();
        } else if name.eq_ignore_ascii_case("transfer-encoding") && value.eq_ignore_ascii_case("chunked") {
            chunked = true;
        }
    }
    if chunked {
        content_length = None;
    }

    Some(Head {
        status,
        body_start: end + 4,
        content_length,
        chunked,
    })
}

/// Decode a chunked body in place. Returns the decoded length.
fn dechunk(buf: &mut [u8]) -> Option<usize> {
    let mut read = 0;
    let mut write = 0;
    loop {
        let line_end = read + buf[read..].windows(2).position(|w| w == b"\r\n")?;
        let size_str = core::str::from_utf8(&buf[read..line_end]).ok()?;
        let size_str = size_str.split(';').next()?.trim();
        let size = usize::from_str_radix(size_str, 16).ok()?;
        read = line_end + 2;
        if size == 0 {
            return Some(write);
        }
        if read + size > buf.len() {
            return None;
        }
        buf.copy_within(read..read + size, write);
        write += size;
        read += size + 2; // skip trailing CRLF
        if read > buf.len() {
            return None;
        }
    }
}
