//! Boot screen tests. The shared ones hold for the public intro and for the
//! private `ba-mark` build; the mark's own tests run only with the feature.

use sign_core::logo::{
    draw_boot_frame, draw_logo, draw_logo_frame, draw_station_frame, BootPhase, CHASE_STEP_MS, INTRO_MS,
    MARK_DONE_MS,
};
#[cfg(feature = "ba-mark")]
use sign_core::logo::{LOGO_HEIGHT, LOGO_WIDTH, LOGO_X, LOGO_Y};
use sign_core::{SignMatrix, SIGN_COLS, SIGN_ROWS};

/// Station names for the bottom sign: one line, two lines (more letters
/// than the title), and one that needs shortening.
const STATIONS: [(&str, &str); 3] = [
    ("Lafayette", "YELLOW LINE"),
    ("Pleasant Hill/Contra Costa Centre", "YELLOW LINE"),
    ("San Francisco International Airport (SFO)", "YELLOW LINE"),
];

fn logo() -> SignMatrix {
    let mut m = SignMatrix::new();
    draw_logo(&mut m);
    m
}

fn frame(t: u32) -> (SignMatrix, BootPhase) {
    let mut m = SignMatrix::new();
    let phase = draw_boot_frame(&mut m, t);
    (m, phase)
}

fn station(t: u32, name: &str, caption: &str) -> (SignMatrix, BootPhase) {
    let mut m = SignMatrix::new();
    let phase = draw_station_frame(&mut m, t, name, caption);
    (m, phase)
}

/// Every 10 ms through the intro and two chase cycles of the hold.
fn times() -> impl Iterator<Item = u32> {
    (0..INTRO_MS + 8 * CHASE_STEP_MS).step_by(10)
}

/// (min x, min y, max x, max y) of the lit LEDs inside `cols` × `rows`.
fn bounds(
    m: &SignMatrix,
    cols: std::ops::Range<usize>,
    rows: std::ops::Range<usize>,
) -> Option<(usize, usize, usize, usize)> {
    let mut b: Option<(usize, usize, usize, usize)> = None;
    for y in rows {
        for x in cols.clone() {
            if m.get(x, y) {
                b = Some(match b {
                    None => (x, y, x, y),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                });
            }
        }
    }
    b
}

fn lit_count(m: &SignMatrix) -> usize {
    (0..SIGN_ROWS)
        .map(|y| (0..SIGN_COLS).filter(|&x| m.get(x, y)).count())
        .sum()
}

fn changed_rows(a: &SignMatrix, b: &SignMatrix) -> Vec<usize> {
    (0..SIGN_ROWS).filter(|&y| a.row_differs(b, y)).collect()
}

/// Mark-relative LED.
#[cfg(feature = "ba-mark")]
fn at(m: &SignMatrix, x: i32, y: i32) -> bool {
    m.get((LOGO_X + x) as usize, (LOGO_Y + y) as usize)
}

/// LEDs reachable from (`x`, `y`) in mark coordinates through LEDs whose
/// state is `lit`, 4-connected, staying inside the mark's box.
#[cfg(feature = "ba-mark")]
fn flood(m: &SignMatrix, x: i32, y: i32, lit: bool) -> Vec<Vec<bool>> {
    let mut seen = vec![vec![false; LOGO_WIDTH]; LOGO_HEIGHT];
    let mut stack = vec![(x, y)];
    while let Some((x, y)) = stack.pop() {
        if x < 0 || y < 0 || x >= LOGO_WIDTH as i32 || y >= LOGO_HEIGHT as i32 {
            continue;
        }
        if seen[y as usize][x as usize] || at(m, x, y) != lit {
            continue;
        }
        seen[y as usize][x as usize] = true;
        stack.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
    }
    seen
}

