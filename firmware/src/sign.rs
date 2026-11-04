//! The screen: two classic BART signs hanging one above the other, one per
//! platform, in Lafayette's gray housing. Each shows its direction's list,
//! or the big arriving screen when a train is at that platform.

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use sign_core::dots::Rgb;
use sign_core::board::Board;
use sign_core::etd::{Departures, Direction};
use sign_core::render::{Renderer, Style};
use sign_core::{SIGN_COLS, SIGN_ROWS, SignMatrix, logo};

use esp_hal::ledc::LowSpeed;
use esp_hal::ledc::channel::{Channel, ChannelIFace};

use crate::display::Lcd;
use crate::labels::{Label, PAIRS, PLATFORM, PLATFORM_PAIRS};

pub const SCREEN_W: usize = 480;

// One sign unit is 155 px tall; rows below are relative to its top.
const UNIT_H: usize = 155;
/// Black gap between the two units.
const GAP: usize = 10;
const UNIT_Y: [usize; 2] = [0, UNIT_H + GAP];
const HOOD_EDGE: usize = 0;
const HOOD: (usize, usize) = (1, 2);
const FRAME_TOP: usize = 4;
/// LED window: 458 × 125 px, 2.6 px per LED.
const FIELD_X: usize = 11;
pub const FIELD_W: usize = 458;
const FIELD_Y: usize = 5;
pub const FIELD_H: usize = 125;
const FRAME_BOT: usize = FIELD_Y + FIELD_H;
/// Painted label, 7 px below the window so the bottom text line breathes.
const LABEL_X: usize = 13;
const LABEL_Y: usize = FRAME_BOT + 8;
/// Light underside of the housing: the divider between the two signs.
const UNDERSIDE: (usize, usize) = (152, 154);

/// Platforms whose trains run this way hang on top (platform 2, toward SF
/// and SFO, at Lafayette); the rest go on the bottom sign.
const TOP: Direction = Direction::South;
/// Labels shown until data says otherwise (Lafayette's numbering).
const DEFAULT_PLATFORM: [u8; 2] = [2, 1];
/// Backlight duty: normal, and once service has ended for the night.
const BRIGHT: u8 = 80;
const NIGHT_GLOW: u8 = 8;
/// Departures older than this give way to a waiting message (reconnecting).
const STALE_AFTER: Duration = Duration::from_secs(180);
/// A sign with more than one screen (several platforms, more than two
/// destinations, or two trains in at once) rotates this often.
const PAGE_TIME: Duration = Duration::from_secs(6);

// Lafayette's gray aluminum housing with a black sun hood.
const HOUSING: Rgb = Rgb(92, 94, 98);
const HOOD_C: Rgb = Rgb(14, 12, 16);
const HOOD_EDGE_C: Rgb = Rgb(60, 60, 64);
const FRAME_C: Rgb = Rgb(40, 34, 36);
const UNDERSIDE_C: Rgb = Rgb(58, 60, 64);
const LABEL_C: Rgb = Rgb(240, 240, 236);

/// True while the boot intro's big motion (lamp test, sweeps, the title
/// typing in) is running. The first HTTPS fetch waits for it: the TLS
/// handshake's software crypto would otherwise stall the animation.
static INTRO_MOVING: AtomicBool = AtomicBool::new(true);

/// Wait until the boot intro's big motion is over.
pub async fn wait_intro_still() {
    while INTRO_MOVING.load(Ordering::Relaxed) {
        Timer::after(Duration::from_millis(50)).await;
    }
}

/// How long the boot screen covers for Wi-Fi and the first fetch before a waiting
/// message ("CONNECTING TO") is shown instead.
pub const WAIT_BEFORE_STATUS: Duration = Duration::from_secs(20);

fn c565(c: Rgb) -> Rgb565 {
    Rgb565::from(RawU16::new(c.to_565()))
}

/// Latest departures and when they were fetched, written by the fetch loop
/// and read by the display.
static LATEST: Mutex<CriticalSectionRawMutex, RefCell<Option<(Departures, Instant)>>> = Mutex::new(RefCell::new(None));

pub fn publish(d: Departures) {
    LATEST.lock(|l| *l.borrow_mut() = Some((d, Instant::now())));
}

