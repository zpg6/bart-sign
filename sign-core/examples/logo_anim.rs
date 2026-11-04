//! Host preview of the boot sequence, drawn with the firmware's own
//! [`Renderer`] so the pixels match the TFT.
//!
//! `cargo +stable run --example logo_anim` writes, into
//! `tools/out/logo_anim/`:
//! - `frames/NNNN.png` and `frames.txt`: every distinct frame sampled at
//!   [`FRAME_MS`], with how long it stays up (for assembling `boot.gif`).
//! - `key_TTTTT.png`: frames at the times in [`KEY_MS`], for a contact sheet.
//! - `final.png`: [`draw_logo`], and `final_grid.png`, its raw LED grid.
//!
//! Each PNG is the 480 × 131 sign centered on the 480 × 320 black screen.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use sign_core::logo::{draw_boot_frame, draw_logo, BootPhase, CHASE_STEP_MS, INTRO_MS};
use sign_core::render::{Renderer, Style};
use sign_core::{SignMatrix, SIGN_COLS, SIGN_ROWS};

const SCREEN_W: usize = 480;
const SCREEN_H: usize = 320;
const SIGN_H: usize = SCREEN_W * SIGN_ROWS / SIGN_COLS;
const SIGN_Y: usize = (SCREEN_H - SIGN_H) / 2;

/// Sampling interval for the animation (25 fps).
const FRAME_MS: u32 = 40;
/// How much of the hold to record after the intro.
const HOLD_RECORD_MS: u32 = 8 * 4 * CHASE_STEP_MS;

const KEY_MS: [u32; 12] = [200, 600, 1100, 1450, 1650, 1850, 2050, 2400, 2700, 3000, 3300, INTRO_MS];

fn screen(r: &Renderer, m: &SignMatrix) -> Vec<u8> {
    let mut out = vec![0u8; SCREEN_W * SCREEN_H * 3];
    let mut line = vec![0u16; SCREEN_W];
    for y in 0..SIGN_H {
        r.render_rows(m, y, &mut line);
        for (x, &p) in line.iter().enumerate() {
            let o = ((SIGN_Y + y) * SCREEN_W + x) * 3;
            // RGB565 to RGB888, replicating the high bits.
            let (r5, g6, b5) = ((p >> 11) as u8, ((p >> 5) & 63) as u8, (p & 31) as u8);
            out[o] = (r5 << 3) | (r5 >> 2);
            out[o + 1] = (g6 << 2) | (g6 >> 4);
            out[o + 2] = (b5 << 3) | (b5 >> 2);
        }
    }
    out
}

fn grid(m: &SignMatrix, scale: usize) -> Vec<u8> {
    let (w, h) = (SIGN_COLS * scale, SIGN_ROWS * scale);
    let mut out = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let lit = m.get(x / scale, y / scale);
            let edge = x % scale == scale - 1 || y % scale == scale - 1;
            let c = match (lit, edge) {
                (true, _) => [255, 40, 24],
                (false, true) => [12, 3, 3],
                (false, false) => [36, 8, 8],
            };
            out[(y * w + x) * 3..][..3].copy_from_slice(&c);
        }
    }
    out
}

fn save(path: &Path, w: usize, h: usize, rgb: &[u8]) {
    let file = BufWriter::new(File::create(path).expect("create png"));
    let mut enc = png::Encoder::new(file, w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut wr| wr.write_image_data(rgb))
        .expect("write png");
}

fn frame(t: u32) -> (SignMatrix, BootPhase) {
    let mut m = SignMatrix::new();
    let phase = draw_boot_frame(&mut m, t);
    (m, phase)
}

fn main() {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "tools", "out", "logo_anim"]
        .iter()
        .collect();
    let frames_dir = dir.join("frames");
    let _ = fs::remove_dir_all(&frames_dir);
    fs::create_dir_all(&frames_dir).expect("create output dir");
    let r = Renderer::new(SCREEN_W, SIGN_H, &Style::BART);

    // Distinct frames and how long each stays up.
    let end = INTRO_MS + HOLD_RECORD_MS;
    let mut list = BufWriter::new(File::create(dir.join("frames.txt")).expect("frames.txt"));
    let mut t = 0;
    let mut n = 0;
    while t < end {
        let (m, phase) = frame(t);
        let mut until = t + FRAME_MS;
        while until < end && frame(until).0 == m {
            until += FRAME_MS;
        }
        let name = format!("{n:04}.png");
        save(&frames_dir.join(&name), SCREEN_W, SCREEN_H, &screen(&r, &m));
        writeln!(list, "{name} {} {t} {phase:?}", until - t).expect("write frames.txt");
        n += 1;
        t = until;
    }
    println!("{n} distinct frames, intro {INTRO_MS} ms");

    for &k in &KEY_MS {
        let (m, _) = frame(k);
        save(
            &dir.join(format!("key_{k:05}.png")),
            SCREEN_W,
            SCREEN_H,
            &screen(&r, &m),
        );
    }

    let mut m = SignMatrix::new();
    draw_logo(&mut m);
    save(&dir.join("final.png"), SCREEN_W, SCREEN_H, &screen(&r, &m));
    save(&dir.join("final_grid.png"), SIGN_COLS * 5, SIGN_ROWS * 5, &grid(&m, 5));

    // Rows the firmware would redraw per frame, sampled at FRAME_MS.
    let mut prev = SignMatrix::new();
    let mut worst = (0, 0);
    let mut t = 0;
    while t < end {
        let (m, _) = frame(t);
        let rows = (0..SIGN_ROWS).filter(|&y| m.row_differs(&prev, y)).count();
        if rows > worst.0 && t > 0 {
            worst = (rows, t);
        }
        prev = m;
        t += FRAME_MS;
    }
    println!("most rows changed between frames: {} at {} ms", worst.0, worst.1);
}
