#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::sync::atomic::{AtomicBool, Ordering};

use bart_sign::display::{self, DisplayPins};
use bart_sign::net::{self, NetBuffers};
use bart_sign::settings::{Line, STATIONS, Settings};
use bart_sign::sign::{self, SCREEN_W, STRIP_ROWS, Sign};
use bart_sign::{button, portal, status, store};
use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{DriveMode, Input, InputConfig, Pull};
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{self, LSGlobalClkSource, Ledc, LowSpeed};
use esp_hal::time::Rate;
use esp_hal::peripherals::{AES, RSA, WIFI};
use esp_hal::ram;
use esp_hal::rng::{Trng, TrngSource};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use esp_radio::wifi::ap::AccessPointConfig;
use esp_radio::wifi::PowerSaveMode;
use esp_radio::wifi::scan::ScanConfig;
use esp_radio::wifi::sta::StationConfig;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, ControllerConfig, Interface, WifiController,
};
use log::{error, info, warn};
use mbedtls_rs::Tls;
use mbedtls_rs::sys::hook::backend::esp::EspAccel;
use sign_core::render::Renderer;
use static_cell::StaticCell;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

/// Size of the `.dram2_uninit` region on the ESP32: RAM the 2nd-stage
/// bootloader used, reclaimed as heap after boot.
const RECLAIMED_HEAP: usize = 98_768;
/// Extra heap carved out of regular DRAM (.bss). Wi-Fi needs ~60-80 KB,
/// one MbedTLS session needs ~40-50 KB (2 x 16 KB record buffers plus
/// handshake state), parsed CA roots ~6 KB.
const MAIN_HEAP: usize = 40 * 1024;

/// BART asks for no more than one request a minute per client.
const FETCH_PERIOD_SECS: u64 = 60;
/// Between the last train of the night and the first of the morning.
const NIGHT_FETCH_SECS: u64 = 300;
const FETCH_BACKOFF_MAX_SECS: u64 = 300;

/// esp-radio defaults to 5 dBm max TX power, which many APs barely hear.
/// 80 = 20 dBm (unit is 0.25 dBm), the ESP-IDF default.
const TX_POWER: i8 = 80;

/// Set once the sign has shown real departures.
static HAVE_DATA: AtomicBool = AtomicBool::new(false);

macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: StaticCell<$t> = StaticCell::new();
        STATIC_CELL.uninit().write($val)
    }};
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    println!("bart-sign: boot");

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: RECLAIMED_HEAP);
    esp_alloc::heap_allocator!(size: MAIN_HEAP);

    info!("heap ready");
    // Display first, so a wiring problem shows up before any networking.
    let pixel_buf = mk_static!([u8; 4096], [0; 4096]);
    let lcd = display::init(
        peripherals.SPI3,
        peripherals.DMA_SPI3,
        DisplayPins {
            sck: peripherals.GPIO18.into(),
            mosi: peripherals.GPIO23.into(),
            cs: peripherals.GPIO5.into(),
            dc: peripherals.GPIO27.into(),
            rst: peripherals.GPIO33.into(),
        },
        pixel_buf,
    )
    .expect("display init failed");
    info!("display init done");
    let renderer = mk_static!(Renderer, sign::renderer());
    let strip = mk_static!([u16; SCREEN_W * STRIP_ROWS], [0; SCREEN_W * STRIP_ROWS]);
    let mut sign = Sign::new(lcd, renderer, strip);
    // Clear before the backlight comes on: panel RAM is random at power-up.
    sign.clear();
    // Backlight on PWM so the sign can dim at night (GPIO32, 20 kHz).
    let ledc = mk_static!(Ledc<'static>, Ledc::new(peripherals.LEDC));
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let bl_timer = mk_static!(ledc::timer::Timer<'static, LowSpeed>, ledc.timer(ledc::timer::Number::Timer0));
    bl_timer
        .configure(ledc::timer::config::Config {
            duty: ledc::timer::config::Duty::Duty10Bit,
            clock_source: ledc::timer::LSClockSource::APBClk,
            frequency: Rate::from_khz(20),
        })
        .expect("backlight timer");
    let mut backlight = ledc.channel(ledc::channel::Number::Channel0, peripherals.GPIO32);
    backlight
        .configure(ledc::channel::config::Config {
            timer: bl_timer,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .expect("backlight channel");
    sign.set_backlight(backlight);
    info!("display up");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);
    info!("esp-rtos started");

    store::init(peripherals.FLASH);
    let settings = store::load();

    // BOOT button: active low, with an external pull-up on the board.
    let boot_button = Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up));

    // True RNG: needs the RNG and ADC1 entropy source alive for the program's
    // lifetime. Wi-Fi uses ADC2 on the ESP32, so ADC1 is free.
    let _trng_source = mk_static!(TrngSource<'static>, TrngSource::new(peripherals.RNG, peripherals.ADC1));
    let trng = mk_static!(Trng, Trng::try_new().expect("TRNG source not enabled"));
    let seed = (trng.random() as u64) << 32 | trng.random() as u64;

    // One socket pool for whichever mode runs. Setup mode uses 5 (DHCP,
    // DNS, 3 x HTTP); normal mode uses 3 (DHCP client, DNS client, HTTPS).
    let resources = mk_static!(StackResources<6>, StackResources::new());

    spawner.spawn(sign::run(sign).expect("sign task spawn"));
    spawner.spawn(button::run(boot_button).expect("button task spawn"));

    match settings {
        None => {
            info!("no saved settings: setup mode");
            setup_mode(spawner, peripherals.WIFI, resources, seed).await
        }
        Some(s) => {
            info!(
                "settings: ssid {:?}, station {}, line {:?}",
                s.ssid.as_str(),
                s.station,
                s.line
            );
            normal_mode(spawner, peripherals.WIFI, peripherals.RSA, peripherals.AES, trng, resources, seed, s).await
        }
    }
}

/// No settings yet: run the open access point and the setup page until the
/// user saves (the HTTP task then restarts the chip).
async fn setup_mode(
    spawner: Spawner,
    wifi: WIFI<'static>,
    resources: &'static mut StackResources<6>,
    seed: u64,
) -> ! {
    status::set_waiting(&["SETUP MODE", "STARTING"]);

    // Starts in station mode (the default), which can scan.
    let mut controller = WifiController::new(wifi, ControllerConfig::default()).expect("Wi-Fi init failed");
    controller.set_max_tx_power(TX_POWER).expect("set_max_tx_power failed");
    portal::scan(&mut controller).await;

    let iface = Interface::access_point();
    let name = portal::ap_name(iface.mac_address());
    controller
        .set_config(&WifiConfig::AccessPoint(
            AccessPointConfig::default()
                .with_ssid(name.as_str().try_into().expect("AP name"))
                .with_authentication(AuthenticationMethodConfig::Open),
        ))
        .expect("access point start failed");
    info!("AP {:?} up, open, portal at http://{}/", name.as_str(), portal::PORTAL_IP);

    let (stack, runner) = embassy_net::new(iface, portal::ip_config(), resources, seed);
    spawner.spawn(net_task(runner).expect("net_task spawn"));
    spawner.spawn(portal::ap_task(controller).expect("ap_task spawn"));
    spawner.spawn(portal::dhcp_task(stack).expect("dhcp_task spawn"));
    spawner.spawn(portal::dns_task(stack).expect("dns_task spawn"));
    for id in 0..portal::HTTP_WORKERS {
        spawner.spawn(portal::http_task(stack, id).expect("http_task spawn"));
    }

    status::set(&["JOIN WI-FI", &name, "OPEN 192.168.4.1"]);

    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("setup mode: heap used {} / free {}", esp_alloc::HEAP.used(), esp_alloc::HEAP.free());
        Timer::after(Duration::from_secs(55)).await;
    }
}