/// Latest departures, counted down by the time since they were fetched.
fn current() -> Option<Departures> {
    LATEST.lock(|l| {
        l.borrow().as_ref().map(|(d, at)| d.aged((at.elapsed().as_secs() / 60) as u16))
    })
}

/// A status message (e.g. Wi-Fi setup instructions) that takes over the
/// sign, drawn in the small LED font, up to three centered lines.
pub type MessageLines = heapless::Vec<heapless::String<24>, 3>;
/// The message and whether it is urgent. Non-urgent ("waiting") messages
/// such as "CONNECTING TO" don't cut the boot screen short.
static MESSAGE: Mutex<CriticalSectionRawMutex, RefCell<Option<(MessageLines, bool)>>> = Mutex::new(RefCell::new(None));

/// Show `lines` on the sign instead of departures until [`clear_message`],
/// right away, even during the boot screen (setup instructions, errors).
/// Lines longer than the sign are clipped; keep them under ~20 characters.
pub fn show_message(lines: &[&str]) {
    set_message(lines, true);
}

/// Like [`show_message`], for progress that is usually brief
/// ("CONNECTING TO"): the boot screen stays up while it is the only news, and
/// it shows only if the wait outlasts [`WAIT_BEFORE_STATUS`].
pub fn show_waiting(lines: &[&str]) {
    set_message(lines, false);
}

fn set_message(lines: &[&str], urgent: bool) {
    let mut v = MessageLines::new();
    for l in lines.iter().take(3) {
        let mut s = heapless::String::new();
        for c in l.chars() {
            if s.push(c).is_err() {
                break;
            }
        }
        let _ = v.push(s);
    }
    MESSAGE.lock(|m| *m.borrow_mut() = Some((v, urgent)));
}

/// The station the sign is set to, shown on the bottom sign during boot.
type StationCard = (heapless::String<48>, heapless::String<24>);
static STATION: Mutex<CriticalSectionRawMutex, RefCell<Option<StationCard>>> = Mutex::new(RefCell::new(None));

/// Set the boot screen's station card: e.g. "Lafayette", "YELLOW LINE".
pub fn set_station(name: &str, caption: &str) {
    let mut n = heapless::String::new();
    let mut c = heapless::String::new();
    for ch in name.chars() {
        if n.push(ch).is_err() {
            break;
        }
    }
    for ch in caption.chars() {
        if c.push(ch).is_err() {
            break;
        }
    }
    STATION.lock(|s| *s.borrow_mut() = Some((n, c)));
}

pub fn clear_message() {
    MESSAGE.lock(|m| *m.borrow_mut() = None);
}

fn draw_message_lines(m: &mut SignMatrix, lines: &MessageLines) {
    use sign_core::font::Font;
    let rows: &[i32] = match lines.len() {
        0 | 1 => &[20],
        2 => &[14, 27],
        _ => &[3, 20, 37],
    };
    for (line, &y) in lines.iter().zip(rows) {
        let x = (SIGN_COLS as i32 - Font::Small.width(line)) / 2;
        Font::Small.draw(m, x.max(0), y, line);
    }
}

pub const STRIP_ROWS: usize = 8;

pub fn renderer() -> Renderer {
    Renderer::new(FIELD_W, FIELD_H, &Style::BART)
}

pub struct Sign {
    lcd: Lcd,
    renderer: &'static Renderer,
    /// Pixel staging for one strip; at least FIELD_W × STRIP_ROWS long.
    strip: &'static mut [u16],
    shown: [SignMatrix; 2],
    /// Platforms the label currently shows, as (a, b); b = 0 for one.
    label: [(u8, u8); 2],
    backlight: Option<Channel<'static, LowSpeed>>,
    brightness: u8,
}

impl Sign {
    pub fn new(lcd: Lcd, renderer: &'static Renderer, strip: &'static mut [u16]) -> Self {
        assert!(strip.len() >= FIELD_W * STRIP_ROWS);
        Self {
            lcd,
            renderer,
            strip,
            shown: [SignMatrix::new(), SignMatrix::new()],
            label: [(0, 0); 2],
            backlight: None,
            brightness: 0,
        }
    }

