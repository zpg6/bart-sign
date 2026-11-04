//! The setup portal's HTTP side: request parsing, form decoding and the
//! config page itself.
//!
//! Pure logic, no hardware: `host-tests/` runs the unit tests on the Mac.

use alloc::string::String;
use core::fmt::Write;

use crate::settings::{DEFAULT_LINE, DEFAULT_STATION, Line, STATIONS};

/// The fields of an HTTP request the portal cares about.
#[derive(Debug, PartialEq, Eq)]
pub struct Request<'a> {
    pub method: &'a str,
    /// Path without the query string.
    pub path: &'a str,
    pub host: Option<&'a str>,
    pub content_length: usize,
    /// Bytes up to and including the blank line after the headers.
    pub head_len: usize,
}

/// Parse the request line and headers. `None` until the blank line that ends
/// the headers has arrived, or if the request line is malformed.
pub fn parse_request(buf: &[u8]) -> Option<Request<'_>> {
    let end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
    // Header values may hold non-UTF-8 bytes; only the request line and the
    // two headers below need to be text.
    let mut lines = buf[..end].split(|&b| b == b'\n').map(|l| l.strip_suffix(b"\r").unwrap_or(l));

    let request_line = core::str::from_utf8(lines.next()?).ok()?;
    let mut parts = request_line.split(' ');
    let method = parts.next().filter(|m| !m.is_empty())?;
    let target = parts.next()?;
    let path = target.split('?').next().unwrap_or(target);

    let mut host = None;
    let mut content_length = 0;
    for line in lines {
        let Ok(line) = core::str::from_utf8(line) else { continue };
        let Some((name, value)) = line.split_once(':') else { continue };
        let value = value.trim();
        if name.eq_ignore_ascii_case("host") {
            host = Some(value);
        } else if name.eq_ignore_ascii_case("content-length") {
            content_length = value.parse().ok()?;
        }
    }
    Some(Request {
        method,
        path,
        host,
        content_length,
        head_len: end + 4,
    })
}

/// A form value with a bad `%` escape, invalid UTF-8, or too long.
#[derive(Debug, PartialEq, Eq)]
pub struct DecodeError;

/// Decode an `application/x-www-form-urlencoded` component (`+` is a space,
/// `%XX` a byte) into `out`.
pub fn url_decode<const N: usize>(s: &str, out: &mut heapless::String<N>) -> Result<(), DecodeError> {
    let mut bytes: heapless::Vec<u8, N> = heapless::Vec::new();
    let mut it = s.bytes();
    while let Some(b) = it.next() {
        let b = match b {
            b'+' => b' ',
            b'%' => {
                let hi = hex_val(it.next().ok_or(DecodeError)?)?;
                let lo = hex_val(it.next().ok_or(DecodeError)?)?;
                hi << 4 | lo
            }
            b => b,
        };
        bytes.push(b).map_err(|_| DecodeError)?;
    }
    let text = core::str::from_utf8(&bytes).map_err(|_| DecodeError)?;
    out.clear();
    out.push_str(text).map_err(|_| DecodeError)
}

fn hex_val(b: u8) -> Result<u8, DecodeError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(DecodeError),
    }
}

/// The setup form as submitted. Fields are already URL-decoded.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Form {
    /// Network picked from the scan list.
    pub ssid: heapless::String<32>,
    /// Network name typed by hand. Wins over `ssid` when not empty.
    pub ssid_typed: heapless::String<32>,
    pub password: heapless::String<64>,
    pub station: heapless::String<8>,
    pub line: heapless::String<8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FormError {
    /// A field failed to decode or was too long for its buffer.
    Field(&'static str),
}

impl Form {
    pub fn parse(body: &str) -> Result<Form, FormError> {
        let mut f = Form::default();
        for pair in body.split('&').filter(|p| !p.is_empty()) {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            let res = match name {
                "ssid" => url_decode(value, &mut f.ssid),
                "ssid2" => url_decode(value, &mut f.ssid_typed),
                "pass" => url_decode(value, &mut f.password),
                "stn" => url_decode(value, &mut f.station),
                "line" => url_decode(value, &mut f.line),
                _ => Ok(()),
            };
            res.map_err(|_| {
                FormError::Field(match name {
                    "ssid" | "ssid2" => "network name",
                    "pass" => "password",
                    "stn" => "station",
                    _ => "line",
                })
            })?;
        }
        Ok(f)
    }

    /// The network to join: the typed name if any, else the picked one.
    /// Surrounding spaces in a typed name are almost always a phone keyboard
    /// accident, so they are dropped.
    pub fn network(&self) -> &str {
        let typed = self.ssid_typed.trim();
        if typed.is_empty() { &self.ssid } else { typed }
    }
}

/// Escape text for HTML element content and double-quoted attributes.
pub fn escape(out: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
}

/// What the page pre-selects: the defaults, or the values from a rejected
/// submission.
pub struct Defaults<'a> {
    pub ssid: &'a str,
    pub station: &'a str,
    pub line: &'a str,
}

