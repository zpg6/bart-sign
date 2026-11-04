//! Both signs' boot frames side by side in time, as raw PNG bands for
//! compositing: `cargo run --example boot_pair -- <out-dir> [name] [caption]`.
use sign_core::logo::{self, BootPhase};
use sign_core::render::{Renderer, Style};
use sign_core::SignMatrix;

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
    let a: Vec<String> = std::env::args().collect();
    let out = &a[1];
    let name = a.get(2).map_or("Lafayette", |s| s.as_str());
    let caption = a.get(3).map_or("YELLOW LINE", |s| s.as_str());
    let (w, h) = (458, 125);
    let r = Renderer::new(w, h, &Style::BART);
    let mut px = vec![0u16; w * h];
    let mut t = 0u32;
    let mut i = 0;
    while t <= logo::INTRO_MS + 1200 {
        let mut top = SignMatrix::new();
        let mut bottom = SignMatrix::new();
        let phase = logo::draw_boot_frame(&mut top, t);
        logo::draw_station_frame(&mut bottom, t, name, caption);
        r.render_rows(&top, 0, &mut px);
        save(&format!("{out}/{i:04}_top.png"), w, h, &px);
        r.render_rows(&bottom, 0, &mut px);
        save(&format!("{out}/{i:04}_bot.png"), w, h, &px);
        let step = if phase == BootPhase::Hold { logo::CHASE_STEP_MS } else { 40 };
        println!("{i:04} {t} {step}");
        t += step;
        i += 1;
    }
}
