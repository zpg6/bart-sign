//! Every station through the real board, layout and renderer: writes both
//! signs' LED bands for every rotation screen, plus an index for composing
//! mockups. `cargo run --release --example mockups -- <fixtures-dir> <out-dir>`
use sign_core::board::Board;
use sign_core::etd::{parse, Direction};
use sign_core::render::{Renderer, Style};
use sign_core::SignMatrix;
use std::io::Write;

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
    let (dir, out) = (&a[1], &a[2]);
    let (w, h) = (458, 125);
    let r = Renderer::new(w, h, &Style::BART);
    let mut px = vec![0u16; w * h];
    let mut files: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "json")).collect();
    files.sort();
    let mut index = std::fs::File::create(format!("{out}/index.tsv")).unwrap();
    for f in files {
        let code = f.file_stem().unwrap().to_string_lossy().into_owned();
        let d = parse(&std::fs::read(&f).unwrap(), None).unwrap();
        let b = Board::new(&d, Direction::South);
        let screens = b.screens(0).max(b.screens(1));
        for tick in 0..screens {
            let mut labels = [0u8; 2];
            for (slot, label) in labels.iter_mut().enumerate() {
                let mut m = SignMatrix::new();
                let info = b.draw(&mut m, slot, tick);
                *label = info.platforms.first().copied().unwrap_or(0);
                r.render_rows(&m, 0, &mut px);
                save(&format!("{out}/{code}_{tick}_{slot}.png"), w, h, &px);
            }
            let plats: Vec<String> = (0..2).map(|s| b.slots[s].iter().map(|p| p.number.to_string()).collect::<Vec<_>>().join(",")).collect();
            writeln!(index, "{code}\t{tick}\t{screens}\t{}\t{}\t{}\t{}", labels[0], labels[1], plats[0], plats[1]).unwrap();
        }
    }
}
