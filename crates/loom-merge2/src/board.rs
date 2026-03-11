use crossterm::style::Color;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::item::Item;

/// A single cell on the merge-2 board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cell {
    Item(Item),
    Empty,
    Generator {
        #[serde(with = "loom_engine::color_serde")]
        color: Color,
        /// Remaining charges. `None` = infinite.
        charges: Option<u16>,
        /// Ticks between spawns.
        interval: u32,
        /// Ticks until next spawn.
        cooldown: u32,
    },
    Blocked,
}

impl Cell {
    pub fn is_item(&self) -> bool { matches!(self, Cell::Item(_)) }
    pub fn is_empty(&self) -> bool { matches!(self, Cell::Empty) }
    pub fn is_generator(&self) -> bool { matches!(self, Cell::Generator { .. }) }
    pub fn is_blocked(&self) -> bool { matches!(self, Cell::Blocked) }

    pub fn item(&self) -> Option<&Item> {
        match self { Cell::Item(i) => Some(i), _ => None }
    }
}

/// The merge-2 game board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub cells: Vec<Vec<Cell>>,
    pub height: usize,
    pub width: usize,
}

impl Board {
    /// Create a random board with generators on edges, blocked cells in the interior,
    /// and the rest filled with tier-1 items.
    pub fn make_random(
        height: usize,
        width: usize,
        palette: &[Color],
        generator_count: usize,
        generator_charges: u16,
        generator_interval: u32,
        blocked_count: usize,
    ) -> Self {
        let mut rng = rand::rng();

        // Start with all empty
        let mut cells = vec![vec![Cell::Empty; width]; height];

        // Collect edge positions for generators
        let mut edge_positions: Vec<(usize, usize)> = Vec::new();
        for r in 0..height {
            for c in 0..width {
                if r == 0 || r == height - 1 || c == 0 || c == width - 1 {
                    edge_positions.push((r, c));
                }
            }
        }
        edge_positions.shuffle(&mut rng);

        // Place generators — ensure at least one per palette color used
        let gen_count = generator_count.min(edge_positions.len());
        for i in 0..gen_count {
            let (r, c) = edge_positions[i];
            let color = palette[i % palette.len()];
            let charges = if generator_charges == 0 { None } else { Some(generator_charges) };
            cells[r][c] = Cell::Generator {
                color,
                charges,
                interval: generator_interval,
                cooldown: generator_interval,
            };
        }

        // Collect interior positions for blocked cells
        let mut interior: Vec<(usize, usize)> = Vec::new();
        for r in 1..height.saturating_sub(1) {
            for c in 1..width.saturating_sub(1) {
                if cells[r][c].is_empty() {
                    interior.push((r, c));
                }
            }
        }
        interior.shuffle(&mut rng);

        let block_count = blocked_count.min(interior.len());
        for i in 0..block_count {
            let (r, c) = interior[i];
            cells[r][c] = Cell::Blocked;
        }

        // Fill remaining empty cells with random tier-1 items
        for r in 0..height {
            for c in 0..width {
                if cells[r][c].is_empty() {
                    let color = *palette.choose(&mut rng).unwrap();
                    cells[r][c] = Cell::Item(Item::new(color, 1));
                }
            }
        }

        Board { cells, height, width }
    }

    /// Get the item at a position, if any.
    pub fn item_at(&self, r: usize, c: usize) -> Option<&Item> {
        self.cells.get(r).and_then(|row| row.get(c)).and_then(|cell| cell.item())
    }

    /// Remove and return the item at a position.
    pub fn take_item(&mut self, r: usize, c: usize) -> Option<Item> {
        if let Some(Cell::Item(item)) = self.cells.get(r).and_then(|row| row.get(c)) {
            let item = item.clone();
            self.cells[r][c] = Cell::Empty;
            Some(item)
        } else {
            None
        }
    }

    /// Whether two positions are 4-directionally adjacent.
    pub fn is_adjacent(a: (usize, usize), b: (usize, usize)) -> bool {
        let dr = (a.0 as i32 - b.0 as i32).abs();
        let dc = (a.1 as i32 - b.1 as i32).abs();
        (dr == 1 && dc == 0) || (dr == 0 && dc == 1)
    }

    /// Whether the items at two positions can merge.
    pub fn can_merge(&self, a: (usize, usize), b: (usize, usize)) -> bool {
        if !Self::is_adjacent(a, b) { return false; }
        match (self.item_at(a.0, a.1), self.item_at(b.0, b.1)) {
            (Some(ia), Some(ib)) => ia.can_merge(ib),
            _ => false,
        }
    }

    /// Perform a merge: place the merged item at `dst`, clear `src`.
    /// Returns the resulting item.
    pub fn do_merge(&mut self, src: (usize, usize), dst: (usize, usize)) -> Option<Item> {
        let src_item = self.item_at(src.0, src.1)?.clone();
        let dst_item = self.item_at(dst.0, dst.1)?;
        if !src_item.can_merge(dst_item) { return None; }
        let merged = src_item.merged();
        self.cells[src.0][src.1] = Cell::Empty;
        self.cells[dst.0][dst.1] = Cell::Item(merged.clone());
        Some(merged)
    }

