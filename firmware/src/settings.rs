//! The user's settings (Wi-Fi network, station, line) and their on-flash
//! record format.
//!
//! Pure logic, no hardware: `host-tests/` runs the unit tests on the Mac.
//!
//! Record layout (little-endian, 112 bytes, written at offset 0 of the
//! `config` partition):
//!
//! | offset | size | field                                   |
//! |--------|------|-----------------------------------------|
//! | 0      | 4    | magic `b"BSGN"`                         |
//! | 4      | 1    | format version (1)                      |
//! | 5      | 1    | line ([`Line`] as u8)                   |
//! | 6      | 1    | SSID length (1..=32)                    |
//! | 7      | 1    | password length (0 or 8..=64)           |
//! | 8      | 4    | station abbreviation, ASCII, 0-padded   |
//! | 12     | 32   | SSID bytes, 0-padded                    |
//! | 44     | 64   | password bytes, 0-padded                |
//! | 108    | 4    | CRC-32 (IEEE) of bytes 0..108           |
//!
//! Erased flash (all 0xFF) fails the magic check, so a blank partition reads
//! as "no settings".

use heapless::String;

pub const MAGIC: [u8; 4] = *b"BSGN";
pub const VERSION: u8 = 1;
pub const RECORD_LEN: usize = 112;
const CRC_AT: usize = RECORD_LEN - 4;

pub const DEFAULT_STATION: &str = "LAFY";
pub const DEFAULT_LINE: Line = Line::Yellow;

/// Which trains the sign shows. Values are stored on flash; never renumber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Line {
    All = 0,
    Yellow = 1,
    Red = 2,
    Green = 3,
    Orange = 4,
    Blue = 5,
}

impl Line {
    pub const LIST: [Line; 6] = [Line::All, Line::Yellow, Line::Red, Line::Green, Line::Orange, Line::Blue];

    /// The form value, which is also BART's `color` field for real lines.
    pub const fn code(self) -> &'static str {
        match self {
            Line::All => "ALL",
            Line::Yellow => "YELLOW",
            Line::Red => "RED",
            Line::Green => "GREEN",
            Line::Orange => "ORANGE",
            Line::Blue => "BLUE",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Line::All => "All lines",
            Line::Yellow => "Yellow",
            Line::Red => "Red",
            Line::Green => "Green",
            Line::Orange => "Orange",
            Line::Blue => "Blue",
        }
    }

    /// The line filter for `sign_core::etd::parse`: `None` shows every train.
    pub const fn filter(self) -> Option<&'static str> {
        match self {
            Line::All => None,
            l => Some(l.code()),
        }
    }

    pub fn from_code(s: &str) -> Option<Line> {
        Line::LIST.into_iter().find(|l| l.code().eq_ignore_ascii_case(s))
    }

    fn from_u8(v: u8) -> Option<Line> {
        Line::LIST.into_iter().find(|l| *l as u8 == v)
    }
}

/// Every BART station, as `(abbreviation, name)`, sorted by name. From
/// `https://api.bart.gov/api/stn.aspx?cmd=stns` (50 stations, Nov 2026).
pub const STATIONS: &[(&str, &str)] = &[
    ("12TH", "12th St./Oakland City Center"),
    ("16TH", "16th St./Mission"),
    ("19TH", "19th St./Oakland"),
    ("24TH", "24th St./Mission"),
    ("ANTC", "Antioch"),
    ("ASHB", "Ashby"),
    ("BALB", "Balboa Park"),
    ("BAYF", "Bay Fair"),
    ("BERY", "Berryessa/North San Jose"),
    ("CAST", "Castro Valley"),
    ("CIVC", "Civic Center/UN Plaza"),
    ("COLS", "Coliseum"),
    ("COLM", "Colma"),
    ("CONC", "Concord"),
    ("DALY", "Daly City"),
    ("DBRK", "Downtown Berkeley"),
    ("DUBL", "Dublin/Pleasanton"),
    ("DELN", "El Cerrito del Norte"),
    ("PLZA", "El Cerrito Plaza"),
    ("EMBR", "Embarcadero"),
    ("FRMT", "Fremont"),
    ("FTVL", "Fruitvale"),
    ("GLEN", "Glen Park"),
    ("HAYW", "Hayward"),
    ("LAFY", "Lafayette"),
    ("LAKE", "Lake Merritt"),
    ("MCAR", "MacArthur"),
    ("MLBR", "Millbrae"),
    ("MLPT", "Milpitas"),
    ("MONT", "Montgomery St."),
    ("NBRK", "North Berkeley"),
    ("NCON", "North Concord/Martinez"),
    ("OAKL", "Oakland International Airport (OAK)"),
    ("ORIN", "Orinda"),
    ("PCTR", "Pittsburg Center"),
    ("PITT", "Pittsburg/Bay Point"),
    ("PHIL", "Pleasant Hill/Contra Costa Centre"),
    ("POWL", "Powell St."),
    ("RICH", "Richmond"),
    ("ROCK", "Rockridge"),
    ("SBRN", "San Bruno"),
    ("SFIA", "San Francisco International Airport (SFO)"),
    ("SANL", "San Leandro"),
    ("SHAY", "South Hayward"),
    ("SSAN", "South San Francisco"),
    ("UCTY", "Union City"),
    ("WCRK", "Walnut Creek"),
    ("WARM", "Warm Springs/South Fremont"),
    ("WDUB", "West Dublin/Pleasanton"),
    ("WOAK", "West Oakland"),
];

