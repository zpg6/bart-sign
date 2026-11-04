//! Rasterizes the sign's LEDs into RGB565 pixels at any (fractional)
//! pitch: 176 LEDs across 480 px is 2.73 px per LED. Each pixel looks up
//! which LED it belongs to and how far it is from that LED's center, then
//! reads a precomputed brightness table, so a row of pixels costs one table
//! lookup per pixel and no float math.

use crate::dots::Rgb;
use crate::{SignMatrix, SIGN_COLS, SIGN_ROWS};

/// Sub-pixel resolution of the distance tables.
const Q: i32 = 8;
/// Pixel-to-LED-center offsets are tabulated over ±2 px, in 1/Q px.
const HALF: i32 = 2 * Q;
const SIDE: usize = (2 * HALF + 1) as usize;

pub const MAX_W: usize = 480;
pub const MAX_H: usize = 320;

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub background: Rgb,
    pub lit_rim: Rgb,
    pub lit_core: Rgb,
    pub unlit: Rgb,
    /// LED dot diameter as a fraction of pitch, in 1/256.
    pub dot_256: u16,
}

impl Style {
    pub const BART: Style = Style {
        background: Rgb(6, 2, 2),
        lit_rim: Rgb(235, 22, 14),
        lit_core: Rgb(255, 120, 90),
        unlit: Rgb(40, 8, 8),
        dot_256: 200,
    };
}

pub struct Renderer {
    width: usize,
    height: usize,
    /// Per pixel column: LED column, and offset from its center in 1/Q px.
    cols: [(u8, i8); MAX_W],
    rows: [(u8, i8); MAX_H],
    lit: [u16; SIDE * SIDE],
    unlit: [u16; SIDE * SIDE],
}

fn map_axis(px: usize, leds: usize, out: &mut [(u8, i8)]) {
    // Pixel p's center is at p + 1/2; LED i's center at (i + 1/2) * px/leds.
    let (px, leds) = (px as i64, leds as i64);
    for (p, slot) in out.iter_mut().enumerate().take(px as usize) {
        let p = p as i64;
        let led = ((2 * p + 1) * leds / (2 * px)).min(leds - 1);
        let off = Q as i64 * ((2 * p + 1) * leds - (2 * led + 1) * px) / (2 * leds);
        *slot = (led as u8, off.clamp(-HALF as i64, HALF as i64) as i8);
    }
}

impl Renderer {
    /// A renderer for the whole sign drawn into `width` × `height` pixels.
    pub fn new(width: usize, height: usize, style: &Style) -> Self {
        assert!(width <= MAX_W && height <= MAX_H);
        let mut r = Renderer {
            width,
            height,
            cols: [(0, 0); MAX_W],
            rows: [(0, 0); MAX_H],
            lit: [0; SIDE * SIDE],
            unlit: [0; SIDE * SIDE],
        };
        map_axis(width, SIGN_COLS, &mut r.cols);
        map_axis(height, SIGN_ROWS, &mut r.rows);

        // Dot radius in 1/Q px, from the horizontal pitch.
        let pitch_q = (width as i32 * Q) / SIGN_COLS as i32; // ~21 for 2.73 px
        let radius = pitch_q * style.dot_256 as i32 / 256 / 2;
        let r2 = radius * radius;
        let glow = radius + Q; // faint halo one pixel past the dot
        let g2 = glow * glow;
        const SUB: i32 = 4;
        for dy in -HALF..=HALF {
            for dx in -HALF..=HALF {
                let mut covered = 0;
                let mut heat = 0;
                let mut halo = 0;
                for sy in 0..SUB {
                    for sx in 0..SUB {
                        let x = dx * SUB + sx * 2 - (SUB - 1);
                        let y = dy * SUB + sy * 2 - (SUB - 1);
                        let d2 = (x * x + y * y) / (SUB * SUB);
                        if d2 <= r2 {
                            covered += 1;
                            heat += 255 - d2 * 255 / r2.max(1);
                        } else if d2 <= g2 {
                            halo += (g2 - d2) * 60 / (g2 - r2).max(1);
                        }
                    }
                }
                let n = SUB * SUB;
                let cov = (covered * 255 / n) as u8;
                let core = if covered > 0 { (heat / covered) as u8 } else { 0 };
                let halo = (halo / n) as u8;
                let i = ((dy + HALF) as usize) * SIDE + (dx + HALF) as usize;
                let lit = style.lit_rim.mix(style.lit_core, core);
                let glow_px = style.background.mix(style.lit_rim, halo);
                r.lit[i] = glow_px.mix(lit, cov).to_565();
                r.unlit[i] = style.background.mix(style.unlit, cov).to_565();
            }
        }
        r
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// The LED row that pixel row `y` belongs to.
    pub fn led_row(&self, y: usize) -> usize {
        self.rows[y].0 as usize
    }

    /// Fill `out` (`width` × n pixels) with pixel rows `y0..y0+n`.
    pub fn render_rows(&self, m: &SignMatrix, y0: usize, out: &mut [u16]) {
        let w = self.width;
        for (k, line) in out.chunks_exact_mut(w).enumerate() {
            let y = y0 + k;
            if y >= self.height {
                break;
            }
            let (led_y, oy) = self.rows[y];
            let base = ((oy as i32 + HALF) as usize) * SIDE;
            for (x, px) in line.iter_mut().enumerate() {
                let (led_x, ox) = self.cols[x];
                let i = base + (ox as i32 + HALF) as usize;
                *px = if m.get(led_x as usize, led_y as usize) { self.lit[i] } else { self.unlit[i] };
            }
        }
    }
}
