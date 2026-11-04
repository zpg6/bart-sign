//! Every BART station through the real layout code: live snapshots of all
//! 49 stations (tests/fixtures/all, taken 2026-11-02 22:20 PST), every line
//! filter, both directions, every page; plus every station name as a
//! destination, since any station can be a short-turn terminus.

use sign_core::etd::{parse, DestGroup, Direction, Minutes, Train};
use sign_core::font::Font;
use sign_core::layout::{self, fit_name, page_count, LEFT, RIGHT};
use sign_core::{SignMatrix, SIGN_COLS, SIGN_ROWS};

const LINES: [Option<&str>; 6] = [None, Some("YELLOW"), Some("RED"), Some("GREEN"), Some("ORANGE"), Some("BLUE")];

fn fixtures() -> Vec<(String, Vec<u8>)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/all");
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .map(|p| (p.file_stem().unwrap().to_string_lossy().into_owned(), std::fs::read(&p).unwrap()))
        .collect();
    out.sort();
    assert!(out.len() >= 45, "expected a snapshot per station, found {}", out.len());
    out
}

fn station_names() -> Vec<String> {
    let json = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/stations.json")).unwrap();
    // Light scan for "name":"..." to avoid a JSON dependency.
    json.split("\"name\":\"").skip(1).map(|s| s[..s.find('"').unwrap()].to_string()).collect()
}

/// Lit columns of `m` within LED rows `rows`.
fn lit_cols(m: &SignMatrix, rows: std::ops::Range<usize>) -> Vec<usize> {
    (0..SIGN_COLS).filter(|&x| rows.clone().any(|y| m.get(x, y))).collect()
}

/// Nothing outside the text area, and on each list name row the name and
/// the minutes are separated by a clear gap.
fn check_frame(m: &SignMatrix, what: &str) {
    for y in 0..SIGN_ROWS {
        assert!(!m.get(0, y) && !m.get(SIGN_COLS - 1, y), "{what}: lit LED in the edge column, row {y}");
    }
    for rows in [3..11, 29..37] {
        let cols = lit_cols(m, rows.clone());
        if cols.is_empty() {
            continue;
        }
        let widest_gap = cols.windows(2).map(|w| w[1] - w[0] - 1).max().unwrap_or(0);
        if *cols.last().unwrap() as i32 == RIGHT {
            // Minutes are right-aligned: something sits flush right, so the
            // name (if any) must end well before them.
            if cols[0] as i32 == LEFT && cols.len() > 1 {
                assert!(widest_gap >= 6, "{what}: name runs into minutes on rows {rows:?} (gap {widest_gap})");
            }
        }
    }
}

#[test]
fn every_station_every_line_every_page() {
    let mut screens = 0;
    for (code, json) in fixtures() {
        for line in LINES {
            let d = parse(&json, line).unwrap_or_else(|e| panic!("{code}: parse failed: {e:?}"));
            for dir in [Direction::South, Direction::North] {
                let groups = d.direction(dir);
                for page in 0..page_count(groups) {
                    let mut m = SignMatrix::new();
                    let info = layout::draw_direction_page(&mut m, groups, page);
                    let what = format!("{code} {line:?} {dir:?} page {page}");
                    check_frame(&m, &what);
                    if !groups.is_empty() {
                        assert!(!info.platforms.is_empty(), "{what}: no platform for the label");
                    }
                    screens += 1;
                }
            }
        }
    }
    assert!(screens > 400, "only {screens} screens checked");
}

#[test]
fn multi_platform_directions_report_both_platforms() {
    // MacArthur northbound: Richmond on platform 1, Antioch on platform 3.
    let json = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/all/MCAR.json")).unwrap();
    let d = parse(&json, None).unwrap();
    let mut m = SignMatrix::new();
    let info = layout::draw_direction(&mut m, &d.north);
    let mut p = info.platforms.to_vec();
    p.sort();
    assert_eq!(p, [1, 3]);
}

fn group(dest: &str, mins: &[Minutes], cars: u8) -> DestGroup {
    let mut g = DestGroup {
        destination: Default::default(),
        abbreviation: Default::default(),
        platform: 1,
        direction: Direction::North,
        trains: Default::default(),
    };
    g.destination.push_str(dest).unwrap();
    for &m in mins {
        g.trains.push(Train { minutes: m, cars }).unwrap();
    }
    g
}

