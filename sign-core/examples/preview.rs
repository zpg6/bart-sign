//! Render sign screens to PNG with the same renderer the firmware uses.
//! `cargo run --example preview -- <etd.json> <out-dir>`
use sign_core::etd::{parse, Direction};
use sign_core::render::{Renderer, Style};
use sign_core::{layout, SignMatrix};

fn save(path: &str, w: usize, h: usize, px: &[u16]) {
    let mut rgb = Vec::with_capacity(w * h * 3);
    for &p in px {
        rgb.push((((p >> 11) & 31) * 255 / 31) as u8);
        rgb.push((((p >> 5) & 63) * 255 / 63) as u8);
        rgb.push(((p & 31) * 255 / 31) as u8);
    }
    let f = std::fs::File::create(path).unwrap();
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    e.set_color(png::ColorType::Rgb);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&rgb).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json = std::fs::read(&args[1]).unwrap();
    let out = &args[2];
    let d = parse(&json, Some("yellow")).unwrap();
    let r = Renderer::new(480, 131, &Style::BART);
    let mut px = vec![0u16; 480 * 131];
    let mut shot = |name: &str, m: &SignMatrix| {
        r.render_rows(m, 0, &mut px);
        save(&format!("{out}/{name}.png"), 480, 131, &px);
    };
    for (name, dir) in [("south", Direction::South), ("north", Direction::North)] {
        let mut m = SignMatrix::new();
        layout::draw_direction(&mut m, d.direction(dir));
        shot(name, &m);
    }
    let mut m = SignMatrix::new();
    layout::draw_arriving(&mut m, "Pittsburg/Bay Point", 9);
    shot("arriving_pitt", &m);
    let mut m = SignMatrix::new();
    layout::draw_arriving(&mut m, "Antioch", 10);
    shot("arriving_antioch", &m);
    let mut m = SignMatrix::new();
    sign_core::logo::draw_logo(&mut m);
    shot("logo", &m);
}