#[test]
fn final_frame_is_centered_inside_the_border() {
    let m = logo();
    // Inside the border: rows 2..46, columns 2..174.
    let (x0, y0, x1, y1) = bounds(&m, 2..SIGN_COLS - 2, 2..SIGN_ROWS - 2).expect("final frame lights something");
    let (left, right) = (x0, SIGN_COLS - 1 - x1);
    let (top, bottom) = (y0, SIGN_ROWS - 1 - y1);
    assert!(left.abs_diff(right) <= 1, "left {left} right {right}");
    assert!(top.abs_diff(bottom) <= 1, "top {top} bottom {bottom}");
    // At least 4 dark rows between the content and the border runs.
    assert!(top >= 2 + 4 && bottom >= 2 + 4, "top {top} bottom {bottom}");
}

#[cfg(feature = "ba-mark")]
#[test]
fn mark_sits_at_its_constants() {
    let m = logo();
    let (x0, y0, _, y1) = bounds(&m, 2..SIGN_COLS - 2, 2..SIGN_ROWS - 2).expect("final frame lights something");
    assert_eq!((x0 as i32, y0 as i32), (LOGO_X, LOGO_Y), "mark at its constants");
    assert_eq!(y1 - y0 + 1, LOGO_HEIGHT);
}

#[test]
fn border_blocks_are_2x2_every_8_leds() {
    let m = logo();
    for y in [0, 1, SIGN_ROWS - 2, SIGN_ROWS - 1] {
        let lit: Vec<usize> = (0..SIGN_COLS).filter(|&x| m.get(x, y)).collect();
        assert_eq!(lit.len(), 2 * 22, "row {y}");
        for pair in lit.chunks(2) {
            assert_eq!(pair[1], pair[0] + 1, "row {y}: blocks are 2 wide");
        }
        for w in lit.chunks(2).collect::<Vec<_>>().windows(2) {
            assert_eq!(w[1][0] - w[0][0], 8, "row {y}: pitch");
        }
    }
    for x in [0, 1, SIGN_COLS - 2, SIGN_COLS - 1] {
        let lit: Vec<usize> = (2..SIGN_ROWS - 2).filter(|&y| m.get(x, y)).collect();
        assert_eq!(lit.len(), 2 * 5, "column {x}");
        for w in lit.chunks(2).collect::<Vec<_>>().windows(2) {
            assert_eq!(w[1][0] - w[0][0], 8, "column {x}: pitch");
        }
    }
}

#[cfg(feature = "ba-mark")]
#[test]
fn counters_are_closed_and_letters_are_separate() {
    let m = logo();
    // Counter centers are dark, stems are lit.
    let (b_counter, a_counter) = ((18, 22), (42, 22));
    assert!(!at(&m, b_counter.0, b_counter.1));
    assert!(!at(&m, a_counter.0, a_counter.1));
    assert!(at(&m, 4, 22));
    assert!(at(&m, LOGO_WIDTH as i32 - 4, 22));

    // Each counter is enclosed: its dark region stays well inside the mark.
    for (x, y) in [b_counter, a_counter] {
        let dark = flood(&m, x, y, false);
        let n: usize = dark.iter().flatten().filter(|&&d| d).count();
        assert!(
            (140..=170).contains(&n),
            "counter at {x},{y} has {n} LEDs; it leaks or is too small"
        );
        assert!(!dark[0][x as usize] && !dark[LOGO_HEIGHT - 1][x as usize]);
    }

    // The seam keeps the b and the a apart: from the b's stem, the a's
    // stem is unreachable through lit LEDs.
    let b = flood(&m, 4, 22, true);
    assert!(!b[22][LOGO_WIDTH - 4], "b and a run together");
    assert!(b[0][0] && b[LOGO_HEIGHT - 2][0], "b stem should be one piece");
    // And the a is one piece with its stem and its sliver of wall.
    let a = flood(&m, LOGO_WIDTH as i32 - 4, 22, true);
    assert!(a[22][a_counter.0 as usize - 7], "the a's left wall is missing");
}

#[test]
fn frames_stay_on_the_sign_and_clear_of_the_border() {
    for t in times() {
        if t < 800 {
            continue; // lamp test lights everything
        }
        let (m, _) = frame(t);
        assert_clear_of_border(&m, t);
        for (name, caption) in STATIONS {
            let mut m = SignMatrix::new();
            draw_station_frame(&mut m, t, name, caption);
            assert_clear_of_border(&m, t);
        }
    }
}

