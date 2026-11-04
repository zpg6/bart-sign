//! Boot screen: the sign tests its lamps, puts up the dotted marquee border
//! BART uses to frame service messages, and types "BAY AREA" over
//! "RAPID TRANSIT" in the sign's big font while the bottom sign types the
//! station name.
//!
//! Timeline (ms from power-on):
//!
//! | ms          | what                                                      |
//! |-------------|-----------------------------------------------------------|
//! | 0–400       | lamp test: every LED on                                   |
//! | 400–800     | rows go dark top to bottom                                |
//! | 900–1450    | border blocks light one by one, clockwise from top left   |
//! | 1500–2940   | "BAY AREA" / "RAPID TRANSIT" types in, a letter every     |
//! |             | 80 ms; the station name below types on the same beat and  |
//! |             | lands its last letter on the same tick                    |
//! | 3200        | the station's caption ("YELLOW LINE") appears             |
//! | 3600 on     | hold: final frame, top and bottom border runs chase       |
//!
//! The border is the one on the 2011 "LOOK FOR DESTINATION DISPLAY…" message
//! (see notes/font-measurements-b.md): 2 × 2
//! blocks every 8 LEDs on rows 0–1 and 46–47 and down both sides. In that
//! photo the top and bottom blocks leave a dim copy 2 LEDs to one side
//! (they were moving during the exposure) and the side blocks don't, so the
//! hold moves the top run right and the bottom run left 2 LEDs per step and
//! keeps the sides still. Each step redraws four LED rows.
//!
//! Personal builds can swap the typed title for a private mark with the
//! `ba-mark` feature, which reads `private/ba_mark.rs` (not in the
//! repository). The lamp test, border and API are the same either way.
//!
//! Frames are pure functions of the time, cost a few thousand `set` calls,
//! and need no allocation. Apart from the lamp test, frames change only a
//! few LED rows.

use crate::font::Font;
use crate::layout;
use crate::{SignMatrix, SIGN_COLS, SIGN_ROWS};

/// What the boot screen is doing at a given time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootPhase {
    /// The intro is still moving. Keep drawing frames; don't cut away yet.
    Intro,
    /// The final composition is up and only the border is chasing. Safe to
    /// cut to departures whenever they're ready.
    Hold,
}

// Timeline, in ms from power-on (see the table above).
/// Every LED on.
const LAMP_END: u32 = 400;
/// Rows go dark top to bottom.
const SCAN_END: u32 = 800;
/// The border's blocks light one at a time, clockwise from top left.
const BORDER_START: u32 = 900;
const BORDER_END: u32 = 1450;

/// The border chase moves 2 LEDs this often during the hold. Redraw at least
/// this often to show every step; the pattern repeats every 4 steps.
pub const CHASE_STEP_MS: u32 = 200;
/// Former name of [`INTRO_MS`], kept for older callers.
pub const WIPE_MS: u32 = INTRO_MS;

// ---- the border ----------------------------------------------------------

/// The field stays inside these columns, clear of the border.
const FIELD_LEFT: i32 = 3;
const FIELD_RIGHT: i32 = SIGN_COLS as i32 - 4;
/// Border blocks are 2 × 2 LEDs, one every 8 LEDs.
const PITCH: i32 = 8;
/// Top and bottom rows: 22 blocks at columns 3, 11, … 171.
const RUN_X0: i32 = FIELD_LEFT;
const RUN_BLOCKS: i32 = 22;
/// The sides hold still, five blocks each, spaced like the runs.
const SIDE_ROWS: [i32; 5] = [7, 15, 23, 31, 39];
const RING_BLOCKS: u32 = 2 * RUN_BLOCKS as u32 + 2 * SIDE_ROWS.len() as u32;
const RIGHT_COL: i32 = SIGN_COLS as i32 - 2;
const BOTTOM_ROW: i32 = SIGN_ROWS as i32 - 2;

fn block(m: &mut SignMatrix, x: i32, y: i32) {
    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        m.set(x + dx, y + dy);
    }
}

