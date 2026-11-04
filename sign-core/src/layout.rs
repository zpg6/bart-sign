//! What goes where on the 176 × 48 sign, measured from station photos.
//!
//! List mode (two destinations, four lines):
//! ```text
//! rows  3-10  SF AIRPORT                     1,13 MIN
//! rows 14-21  10-CAR, 3-DOOR
//! rows 29-36  SFO/MILLBRAE                  12,32 MIN
//! rows 40-47  9-CAR. 3-DOOR      <- comma tail clipped by the bottom edge
//! ```
//! Arriving mode: the destination in the big font, centered on rows 10-23
//! (or two big lines on rows 2-15 and 19-32), car line centered on rows 38-45.
//!
//! A direction with more than two destinations (Montgomery St at rush hour
//! has four northbound) pages through them two at a time, the way the real
//! signs alternate. While a train is at the platform its arriving screen
//! takes over; two at once (MacArthur's cross-platform transfers) alternate.

use core::fmt::Write;

use heapless::{String, Vec};

use crate::etd::{DestGroup, Minutes};
use crate::font::Font;
use crate::{SignMatrix, SIGN_COLS};

/// First and last LED column text may use.
pub const LEFT: i32 = 1;
pub const RIGHT: i32 = SIGN_COLS as i32 - 2;
/// Minimum dark columns between a destination and its minutes.
const NAME_GAP: i32 = 8;

const LIST_ROWS: [(i32, i32); 2] = [(3, 14), (29, 40)];

/// Every BART train since 2023 is a three-door Fleet of the Future train,
/// and the API does not report doors.
const DOORS: u8 = 3;

pub type Name = String<48>;

fn upper(s: &str) -> Name {
    let mut out = Name::new();
    for c in s.chars() {
        let _ = out.push(c.to_ascii_uppercase());
    }
    out
}

/// Shorter forms in the style of BART's own signs ("MILLBRAE/CALTRN"),
/// tried before falling back to generic shortening.
fn alias(full: &str) -> Option<&'static str> {
    Some(match full {
        "PITTSBURG/BAY POINT" => "PITTSBURG/BAY PT",
        "DUBLIN/PLEASANTON" => "DUBLIN/PLSNTN",
        "WEST DUBLIN/PLEASANTON" => "W DUBLIN/PLSNTN",
        "MILLBRAE/CALTRAIN" => "MILLBRAE/CALTRN",
        "WARM SPRINGS/SOUTH FREMONT" => "WARM SPRINGS",
        "BERRYESSA/NORTH SAN JOSE" => "BERRYESSA",
        "12TH ST./OAKLAND CITY CENTER" => "12TH ST OAKLAND",
        "19TH ST./OAKLAND" => "19TH ST OAKLAND",
        "16TH ST./MISSION" => "16TH ST MISSION",
        "24TH ST./MISSION" => "24TH ST MISSION",
        "PLEASANT HILL/CONTRA COSTA CENTRE" => "PLEASANT HILL",
        "CIVIC CENTER/UN PLAZA" => "CIVIC CENTER",
        "NORTH CONCORD/MARTINEZ" => "NORTH CONCORD",
        "SAN FRANCISCO INTERNATIONAL AIRPORT (SFO)" => "SF AIRPORT",
        "OAKLAND INTERNATIONAL AIRPORT (OAK)" => "OAK AIRPORT",
        "EL CERRITO DEL NORTE" => "DEL NORTE",
        "EL CERRITO PLAZA" => "EL CERRITO PLZ",
        "SOUTH SAN FRANCISCO" => "S SAN FRANCISCO",
        "SOUTH HAYWARD" => "S HAYWARD",
        "NORTH BERKELEY" => "N BERKELEY",
        "DOWNTOWN BERKELEY" => "DOWNTOWN BERK",
        "PITTSBURG CENTER" => "PITTSBURG CTR",
        "MONTGOMERY ST." => "MONTGOMERY",
        _ => return None,
    })
}

/// Two-line forms for the big arriving font, for names with no slash to
/// split at or whose halves are still too wide.
fn two_lines(full: &str) -> Option<(&'static str, &'static str)> {
    Some(match full {
        "12TH ST./OAKLAND CITY CENTER" => ("12TH ST", "OAKLAND"),
        "PLEASANT HILL/CONTRA COSTA CENTRE" => ("PLEASANT HILL", "CONTRA COSTA"),
        "SAN FRANCISCO INTERNATIONAL AIRPORT (SFO)" => ("SAN FRANCISCO", "AIRPORT"),
        "OAKLAND INTERNATIONAL AIRPORT (OAK)" => ("OAKLAND", "AIRPORT"),
        "SOUTH SAN FRANCISCO" => ("SOUTH", "SAN FRANCISCO"),
        "DOWNTOWN BERKELEY" => ("DOWNTOWN", "BERKELEY"),
        "EL CERRITO DEL NORTE" => ("EL CERRITO", "DEL NORTE"),
        "EL CERRITO PLAZA" => ("EL CERRITO", "PLAZA"),
        "PITTSBURG CENTER" => ("PITTSBURG", "CENTER"),
        _ => return None,
    })
}