impl Default for Defaults<'_> {
    fn default() -> Self {
        Self {
            ssid: "",
            station: DEFAULT_STATION,
            line: DEFAULT_LINE.code(),
        }
    }
}

/// The page head up to `<main>`: bart.gov's blue (its `h1`, link and tile
/// color), white page, Arial. `$css` styles the header badge `$badge`.
macro_rules! head {
    ($css:literal, $badge:literal) => {
        concat!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<meta name=\"theme-color\" content=\"#0064a8\"><title>BART Platform Sign</title><style>\
body{margin:0;background:#fff;color:#222;font:17px/1.4 Helvetica,Arial,sans-serif}\
header{background:#0064a8;color:#fff;padding:12px 16px;display:flex;align-items:center;gap:10px;\
font-weight:700;font-size:20px}\
",
            $css,
            "main{max-width:460px;margin:0 auto;padding:4px 16px 24px}\
h1{color:#0064a8;font-size:22px;margin:18px 0 4px}\
label{display:block;margin:18px 0 6px;font-weight:700}\
input,select,button{width:100%;box-sizing:border-box;font:inherit;min-height:50px;padding:10px 12px;\
border:1px solid #888;border-radius:6px;background:#fff;color:#222}\
select+input{margin-top:8px}\
.cb{display:flex;gap:10px;align-items:center;font-weight:400;margin-top:10px}.cb input{width:24px;min-height:24px}\
button{background:#0064a8;color:#fff;border:0;margin-top:26px;font-weight:700}\
.note{color:#505050;font-size:15px}.err{border-left:5px solid #b50012;background:#fbeaea;padding:10px 12px}\
</style></head><body><header>",
            $badge,
            "BART Platform Sign</header><main>"
        )
    };
}

#[cfg(feature = "ba-mark")]
const HEAD: &str = head!(
    ".ba{background:#fff;color:#0064a8;border-radius:6px;padding:0 6px;font-size:22px;letter-spacing:-1px}",
    "<span class=\"ba\">ba</span>"
);

/// A lit red LED, like one dot of the sign.
#[cfg(not(feature = "ba-mark"))]
const HEAD: &str = head!(
    ".led{width:12px;height:12px;border-radius:50%;background:#ff3b1f;box-shadow:0 0 6px 1px #ff6040;flex:none}",
    "<span class=\"led\"></span>"
);