/// The table entry for `abbr` (case-insensitive), as the canonical
/// upper-case abbreviation.
pub fn find_station(abbr: &str) -> Option<&'static str> {
    STATIONS.iter().map(|(a, _)| *a).find(|a| a.eq_ignore_ascii_case(abbr))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub ssid: String<32>,
    /// Empty for an open network.
    pub password: String<64>,
    /// Always an entry of [`STATIONS`].
    pub station: &'static str,
    pub line: Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalid {
    SsidEmpty,
    SsidTooLong,
    /// WPA2 needs 8..=63 characters, or exactly 64 hex digits.
    PasswordLength,
    PasswordChars,
    UnknownStation,
    UnknownLine,
}

impl Invalid {
    /// Shown to the user on the setup page.
    pub const fn message(self) -> &'static str {
        match self {
            Invalid::SsidEmpty => "Pick a Wi-Fi network or type its name.",
            Invalid::SsidTooLong => "That network name is longer than 32 bytes.",
            Invalid::PasswordLength => {
                "The password must be 8 to 63 characters (or empty for an open network)."
            }
            Invalid::PasswordChars => "The password may only contain printable ASCII characters.",
            Invalid::UnknownStation => "Pick a station from the list.",
            Invalid::UnknownLine => "Pick a line from the list.",
        }
    }
}

impl Settings {
    /// Check and build settings from user input.
    pub fn new(ssid: &str, password: &str, station: &str, line: &str) -> Result<Settings, Invalid> {
        if ssid.is_empty() {
            return Err(Invalid::SsidEmpty);
        }
        let ssid = String::try_from(ssid).map_err(|_| Invalid::SsidTooLong)?;
        check_password(password)?;
        let password = String::try_from(password).map_err(|_| Invalid::PasswordLength)?;
        let station = find_station(station).ok_or(Invalid::UnknownStation)?;
        let line = Line::from_code(line).ok_or(Invalid::UnknownLine)?;
        Ok(Settings {
            ssid,
            password,
            station,
            line,
        })
    }

    pub fn encode(&self) -> [u8; RECORD_LEN] {
        let mut r = [0u8; RECORD_LEN];
        r[0..4].copy_from_slice(&MAGIC);
        r[4] = VERSION;
        r[5] = self.line as u8;
        r[6] = self.ssid.len() as u8;
        r[7] = self.password.len() as u8;
        r[8..8 + self.station.len()].copy_from_slice(self.station.as_bytes());
        r[12..12 + self.ssid.len()].copy_from_slice(self.ssid.as_bytes());
        r[44..44 + self.password.len()].copy_from_slice(self.password.as_bytes());
        let crc = crc32(&r[..CRC_AT]);
        r[CRC_AT..].copy_from_slice(&crc.to_le_bytes());
        r
    }