    /// Hand over the backlight PWM channel and light the screen.
    pub fn set_backlight(&mut self, ch: Channel<'static, LowSpeed>) {
        self.backlight = Some(ch);
        self.set_brightness(BRIGHT);
    }

    fn set_brightness(&mut self, pct: u8) {
        if pct == self.brightness {
            return;
        }
        if let Some(ch) = &self.backlight
            && ch.set_duty(pct).is_ok()
        {
            self.brightness = pct;
        }
    }

    fn fill(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgb) {
        let r = Rectangle::new(Point::new(x as i32, y as i32), Size::new(w as u32, h as u32));
        let _ = self.lcd.fill_solid(&r, c565(c));
    }

    /// Paint both housings and all LEDs dark. Done once at boot.
    pub fn clear(&mut self) {
        let _ = self.lcd.clear(Rgb565::BLACK);
        for unit in 0..2 {
            let y0 = UNIT_Y[unit];
            self.fill(0, y0, SCREEN_W, UNIT_H, HOUSING);
            self.fill(0, y0 + HOOD.0, SCREEN_W, HOOD.1 - HOOD.0 + 1, HOOD_C);
            self.fill(0, y0 + HOOD_EDGE, SCREEN_W, 1, HOOD_EDGE_C);
            self.fill(FIELD_X - 1, y0 + FRAME_TOP, FIELD_W + 2, FRAME_BOT - FRAME_TOP + 1, FRAME_C);
            self.fill(0, y0 + UNDERSIDE.0, SCREEN_W, UNDERSIDE.1 - UNDERSIDE.0 + 1, UNDERSIDE_C);
            self.label[unit] = (0, 0);
            self.set_label(unit, &[DEFAULT_PLATFORM[unit]]);
            self.show(unit, &SignMatrix::new(), true);
        }
    }

    /// Repaint the LED rows of `unit` that differ from what is showing.
    fn show(&mut self, unit: usize, m: &SignMatrix, force: bool) {
        let mut dirty = [force; SIGN_ROWS];
        if !force {
            for (y, d) in dirty.iter_mut().enumerate() {
                *d = m.row_differs(&self.shown[unit], y);
            }
            if !dirty.contains(&true) {
                return;
            }
        }
        let y0 = UNIT_Y[unit] + FIELD_Y;
        let mut y = 0;
        while y < FIELD_H {
            if !dirty[self.renderer.led_row(y)] {
                y += 1;
                continue;
            }
            // Extend over consecutive dirty pixel rows, up to a strip.
            let mut n = 1;
            while n < STRIP_ROWS && y + n < FIELD_H && dirty[self.renderer.led_row(y + n)] {
                n += 1;
            }
            let buf = &mut self.strip[..FIELD_W * n];
            self.renderer.render_rows(m, y, buf);
            let sy = (y0 + y) as u16;
            let _ = self.lcd.set_pixels(
                FIELD_X as u16,
                sy,
                (FIELD_X + FIELD_W - 1) as u16,
                sy + n as u16 - 1,
                buf.iter().map(|&p| Rgb565::from(RawU16::new(p))),
            );
            y += n;
        }
        self.shown[unit] = m.clone();
    }

    /// The "PLATFORM n" (or "PLATFORMS a & b") painted on the bezel under a
    /// unit's LEDs, for the platforms of the trains it is showing.
    fn set_label(&mut self, unit: usize, platforms: &[u8]) {
        let valid = |p: &u8| (1..=9).contains(p);
        let mut key = match platforms.iter().copied().filter(valid).collect::<heapless::Vec<u8, 2>>()[..] {
            [a] => (a, 0),
            [a, b] => (a.min(b), a.max(b)),
            _ => (DEFAULT_PLATFORM[unit], 0),
        };
        let pair = PAIRS.iter().position(|&p| p == key);
        if key.1 != 0 && pair.is_none() {
            key.1 = 0; // no bitmap for that pair: name the first platform
        }
        if key == self.label[unit] {
            return;
        }
        self.label[unit] = key;
        let label: &Label = match pair {
            Some(i) if key.1 != 0 => &PLATFORM_PAIRS[i],
            _ => &PLATFORM[key.0 as usize],
        };
        let y0 = UNIT_Y[unit] + LABEL_Y;
        // Clear the widest label's area, then blend the paint over the bezel.
        self.fill(LABEL_X, y0, 200, 15, HOUSING);
        let shades = [0u8, 85, 170, 255].map(|t| c565(HOUSING.mix(LABEL_C, t)));
        let pixels = (0..label.height)
            .flat_map(|y| (0..label.width).map(move |x| (x, y)))
            .map(|(x, y)| shades[label.level(x, y) as usize]);
        let _ = self.lcd.set_pixels(
            LABEL_X as u16,
            y0 as u16,
            (LABEL_X + label.width - 1) as u16,
            (y0 + label.height - 1) as u16,
            pixels,
        );
    }
}