/// How much a name had to give up to fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fit {
    Full,
    /// BART-style abbreviation, or the part before the slash.
    Short,
    /// Words dropped or letters cut: still recognizable, but a last resort.
    Cut,
}

/// The destination in sign style, as wide as possible within `max` LED
/// columns of `font`: the full name, BART-style abbreviation, the part
/// before a slash, then dropping trailing words, then cutting letters.
/// Never returns something wider than `max`.
pub fn fit_name(dest: &str, font: Font, max: i32) -> (Name, Fit) {
    let full = upper(dest.trim());
    let fits = |s: &str| font.width(s) <= max;
    if fits(&full) {
        return (full, Fit::Full);
    }
    let mut s = Name::new();
    if let Some(a) = alias(&full).filter(|a| fits(a)) {
        let _ = s.push_str(a);
        return (s, Fit::Short);
    }
    let base = alias(&full).unwrap_or(&full);
    let _ = s.push_str(base.split('/').next().unwrap_or(base).trim());
    if fits(&s) {
        return (s, Fit::Short);
    }
    while !fits(&s) {
        match s.rfind(' ') {
            Some(i) if i > 0 => s.truncate(i),
            _ => break,
        }
        // Never end on a dangling connector ("EL CERRITO DEL").
        while let Some(i) = s.rfind(' ').filter(|&i| matches!(&s[i + 1..], "DEL" | "DE" | "LA" | "THE" | "OF" | "AND" | "&")) {
            s.truncate(i);
        }
    }
    while !fits(&s) && !s.is_empty() {
        s.pop();
    }
    (s, Fit::Cut)
}

fn minutes_text(trains: &[Minutes], count: usize) -> Option<String<24>> {
    let mut s: String<24> = String::new();
    let mut n = 0;
    for m in trains {
        let Minutes::In(min) = m else { continue };
        if n == count {
            break;
        }
        if n > 0 {
            s.push(',').ok()?;
        }
        write!(s, "{min}").ok()?;
        n += 1;
    }
    if n == 0 {
        return None;
    }
    s.push_str(" MIN").ok()?;
    Some(s)
}

fn car_text(cars: u8) -> String<24> {
    let mut s = String::new();
    let _ = write!(s, "{cars}-CAR, {DOORS}-DOOR");
    s
}

fn is_leaving(g: &DestGroup) -> bool {
    g.trains.first().is_some_and(|t| t.minutes == Minutes::Leaving)
}

/// Four-line list of up to two destinations.
pub fn draw_list(m: &mut SignMatrix, groups: &[DestGroup]) {
    for (g, &(name_y, car_y)) in groups.iter().zip(LIST_ROWS.iter()) {
        let mins: Vec<Minutes, 4> = g.trains.iter().map(|t| t.minutes).collect();
        // Two times if the name still reads well next to them, else one,
        // else (only a train at the platform) the name alone.
        let mut drawn = false;
        for count in [2, 1] {
            let Some(min_text) = minutes_text(&mins, count) else { break };
            let room = RIGHT + 1 - Font::Small.width(&min_text) - NAME_GAP - LEFT;
            let (name, fit) = fit_name(&g.destination, Font::Small, room);
            // A second time is worth an abbreviation, not a mangled name.
            if count == 2 && fit == Fit::Cut {
                continue;
            }
            Font::Small.draw(m, LEFT, name_y, &name);
            Font::Small.draw(m, RIGHT + 1 - Font::Small.width(&min_text), name_y, &min_text);
            drawn = true;
            break;
        }
        if !drawn {
            let (name, _) = fit_name(&g.destination, Font::Small, RIGHT - LEFT + 1);
            Font::Small.draw(m, LEFT, name_y, &name);
        }
        let cars = g
            .trains
            .iter()
            .find(|t| matches!(t.minutes, Minutes::In(_)))
            .or(g.trains.first())
            .map_or(0, |t| t.cars);
        if cars > 0 {
            Font::Small.draw(m, LEFT, car_y, &car_text(cars));
        }
    }
}

fn centered(font: Font, text: &str) -> i32 {
    (SIGN_COLS as i32 - font.width(text)) / 2
}

/// A name in the big font, centered: one line if it fits, else two (split
/// at the slash or a known two-line form), else shortened.
fn draw_big_name(m: &mut SignMatrix, name: &str) {
    let width = RIGHT - LEFT + 1;
    let full = upper(name.trim());
    let fits = |t: &str| Font::Big.width(t) <= width;
    let split = full.split_once('/').map(|(a, b)| (a.trim(), b.trim()));
    let pair = split.filter(|(a, b)| fits(a) && fits(b)).or(two_lines(&full).filter(|(a, b)| fits(a) && fits(b)));
    if fits(&full) {
        Font::Big.draw(m, centered(Font::Big, &full), 10, &full);
    } else if let Some((a, b)) = pair {
        // Two big lines: PITTSBURG / BAY POINT, 12TH ST / OAKLAND.
        Font::Big.draw(m, centered(Font::Big, a), 2, a);
        Font::Big.draw(m, centered(Font::Big, b), 19, b);
    } else {
        let (short, _) = fit_name(name, Font::Big, width);
        Font::Big.draw(m, centered(Font::Big, &short), 10, &short);
    }
}