/// Top-left LED of border block `i`, counting clockwise from the top left.
fn ring_block(i: u32) -> (i32, i32) {
    let i = i as i32;
    let sides = SIDE_ROWS.len() as i32;
    if i < RUN_BLOCKS {
        (RUN_X0 + PITCH * i, 0)
    } else if i < RUN_BLOCKS + sides {
        (RIGHT_COL, SIDE_ROWS[(i - RUN_BLOCKS) as usize])
    } else if i < 2 * RUN_BLOCKS + sides {
        (RUN_X0 + PITCH * (2 * RUN_BLOCKS + sides - 1 - i), BOTTOM_ROW)
    } else {
        (0, SIDE_ROWS[(2 * RUN_BLOCKS + 2 * sides - 1 - i) as usize])
    }
}

/// The first `n` border blocks, clockwise from the top left.
fn draw_border_build(m: &mut SignMatrix, n: u32) {
    for i in 0..n.min(RING_BLOCKS) {
        let (x, y) = ring_block(i);
        block(m, x, y);
    }
}

/// The whole border, chase step `step`: the top run moved `2 * step` LEDs
/// right and the bottom run as far left, so the blocks run clockwise.
/// Blocks that would cross the end of a run are left out. The sides hold
/// still.
fn draw_border(m: &mut SignMatrix, step: u32) {
    let shift = 2 * (step % (PITCH as u32 / 2)) as i32;
    for k in 0..RUN_BLOCKS {
        let top = RUN_X0 + shift + PITCH * k;
        let bottom = FIELD_RIGHT - 1 - shift - PITCH * k;
        for (x, y) in [(top, 0), (bottom, BOTTOM_ROW)] {
            if (FIELD_LEFT..FIELD_RIGHT).contains(&x) {
                block(m, x, y);
            }
        }
    }
    for &y in &SIDE_ROWS {
        block(m, 0, y);
        block(m, RIGHT_COL, y);
    }
}

/// How many steps of `step` ms have started since `start`, at most `max`.
fn steps_started(t: u32, start: u32, step: u32, max: u32) -> usize {
    if t < start {
        0
    } else {
        ((t - start) / step + 1).min(max) as usize
    }
}

// ---- frames --------------------------------------------------------------

/// Draw boot frame `t_ms` after power-on. Lights LEDs only; clear the matrix
/// first. Frames depend only on `t_ms`, so any frame rate gives the same
/// timing.
///
/// Returns [`BootPhase::Intro`] before [`INTRO_MS`] and [`BootPhase::Hold`]
/// from then on. Keep the boot screen up for the whole intro; once it holds,
/// cut to departures as soon as they arrive, or keep redrawing (at least
/// every [`CHASE_STEP_MS`]) for as long as it takes.
pub fn draw_boot_frame(m: &mut SignMatrix, t_ms: u32) -> BootPhase {
    let t = t_ms;
    if t >= INTRO_MS {
        draw_final(m, (t - INTRO_MS) / CHASE_STEP_MS);
        return BootPhase::Hold;
    }
    if t < LAMP_END {
        for y in 0..SIGN_ROWS as i32 {
            for x in 0..SIGN_COLS as i32 {
                m.set(x, y);
            }
        }
        return BootPhase::Intro;
    }
    if t < SCAN_END {
        let dark = ((t - LAMP_END) * SIGN_ROWS as u32 / (SCAN_END - LAMP_END)) as i32;
        for y in dark + 1..SIGN_ROWS as i32 {
            for x in 0..SIGN_COLS as i32 {
                m.set(x, y);
            }
        }
        return BootPhase::Intro;
    }
    if t >= BORDER_START {
        let lit = (t - BORDER_START) * RING_BLOCKS / (BORDER_END - BORDER_START) + 1;
        draw_border_build(m, lit);
    }
    draw_top_intro(m, t);
    BootPhase::Intro
}

fn draw_final(m: &mut SignMatrix, chase_step: u32) {
    draw_border(m, chase_step);
    draw_top_final(m);
}