fn assert_clear_of_border(m: &SignMatrix, t: u32) {
    for y in 2..SIGN_ROWS - 2 {
        // Columns 2 and 173 separate the field from the side blocks.
        assert!(
            !m.get(2, y) && !m.get(SIGN_COLS - 3, y),
            "field spills into the border at {t} ms, row {y}"
        );
    }
    for x in 2..SIGN_COLS - 2 {
        assert!(!m.get(x, 2) && !m.get(x, SIGN_ROWS - 3), "row 2 or 45 lit at {t} ms");
    }
}

#[test]
fn frames_are_deterministic() {
    for t in times().step_by(7) {
        let (a, pa) = frame(t);
        let (b, pb) = frame(t);
        assert!(a == b && pa == pb, "frame at {t} ms differs between calls");
        for (name, caption) in STATIONS {
            let (a, pa) = station(t, name, caption);
            let (b, pb) = station(t, name, caption);
            assert!(a == b && pa == pb, "{name} at {t} ms differs between calls");
        }
    }
}

#[test]
fn phases_switch_once_at_intro_ms() {
    for t in times() {
        let want = if t < INTRO_MS {
            BootPhase::Intro
        } else {
            BootPhase::Hold
        };
        assert_eq!(frame(t).1, want, "at {t} ms");
        for (name, caption) in STATIONS {
            assert_eq!(station(t, name, caption).1, want, "{name} at {t} ms");
        }
    }
    assert_eq!(frame(INTRO_MS - 1).1, BootPhase::Intro);
    assert_eq!(frame(INTRO_MS).1, BootPhase::Hold);
    assert_eq!(frame(u32::MAX).1, BootPhase::Hold);
    assert!((3000..=6000).contains(&INTRO_MS), "intro should be 3-6 s");
}

#[test]
fn mark_done_falls_inside_the_intro() {
    // The firmware starts TLS at MARK_DONE_MS, after the lamp test and
    // scan and before the hold.
    assert!((1000..INTRO_MS).contains(&MARK_DONE_MS), "MARK_DONE_MS {MARK_DONE_MS}");
}

#[test]
fn lamp_test_lights_everything_then_clears() {
    assert_eq!(lit_count(&frame(0).0), SIGN_COLS * SIGN_ROWS);
    assert_eq!(lit_count(&frame(850).0), 0, "dark pause after the scan");
    // The scan clears rows from the top.
    let (m, _) = frame(600);
    assert!(!m.get(0, 0) && m.get(0, SIGN_ROWS - 1));
}

#[test]
fn final_frame_is_draw_logo() {
    let full = logo();
    // The last intro frame is already final, so cutting over is seamless.
    assert!(frame(INTRO_MS - 1).0 == full, "intro does not end on the final frame");
    assert!(frame(INTRO_MS).0 == full);
    // The chase repeats every 4 steps.
    for k in 0..8 {
        let t = INTRO_MS + 4 * CHASE_STEP_MS * k;
        assert!(frame(t).0 == full, "hold at {t} ms is not the final frame");
    }
}

#[test]
fn hold_changes_only_the_border_runs() {
    let full = logo();
    let edges = vec![0, 1, SIGN_ROWS - 2, SIGN_ROWS - 1];
    let mut prev = full.clone();
    for k in 1..=8 {
        let (m, _) = frame(INTRO_MS + k * CHASE_STEP_MS);
        assert_eq!(changed_rows(&prev, &m), edges, "step {k}");
        // Same number of LEDs within one block either way.
        assert!(lit_count(&m).abs_diff(lit_count(&full)) <= 2 * 4 * 2);
        prev = m;
    }
}