/// A train at the platform: big destination, car line below.
pub fn draw_arriving(m: &mut SignMatrix, destination: &str, cars: u8) {
    draw_big_name(m, destination);
    if cars > 0 {
        let car = car_text(cars);
        Font::Small.draw(m, centered(Font::Small, &car), 38, &car);
    }
}

/// A title card in the arriving layout: `name` big, `caption` small below.
/// The boot screen uses it to show which station the sign is set to.
pub fn draw_title(m: &mut SignMatrix, name: &str, caption: &str) {
    draw_big_name(m, name);
    Font::Small.draw(m, centered(Font::Small, caption), 38, caption);
}

/// The big lines [`draw_card`] shows for `name`: one, or two split like the
/// arriving screen, shortened if need be to fit inside the boot border.
pub fn card_lines(name: &str) -> Vec<Name, 2> {
    const INNER: i32 = 170;
    let full = upper(name.trim());
    let fits = |t: &str| Font::Big.width(t) <= INNER;
    let split = full.split_once('/').map(|(a, b)| (a.trim(), b.trim()));
    let pair = split.filter(|(a, b)| fits(a) && fits(b)).or(two_lines(&full).filter(|(a, b)| fits(a) && fits(b)));
    let mut lines = Vec::new();
    if fits(&full) {
        let _ = lines.push(full.clone());
    } else if let Some((a, b)) = pair {
        for s in [a, b] {
            let mut l = Name::new();
            let _ = l.push_str(s);
            let _ = lines.push(l);
        }
    } else {
        let _ = lines.push(fit_name(name, Font::Big, INNER).0);
    }
    lines
}

/// The boot screen's station card, inside the boot border (columns 3-172,
/// rows 2-45): `name` big (one line, or two split like the arriving
/// screen), `caption` small below. Only the first `chars` characters of the
/// name are lit, so it can type in; the caption shows once `caption` is
/// `Some`. Letters don't move as they appear: positions come from the
/// whole text.
pub fn draw_card(m: &mut SignMatrix, name: &str, chars: usize, caption: Option<&str>) {
    let lines = card_lines(name);
    let (rows, caption_y): (&[i32], i32) = if lines.len() == 2 { (&[3, 18], 36) } else { (&[10], 30) };
    let mut left = chars;
    for (line, &y) in lines.iter().zip(rows) {
        let n = line.chars().count().min(left);
        let shown: String<48> = line.chars().take(n).collect();
        Font::Big.draw(m, centered(Font::Big, line), y, &shown);
        left -= n;
    }
    if let Some(c) = caption {
        Font::Small.draw(m, centered(Font::Small, c), caption_y, c);
    }
}

/// One or two centered small-font lines, for status messages.
pub fn draw_message(m: &mut SignMatrix, line1: &str, line2: &str) {
    if line2.is_empty() {
        Font::Small.draw(m, centered(Font::Small, line1), 20, line1);
    } else {
        Font::Small.draw(m, centered(Font::Small, line1), 14, line1);
        Font::Small.draw(m, centered(Font::Small, line2), 27, line2);
    }
}

/// How many screens a direction cycles through: one per train at the
/// platform while any is, otherwise one per two destinations. At least 1.
pub fn page_count(groups: &[DestGroup]) -> usize {
    let leaving = groups.iter().filter(|g| is_leaving(g)).count();
    if leaving > 0 {
        leaving
    } else {
        groups.len().div_ceil(2).max(1)
    }
}

/// What a drawn page shows, for the bezel label.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageInfo {
    /// Distinct platforms of the trains on screen, in order of appearance.
    pub platforms: Vec<u8, 2>,
}

fn note_platform(info: &mut PageInfo, p: u8) {
    if p != 0 && !info.platforms.contains(&p) {
        let _ = info.platforms.push(p);
    }
}

/// Screen `page` (taken modulo [`page_count`]) for a direction.
pub fn draw_direction_page(m: &mut SignMatrix, groups: &[DestGroup], page: usize) -> PageInfo {
    let mut info = PageInfo::default();
    if groups.is_empty() {
        draw_message(m, "NO TRAINS", "");
        return info;
    }
    let page = page % page_count(groups);
    if let Some(g) = groups.iter().filter(|g| is_leaving(g)).nth(page) {
        draw_arriving(m, &g.destination, g.trains[0].cars);
        note_platform(&mut info, g.platform);
        return info;
    }
    let shown = &groups[page * 2..(page * 2 + 2).min(groups.len())];
    draw_list(m, shown);
    for g in shown {
        note_platform(&mut info, g.platform);
    }
    info
}

/// The first screen for a direction.
pub fn draw_direction(m: &mut SignMatrix, groups: &[DestGroup]) -> PageInfo {
    draw_direction_page(m, groups, 0)
}