    /// Whether any valid merge exists on the board.
    pub fn has_any_merge(&self) -> bool {
        for r in 0..self.height {
            for c in 0..self.width {
                if let Some(item) = self.item_at(r, c) {
                    // Check right neighbor
                    if c + 1 < self.width {
                        if let Some(other) = self.item_at(r, c + 1) {
                            if item.can_merge(other) { return true; }
                        }
                    }
                    // Check down neighbor
                    if r + 1 < self.height {
                        if let Some(other) = self.item_at(r + 1, c) {
                            if item.can_merge(other) { return true; }
                        }
                    }
                }
            }
        }
        false
    }

    /// Whether the board has no empty cells (generators and blocked don't count as empty).
    pub fn is_full(&self) -> bool {
        !self.cells.iter().any(|row| row.iter().any(|c| c.is_empty()))
    }

    /// Count empty cells.
    pub fn empty_count(&self) -> usize {
        self.cells.iter().flat_map(|row| row.iter()).filter(|c| c.is_empty()).count()
    }

    /// Tick all generators. Returns true if any item was spawned.
    pub fn tick_generators(&mut self) -> bool {
        self.tick_generators_with_golden(false)
    }

    /// Tick generators. If `golden` is true, 30% chance to spawn T2 instead of T1.
    pub fn tick_generators_with_golden(&mut self, golden: bool) -> bool {
        let mut spawned = false;
        let mut rng = rand::rng();

        // Collect generator info first to avoid borrow issues
        let mut gen_info: Vec<(usize, usize, Color, Option<u16>)> = Vec::new();
        for r in 0..self.height {
            for c in 0..self.width {
                if let Cell::Generator { color, charges, interval: _, cooldown } = &mut self.cells[r][c] {
                    if *cooldown > 0 {
                        *cooldown -= 1;
                        continue;
                    }
                    // Ready to spawn
                    match charges {
                        Some(0) => continue, // exhausted
                        _ => gen_info.push((r, c, *color, *charges)),
                    }
                }
            }
        }

        for (r, c, color, _charges) in gen_info {
            // Find adjacent empty cells
            let neighbors = Self::adjacent_positions(r, c, self.height, self.width);
            let empty: Vec<(usize, usize)> = neighbors.into_iter()
                .filter(|&(nr, nc)| self.cells[nr][nc].is_empty())
                .collect();

            if let Some(&(nr, nc)) = empty.choose(&mut rng) {
                let tier = if golden && rng.random_range(0u8..100) < 30 { 2 } else { 1 };
                self.cells[nr][nc] = Cell::Item(Item::new(color, tier));
                // Update generator state
                if let Cell::Generator { charges, interval, cooldown, .. } = &mut self.cells[r][c] {
                    *cooldown = *interval;
                    if let Some(n) = charges {
                        *n = n.saturating_sub(1);
                    }
                }
                spawned = true;
            }
        }

        spawned
    }

    /// Clear up to `count` random Item cells. Returns how many were cleared.
    pub fn clear_random_items(&mut self, count: usize) -> usize {
        let mut rng = rand::rng();
        let mut item_positions: Vec<(usize, usize)> = Vec::new();
        for r in 0..self.height {
            for c in 0..self.width {
                if self.cells[r][c].is_item() {
                    item_positions.push((r, c));
                }
            }
        }
        item_positions.shuffle(&mut rng);
        let to_clear = count.min(item_positions.len());
        for i in 0..to_clear {
            let (r, c) = item_positions[i];
            self.cells[r][c] = Cell::Empty;
        }
        to_clear
    }