#[test]
fn intro_mostly_changes_few_rows() {
    // At 40 fps, count frames where the firmware would redraw most rows.
    let mut prev = SignMatrix::new();
    let mut heavy = 0;
    let mut t = 0;
    while t < INTRO_MS {
        let (m, _) = frame(t);
        if changed_rows(&prev, &m).len() > 24 {
            heavy += 1;
        }
        prev = m;
        t += 25;
    }
    // The lamp test's on and off, and the slide (36 rows a frame for under
    // a second), and nothing else.
    assert!(heavy <= 40, "{heavy} heavy frames");
}

#[cfg(feature = "ba-mark")]
#[test]
fn slide_brings_the_letters_together() {
    // The b only moves right and the a only moves left, and they lock
    // together at the end of the slide.
    let stem_row = (LOGO_Y + 2) as usize; // only the b's stem is lit here
    let bowl_row = (LOGO_Y + 22) as usize; // through both counters
    let mut last_b = 0;
    let mut last_a = SIGN_COLS;
    let mut a_seen = 0;
    for t in (1500..=2400).step_by(10) {
        let (m, _) = frame(t);
        // The b's stem right edge; once it is in view, the b's right wall
        // is 24 LEDs further, and the next lit LED is the a. Once the a
        // reaches the seam the b hides its edge, so stop following it.
        // Columns 3..173 skip the side blocks of the border.
        let Some(b) = (3..SIGN_COLS - 3).rev().find(|&x| m.get(x, stem_row)) else {
            continue;
        };
        assert!(b >= last_b, "b moved left at {t} ms");
        last_b = b;
        let b_wall = b + 24;
        let a = (b_wall + 1..SIGN_COLS - 3).find(|&x| m.get(x, bowl_row));
        if let Some(a) = a.filter(|&a| a > b_wall + 2) {
            assert!(a <= last_a, "a moved right at {t} ms");
            last_a = a;
            a_seen += 1;
        }
    }
    assert!(a_seen > 10);
    assert_eq!(last_b as i32, LOGO_X + 9, "b stem ends in place");
    let (m, _) = frame(2400);
    let mut full = SignMatrix::new();
    draw_logo(&mut full);
    for y in 0..LOGO_HEIGHT as i32 {
        for x in 0..LOGO_WIDTH as i32 {
            if y >= 6 {
                assert_eq!(at(&m, x, y), at(&full, x, y), "mark differs at {x},{y} after the slide");
            }
        }
    }
}

#[test]
fn draw_logo_frame_still_works() {
    for t in [0, 1000, 2000, INTRO_MS - 1, INTRO_MS, INTRO_MS + 777, u32::MAX] {
        let mut m = SignMatrix::new();
        let done = draw_logo_frame(&mut m, t);
        let (want, phase) = frame(t);
        assert_eq!(done, phase == BootPhase::Hold, "at {t} ms");
        assert!(m == want, "draw_logo_frame differs from draw_boot_frame at {t} ms");
    }
}

// ---- the public intro ----------------------------------------------------

/// The field inside the border (rows 2..46, columns 2..174), as rows of
/// bits, so frames can be compared without the border.
#[cfg(not(feature = "ba-mark"))]
fn field(m: &SignMatrix) -> Vec<Vec<bool>> {
    (2..SIGN_ROWS - 2).map(|y| (2..SIGN_COLS - 2).map(|x| m.get(x, y)).collect()).collect()
}

/// Times (every ms from the border's end to the hold) at which the field
/// changes, for the top sign or a station.
#[cfg(not(feature = "ba-mark"))]
fn field_changes(draw: impl Fn(u32) -> SignMatrix) -> Vec<u32> {
    let mut prev = field(&draw(1460));
    let mut out = Vec::new();
    for t in 1461..INTRO_MS + 4 * CHASE_STEP_MS {
        let f = field(&draw(t));
        if f != prev {
            out.push(t);
        }
        prev = f;
    }
    out
}