/// The config page. `networks` is the scan list, strongest first.
pub fn config_page(networks: &[&str], d: &Defaults<'_>, error: Option<&str>) -> String {
    let mut p = String::with_capacity(6 * 1024);
    p.push_str(HEAD);
    p.push_str("<h1>Set up your sign</h1>");
    if let Some(e) = error {
        p.push_str("<p class=\"err\">");
        escape(&mut p, e);
        p.push_str("</p>");
    }
    p.push_str(
        "<form method=\"post\" action=\"/save\">\
<label for=\"s\">Wi-Fi network</label><select id=\"s\" name=\"ssid\">",
    );
    let known = networks.contains(&d.ssid);
    p.push_str(if networks.is_empty() {
        "<option value=\"\">No networks found, type one below</option>"
    } else {
        "<option value=\"\">Choose a network</option>"
    });
    for n in networks {
        option(&mut p, n, n, *n == d.ssid);
    }
    p.push_str(
        "</select><input name=\"ssid2\" maxlength=\"32\" autocapitalize=\"off\" autocorrect=\"off\" \
spellcheck=\"false\" placeholder=\"Or type a network name\" value=\"",
    );
    if !known {
        escape(&mut p, d.ssid);
    }
    p.push_str(
        "\"><label for=\"p\">Password</label>\
<input id=\"p\" name=\"pass\" type=\"password\" maxlength=\"64\" autocapitalize=\"off\" autocorrect=\"off\" \
spellcheck=\"false\" placeholder=\"Leave empty for an open network\">\
<label class=\"cb\"><input type=\"checkbox\" onclick=\"p.type=this.checked?'text':'password'\">Show password</label>\
<label for=\"st\">Station</label><select id=\"st\" name=\"stn\">",
    );
    for (abbr, name) in STATIONS {
        option(&mut p, abbr, name, abbr.eq_ignore_ascii_case(d.station));
    }
    p.push_str("</select><label for=\"l\">Line</label><select id=\"l\" name=\"line\">");
    for l in Line::LIST {
        option(&mut p, l.code(), l.label(), l.code().eq_ignore_ascii_case(d.line));
    }
    p.push_str(
        "</select><button type=\"submit\">Save and connect</button></form>\
<p class=\"note\">The sign works with 2.4 GHz Wi-Fi only. To start over later, \
hold the BOOT button on the sign for 5 seconds.</p></main></body></html>",
    );
    p
}

fn option(p: &mut String, value: &str, label: &str, selected: bool) {
    p.push_str("<option value=\"");
    escape(p, value);
    p.push_str(if selected { "\" selected>" } else { "\">" });
    escape(p, label);
    p.push_str("</option>");
}

/// The answer to a good POST /save.
pub fn saved_page(network: &str) -> String {
    let mut p = String::with_capacity(1024);
    p.push_str(HEAD);
    p.push_str("<h1>Saved &mdash; the sign will restart and connect</h1><p>Joining <b>");
    escape(&mut p, network);
    p.push_str(
        "</b> in a few seconds.</p><p class=\"note\">You can close this page. This setup network goes away now. \
If the sign shows CAN'T JOIN, hold its BOOT button for 5 seconds and set it up again.</p></main></body></html>",
    );
    p
}

/// Status line and headers for a complete response with a body.
pub fn response_head(status: &str, content_type: &str, len: usize) -> String {
    let mut h = String::with_capacity(160);
    let _ = write!(
        h,
        "HTTP/1.0 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {len}\r\n\
Cache-Control: no-store\r\nConnection: close\r\n\r\n"
    );
    h
}

/// Send every request for another host to the portal. This is what makes
/// the phone's captive-portal check (Apple `/hotspot-detect.html`, Android
/// `/generate_204`, Windows `/connecttest.txt`, ...) fail and pop up the page.
pub const REDIRECT: &str = "HTTP/1.0 302 Found\r\nLocation: http://192.168.4.1/\r\n\
Content-Length: 0\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n";

