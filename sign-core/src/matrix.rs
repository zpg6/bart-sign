//! A 1-bit LED matrix: which LEDs on the sign are lit.

/// Fixed-size bit grid, `W` columns by `H` rows. `WORDS` must be at least
/// `(W * H).div_ceil(32)`; use the [`matrix!`](crate::matrix) helper type
/// alias to get it right.
#[derive(Clone, PartialEq, Eq)]
pub struct Matrix<const W: usize, const H: usize, const WORDS: usize> {
    bits: [u32; WORDS],
}

impl<const W: usize, const H: usize, const WORDS: usize> Default for Matrix<W, H, WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize, const H: usize, const WORDS: usize> Matrix<W, H, WORDS> {
    pub const WIDTH: usize = W;
    pub const HEIGHT: usize = H;

    pub const fn new() -> Self {
        assert!(WORDS * 32 >= W * H, "Matrix storage too small");
        Self { bits: [0; WORDS] }
    }

    pub fn clear(&mut self) {
        self.bits = [0; WORDS];
    }

    /// Light the LED at (x, y). Out-of-range coordinates are clipped, so
    /// text can run off the edge the way it does on the real sign.
    pub fn set(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 || x as usize >= W || y as usize >= H {
            return;
        }
        let i = y as usize * W + x as usize;
        self.bits[i / 32] |= 1 << (i % 32);
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        if x >= W || y >= H {
            return false;
        }
        let i = y * W + x;
        self.bits[i / 32] & (1 << (i % 32)) != 0
    }

    /// True if any LED in row `y` differs between `self` and `other`.
    pub fn row_differs(&self, other: &Self, y: usize) -> bool {
        (0..W).any(|x| self.get(x, y) != other.get(x, y))
    }
}

/// `matrix!(W, H)` expands to `Matrix<W, H, {words}>` with the storage size
/// computed for you.
#[macro_export]
macro_rules! matrix {
    ($w:expr, $h:expr) => {
        $crate::matrix::Matrix<{ $w }, { $h }, { ($w * $h).div_ceil(32) }>
    };
}