/// The bottom sign's half of the boot sequence, in step with
/// [`draw_boot_frame`] on the top sign: the same lamp test and scan, the
/// same border building and chasing, then the station `name` typing in
/// alongside the top sign and the `caption` (e.g. "YELLOW LINE") appearing
/// once the typing is done.
pub fn draw_station_frame(m: &mut SignMatrix, t_ms: u32, name: &str, caption: &str) -> BootPhase {
    let t = t_ms;
    if t < SCAN_END {
        return draw_boot_frame(m, t);
    }
    if t >= INTRO_MS {
        draw_border(m, (t - INTRO_MS) / CHASE_STEP_MS);
    } else if t >= BORDER_START {
        let lit = (t - BORDER_START) * RING_BLOCKS / (BORDER_END - BORDER_START) + 1;
        draw_border_build(m, lit);
    }
    let chars = if t >= INTRO_MS { usize::MAX } else { station_chars(t, name) };
    let caption = (t >= CAPTION_MS).then_some(caption);
    layout::draw_card(m, name, chars, caption);
    if t >= INTRO_MS {
        BootPhase::Hold
    } else {
        BootPhase::Intro
    }
}

/// Draw the finished boot screen: border and title. Lights LEDs only;
/// clear the matrix first. This is the frame at [`INTRO_MS`], and every
/// fourth chase step after it.
pub fn draw_logo(m: &mut SignMatrix) {
    draw_final(m, 0);
}

/// Older entry point: draws boot frame `t_ms` and returns true once the
/// intro is over (see [`draw_boot_frame`]).
pub fn draw_logo_frame(m: &mut SignMatrix, t_ms: u32) -> bool {
    draw_boot_frame(m, t_ms) == BootPhase::Hold
}


// ---- the composition -----------------------------------------------------
//
// What fills the border comes in one of two variants with the same items:
// `INTRO_MS`, `MARK_DONE_MS`, `CAPTION_MS`, `draw_top_intro`,
// `draw_top_final` and `station_chars`. The public one is the `title`
// module. The `ba-mark` feature swaps in a private file instead (see the
// end of this file).

#[cfg(not(feature = "ba-mark"))]
pub use title::{INTRO_MS, MARK_DONE_MS};
#[cfg(not(feature = "ba-mark"))]
use title::{draw_top_final, draw_top_intro, station_chars, CAPTION_MS};

#[cfg(not(feature = "ba-mark"))]
mod title {
    use super::*;

    /// The top sign's title, typed in the big font.
    pub(super) const TITLE: [&str; 2] = ["BAY AREA", "RAPID TRANSIT"];
    /// Rows of the two lines: 14 LEDs tall with 3 dark rows between, like
    /// the two-line arriving screen, centered on the sign (rows 8–38).
    const TITLE_Y: [i32; 2] = [8, 25];
    /// Letters in [`TITLE`], spaces not counted (checked by a test).
    pub(super) const TITLE_LETTERS: u32 = 19;

    /// The first letter lights here, then one every `TYPE_STEP`. Spaces
    /// are free, on both signs.
    const TYPE_START: u32 = 1500;
    const TYPE_STEP: u32 = 80;
    /// Both signs light their last letter at this moment.
    const TYPE_END: u32 = TYPE_START + TYPE_STEP * (TITLE_LETTERS - 1);

    /// The typing is over from here on: the caption and the border chase
    /// light only a few LEDs at a time, so the firmware can start heavy work
    /// (the TLS handshake) without making the typing stutter.
    pub const MARK_DONE_MS: u32 = TYPE_END + TYPE_STEP;
    /// The bottom sign's caption appears a beat after the typing ends.
    pub(super) const CAPTION_MS: u32 = 3200;
    /// From here on the frame is final apart from the border chase, and
    /// [`draw_boot_frame`] returns [`BootPhase::Hold`].
    pub const INTRO_MS: u32 = 3600;