/// Whether a request should get [`REDIRECT`] instead of the page.
pub fn should_redirect(req: &Request<'_>, portal_ip: &str) -> bool {
    let host = req.host.map(|h| h.split(':').next().unwrap_or(h));
    let foreign_host = host.is_some_and(|h| h != portal_ip);
    let probe = matches!(
        req.path,
        "/hotspot-detect.html" | "/generate_204" | "/gen_204" | "/connecttest.txt" | "/ncsi.txt" | "/success.txt"
    );
    foreign_host || probe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_urlencoded() {
        let mut s: heapless::String<32> = heapless::String::new();
        url_decode("My+Wi-Fi%21%20%26%3D%2B", &mut s).unwrap();
        assert_eq!(s, "My Wi-Fi! &=+");
        url_decode("caf%C3%A9", &mut s).unwrap();
        assert_eq!(s, "café");
        assert!(url_decode("bad%2", &mut s).is_err());
        assert!(url_decode("bad%zz", &mut s).is_err());
        assert!(url_decode("%FF", &mut s).is_err());
        let mut tiny: heapless::String<3> = heapless::String::new();
        assert!(url_decode("abcd", &mut tiny).is_err());
        assert!(url_decode("%41%42%43", &mut tiny).is_ok());
    }

    #[test]
    fn parses_form() {
        let f = Form::parse("ssid=Home+Net&ssid2=&pass=p%40ss+word%26%3D&stn=ROCK&line=RED").unwrap();
        assert_eq!(f.network(), "Home Net");
        assert_eq!(f.password, "p@ss word&=");
        assert_eq!(f.station, "ROCK");
        assert_eq!(f.line, "RED");

        let f = Form::parse("ssid=Home&ssid2=+Hidden+Net+&pass=&stn=LAFY&line=ALL&extra=1").unwrap();
        assert_eq!(f.network(), "Hidden Net");
        assert_eq!(f.password, "");

        let long = alloc::format!("ssid={}", "x".repeat(33));
        assert_eq!(Form::parse(&long), Err(FormError::Field("network name")));
        assert_eq!(Form::parse("pass=%G1"), Err(FormError::Field("password")));
    }

    #[test]
    fn form_to_settings() {
        use crate::settings::Settings;
        let f = Form::parse("ssid=Net&pass=12345678&stn=lafy&line=yellow").unwrap();
        let s = Settings::new(f.network(), &f.password, &f.station, &f.line).unwrap();
        assert_eq!(s.station, "LAFY");
        assert_eq!(s.line, Line::Yellow);
    }

    #[test]
    fn parses_requests() {
        assert_eq!(parse_request(b"GET / HTTP/1.1\r\nHost: x"), None);
        let r = parse_request(b"GET /generate_204?x=1 HTTP/1.1\r\nHost: connectivitycheck.gstatic.com\r\n\r\n")
            .unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.path, "/generate_204");
        assert_eq!(r.host, Some("connectivitycheck.gstatic.com"));
        assert!(should_redirect(&r, "192.168.4.1"));

        let raw = b"POST /save HTTP/1.1\r\nhost: 192.168.4.1\r\nCONTENT-LENGTH: 12\r\n\r\nssid=a&pass=";
        let r = parse_request(raw).unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.content_length, 12);
        assert_eq!(&raw[r.head_len..], b"ssid=a&pass=");
        assert!(!should_redirect(&r, "192.168.4.1"));

        let r = parse_request(b"GET /hotspot-detect.html HTTP/1.0\r\nHost: 192.168.4.1:80\r\n\r\n").unwrap();
        assert!(should_redirect(&r, "192.168.4.1"));
        let r = parse_request(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        assert_eq!(r.host, None);
        assert!(!should_redirect(&r, "192.168.4.1"));
        assert_eq!(parse_request(b"POST / HTTP/1.1\r\nContent-Length: x\r\n\r\n"), None);
    }

    #[test]
    fn page_escapes_and_selects() {
        let page = config_page(
            &["Cafe <Free>", "Home"],
            &Defaults {
                ssid: "Home",
                station: "ROCK",
                line: "RED",
            },
            Some("Bad & wrong"),
        );
        assert!(page.contains("<option value=\"Cafe &lt;Free&gt;\">Cafe &lt;Free&gt;</option>"));
        assert!(page.contains("<option value=\"Home\" selected>"));
        assert!(page.contains("<option value=\"ROCK\" selected>Rockridge</option>"));
        assert!(page.contains("<option value=\"RED\" selected>Red</option>"));
        assert!(page.contains("Bad &amp; wrong"));
        assert_eq!(page.matches("<option").count(), 1 + 2 + STATIONS.len() + Line::LIST.len());
        // Default page: Lafayette and Yellow.
        let page = config_page(&[], &Defaults::default(), None);
        assert!(page.contains("<option value=\"LAFY\" selected>Lafayette</option>"));
        assert!(page.contains("<option value=\"YELLOW\" selected>Yellow</option>"));
        assert!(page.len() < 6 * 1024, "page is {} bytes", page.len());
    }
}
