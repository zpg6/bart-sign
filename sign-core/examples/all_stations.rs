//! Contact sheets for eyeballing every station and every station name.
//! `cargo run --example all_stations -- <out-dir>`
use sign_core::etd::{parse, DestGroup, Direction, Minutes, Train};
use sign_core::layout::{self, page_count};
use sign_core::render::{Renderer, Style};
use sign_core::SignMatrix;

const W: usize = 480;
const H: usize = 131;

struct Sheet { cols: usize, tiles: Vec<Vec<u16>> }

impl Sheet {
    fn save(&self, path: &str) {
        let rows = self.tiles.len().div_ceil(self.cols);
        let (sw, sh) = (self.cols * (W + 4), rows * (H + 4));
        let mut rgb = vec![40u8; sw * sh * 3];
        for (i, t) in self.tiles.iter().enumerate() {
            let (ox, oy) = ((i % self.cols) * (W + 4), (i / self.cols) * (H + 4));
            for y in 0..H {
                for x in 0..W {
                    let p = t[y * W + x];
                    let o = ((oy + y) * sw + ox + x) * 3;
                    rgb[o] = (((p >> 11) & 31) * 255 / 31) as u8;
                    rgb[o + 1] = (((p >> 5) & 63) * 255 / 63) as u8;
                    rgb[o + 2] = ((p & 31) * 255 / 31) as u8;
                }
            }
        }
        let f = std::fs::File::create(path).unwrap();
        let mut e = png::Encoder::new(std::io::BufWriter::new(f), sw as u32, sh as u32);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(&rgb).unwrap();
    }
}

fn main() {
    let out = std::env::args().nth(1).expect("out dir");
    let r = Renderer::new(W, H, &Style::BART);
    let tile = |m: &SignMatrix| {
        let mut px = vec![0u16; W * H];
        r.render_rows(m, 0, &mut px);
        px
    };
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    // 1. Every live station, all lines, every page of both directions.
    let mut files: Vec<_> = std::fs::read_dir(format!("{base}/all")).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    let mut sheet = Sheet { cols: 4, tiles: vec![] };
    for f in &files {
        let d = parse(&std::fs::read(f).unwrap(), None).unwrap();
        for dir in [Direction::South, Direction::North] {
            let g = d.direction(dir);
            for page in 0..page_count(g) {
                let mut m = SignMatrix::new();
                layout::draw_direction_page(&mut m, g, page);
                sheet.tiles.push(tile(&m));
            }
        }
    }
    sheet.save(&format!("{out}/live_all_stations.png"));

    // 2. Every station name as a destination: list (worst-case minutes) and arriving.
    let json = std::fs::read_to_string(format!("{base}/stations.json")).unwrap();
    let names: Vec<String> = json.split("\"name\":\"").skip(1).map(|s| s[..s.find('"').unwrap()].to_string()).collect();
    let mut list = Sheet { cols: 4, tiles: vec![] };
    let mut arriving = Sheet { cols: 4, tiles: vec![] };
    for pair in names.chunks(2) {
        let groups: Vec<DestGroup> = pair.iter().enumerate().map(|(i, n)| {
            let mut g = DestGroup { destination: Default::default(), abbreviation: Default::default(), platform: 1, direction: Direction::North, trains: Default::default() };
            g.destination.push_str(n).unwrap();
            let mins = if i == 0 { [105, 125] } else { [8, 23] };
            for m in mins { g.trains.push(Train { minutes: Minutes::In(m), cars: 10 }).unwrap(); }
            g
        }).collect();
        let mut m = SignMatrix::new();
        layout::draw_list(&mut m, &groups);
        list.tiles.push(tile(&m));
        for n in pair {
            let mut m = SignMatrix::new();
            layout::draw_arriving(&mut m, n, 10);
            arriving.tiles.push(tile(&m));
        }
    }
    list.save(&format!("{out}/names_list.png"));
    arriving.save(&format!("{out}/names_arriving.png"));
    println!("{} station screens, {} names", sheet.tiles.len(), names.len());
}