#[test]
fn every_station_name_fits_as_a_destination() {
    let names = station_names();
    assert_eq!(names.len(), 50);
    let worst = [Minutes::In(105), Minutes::In(125)];
    for name in &names {
        // List mode with the widest realistic minutes and a 10-car train.
        let groups = [group(name, &worst, 10), group(name, &[Minutes::In(8)], 10)];
        let mut m = SignMatrix::new();
        layout::draw_list(&mut m, &groups);
        check_frame(&m, &format!("list {name}"));

        // Arriving mode in the big font.
        let mut m = SignMatrix::new();
        layout::draw_arriving(&mut m, name, 10);
        check_frame(&m, &format!("arriving {name}"));
        let lit = lit_cols(&m, 2..33);
        assert!(!lit.is_empty(), "arriving {name}: no destination drawn");

        // The fitted name is never empty, always fits, and every character
        // is in the font.
        for (font, max) in [(Font::Small, 95), (Font::Big, RIGHT - LEFT + 1)] {
            let (fitted, _) = fit_name(name, font, max);
            assert!(!fitted.is_empty(), "{name}: fitted to nothing in {font:?}");
            assert!(font.width(&fitted) <= max, "{name}: '{fitted}' too wide in {font:?}");
        }
        let (small, _) = fit_name(name, Font::Small, RIGHT - LEFT + 1);
        for c in small.chars() {
            assert!(Font::Small.width(&c.to_string()) > 0 || c == ' ', "{name}: '{c}' missing from the small font");
        }
    }
}

#[test]
fn four_destinations_page_in_pairs() {
    // Montgomery St northbound at rush hour.
    let groups: Vec<_> = ["Antioch", "Berryessa", "Dublin/Pleasanton", "Richmond"]
        .iter()
        .map(|d| group(d, &[Minutes::In(3), Minutes::In(18)], 8))
        .collect();
    assert_eq!(page_count(&groups), 2);
    let mut a = SignMatrix::new();
    let mut b = SignMatrix::new();
    layout::draw_direction_page(&mut a, &groups, 0);
    layout::draw_direction_page(&mut b, &groups, 1);
    assert!(a != b);
    let mut c = SignMatrix::new();
    layout::draw_direction_page(&mut c, &groups, 2);
    assert!(a == c, "pages wrap around");
}

#[test]
fn two_trains_at_the_platform_alternate() {
    let groups = [
        group("Richmond", &[Minutes::Leaving, Minutes::In(20)], 6),
        group("Antioch", &[Minutes::Leaving], 10),
        group("Dublin/Pleasanton", &[Minutes::In(4)], 6),
    ];
    assert_eq!(page_count(&groups), 2);
    let mut a = SignMatrix::new();
    let mut b = SignMatrix::new();
    layout::draw_direction_page(&mut a, &groups, 0);
    layout::draw_direction_page(&mut b, &groups, 1);
    let mut richmond = SignMatrix::new();
    layout::draw_arriving(&mut richmond, "Richmond", 6);
    assert!(a == richmond);
    let mut antioch = SignMatrix::new();
    layout::draw_arriving(&mut antioch, "Antioch", 10);
    assert!(b == antioch);
}

#[test]
fn empty_direction_says_no_trains() {
    let mut m = SignMatrix::new();
    let info = layout::draw_direction_page(&mut m, &[], 3);
    assert!(info.platforms.is_empty());
    assert!(!lit_cols(&m, 0..SIGN_ROWS).is_empty());
}

mod board {
    use super::*;
    use sign_core::board::Board;

    /// Every destination at every station, under every line filter, shows
    /// up on exactly one sign, and each sign's rotation reaches it.
    #[test]
    fn all_stops_accounted_for() {
        for (code, json) in fixtures() {
            for line in LINES {
                let d = parse(&json, line).unwrap();
                let b = Board::new(&d, Direction::South);
                let expected: usize = d.north.len() + d.south.len();
                let placed: usize = b.slots.iter().flatten().map(|p| p.groups.len()).sum();
                assert_eq!(placed, expected, "{code} {line:?}: destinations lost");
                // Each platform on one sign only.
                let mut seen = vec![];
                for p in b.slots.iter().flatten() {
                    assert!(!seen.contains(&p.number), "{code}: platform {} on both signs", p.number);
                    seen.push(p.number);
                }
                for slot in 0..2 {
                    let mut labels = vec![];
                    for tick in 0..b.screens(slot) {
                        let mut m = SignMatrix::new();
                        let info = b.draw(&mut m, slot, tick);
                        check_frame(&m, &format!("{code} {line:?} slot {slot} tick {tick}"));
                        assert!(info.platforms.len() <= 1, "{code}: a sign mixed platforms");
                        labels.extend(info.platforms.iter().copied());
                    }
                    for p in &b.slots[slot] {
                        assert!(labels.contains(&p.number), "{code} {line:?}: platform {} never shown", p.number);
                    }
                }
            }
        }
    }

    #[test]
    fn lafayette_puts_platform_2_on_top() {
        let json = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/lafy_leaving.json")).unwrap();
        let b = Board::new(&parse(&json, Some("yellow")).unwrap(), Direction::South);
        assert_eq!(b.slots[0].iter().map(|p| p.number).collect::<Vec<_>>(), [2]);
        assert_eq!(b.slots[1].iter().map(|p| p.number).collect::<Vec<_>>(), [1]);
        assert_eq!((b.screens(0), b.screens(1)), (1, 1));
    }