/// Settings present: join the saved network and show departures.
#[allow(clippy::too_many_arguments)]
async fn normal_mode(
    spawner: Spawner,
    wifi: WIFI<'static>,
    rsa: RSA<'static>,
    aes: AES<'static>,
    trng: &'static mut Trng,
    resources: &'static mut StackResources<6>,
    seed: u64,
    settings: Settings,
) -> ! {
    // The bottom sign's boot card: which station this sign is set to.
    let name = STATIONS.iter().find(|(a, _)| *a == settings.station).map_or(settings.station, |(_, n)| *n);
    let mut caption: heapless::String<24> = heapless::String::new();
    match settings.line {
        Line::All => drop(caption.push_str("ALL LINES")),
        line => drop(core::fmt::write(&mut caption, format_args!("{} LINE", line.code()))),
    }
    sign::set_station(name, &caption);

    // Hardware RSA (mod-exp) and AES for MbedTLS. The ESP32's SHA block is
    // not supported by the esp-hal work-queue backend, and it has no ECC
    // block, so SHA and P-256 run in software. Hook before any TLS use.
    let mut accel = EspAccel::new().with_rsa(rsa).with_aes(aes);
    let accel_queue = accel.start();
    // SAFETY: called before any MbedTLS state exists; `accel_queue` and the
    // guard live until this function returns, which is never.
    let _hooks = unsafe { accel_queue.hook() };

    let tls = Tls::new(trng).expect("Tls already created");
    let ca_chain = net::load_ca_chain().expect("CA bundle failed to parse");

    let auth = if settings.password.is_empty() {
        AuthenticationMethodConfig::Open
    } else {
        AuthenticationMethodConfig::Wpa2Personal(settings.password.as_str().try_into().expect("password too long"))
    };
    let station = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(settings.ssid.as_str().try_into().expect("SSID too long"))
            .with_authentication(auth),
    );
    let mut controller = WifiController::new(wifi, ControllerConfig::default().with_initial_config(station))
        .expect("Wi-Fi init failed");
    controller.set_max_tx_power(TX_POWER).expect("set_max_tx_power failed");
    // Modem sleep: the radio wakes for each DTIM beacon instead of listening
    // nonstop. About 80 mA less; we only talk to the network once a minute.
    if let Err(e) = controller.set_power_saving(PowerSaveMode::Minimum) {
        warn!("Wi-Fi power save not enabled: {e:?}");
    }

    let (stack, runner) = embassy_net::new(
        Interface::station(),
        embassy_net::Config::dhcpv4(Default::default()),
        resources,
        seed,
    );

    spawner.spawn(wifi_task(controller, settings.ssid.clone()).expect("wifi_task spawn"));
    spawner.spawn(net_task(runner).expect("net_task spawn"));

    let bufs = mk_static!(NetBuffers, NetBuffers::new());
    let filter = settings.line.filter();
    let mut rng = seed | 1;
    let mut failures: u32 = 0;
    let mut quiet = false;

    loop {
        if !stack.is_config_up() {
            info!("waiting for an IP address");
            wait_for_ip(stack).await;
        }

        // Keep the boot intro smooth: the TLS handshake is heavy software
        // crypto, so the first fetch waits for the intro's still frame.
        sign::wait_intro_still().await;
        let ok = match net::fetch_etd(stack, tls.reference(), &ca_chain, settings.station, bufs).await {
            Ok(body) => match sign_core::etd::parse(body, filter) {
                Ok(d) => {
                    info!(
                        "ETD {} {}: {} south, {} north",
                        settings.station,
                        d.time.as_str(),
                        d.south.len(),
                        d.north.len()
                    );
                    // Service over for the night: no need to ask every minute.
                    quiet = d.is_empty() && d.is_late();
                    sign::publish(d);
                    HAVE_DATA.store(true, Ordering::Relaxed);
                    status::clear();
                    true
                }
                Err(e) => {
                    error!("ETD parse failed: {e:?} ({} bytes)", body.len());
                    false
                }
            },
            Err(e) => {
                error!("fetch_etd failed: {e:?}");
                false
            }
        };

        info!("heap used {} / free {}", esp_alloc::HEAP.used(), esp_alloc::HEAP.free());

        failures = if ok { 0 } else { failures + 1 };
        let base = if ok && quiet { NIGHT_FETCH_SECS } else { retry_delay_secs(failures) };
        // A little jitter (up to 5 %) so a room full of signs drifts apart.
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let jitter_ms = rng % (base * 50 + 1);
        let delay = Duration::from_secs(base) + Duration::from_millis(jitter_ms);
        info!("next fetch in {} s", delay.as_secs());
        Timer::after(delay).await;
    }
}