#[embassy_executor::task]
pub async fn run(mut sign: Sign) {
    let boot = Instant::now();

    // Boot: the two signs play one sequence (see logo::draw_station_frame)
    // while Wi-Fi connects in
    // the background. Cut to trains as soon as the intro is done and they're
    // in; show an urgent message (setup, errors) once the intro is done; and
    // after WAIT_BEFORE_STATUS show what we're still waiting on.
    let mut frame = SignMatrix::new();
    loop {
        let t = boot.elapsed();
        let t_ms = t.as_millis().min(u32::MAX as u64) as u32;
        frame.clear();
        let phase = logo::draw_boot_frame(&mut frame, t_ms);
        sign.show(0, &frame, false);
        // The bottom sign runs in step: same lamp test and border, then the
        // station name types in alongside the top sign. Without a station
        // (setup mode) it mirrors the top sign.
        STATION.lock(|s| {
            if let Some((name, caption)) = s.borrow().as_ref() {
                frame.clear();
                logo::draw_station_frame(&mut frame, t_ms, name, caption);
            }
        });
        sign.show(1, &frame, false);
        let have_data = LATEST.lock(|l| l.borrow().is_some());
        let message = MESSAGE.lock(|m| m.borrow().as_ref().map(|(_, urgent)| *urgent));
        let holding = phase == logo::BootPhase::Hold;
        if t_ms >= logo::MARK_DONE_MS {
            INTRO_MOVING.store(false, Ordering::Relaxed);
        }
        let ready = match message {
            Some(true) => true,
            Some(false) => have_data || t >= WAIT_BEFORE_STATUS,
            None => have_data,
        };
        if holding && ready {
            log::info!("boot screen done after {} ms", t.as_millis());
            break;
        }
        let step = if holding { logo::CHASE_STEP_MS } else { 30 };
        Timer::after(Duration::from_millis(step as u64)).await;
    }

    loop {
        // Urgent messages always show. A waiting one ("CONNECTING TO" after a
        // Wi-Fi blip) only once the trains we have are too old to trust;
        // until then the counted-down departures stay up.
        let fresh = LATEST.lock(|l| l.borrow().as_ref().is_some_and(|(_, at)| at.elapsed() < STALE_AFTER));
        let message = MESSAGE.lock(|m| m.borrow().clone()).filter(|(_, urgent)| *urgent || !fresh);
        if let Some((lines, _)) = message {
            // Status messages take the top sign; the bottom one goes dark.
            let mut frame = SignMatrix::new();
            draw_message_lines(&mut frame, &lines);
            sign.set_brightness(BRIGHT);
            sign.show(0, &frame, false);
            sign.show(1, &SignMatrix::new(), false);
        } else {
            let deps = current();
            // Service over for the night: just a glow, like a sign in a
            // closed station, until the first trains of the morning.
            let night = deps.as_ref().is_some_and(|d| d.is_empty() && d.is_late());
            sign.set_brightness(if night { NIGHT_GLOW } else { BRIGHT });
            let board = deps.map(|d| Board::new(&d, TOP)).unwrap_or_default();
            let tick = (Instant::now().as_millis() / PAGE_TIME.as_millis()) as usize;
            for unit in 0..2 {
                let mut frame = SignMatrix::new();
                let info = board.draw(&mut frame, unit, tick);
                if !info.platforms.is_empty() {
                    sign.set_label(unit, &info.platforms);
                }
                sign.show(unit, &frame, false);
            }
        }
        Timer::after(Duration::from_millis(250)).await;
    }
}
