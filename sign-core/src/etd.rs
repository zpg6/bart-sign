//! Parsing for BART's real-time departure endpoint:
//! `GET https://api.bart.gov/api/etd.aspx?cmd=etd&orig=LAFY&key=...&json=y`
//!
//! The response is BART's XML converted to JSON, so the shape is a little odd
//! (`"?xml"`, `"#cdata-section"`, every number is a string). We only describe
//! the fields we use; serde-json-core skips the rest.

use heapless::{String, Vec};
use serde::Deserialize;

/// Most destinations a single station lists (Montgomery St lists 8).
pub const MAX_DESTS: usize = 12;
/// BART gives at most three estimates per destination.
pub const MAX_ESTIMATES: usize = 4;
/// Destination groups kept per direction after filtering.
pub const MAX_GROUPS: usize = 8;

#[derive(Deserialize)]
struct Response<'a> {
    #[serde(borrow)]
    root: Root<'a>,
}

#[derive(Deserialize)]
struct Root<'a> {
    #[serde(borrow, default)]
    time: &'a str,
    #[serde(borrow, default)]
    station: Vec<Station<'a>, 1>,
}

#[derive(Deserialize)]
struct Station<'a> {
    #[serde(borrow, default)]
    etd: Vec<Etd<'a>, MAX_DESTS>,
}

#[derive(Deserialize)]
struct Etd<'a> {
    #[serde(borrow)]
    destination: &'a str,
    #[serde(borrow, default)]
    abbreviation: &'a str,
    #[serde(borrow, default)]
    estimate: Vec<Estimate<'a>, MAX_ESTIMATES>,
}

#[derive(Deserialize)]
struct Estimate<'a> {
    #[serde(borrow)]
    minutes: &'a str,
    #[serde(borrow, default)]
    direction: &'a str,
    #[serde(borrow, default)]
    length: &'a str,
    #[serde(borrow, default)]
    color: &'a str,
    #[serde(borrow, default)]
    cancelflag: &'a str,
    #[serde(borrow, default)]
    platform: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Minutes {
    /// BART reports `"Leaving"`: the train is at the platform.
    Leaving,
    In(u16),
}