/// 60 s normally. Each failed fetch in a row doubles it: 120, 240, then
/// 300 s at most.
fn retry_delay_secs(failures: u32) -> u64 {
    (FETCH_PERIOD_SECS << failures.min(8)).min(FETCH_BACKOFF_MAX_SECS)
}

async fn wait_for_ip(stack: Stack<'_>) {
    stack.wait_config_up().await;
    if let Some(cfg) = stack.config_v4() {
        info!("got IP {}", cfg.address);
    }
}

/// Log what the radio can hear, so "network not found" is diagnosable. Only
/// after a failed connect: a scan costs over a second of boot time.
async fn log_scan(controller: &mut WifiController<'static>, ssid: &str) {
    match controller.scan_async(&ScanConfig::default().with_max(20)).await {
        Ok(aps) => {
            info!("Wi-Fi scan: {} networks", aps.len());
            for ap in &aps {
                info!("  {:?} ch{} {}dBm {:?}", ap.ssid.as_str(), ap.channel, ap.signal_strength, ap.auth_method);
            }
            if !aps.iter().any(|ap| ap.ssid.as_str() == ssid) {
                warn!("{ssid:?} not in scan results");
            }
        }
        Err(e) => warn!("Wi-Fi scan failed: {e:?}"),
    }
}

#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>, ssid: heapless::String<32>) {
    let shown = status::sign_text(&ssid);
    let connecting: [&str; 2] = ["CONNECTING TO", &shown];
    status::set_waiting(&connecting);

    // The first retry is quick: right after a reset the router can still
    // hold this board's old session and refuse the first attempt.
    let mut backoff = Duration::from_millis(500);
    let mut failures = 0u32;
    loop {
        info!("connecting to {ssid:?}");
        match controller.connect_async().await {
            Ok(info) => {
                info!("Wi-Fi connected: {info:?}");
                backoff = Duration::from_millis(500);
                failures = 0;
                if HAVE_DATA.load(Ordering::Relaxed) {
                    status::clear();
                } else {
                    status::set_waiting(&["CONNECTED", "LOADING TRAINS"]);
                }
                let why = controller.wait_for_disconnect_async().await;
                warn!("Wi-Fi disconnected: {why:?}");
                status::set_waiting(&connecting);
            }
            Err(e) => {
                failures += 1;
                error!("Wi-Fi connect failed: {e:?}, retry in {}s", backoff.as_secs());
                if failures == 2 {
                    log_scan(&mut controller, &ssid).await;
                }
                // Settings are never erased here: the network may just be
                // down. Point at the BOOT button instead.
                if failures >= 3 {
                    status::set(&["CAN'T JOIN", &shown, "HOLD BOOT 5S: RESET"]);
                }
                Timer::after(backoff).await;
                backoff = (backoff * 4).min(Duration::from_secs(60));
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface>) {
    runner.run().await
}
