//! A terminal's size in character cells.

/// Columns and rows of a terminal. Never zero: a zero-sized PTY confuses full-screen programs
/// such as `vim` and `top`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Size {
    pub cols: u16,
    pub rows: u16,
}

impl Size {
    /// The smallest size ever sent to a server.
    pub const MIN: Self = Self { cols: 2, rows: 1 };
    /// The largest: far beyond any screen, and it keeps the grid's memory bounded.
    pub const MAX: Self = Self { cols: 1000, rows: 500 };
    /// What a terminal starts with before its window has a size.
    pub const DEFAULT: Self = Self { cols: 80, rows: 24 };

    /// A size within [`Self::MIN`] and [`Self::MAX`].
    pub fn clamped(cols: u16, rows: u16) -> Self {
        Self { cols: cols.clamp(Self::MIN.cols, Self::MAX.cols), rows: rows.clamp(Self::MIN.rows, Self::MAX.rows) }
    }

    /// How many whole cells fit in an area, both in the same unit (logical pixels). Values that
    /// aren't usable (zero, negative, NaN or infinite cell sizes) give [`Self::DEFAULT`] instead of
    /// a nonsensical size.
    pub fn fitting(width: f32, height: f32, cell_width: f32, cell_height: f32) -> Self {
        let usable = |v: f32| v.is_finite() && v > 0.0;
        if !usable(cell_width) || !usable(cell_height) || !width.is_finite() || !height.is_finite() {
            return Self::DEFAULT;
        }
        Self::clamped(cells(width, cell_width, Self::MAX.cols), cells(height, cell_height, Self::MAX.rows))
    }
}

/// Whole cells of `cell` in `length`, at most `max`. `as` saturates, and the clamp keeps it in range.
fn cells(length: f32, cell: f32, max: u16) -> u16 {
    (length / cell).floor().clamp(0.0, f32::from(max)) as u16
}

impl Default for Size {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_cells_fit() {
        assert_eq!(Size::fitting(800.0, 480.0, 8.0, 20.0), Size { cols: 100, rows: 24 });
        assert_eq!(Size::fitting(805.9, 499.0, 8.0, 20.0), Size { cols: 100, rows: 24 });
    }

    #[test]
    fn tiny_and_huge_areas_are_clamped() {
        assert_eq!(Size::fitting(0.0, 0.0, 8.0, 20.0), Size::MIN);
        assert_eq!(Size::fitting(-50.0, -50.0, 8.0, 20.0), Size::MIN);
        assert_eq!(Size::fitting(1.0e9, 1.0e9, 8.0, 20.0), Size::MAX);
    }

    #[test]
    fn unusable_cell_sizes_give_the_default() {
        for (w, h) in [(0.0, 20.0), (8.0, 0.0), (-1.0, 20.0), (f32::NAN, 20.0), (8.0, f32::INFINITY)] {
            assert_eq!(Size::fitting(800.0, 480.0, w, h), Size::DEFAULT);
        }
        assert_eq!(Size::fitting(f32::NAN, 480.0, 8.0, 20.0), Size::DEFAULT);
    }
}
