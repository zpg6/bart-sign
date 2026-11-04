#![no_std]

pub mod board;
pub mod dots;
pub mod etd;
pub mod font;
mod glyphs;
pub mod layout;
pub mod logo;
pub mod matrix;
pub mod render;

/// The classic BART platform sign: 176 × 48 LEDs on a square pitch
/// (11 × 3 modules of 16 × 16), measured from station photos.
pub const SIGN_COLS: usize = 176;
pub const SIGN_ROWS: usize = 48;
pub type SignMatrix = matrix!(SIGN_COLS, SIGN_ROWS);
