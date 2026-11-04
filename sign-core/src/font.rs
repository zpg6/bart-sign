//! The two LED fonts of the classic BART platform sign, measured from
//! station photos (see `notes/font-measurements-a.md` and `-b.md`).
//!
//! - [`Font::Small`]: 8 LEDs tall (plus a 2-row comma descender),
//!   proportional, 3 dark columns between glyphs.
//! - [`Font::Big`]: 14 LEDs tall, 2-LED strokes, 2 dark columns between
//!   glyphs. Used for the destination when a train is at the platform.

use crate::glyphs::{BIG, SMALL};
use crate::SignMatrix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Small,
    Big,
}

impl Font {
    /// Dark columns after every glyph.
    pub const fn gap(self) -> i32 {
        match self {
            Font::Small => 3,
            Font::Big => 2,
        }
    }

    /// Cap height in LEDs.
    pub const fn height(self) -> i32 {
        match self {
            Font::Small => 8,
            Font::Big => 14,
        }
    }

    fn glyph(self, c: char) -> Option<&'static [&'static str]> {
        let c = c.to_ascii_uppercase();
        match self {
            Font::Small => SMALL.iter().find(|(g, _)| *g == c).map(|(_, rows)| &rows[..]),
            Font::Big => BIG.iter().find(|(g, _)| *g == c).map(|(_, rows)| &rows[..]),
        }
    }

    /// Width of `text` in LEDs, without a trailing gap. Characters the font
    /// lacks are skipped.
    pub fn width(self, text: &str) -> i32 {
        let mut w = 0;
        let mut n = 0;
        for c in text.chars() {
            if let Some(rows) = self.glyph(c) {
                w += rows[0].len() as i32;
                n += 1;
            }
        }
        w + (n - 1).max(0) * self.gap()
    }

    /// Draw `text` with its top-left at (`x`, `y`). Returns the x just past
    /// the last glyph (before the gap). LEDs off the sign are clipped, which
    /// is how the real sign loses the comma tail on its bottom line.
    pub fn draw(self, m: &mut SignMatrix, x: i32, y: i32, text: &str) -> i32 {
        let mut cx = x;
        let mut first = true;
        for c in text.chars() {
            let Some(rows) = self.glyph(c) else { continue };
            if !first {
                cx += self.gap();
            }
            first = false;
            for (dy, row) in rows.iter().enumerate() {
                for (dx, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        m.set(cx + dx as i32, y + dy as i32);
                    }
                }
            }
            cx += rows[0].len() as i32;
        }
        cx
    }
}
