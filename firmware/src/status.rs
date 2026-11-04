//! Which status message the sign shows, on top of [`crate::sign`]'s single
//! message slot.
//!
//! Two layers: the *base* message belongs to the current mode ("JOIN WI-FI",
//! "CONNECTING TO", or none once trains show), and the *override* belongs to
//! the reset button. When the button is let go, the base message comes back.

use core::cell::RefCell;

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use sign_core::SIGN_COLS;
use sign_core::font::Font;

use crate::sign::{self, MessageLines};

struct State {
    /// The mode's message and whether it is urgent (see `sign::show_waiting`).
    base: Option<(MessageLines, bool)>,
    overridden: bool,
}

static STATE: Mutex<CriticalSectionRawMutex, RefCell<State>> = Mutex::new(RefCell::new(State {
    base: None,
    overridden: false,
}));

fn to_lines(lines: &[&str]) -> MessageLines {
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
    v
}

fn show(lines: &MessageLines, urgent: bool) {
    let mut refs: heapless::Vec<&str, 3> = heapless::Vec::new();
    for l in lines {
        let _ = refs.push(l.as_str());
    }
    log::info!("sign message: {refs:?}");
    if urgent {
        sign::show_message(&refs);
    } else {
        sign::show_waiting(&refs);
    }
}

fn set_base(lines: &[&str], urgent: bool) {
    let lines = to_lines(lines);
    STATE.lock(|s| {
        let mut s = s.borrow_mut();
        if !s.overridden {
            show(&lines, urgent);
        }
        s.base = Some((lines, urgent));
    });
}

/// Set the mode's message (setup instructions, errors). Shown at once,
/// even over the boot screen, unless the button owns the sign.
pub fn set(lines: &[&str]) {
    set_base(lines, true);
}

/// Set a progress message that is usually brief ("CONNECTING TO"). The boot
/// screen keeps covering it unless the wait drags on.
pub fn set_waiting(lines: &[&str]) {
    set_base(lines, false);
}

/// Drop the mode's message, so departures show.
pub fn clear() {
    STATE.lock(|s| {
        let mut s = s.borrow_mut();
        if !s.overridden && s.base.is_some() {
            log::info!("sign message cleared");
            sign::clear_message();
        }
        s.base = None;
    });
}

/// Take over the sign (reset button feedback).
pub fn set_override(lines: &[&str]) {
    let lines = to_lines(lines);
    STATE.lock(|s| {
        s.borrow_mut().overridden = true;
        show(&lines, true);
    });
}

/// Give the sign back to the mode's message, or departures.
pub fn clear_override() {
    STATE.lock(|s| {
        let mut s = s.borrow_mut();
        s.overridden = false;
        match &s.base {
            Some((lines, urgent)) => show(lines, *urgent),
            None => sign::clear_message(),
        }
    });
}

/// Make `text` drawable in the sign's small font: upper case, characters the
/// font lacks become `-`, and the result is cut to fit the sign's width.
pub fn sign_text(text: &str) -> heapless::String<24> {
    const OK: &str = " ,-/.:'&";
    let mut out = heapless::String::<24>::new();
    for c in text.chars() {
        let c = c.to_ascii_uppercase();
        let c = if c.is_ascii_uppercase() || c.is_ascii_digit() || OK.contains(c) { c } else { '-' };
        if out.push(c).is_err() || Font::Small.width(&out) > SIGN_COLS as i32 {
            out.pop();
            break;
        }
    }
    out
}