    /// Get 4-directionally adjacent positions within bounds.
    fn adjacent_positions(r: usize, c: usize, height: usize, width: usize) -> Vec<(usize, usize)> {
        let mut pos = Vec::new();
        if r > 0 { pos.push((r - 1, c)); }
        if r + 1 < height { pos.push((r + 1, c)); }
        if c > 0 { pos.push((r, c - 1)); }
        if c + 1 < width { pos.push((r, c + 1)); }
        pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_board() -> Board {
        Board {
            cells: vec![
                vec![Cell::Item(Item::new(Color::Red, 1)), Cell::Item(Item::new(Color::Red, 1))],
                vec![Cell::Item(Item::new(Color::Blue, 1)), Cell::Empty],
            ],
            height: 2, width: 2,
        }
    }

    #[test]
    fn adjacency() {
        assert!(Board::is_adjacent((0, 0), (0, 1)));
        assert!(Board::is_adjacent((0, 0), (1, 0)));
        assert!(!Board::is_adjacent((0, 0), (1, 1)));
        assert!(!Board::is_adjacent((0, 0), (0, 2)));
    }

    #[test]
    fn can_merge_adjacent_same() {
        let board = small_board();
        assert!(board.can_merge((0, 0), (0, 1)));
    }

    #[test]
    fn cannot_merge_different_color() {
        let board = small_board();
        assert!(!board.can_merge((0, 0), (1, 0)));
    }

    #[test]
    fn cannot_merge_non_adjacent() {
        let board = Board {
            cells: vec![
                vec![Cell::Item(Item::new(Color::Red, 1)), Cell::Empty, Cell::Item(Item::new(Color::Red, 1))],
            ],
            height: 1, width: 3,
        };
        assert!(!board.can_merge((0, 0), (0, 2)));
    }

    #[test]
    fn do_merge_works() {
        let mut board = small_board();
        let result = board.do_merge((0, 0), (0, 1));
        assert!(result.is_some());
        let merged = result.unwrap();
        assert_eq!(merged.tier, 2);
        assert_eq!(merged.color, Color::Red);
        assert!(board.cells[0][0].is_empty());
        assert_eq!(board.item_at(0, 1).unwrap().tier, 2);
    }

    #[test]
    fn has_any_merge_true() {
        let board = small_board();
        assert!(board.has_any_merge());
    }

    #[test]
    fn has_any_merge_false() {
        let board = Board {
            cells: vec![
                vec![Cell::Item(Item::new(Color::Red, 1)), Cell::Item(Item::new(Color::Blue, 1))],
                vec![Cell::Item(Item::new(Color::Green, 1)), Cell::Item(Item::new(Color::Yellow, 1))],
            ],
            height: 2, width: 2,
        };
        assert!(!board.has_any_merge());
    }

    #[test]
    fn is_full_when_no_empty() {
        let board = Board {
            cells: vec![
                vec![Cell::Item(Item::new(Color::Red, 1)), Cell::Blocked],
                vec![Cell::Item(Item::new(Color::Blue, 1)), Cell::Item(Item::new(Color::Green, 1))],
            ],
            height: 2, width: 2,
        };
        assert!(board.is_full());
    }

    #[test]
    fn is_full_false_with_empty() {
        let board = small_board();
        assert!(!board.is_full());
    }

    #[test]
    fn take_item_removes_and_returns() {
        let mut board = small_board();
        let item = board.take_item(0, 0);
        assert!(item.is_some());
        assert!(board.cells[0][0].is_empty());
    }

    #[test]
    fn take_item_from_empty_returns_none() {
        let mut board = small_board();
        assert!(board.take_item(1, 1).is_none());
    }

    #[test]
    fn clear_random_items_clears() {
        let mut board = Board {
            cells: vec![
                vec![Cell::Item(Item::new(Color::Red, 1)), Cell::Item(Item::new(Color::Red, 1))],
                vec![Cell::Item(Item::new(Color::Red, 1)), Cell::Item(Item::new(Color::Red, 1))],
            ],
            height: 2, width: 2,
        };
        let cleared = board.clear_random_items(2);
        assert_eq!(cleared, 2);
        assert_eq!(board.empty_count(), 2);
    }

    #[test]
    fn make_random_produces_valid_board() {
        let palette = vec![Color::Red, Color::Blue];
        let board = Board::make_random(4, 4, &palette, 2, 5, 8, 1);
        assert_eq!(board.height, 4);
        assert_eq!(board.width, 4);
        // Should have generators
        let gen_count = board.cells.iter().flat_map(|row| row.iter())
            .filter(|c| c.is_generator()).count();
        assert_eq!(gen_count, 2);
        // Should have blocked cells
        let blocked = board.cells.iter().flat_map(|row| row.iter())
            .filter(|c| c.is_blocked()).count();
        assert_eq!(blocked, 1);
    }

    #[test]
    fn generator_spawns_item() {
        let mut board = Board {
            cells: vec![
                vec![Cell::Generator { color: Color::Red, charges: Some(3), interval: 0, cooldown: 0 }, Cell::Empty],
            ],
            height: 1, width: 2,
        };
        let spawned = board.tick_generators();
        assert!(spawned);
        assert!(board.cells[0][1].is_item());
    }

    #[test]
    fn generator_exhausted_does_not_spawn() {
        let mut board = Board {
            cells: vec![
                vec![Cell::Generator { color: Color::Red, charges: Some(0), interval: 0, cooldown: 0 }, Cell::Empty],
            ],
            height: 1, width: 2,
        };
        let spawned = board.tick_generators();
        assert!(!spawned);
    }

    #[test]
    fn generator_cooldown_delays_spawn() {
        let mut board = Board {
            cells: vec![
                vec![Cell::Generator { color: Color::Red, charges: None, interval: 2, cooldown: 2 }, Cell::Empty],
            ],
            height: 1, width: 2,
        };
        // First tick: cooldown 2 -> 1, no spawn
        assert!(!board.tick_generators());
        // Second tick: cooldown 1 -> 0, no spawn
        assert!(!board.tick_generators());
        // Third tick: cooldown 0 -> spawn
        assert!(board.tick_generators());
    }

    #[test]
    fn serde_roundtrip() {
        let board = small_board();
        let json = serde_json::to_string(&board).unwrap();
        let restored: Board = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.height, board.height);
        assert_eq!(restored.width, board.width);
    }
}