    /// Parse a record read from flash. `None` for blank flash, a bad CRC, an
    /// unknown version or out-of-range fields.
    ///
    /// A station that is no longer in [`STATIONS`] falls back to the default
    /// instead of throwing away good Wi-Fi credentials.
    pub fn decode(r: &[u8; RECORD_LEN]) -> Option<Settings> {
        if r[0..4] != MAGIC || r[4] != VERSION {
            return None;
        }
        let crc = u32::from_le_bytes([r[CRC_AT], r[CRC_AT + 1], r[CRC_AT + 2], r[CRC_AT + 3]]);
        if crc != crc32(&r[..CRC_AT]) {
            return None;
        }
        let line = Line::from_u8(r[5])?;
        let (ssid_len, pass_len) = (r[6] as usize, r[7] as usize);
        if !(1..=32).contains(&ssid_len) || pass_len > 64 {
            return None;
        }
        let ssid = core::str::from_utf8(&r[12..12 + ssid_len]).ok()?;
        let password = core::str::from_utf8(&r[44..44 + pass_len]).ok()?;
        check_password(password).ok()?;
        let abbr = &r[8..12];
        let abbr = &abbr[..abbr.iter().position(|&b| b == 0).unwrap_or(4)];
        let station = core::str::from_utf8(abbr)
            .ok()
            .and_then(find_station)
            .unwrap_or(DEFAULT_STATION);
        Some(Settings {
            ssid: String::try_from(ssid).ok()?,
            password: String::try_from(password).ok()?,
            station,
            line,
        })
    }
}

fn check_password(p: &str) -> Result<(), Invalid> {
    if !p.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        return Err(Invalid::PasswordChars);
    }
    let hex_psk = p.len() == 64 && p.bytes().all(|b| b.is_ascii_hexdigit());
    if p.is_empty() || (8..=63).contains(&p.len()) || hex_psk {
        Ok(())
    } else {
        Err(Invalid::PasswordLength)
    }
}

/// CRC-32/ISO-HDLC (the zlib/Ethernet CRC), bitwise. 108 bytes per boot does
/// not need a table.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn round_trip() {
        let s = Settings::new("Home Net", "correct horse", "rock", "red").unwrap();
        assert_eq!(s.station, "ROCK");
        assert_eq!(s.line, Line::Red);
        assert_eq!(Settings::decode(&s.encode()), Some(s));
    }

    #[test]
    fn open_network_and_max_lengths() {
        let s = Settings::new("x", "", "LAFY", "ALL").unwrap();
        assert_eq!(Settings::decode(&s.encode()).unwrap().password, "");
        assert_eq!(s.line.filter(), None);
        let ssid = "s".repeat(32);
        let psk = "a".repeat(64);
        let s = Settings::new(&ssid, &psk, "LAFY", "YELLOW").unwrap();
        assert_eq!(Settings::decode(&s.encode()), Some(s));
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(Settings::new("", "", "LAFY", "ALL"), Err(Invalid::SsidEmpty));
        assert_eq!(Settings::new(&"s".repeat(33), "", "LAFY", "ALL"), Err(Invalid::SsidTooLong));
        assert_eq!(Settings::new("n", "short", "LAFY", "ALL"), Err(Invalid::PasswordLength));
        assert_eq!(Settings::new("n", &"g".repeat(64), "LAFY", "ALL"), Err(Invalid::PasswordLength));
        assert_eq!(Settings::new("n", "pässwörd", "LAFY", "ALL"), Err(Invalid::PasswordChars));
        assert_eq!(Settings::new("n", "", "XXXX", "ALL"), Err(Invalid::UnknownStation));
        assert_eq!(Settings::new("n", "", "LAFY", "PURPLE"), Err(Invalid::UnknownLine));
    }

    #[test]
    fn blank_or_corrupt_flash_is_none() {
        assert_eq!(Settings::decode(&[0xFF; RECORD_LEN]), None);
        assert_eq!(Settings::decode(&[0; RECORD_LEN]), None);
        let mut r = Settings::new("n", "", "LAFY", "ALL").unwrap().encode();
        r[20] ^= 1;
        assert_eq!(Settings::decode(&r), None);
        let mut r = Settings::new("n", "", "LAFY", "ALL").unwrap().encode();
        r[4] = 2;
        assert_eq!(Settings::decode(&r), None);
    }

    #[test]
    fn unknown_station_falls_back() {
        let mut r = Settings::new("n", "", "LAFY", "ALL").unwrap().encode();
        r[8..12].copy_from_slice(b"ZZZZ");
        let crc = crc32(&r[..CRC_AT]);
        r[CRC_AT..].copy_from_slice(&crc.to_le_bytes());
        assert_eq!(Settings::decode(&r).unwrap().station, DEFAULT_STATION);
    }

    #[test]
    fn stations_are_unique_four_letter_codes() {
        assert_eq!(STATIONS.len(), 50);
        for (i, (a, _)) in STATIONS.iter().enumerate() {
            assert_eq!(a.len(), 4);
            assert!(a.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()));
            assert!(STATIONS[i + 1..].iter().all(|(b, _)| a != b));
        }
        assert_eq!(find_station("lafy"), Some("LAFY"));
    }
}
