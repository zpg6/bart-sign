//! Turns lit/unlit LEDs into pixels that look like a dot-matrix sign: each
//! LED is a soft round dot on a `pitch`×`pitch` cell, lit dots with a hotter
//! core, unlit dots faintly visible against the black face.
//!
//! Everything is integer math (no `sqrt`, no float) and works one LED row at
//! a time, so the firmware never needs a full framebuffer.

/// 8-bit-per-channel color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Pack to RGB565, the ST7796S 16-bit pixel format.
    pub const fn to_565(self) -> u16 {
        ((self.0 as u16 >> 3) << 11) | ((self.1 as u16 >> 2) << 5) | (self.2 as u16 >> 3)
    }

    /// `self` blended toward `other` by `t`/255.
    pub const fn mix(self, other: Rgb, t: u8) -> Rgb {
        const fn ch(a: u8, b: u8, t: u8) -> u8 {
            ((a as u16 * (255 - t as u16) + b as u16 * t as u16 + 127) / 255) as u8
        }
        Rgb(ch(self.0, other.0, t), ch(self.1, other.1, t), ch(self.2, other.2, t))
    }
}

/// Largest supported LED pitch in pixels.
pub const MAX_PITCH: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct DotStyle {
    /// Pixels from one LED center to the next.
    pub pitch: u8,
    /// Dot diameter in quarter-pixels (so 3.5 px = 14).
    pub diameter_q: u8,
    pub background: Rgb,
    /// Lit LED color at the rim and at the center.
    pub lit_rim: Rgb,
    pub lit_core: Rgb,
    pub unlit: Rgb,
}

/// Pre-rendered lit and unlit cells, RGB565.
pub struct DotTiles {
    pitch: usize,
    lit: [u16; MAX_PITCH * MAX_PITCH],
    unlit: [u16; MAX_PITCH * MAX_PITCH],
    background: u16,
}

impl DotTiles {
    pub fn new(style: &DotStyle) -> Self {
        let pitch = style.pitch as usize;
        assert!((1..=MAX_PITCH).contains(&pitch));

        // Work in 1/8-pixel units, sampling each pixel 4×4.
        const SUB: i32 = 4;
        let scale = 8i32;
        let center = pitch as i32 * scale / 2;
        let radius = style.diameter_q as i32 * scale / 4 / 2;
        let r2 = radius * radius;

        let mut tiles = Self {
            pitch,
            lit: [0; MAX_PITCH * MAX_PITCH],
            unlit: [0; MAX_PITCH * MAX_PITCH],
            background: style.background.to_565(),
        };

        for py in 0..pitch {
            for px in 0..pitch {
                let mut covered = 0i32;
                let mut heat = 0i32; // summed (1 - d²/r²) * 255 over covered samples
                for sy in 0..SUB {
                    for sx in 0..SUB {
                        let x = px as i32 * scale + (2 * sx + 1) * scale / (2 * SUB) - center;
                        let y = py as i32 * scale + (2 * sy + 1) * scale / (2 * SUB) - center;
                        let d2 = x * x + y * y;
                        if d2 <= r2 {
                            covered += 1;
                            heat += 255 - d2 * 255 / r2.max(1);
                        }
                    }
                }
                let n = SUB * SUB;
                let coverage = (covered * 255 / n) as u8;
                let core = if covered > 0 { (heat / covered) as u8 } else { 0 };

                let lit = style.lit_rim.mix(style.lit_core, core);
                tiles.lit[py * MAX_PITCH + px] = style.background.mix(lit, coverage).to_565();
                tiles.unlit[py * MAX_PITCH + px] = style.background.mix(style.unlit, coverage).to_565();
            }
        }
        tiles
    }

    pub fn pitch(&self) -> usize {
        self.pitch
    }

    pub fn background(&self) -> u16 {
        self.background
    }

    /// Render one row of LEDs into `out`, which is `pitch` pixel rows of
    /// `stride` pixels each. LED `x` lands at pixel column `x0 + x * pitch`;
    /// pixels outside the LED area are left untouched.
    pub fn render_led_row(&self, lit: impl Fn(usize) -> bool, cols: usize, out: &mut [u16], stride: usize, x0: usize) {
        let p = self.pitch;
        for led in 0..cols {
            let tile = if lit(led) { &self.lit } else { &self.unlit };
            let px0 = x0 + led * p;
            if px0 + p > stride {
                break;
            }
            for ty in 0..p {
                let dst = &mut out[ty * stride + px0..ty * stride + px0 + p];
                dst.copy_from_slice(&tile[ty * MAX_PITCH..ty * MAX_PITCH + p]);
            }
        }
    }
}
