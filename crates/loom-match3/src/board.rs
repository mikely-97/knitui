use crossterm::style::Color;
use rand::prelude::*;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub cells: Vec<Vec<Cell>>,
    /// `usize` is used (not `u16` as in the spec) because it indexes directly into `cells`
    /// without casts. `Config::board_height/width` are `u16` and are cast at construction.
    pub height: usize,
    pub width: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub content: CellContent,
    pub modifier: Option<TileModifier>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellContent {
    Empty,
    Gem { color: Color, special: Option<SpecialPiece> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpecialPiece {
    LineBomb(Orientation),
    ColorBomb,
    AreaBomb { radius: u8 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TileModifier {
    /// Encases a gem. Breaks after hp adjacent-match hits (or any direct explosion hit).
    Ice { hp: u8 },
    /// Indestructible by normal matches. Destroyed only by a direct explosion landing on this cell.
    Stone,
    /// Takes hp hits from adjacent clears or direct explosions to destroy.
    Crate { hp: u8 },
    /// Gem matches normally but cannot be swapped by the player.
    Locked,
}

// ── Cell helpers ──────────────────────────────────────────────────────────

impl Cell {
    pub fn gem(color: Color) -> Self {
        Self { content: CellContent::Gem { color, special: None }, modifier: None }
    }

    pub fn empty() -> Self {
        Self { content: CellContent::Empty, modifier: None }
    }

    /// Color of the gem in this cell, if any.
    pub fn color(&self) -> Option<Color> {
        match &self.content {
            CellContent::Gem { color, .. } => Some(*color),
            CellContent::Empty => None,
        }
    }

    /// True if the player can pick this cell as part of a swap.
    /// Stone and Locked cells block swapping; empty cells are also not swappable.
    pub fn is_swappable(&self) -> bool {
        match &self.modifier {
            Some(TileModifier::Stone) | Some(TileModifier::Locked) => return false,
            _ => {}
        }
        matches!(self.content, CellContent::Gem { .. })
    }
}

// ── Board ─────────────────────────────────────────────────────────────────

impl Board {
    /// Build a random board with no pre-existing matches.
    pub fn make_random(
        height: usize,
        width: usize,
        palette: &[Color],
        special_tile_pct: u16,
    ) -> Self {
        let mut rng = rand::rng();

        let mut cells: Vec<Vec<Cell>> = (0..height)
            .map(|_| (0..width).map(|_| Cell::gem(*palette.choose(&mut rng).unwrap())).collect())
            .collect();

        // Iteratively fix horizontal and vertical matches until the board is stable.
        // A single pass of vertical fixing can introduce new horizontal matches, so
        // we repeat until neither pass finds anything to fix.
        loop {
            let mut changed = false;

            // Fix horizontal matches
            for r in 0..height {
                for c in 2..width {
                    while cells[r][c].color() == cells[r][c - 1].color()
                        && cells[r][c].color() == cells[r][c - 2].color()
                    {
                        cells[r][c] = Cell::gem(*palette.choose(&mut rng).unwrap());
                        changed = true;
                    }
                }
            }

            // Fix vertical matches
            for r in 2..height {
                for c in 0..width {
                    while cells[r][c].color() == cells[r - 1][c].color()
                        && cells[r][c].color() == cells[r - 2][c].color()
                    {
                        cells[r][c] = Cell::gem(*palette.choose(&mut rng).unwrap());
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        // Place special tile modifiers
        if special_tile_pct > 0 {
            for r in 0..height {
                for c in 0..width {
                    if rng.random_range(0u16..100) < special_tile_pct {
                        let modifier = match rng.random_range(0u8..4) {
                            0 => TileModifier::Ice { hp: 2 },
                            1 => TileModifier::Stone,
                            2 => TileModifier::Crate { hp: 3 },
                            _ => TileModifier::Locked,
                        };
                        cells[r][c].modifier = Some(modifier);
                    }
                }
            }
        }

        Self { cells, height, width }
    }

    /// Swap the *contents* of two cells. Modifiers stay attached to their cells.
    pub fn swap_cells(&mut self, (r1, c1): (usize, usize), (r2, c2): (usize, usize)) {
        let tmp = self.cells[r1][c1].content.clone();
        self.cells[r1][c1].content = self.cells[r2][c2].content.clone();
        self.cells[r2][c2].content = tmp;
    }

    /// Drop all gems downward by compressing each column toward the bottom.
    /// Stone cells act as barriers; gems cannot fall through them.
    /// Returns true if any gem moved.
    pub fn apply_gravity(&mut self) -> bool {
        let mut moved = false;
        for c in 0..self.width {
            // Split the column into segments separated by stone cells.
            // Each segment is processed independently so gems cannot cross stone barriers.
            let mut seg_start = 0;
            while seg_start < self.height {
                // Find the end of this segment (exclusive): stop at a stone or end of column.
                let mut seg_end = seg_start;
                while seg_end < self.height
                    && !matches!(self.cells[seg_end][c].modifier, Some(TileModifier::Stone))
                {
                    seg_end += 1;
                }

                // Process the segment [seg_start, seg_end).
                // Collect gem contents from bottom to top within the segment.
                let mut gems: Vec<CellContent> = Vec::new();
                for r in (seg_start..seg_end).rev() {
                    if matches!(self.cells[r][c].content, CellContent::Gem { .. }) {
                        gems.push(self.cells[r][c].content.clone());
                    }
                }

                // Re-place gems from bottom of segment upward.
                let mut gi = 0;
                for r in (seg_start..seg_end).rev() {
                    let new_content = gems.get(gi).cloned().unwrap_or(CellContent::Empty);
                    if self.cells[r][c].content != new_content {
                        moved = true;
                    }
                    self.cells[r][c].content = new_content;
                    gi += 1;
                }

                // Advance past the stone cell (if any) to the next segment.
                seg_start = seg_end + 1;
            }
        }
        moved
    }

    /// Fill every Empty non-stone cell with a new random gem from the palette.
    pub fn refill_top(&mut self, palette: &[Color]) {
        let mut rng = rand::rng();
        for c in 0..self.width {
            for r in 0..self.height {
                if matches!(self.cells[r][c].modifier, Some(TileModifier::Stone)) {
                    continue;
                }
                if self.cells[r][c].content == CellContent::Empty {
                    self.cells[r][c].content = CellContent::Gem {
                        color: *palette.choose(&mut rng).unwrap(),
                        special: None,
                    };
                }
            }
        }
    }

    /// Count cells whose modifier satisfies `pred`.
    pub fn count_modifier<F: Fn(&TileModifier) -> bool>(&self, pred: F) -> usize {
        self.cells.iter().flatten()
            .filter(|cell| cell.modifier.as_ref().map_or(false, &pred))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;

    fn four_color_palette() -> Vec<Color> {
        vec![Color::Red, Color::Blue, Color::Green, Color::Yellow]
    }

    // ── Initialization ──────────────────────────────────────────────────────

    #[test]
    fn make_random_correct_dimensions() {
        let b = Board::make_random(8, 8, &four_color_palette(), 0);
        assert_eq!(b.height, 8);
        assert_eq!(b.width, 8);
        assert_eq!(b.cells.len(), 8);
        assert!(b.cells.iter().all(|row| row.len() == 8));
    }

    #[test]
    fn make_random_no_pre_existing_horizontal_match() {
        for _ in 0..10 {
            let b = Board::make_random(8, 8, &four_color_palette(), 0);
            for r in 0..8 {
                for c in 0..6 {
                    let (a, b2, c2) = (
                        b.cells[r][c].color(),
                        b.cells[r][c + 1].color(),
                        b.cells[r][c + 2].color(),
                    );
                    assert!(
                        !(a.is_some() && a == b2 && b2 == c2),
                        "Horizontal match at r={r} c={c}"
                    );
                }
            }
        }
    }

    #[test]
    fn make_random_no_pre_existing_vertical_match() {
        for _ in 0..10 {
            let b = Board::make_random(8, 8, &four_color_palette(), 0);
            for r in 0..6 {
                for c in 0..8 {
                    let (a, b2, c2) = (
                        b.cells[r][c].color(),
                        b.cells[r + 1][c].color(),
                        b.cells[r + 2][c].color(),
                    );
                    assert!(
                        !(a.is_some() && a == b2 && b2 == c2),
                        "Vertical match at r={r} c={c}"
                    );
                }
            }
        }
    }

    #[test]
    fn make_random_zero_special_pct_all_gems() {
        let b = Board::make_random(4, 4, &four_color_palette(), 0);
        for r in 0..4 {
            for c in 0..4 {
                assert!(
                    matches!(b.cells[r][c].content, CellContent::Gem { .. }),
                    "Expected Gem at ({r},{c})"
                );
                assert!(b.cells[r][c].modifier.is_none());
            }
        }
    }

    #[test]
    fn make_random_high_special_pct_produces_modifiers() {
        // 100% special_tile_pct → every cell gets a modifier
        let b = Board::make_random(4, 4, &four_color_palette(), 100);
        let with_modifier = b.cells.iter().flatten().filter(|c| c.modifier.is_some()).count();
        assert!(with_modifier > 0, "Expected some cells with modifiers");
    }

    // ── Cell helpers ────────────────────────────────────────────────────────

    #[test]
    fn cell_gem_has_color() {
        let c = Cell::gem(Color::Red);
        assert_eq!(c.color(), Some(Color::Red));
    }

    #[test]
    fn cell_empty_has_no_color() {
        assert_eq!(Cell::empty().color(), None);
    }

    #[test]
    fn is_swappable_normal_gem() {
        assert!(Cell::gem(Color::Red).is_swappable());
    }

    #[test]
    fn is_swappable_empty_false() {
        assert!(!Cell::empty().is_swappable());
    }

    #[test]
    fn is_swappable_stone_false() {
        let mut c = Cell::gem(Color::Red);
        c.modifier = Some(TileModifier::Stone);
        assert!(!c.is_swappable());
    }

    #[test]
    fn is_swappable_locked_false() {
        let mut c = Cell::gem(Color::Red);
        c.modifier = Some(TileModifier::Locked);
        assert!(!c.is_swappable());
    }

    #[test]
    fn is_swappable_ice_gem_still_swappable() {
        let mut c = Cell::gem(Color::Red);
        c.modifier = Some(TileModifier::Ice { hp: 2 });
        // Ice doesn't prevent swapping — it just takes damage when adjacent matches occur
        assert!(c.is_swappable());
    }

    // ── swap_cells ──────────────────────────────────────────────────────────

    #[test]
    fn swap_cells_exchanges_content() {
        let mut b = Board::make_random(4, 4, &four_color_palette(), 0);
        let orig_00 = b.cells[0][0].content.clone();
        let orig_01 = b.cells[0][1].content.clone();
        b.swap_cells((0, 0), (0, 1));
        assert_eq!(b.cells[0][0].content, orig_01);
        assert_eq!(b.cells[0][1].content, orig_00);
    }

    #[test]
    fn swap_cells_leaves_modifiers_in_place() {
        let mut b = Board::make_random(4, 4, &four_color_palette(), 0);
        b.cells[0][0].modifier = Some(TileModifier::Ice { hp: 2 });
        b.cells[0][1].modifier = None;
        b.swap_cells((0, 0), (0, 1));
        // Modifier stays at (0,0) — only content moves
        assert!(matches!(b.cells[0][0].modifier, Some(TileModifier::Ice { .. })));
        assert!(b.cells[0][1].modifier.is_none());
    }

    // ── apply_gravity ───────────────────────────────────────────────────────

    #[test]
    fn gravity_gem_falls_into_empty_below() {
        let mut b = Board {
            cells: vec![
                vec![Cell::gem(Color::Red)],
                vec![Cell::empty()],
            ],
            height: 2,
            width: 1,
        };
        let moved = b.apply_gravity();
        assert!(moved);
        assert_eq!(b.cells[0][0].content, CellContent::Empty);
        assert!(matches!(b.cells[1][0].content, CellContent::Gem { color: Color::Red, .. }));
    }

    #[test]
    fn gravity_settled_board_returns_false() {
        let mut b = Board {
            cells: vec![
                vec![Cell::gem(Color::Red)],
                vec![Cell::gem(Color::Blue)],
            ],
            height: 2,
            width: 1,
        };
        assert!(!b.apply_gravity());
    }

    #[test]
    fn gravity_multiple_gems_stack_correctly() {
        let mut b = Board {
            cells: vec![
                vec![Cell::gem(Color::Red)],
                vec![Cell::empty()],
                vec![Cell::empty()],
            ],
            height: 3,
            width: 1,
        };
        b.apply_gravity();
        assert_eq!(b.cells[0][0].content, CellContent::Empty);
        assert_eq!(b.cells[1][0].content, CellContent::Empty);
        assert!(matches!(b.cells[2][0].content, CellContent::Gem { color: Color::Red, .. }));
    }

    #[test]
    fn gravity_stone_acts_as_barrier() {
        let mut b = Board {
            cells: vec![
                vec![Cell::gem(Color::Red)],
                vec![{
                    let mut c = Cell::empty();
                    c.modifier = Some(TileModifier::Stone);
                    c
                }],
                vec![Cell::empty()],
            ],
            height: 3,
            width: 1,
        };
        b.apply_gravity();
        // Gem cannot pass through stone → stays at row 0 (can't fall into stone)
        // Stone cell remains stone-empty; cell below stone stays empty
        assert!(matches!(b.cells[0][0].content, CellContent::Gem { .. }));
    }

    // ── refill_top ──────────────────────────────────────────────────────────

    #[test]
    fn refill_fills_empty_cells_from_top() {
        let palette = vec![Color::Red];
        let mut b = Board {
            cells: vec![
                vec![Cell::empty(), Cell::empty()],
                vec![Cell::gem(Color::Blue), Cell::gem(Color::Blue)],
            ],
            height: 2,
            width: 2,
        };
        b.refill_top(&palette);
        assert!(matches!(b.cells[0][0].content, CellContent::Gem { color: Color::Red, .. }));
        assert!(matches!(b.cells[0][1].content, CellContent::Gem { color: Color::Red, .. }));
        // Bottom row unchanged
        assert!(matches!(b.cells[1][0].content, CellContent::Gem { color: Color::Blue, .. }));
    }

    #[test]
    fn refill_does_not_fill_stone_cells() {
        let palette = vec![Color::Red];
        let mut b = Board {
            cells: vec![
                vec![{
                    let mut c = Cell::empty();
                    c.modifier = Some(TileModifier::Stone);
                    c
                }],
                vec![Cell::empty()],
            ],
            height: 2,
            width: 1,
        };
        b.refill_top(&palette);
        // Stone cell at row 0: should not be filled
        assert_eq!(b.cells[0][0].content, CellContent::Empty);
        // Row 1 is empty and non-stone: should be filled
        assert!(matches!(b.cells[1][0].content, CellContent::Gem { .. }));
    }
}