#[cfg(not(feature = "ba-mark"))]
#[test]
fn title_types_a_letter_at_a_time_on_an_even_beat() {
    let changes = field_changes(|t| frame(t).0);
    // "BAY AREA" + "RAPID TRANSIT": 19 letters, spaces free.
    assert_eq!(changes.len(), 19, "{changes:?}");
    let beats: Vec<u32> = changes.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(beats.iter().all(|&b| b == beats[0]), "uneven typing: {beats:?}");
    assert!((40..=150).contains(&beats[0]), "{} ms a letter", beats[0]);
    // Each step adds LEDs and never removes any.
    for &t in &changes {
        let (before, after) = (frame(t - 1).0, frame(t).0);
        for y in 0..SIGN_ROWS {
            for x in 3..SIGN_COLS - 3 {
                assert!(!before.get(x, y) || after.get(x, y), "LED {x},{y} went dark at {t} ms");
            }
        }
    }
}

#[cfg(not(feature = "ba-mark"))]
#[test]
fn typing_finishes_together_before_the_hold() {
    let top = field_changes(|t| frame(t).0);
    let last_letter = *top.last().unwrap();
    assert!(last_letter < MARK_DONE_MS, "title still typing at {MARK_DONE_MS} ms");
    for (name, caption) in STATIONS {
        let changes = field_changes(|t| station(t, name, caption).0);
        // Every change but the last (the caption) is a letter.
        let (&caption_at, letters) = changes.split_last().unwrap();
        assert_eq!(letters.last(), Some(&last_letter), "{name} doesn't land with the title");
        assert!(caption_at > MARK_DONE_MS && caption_at < INTRO_MS, "{name}: caption at {caption_at} ms");
        // A name no longer than the title keeps the title's beat.
        let beat = top[1] - top[0];
        if letters.len() <= top.len() {
            assert!(letters.windows(2).all(|w| w[1] - w[0] == beat), "{name}: {letters:?}");
        }
    }
}

#[cfg(not(feature = "ba-mark"))]
#[test]
fn after_mark_done_only_a_few_leds_change() {
    // The firmware starts TLS at MARK_DONE_MS: from there to the hold the
    // top sign's field is final and the bottom only adds its caption.
    let full = field(&logo());
    for t in MARK_DONE_MS..INTRO_MS {
        assert!(field(&frame(t).0) == full, "top field moves at {t} ms");
    }
    for (name, caption) in STATIONS {
        let changes: Vec<u32> = field_changes(|t| station(t, name, caption).0)
            .into_iter()
            .filter(|&t| t >= MARK_DONE_MS)
            .collect();
        assert_eq!(changes.len(), 1, "{name}: {changes:?}");
    }
}

#[cfg(not(feature = "ba-mark"))]
#[test]
fn station_hold_changes_only_the_border_runs() {
    let edges = vec![0, 1, SIGN_ROWS - 2, SIGN_ROWS - 1];
    for (name, caption) in STATIONS {
        let mut prev = station(INTRO_MS, name, caption).0;
        assert!(station(INTRO_MS - 1, name, caption).0 == prev, "{name}: last intro frame isn't final");
        for k in 1..=8 {
            let (m, _) = station(INTRO_MS + k * CHASE_STEP_MS, name, caption);
            assert_eq!(changed_rows(&prev, &m), edges, "{name}, step {k}");
            prev = m;
        }
    }
}

#[cfg(not(feature = "ba-mark"))]
#[test]
fn title_is_two_big_lines() {
    // Two 14-row lines, rows 8-21 and 25-38, each centered.
    let m = logo();
    let lit_rows: Vec<usize> = (2..SIGN_ROWS - 2).filter(|&y| (2..SIGN_COLS - 2).any(|x| m.get(x, y))).collect();
    assert_eq!(lit_rows.first(), Some(&8));
    assert_eq!(lit_rows.last(), Some(&38));
    assert!(!lit_rows.contains(&22) && !lit_rows.contains(&24), "lines run together");
    for rows in [8..22, 25..39] {
        let (x0, _, x1, _) = bounds(&m, 2..SIGN_COLS - 2, rows).unwrap();
        assert!(x0.abs_diff(SIGN_COLS - 1 - x1) <= 1, "line off center: {x0}..{x1}");
    }
}
