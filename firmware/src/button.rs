//! Factory reset: hold BOOT (GPIO0) for 5 seconds to forget the Wi-Fi
//! settings and restart into setup mode.

use embassy_time::{Duration, Ticker, Timer};
use esp_hal::gpio::Input;
use log::{error, info, warn};

use crate::{status, store};

const POLL: Duration = Duration::from_millis(50);
/// Show the "keep holding" hint after this long.
const HINT_MS: u64 = 1_000;
const RESET_MS: u64 = 5_000;
/// Released for this many polls in a row (100 ms) counts as let go, so a
/// bouncing contact does not cancel a hold.
const RELEASE_POLLS: u32 = 2;

#[embassy_executor::task]
pub async fn run(button: Input<'static>) {
    let mut ticker = Ticker::every(POLL);
    let mut held_ms: u64 = 0;
    let mut released_polls = 0;
    loop {
        ticker.next().await;
        if button.is_low() {
            released_polls = 0;
            held_ms += POLL.as_millis();
        } else if held_ms > 0 {
            released_polls += 1;
            if released_polls < RELEASE_POLLS {
                continue;
            }
            if held_ms >= HINT_MS {
                info!("BOOT released after {held_ms} ms, no reset");
                status::clear_override();
            }
            held_ms = 0;
            released_polls = 0;
            continue;
        } else {
            continue;
        }

        if held_ms == HINT_MS {
            info!("BOOT held, keep holding to reset Wi-Fi");
            status::set_override(&["HOLD 5 S TO", "RESET WI-FI"]);
        }
        if held_ms >= RESET_MS {
            warn!("BOOT held 5 s: erasing settings");
            status::set_override(&["WI-FI RESET"]);
            // Let the sign draw it before the erase blocks interrupts.
            Timer::after(Duration::from_millis(1_500)).await;
            if let Err(e) = store::erase() {
                error!("config erase failed: {e:?}");
            }
            esp_hal::system::software_reset();
        }
    }
}