    #[test]
    fn macarthur_rotates_four_platforms() {
        let json = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/all/MCAR.json")).unwrap();
        let b = Board::new(&parse(&json, None).unwrap(), Direction::South);
        assert_eq!(b.slots[0].iter().map(|p| p.number).collect::<Vec<_>>(), [2, 4]);
        assert_eq!(b.slots[1].iter().map(|p| p.number).collect::<Vec<_>>(), [1, 3]);
        assert_eq!((b.screens(0), b.screens(1)), (2, 2));
    }

    #[test]
    fn lake_merritt_gives_each_platform_a_sign() {
        // Both platforms carry both directions there.
        let json = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/all/LAKE.json")).unwrap();
        let b = Board::new(&parse(&json, None).unwrap(), Direction::South);
        assert_eq!(b.slots[0].len(), 1);
        assert_eq!(b.slots[1].len(), 1);
    }
}

mod night {
    use super::*;
    use sign_core::board::Board;

    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    fn text_rows(m: &SignMatrix) -> Vec<usize> {
        (0..SIGN_ROWS).filter(|&y| (0..SIGN_COLS).any(|x| m.get(x, y))).collect()
    }

    #[test]
    fn clock_is_parsed_from_bart_time() {
        for (file, expect) in [("lafy_south_done.json", 47), ("lafy_service_over.json", 77), ("lafy_late_96min.json", 23 * 60 + 17)] {
            let d = parse(&fixture(file), Some("yellow")).unwrap();
            assert_eq!(d.clock_minutes(), Some(expect), "{file}");
            assert!(d.is_late());
        }
        let d = parse(&fixture("lafy_evening.json"), Some("yellow")).unwrap();
        assert_eq!(d.clock_minutes(), Some(19 * 60 + 17));
        assert!(!d.is_late());
    }

    /// 12:47 AM: the last southbound train (Millbrae, 12:43) has gone; northbound
    /// (Antioch, Pittsburg/Bay Point) still running.
    #[test]
    fn southbound_done_says_no_more_trains() {
        let d = parse(&fixture("lafy_south_done.json"), Some("yellow")).unwrap();
        assert!(d.south.is_empty() && !d.north.is_empty());
        let b = Board::new(&d, Direction::South);
        let mut top = SignMatrix::new();
        b.draw(&mut top, 0, 0);
        let mut expected = SignMatrix::new();
        layout::draw_message(&mut expected, "NO MORE TRAINS", "TONIGHT");
        assert!(top == expected);
        let mut bottom = SignMatrix::new();
        let info = b.draw(&mut bottom, 1, 0);
        assert_eq!(info.platforms.as_slice(), [1]);
        assert!(text_rows(&bottom).len() > 20, "northbound list still shows");
    }

    /// 1:17 AM onwards: BART returns the station with no departures and a
    /// "No data matched your criteria" warning.
    #[test]
    fn service_over_both_signs() {
        let d = parse(&fixture("lafy_service_over.json"), Some("yellow")).unwrap();
        assert!(d.is_empty() && d.is_late());
        let b = Board::new(&d, Direction::South);
        for slot in 0..2 {
            let mut m = SignMatrix::new();
            let info = b.draw(&mut m, slot, 0);
            assert!(info.platforms.is_empty());
            let mut expected = SignMatrix::new();
            layout::draw_message(&mut expected, "NO MORE TRAINS", "TONIGHT");
            assert!(m == expected);
        }
    }

    /// 11:17 PM: a Pittsburg/Bay Point train 96 minutes out still lays out.
    #[test]
    fn long_waits_lay_out() {
        let d = parse(&fixture("lafy_late_96min.json"), Some("yellow")).unwrap();
        let b = Board::new(&d, Direction::South);
        for slot in 0..2 {
            for tick in 0..b.screens(slot) {
                let mut m = SignMatrix::new();
                b.draw(&mut m, slot, tick);
                check_frame(&m, &format!("late slot {slot} tick {tick}"));
            }
        }
    }

    /// Every overnight snapshot, every station.
    #[test]
    fn every_overnight_snapshot() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/night");
        let mut n = 0;
        for f in std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()) {
            let json = std::fs::read(&f).unwrap();
            // The ALL snapshots hold every station; split them like the device sees them.
            let text = String::from_utf8_lossy(&json);
            if f.file_name().unwrap().to_string_lossy().starts_with("ALL_") {
                assert!(text.contains("\"station\""));
                continue;
            }
            let d = parse(&json, None).unwrap();
            let b = Board::new(&d, Direction::South);
            for slot in 0..2 {
                for tick in 0..b.screens(slot) {
                    let mut m = SignMatrix::new();
                    b.draw(&mut m, slot, tick);
                    check_frame(&m, &format!("{f:?} slot {slot}"));
                    n += 1;
                }
            }
        }
        assert!(n >= 40);
    }
}