    /// The top sign's field during the intro, once the scan is over.
    pub(super) fn draw_top_intro(m: &mut SignMatrix, t: u32) {
        let typed = steps_started(t, TYPE_START, TYPE_STEP, TITLE_LETTERS);
        draw(m, chars_for(&TITLE, typed));
    }

    /// The top sign's finished field.
    pub(super) fn draw_top_final(m: &mut SignMatrix) {
        draw(m, usize::MAX);
    }

    /// How many characters of `name` the bottom sign shows at `t`. It types
    /// on the title's beat and lights its last letter with the title's, so
    /// a short name starts later. A name with more letters than the title
    /// starts with it and types a little faster.
    pub(super) fn station_chars(t: u32, name: &str) -> usize {
        let lines = layout::card_lines(name);
        let n = letters(&lines).max(1);
        // The letters after the first are spread evenly over `span`.
        let span = TYPE_STEP * (n.min(TITLE_LETTERS) - 1);
        let start = TYPE_END - span;
        let typed = if t < start {
            0
        } else {
            // A one-letter name (span 0) is whole at once.
            ((t - start) * (n - 1)).checked_div(span).map_or(n, |k| (k + 1).min(n))
        };
        chars_for(&lines, typed as usize)
    }

    /// Characters, spaces included, to show for the first `letters` letters
    /// of `lines` read in order. Counts across lines the way
    /// [`layout::draw_card`] does.
    pub(super) fn chars_for<S: AsRef<str>>(lines: &[S], letters: usize) -> usize {
        let mut left = letters;
        let mut chars = 0;
        for c in lines.iter().flat_map(|l| l.as_ref().chars()) {
            if left == 0 {
                break;
            }
            chars += 1;
            if !c.is_whitespace() {
                left -= 1;
            }
        }
        chars
    }

    /// Letters in `lines`, spaces not counted.
    pub(super) fn letters<S: AsRef<str>>(lines: &[S]) -> u32 {
        lines
            .iter()
            .map(|l| l.as_ref().chars().filter(|c| !c.is_whitespace()).count() as u32)
            .sum()
    }

    /// The first `chars` characters of the title, each line centered on
    /// its full width so letters don't move as they appear.
    fn draw(m: &mut SignMatrix, chars: usize) {
        let mut left = chars;
        for (line, &y) in TITLE.iter().zip(&TITLE_Y) {
            let n = line.len().min(left);
            let x = (SIGN_COLS as i32 - Font::Big.width(line)) / 2;
            Font::Big.draw(m, x, y, &line[..n]);
            left -= n;
        }
    }
}

#[cfg(all(test, not(feature = "ba-mark")))]
mod tests {
    use super::title::{chars_for, letters, TITLE, TITLE_LETTERS};
    use super::*;

    #[test]
    fn title_letters_match_the_text() {
        assert_eq!(letters(&TITLE), TITLE_LETTERS);
    }

    #[test]
    fn title_fits_inside_the_border() {
        for line in TITLE {
            let w = Font::Big.width(line);
            assert!(w <= FIELD_RIGHT - FIELD_LEFT + 1, "{line} is {w} LEDs wide");
        }
    }

    #[test]
    fn chars_for_skips_spaces() {
        assert_eq!(chars_for(&TITLE, 0), 0);
        assert_eq!(chars_for(&TITLE, 3), 3); // BAY
        assert_eq!(chars_for(&TITLE, 4), 5); // BAY A
        assert_eq!(chars_for(&TITLE, 7), 8); // BAY AREA
        assert_eq!(chars_for(&TITLE, 8), 9); // and the R
        assert_eq!(chars_for(&TITLE, 99), 21);
    }
}

// ---- the private mark ----------------------------------------------------
//
// If the build stops here with "couldn't read .../private/ba_mark.rs", the
// `ba-mark` feature is on but the private file isn't there. It is not part
// of the repository. Build without `--features ba-mark` to get the public
// intro. This stays the last item so the file's own tests come last.

#[cfg(feature = "ba-mark")]
include!(concat!(env!("CARGO_MANIFEST_DIR"), "/private/ba_mark.rs"));
