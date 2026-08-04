// Portable, crossterm-free rendering types: a universal cell-grid surface
// that every frontend (terminal, web, ...) can draw to and blit from.

use serde::{Deserialize, Serialize};

/// Mirrors `crossterm::style::Color`'s variants 1:1 so the terminal backend's
/// conversion is a trivial `From` impl and existing color-derived logic can
/// port by renaming a type path rather than redesigning around a different
/// shape (e.g. nested named-color variants).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Reset,
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
    Rgb { r: u8, g: u8, b: u8 },
    AnsiValue(u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::Reset
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct Attrs: u8 {
        const BOLD      = 0b0001;
        const UNDERLINE = 0b0010;
        const REVERSE   = 0b0100;
        const DIM       = 0b1000;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub glyph: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { glyph: ' ', style: Style::default() }
    }
}

/// A grid of styled cells that a frontend can draw to. Every frontend
/// (terminal, web canvas, ...) implements this; game rendering code targets
/// only this trait, never a concrete output stream.
pub trait Surface {
    fn size(&self) -> (u16, u16);
    fn set(&mut self, x: u16, y: u16, cell: Cell);
    fn clear(&mut self, style: Style);

    fn print(&mut self, x: u16, y: u16, text: &str, style: Style) {
        for (i, ch) in text.chars().enumerate() {
            self.set(x + i as u16, y, Cell { glyph: ch, style });
        }
    }
}

/// An owned, in-memory `Surface`. Frontends blit this to the real output
/// (terminal escape sequences, canvas draw calls, ...) after a render pass.
#[derive(Clone)]
pub struct CellGrid {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl CellGrid {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width as usize * height as usize],
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Cell {
        self.cells[self.index(x, y)]
    }

    fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }
}

impl Surface for CellGrid {
    fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x < self.width && y < self.height {
            let idx = self.index(x, y);
            self.cells[idx] = cell;
        }
    }

    fn clear(&mut self, style: Style) {
        for c in &mut self.cells {
            *c = Cell { glyph: ' ', style };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_blank() {
        let grid = CellGrid::new(3, 2);
        assert_eq!(grid.size(), (3, 2));
        assert_eq!(grid.get(0, 0), Cell::default());
    }

    #[test]
    fn set_writes_a_cell() {
        let mut grid = CellGrid::new(3, 2);
        let style = Style { fg: Color::Red, bg: Color::Reset, attrs: Attrs::BOLD };
        grid.set(1, 1, Cell { glyph: 'x', style });
        assert_eq!(grid.get(1, 1), Cell { glyph: 'x', style });
    }

    #[test]
    fn set_out_of_bounds_is_ignored() {
        let mut grid = CellGrid::new(2, 2);
        grid.set(5, 5, Cell { glyph: 'x', style: Style::default() });
        assert_eq!(grid.get(0, 0), Cell::default());
    }

    #[test]
    fn print_writes_each_char() {
        let mut grid = CellGrid::new(5, 1);
        grid.print(0, 0, "hi", Style::default());
        assert_eq!(grid.get(0, 0).glyph, 'h');
        assert_eq!(grid.get(1, 0).glyph, 'i');
    }

    #[test]
    fn clear_resets_all_cells_to_given_style() {
        let mut grid = CellGrid::new(2, 2);
        let style = Style { fg: Color::Blue, ..Default::default() };
        grid.clear(style);
        assert_eq!(grid.get(0, 0), Cell { glyph: ' ', style });
        assert_eq!(grid.get(1, 1), Cell { glyph: ' ', style });
    }
}