impl Minutes {
    fn sort_key(self) -> u16 {
        match self {
            Minutes::Leaving => 0,
            Minutes::In(m) => m,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Train {
    pub minutes: Minutes,
    pub cars: u8,
}

/// All upcoming trains to one destination, soonest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DestGroup {
    /// Destination as BART spells it, e.g. `"SF Airport"`. Long enough for
    /// any station name ("Pleasant Hill/Contra Costa Centre"), since any
    /// station can be a short-turn terminus.
    pub destination: String<48>,
    pub abbreviation: String<4>,
    /// Platform number the trains use (0 if BART didn't say).
    pub platform: u8,
    pub direction: Direction,
    pub trains: Vec<Train, MAX_ESTIMATES>,
}

impl DestGroup {
    fn first(&self) -> u16 {
        self.trains.first().map_or(u16::MAX, |t| t.minutes.sort_key())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Departures {
    /// Server time, e.g. `"07:12:32 PM PST"`.
    pub time: String<16>,
    pub south: Vec<DestGroup, MAX_GROUPS>,
    pub north: Vec<DestGroup, MAX_GROUPS>,
}

impl Departures {
    /// Count every estimate down by `elapsed` whole minutes since the fetch,
    /// so the sign keeps ticking between fetches. A train due now stays at
    /// 1 MIN (only BART's own "Leaving" puts it at the platform); trains
    /// whose time has passed, and "Leaving" trains from an earlier fetch,
    /// are dropped, along with destinations left with no trains.
    pub fn aged(&self, elapsed: u16) -> Departures {
        let mut d = self.clone();
        if elapsed == 0 {
            return d;
        }
        for groups in [&mut d.north, &mut d.south] {
            for g in groups.iter_mut() {
                g.trains.retain(|t| match t.minutes {
                    Minutes::Leaving => false,
                    Minutes::In(m) => m >= elapsed,
                });
                for t in g.trains.iter_mut() {
                    if let Minutes::In(m) = t.minutes {
                        t.minutes = Minutes::In((m - elapsed).max(1));
                    }
                }
            }
            groups.retain(|g| !g.trains.is_empty());
        }
        d
    }

    /// BART's clock at the time of the fetch, in minutes after midnight
    /// (station local time), parsed from `"01:17:47 AM PST"`.
    pub fn clock_minutes(&self) -> Option<u16> {
        let t = self.time.as_str();
        let (hm, rest) = t.split_once(':')?;
        let h: u16 = hm.trim().parse().ok()?;
        let m: u16 = rest.get(..2)?.parse().ok()?;
        if h == 0 || h > 12 || m > 59 {
            return None;
        }
        let pm = rest.contains("PM");
        Some((h % 12 + if pm { 12 } else { 0 }) * 60 + m)
    }

    /// Late evening or night (9 PM to 5 AM by BART's clock): an empty
    /// direction means service is over, not a disruption.
    pub fn is_late(&self) -> bool {
        self.clock_minutes().is_some_and(|m| !(5 * 60..21 * 60).contains(&m))
    }

    /// No trains at all, in either direction.
    pub fn is_empty(&self) -> bool {
        self.north.is_empty() && self.south.is_empty()
    }

    pub fn direction(&self, dir: Direction) -> &[DestGroup] {
        match dir {
            Direction::North => &self.north,
            Direction::South => &self.south,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Json,
}

/// Parse an ETD response, keeping only trains whose line `color` matches
/// (case-insensitive) when `line` is given. Cancelled trains are dropped,
/// matching the platform signs. Groups in each direction are ordered by
/// their soonest train.
pub fn parse(json: &[u8], line: Option<&str>) -> Result<Departures, Error> {
    let (resp, _): (Response, _) = serde_json_core::from_slice(json).map_err(|_| Error::Json)?;

    let mut out = Departures::default();
    let _ = out.time.push_str(truncate(resp.root.time, 16));

    let Some(station) = resp.root.station.first() else {
        return Ok(out);
    };

    for etd in &station.etd {
        for est in &etd.estimate {
            if est.cancelflag == "1" {
                continue;
            }
            if let Some(line) = line {
                if !est.color.eq_ignore_ascii_case(line) {
                    continue;
                }
            }
            let dir = match est.direction {
                "North" => Direction::North,
                "South" => Direction::South,
                _ => continue,
            };
            let minutes = if est.minutes.eq_ignore_ascii_case("leaving") {
                Minutes::Leaving
            } else {
                match est.minutes.parse() {
                    Ok(m) => Minutes::In(m),
                    Err(_) => continue,
                }
            };
            let train = Train {
                minutes,
                cars: est.length.parse().unwrap_or(0),
            };

            let groups = match dir {
                Direction::North => &mut out.north,
                Direction::South => &mut out.south,
            };
            // Group by name (not abbreviation: BART gives both "Millbrae" and
            // "SFO/Millbrae" MLBR) and platform, since a platform's sign
            // only shows its own trains.
            let platform: u8 = est.platform.parse().unwrap_or(0);
            let idx = match groups
                .iter()
                .position(|g| g.destination == truncate(etd.destination, 48) && g.platform == platform)
            {
                Some(i) => i,
                None => {
                    let mut g = DestGroup {
                        destination: String::new(),
                        abbreviation: String::new(),
                        platform,
                        direction: dir,
                        trains: Vec::new(),
                    };
                    let _ = g.destination.push_str(truncate(etd.destination, 48));
                    let _ = g.abbreviation.push_str(truncate(etd.abbreviation, 4));
                    if groups.push(g).is_err() {
                        continue;
                    }
                    groups.len() - 1
                }
            };
            let _ = groups[idx].trains.push(train);
        }
    }

    for groups in [&mut out.north, &mut out.south] {
        for g in groups.iter_mut() {
            g.trains.sort_unstable_by_key(|t| t.minutes.sort_key());
        }
        groups.sort_unstable_by_key(DestGroup::first);
    }
    Ok(out)
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}
