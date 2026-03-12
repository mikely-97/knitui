use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::item::{Family, Item, Piece};

/// A single cell on the merge-2 board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Piece(Piece),
    /// Frozen cell: content is visible but locked. Must merge identical piece into it to thaw.
    Frozen(Piece),
    HardGenerator {
        family: Family,
        /// Generator tier (1-8). Higher tiers produce higher-base-tier items.
        tier: u8,
        cooldown_remaining: u32,
    },
    SoftGenerator {
        family: Family,
        /// Generator tier (1-8).
        tier: u8,
        charges: u16,
        cooldown_remaining: u32,
    },
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }
    pub fn is_piece(&self) -> bool {
        matches!(self, Cell::Piece(_))
    }
    pub fn is_frozen(&self) -> bool {
        matches!(self, Cell::Frozen(_))
    }
    pub fn is_hard_generator(&self) -> bool {
        matches!(self, Cell::HardGenerator { .. })
    }
    pub fn is_soft_generator(&self) -> bool {
        matches!(self, Cell::SoftGenerator { .. })
    }
    pub fn is_any_generator(&self) -> bool {
        self.is_hard_generator() || self.is_soft_generator()
    }

    /// Get the piece in this cell (if it's a Piece or Frozen cell).
    pub fn piece(&self) -> Option<&Piece> {
        match self {
            Cell::Piece(p) | Cell::Frozen(p) => Some(p),
            _ => None,
        }
    }

    /// Get the family of whatever is in this cell.
    pub fn family(&self) -> Option<Family> {
        match self {
            Cell::Piece(p) | Cell::Frozen(p) => Some(p.family()),
            Cell::HardGenerator { family, .. } | Cell::SoftGenerator { family, .. } => {
                Some(*family)
            }
            Cell::Empty => None,
        }
    }
}

/// The merge-2 game board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub cells: Vec<Vec<Cell>>,
    pub rows: usize,
    pub cols: usize,
}

impl Board {
    pub fn new_empty(rows: usize, cols: usize) -> Self {
        Board {
            cells: vec![vec![Cell::Empty; cols]; rows],
            rows,
            cols,
        }
    }

    pub fn in_bounds(&self, r: usize, c: usize) -> bool {
        r < self.rows && c < self.cols
    }

    pub fn cell(&self, r: usize, c: usize) -> &Cell {
        &self.cells[r][c]
    }

    pub fn cell_mut(&mut self, r: usize, c: usize) -> &mut Cell {
        &mut self.cells[r][c]
    }

    /// Get the piece at a position (from Piece or Frozen cell).
    pub fn piece_at(&self, r: usize, c: usize) -> Option<&Piece> {
        self.cells.get(r).and_then(|row| row.get(c)).and_then(|cell| cell.piece())
    }

    /// Get a free (non-frozen) piece at a position.
    pub fn free_piece_at(&self, r: usize, c: usize) -> Option<&Piece> {
        match self.cells.get(r).and_then(|row| row.get(c)) {
            Some(Cell::Piece(p)) => Some(p),
            _ => None,
        }
    }

    /// Remove and return the piece at a free (non-frozen) position.
    pub fn take_piece(&mut self, r: usize, c: usize) -> Option<Piece> {
        if let Cell::Piece(p) = &self.cells[r][c] {
            let piece = p.clone();
            self.cells[r][c] = Cell::Empty;
            Some(piece)
        } else {
            None
        }
    }

    /// Whether two positions can merge.
    /// No adjacency requirement — any matching pair anywhere on the board.
    /// dst can be a Frozen cell if it contains a matching piece.
    pub fn can_merge(&self, src: (usize, usize), dst: (usize, usize)) -> bool {
        if src == dst {
            return false;
        }
        let src_piece = match &self.cells[src.0][src.1] {
            Cell::Piece(p) => p,
            _ => return false,
        };
        let dst_piece = match &self.cells[dst.0][dst.1] {
            Cell::Piece(p) | Cell::Frozen(p) => p,
            _ => return false,
        };
        src_piece.can_merge(dst_piece)
    }

    /// Perform a merge: remove src piece, merge into dst.
    /// If dst is Frozen, thaw it.
    /// Returns the resulting piece and whether a thaw occurred.
    pub fn do_merge(&mut self, src: (usize, usize), dst: (usize, usize)) -> Option<MergeResult> {
        if !self.can_merge(src, dst) {
            return None;
        }

        let was_frozen = self.cells[dst.0][dst.1].is_frozen();

        // Determine result piece
        let src_piece = match &self.cells[src.0][src.1] {
            Cell::Piece(p) => p.clone(),
            _ => return None,
        };
        let dst_piece = match &self.cells[dst.0][dst.1] {
            Cell::Piece(p) | Cell::Frozen(p) => p.clone(),
            _ => return None,
        };

        let result_piece = match (&src_piece, &dst_piece) {
            (Piece::Regular(a), Piece::Regular(_b)) => Piece::Regular(a.merged()),
            // Two blueprints of same family merge → result handled by engine (becomes generator)
            (Piece::Blueprint(fam), Piece::Blueprint(_)) => Piece::Blueprint(*fam),
            _ => return None,
        };

        // Clear src, place result at dst
        self.cells[src.0][src.1] = Cell::Empty;
        self.cells[dst.0][dst.1] = Cell::Piece(result_piece.clone());

        Some(MergeResult {
            piece: result_piece,
            thawed: was_frozen,
            dst,
        })
    }

    /// Whether any valid merge exists on the board.
    /// Checks all pairs of free pieces + free→frozen merges.
    pub fn has_any_merge(&self) -> bool {
        let mut free_pieces: Vec<((usize, usize), &Piece)> = Vec::new();
        let mut all_pieces: Vec<((usize, usize), &Piece)> = Vec::new();

        for r in 0..self.rows {
            for c in 0..self.cols {
                match &self.cells[r][c] {
                    Cell::Piece(p) => {
                        free_pieces.push(((r, c), p));
                        all_pieces.push(((r, c), p));
                    }
                    Cell::Frozen(p) => {
                        all_pieces.push(((r, c), p));
                    }
                    _ => {}
                }
            }
        }

        // For each free piece, check if any other piece (free or frozen) matches
        for (i, (_pos_a, piece_a)) in free_pieces.iter().enumerate() {
            for (_pos_b, piece_b) in all_pieces.iter() {
                if std::ptr::eq(*piece_a, *piece_b) {
                    continue;
                }
                if piece_a.can_merge(piece_b) {
                    // Need at least the src to be free, dst can be free or frozen
                    // But if dst is also free, we need src != dst (already guaranteed by ptr check)
                    // If dst is frozen, this is a valid thaw-merge
                    return true;
                }
            }
            // Also check against other free pieces (for the i+1.. range to avoid double-count)
            for (_pos_b, piece_b) in free_pieces.iter().skip(i + 1) {
                if piece_a.can_merge(piece_b) {
                    return true;
                }
            }
        }

        false
    }

    /// Whether the board has no empty cells.
    pub fn is_full(&self) -> bool {
        !self.cells.iter().any(|row| row.iter().any(|c| c.is_empty()))
    }

    /// Count empty cells.
    pub fn empty_count(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|c| c.is_empty())
            .count()
    }

    /// Count frozen cells.
    pub fn frozen_count(&self) -> usize {
        self.cells
            .iter()
            .flat_map(|row| row.iter())
            .filter(|c| c.is_frozen())
            .count()
    }

    /// Get 4-directionally adjacent positions within bounds.
    pub fn adjacent_positions(&self, r: usize, c: usize) -> Vec<(usize, usize)> {
        let mut pos = Vec::with_capacity(4);
        if r > 0 {
            pos.push((r - 1, c));
        }
        if r + 1 < self.rows {
            pos.push((r + 1, c));
        }
        if c > 0 {
            pos.push((r, c - 1));
        }
        if c + 1 < self.cols {
            pos.push((r, c + 1));
        }
        pos
    }

    /// Find a random empty cell anywhere on the board. Returns None if the board is full.
    pub fn find_empty_adjacent(&self, _r: usize, _c: usize) -> Option<(usize, usize)> {
        self.find_any_empty()
    }

    /// Find a random empty cell anywhere on the board.
    pub fn find_any_empty(&self) -> Option<(usize, usize)> {
        let mut rng = rand::rng();
        let mut empties: Vec<(usize, usize)> = (0..self.rows)
            .flat_map(|r| (0..self.cols).map(move |c| (r, c)))
            .filter(|&(r, c)| self.cells[r][c].is_empty())
            .collect();
        empties.shuffle(&mut rng);
        empties.into_iter().next()
    }

    /// Thaw adjacent frozen cells (for thaw_aura blessing).
    /// Returns the positions that were thawed.
    pub fn thaw_adjacent(&mut self, r: usize, c: usize) -> Vec<(usize, usize)> {
        let neighbors = self.adjacent_positions(r, c);
        let mut thawed = Vec::new();
        for (nr, nc) in neighbors {
            if let Cell::Frozen(piece) = &self.cells[nr][nc] {
                let piece = piece.clone();
                self.cells[nr][nc] = Cell::Piece(piece);
                thawed.push((nr, nc));
            }
        }
        thawed
    }

    /// Clear up to `count` random free piece cells. Returns how many were cleared.
    pub fn clear_random_pieces(&mut self, count: usize) -> usize {
        let mut rng = rand::rng();
        let mut positions: Vec<(usize, usize)> = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.cells[r][c].is_piece() {
                    positions.push((r, c));
                }
            }
        }
        positions.shuffle(&mut rng);
        let to_clear = count.min(positions.len());
        for &(r, c) in &positions[..to_clear] {
            self.cells[r][c] = Cell::Empty;
        }
        to_clear
    }
}

/// Result of a successful merge.
#[derive(Clone, Debug)]
pub struct MergeResult {
    pub piece: Piece,
    pub thawed: bool,
    pub dst: (usize, usize),
}

/// Initial cell state for board layout definitions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CellInit {
    Empty,
    Frozen,
    FrozenItem(Family, u8),
    FrozenBlueprint(Family),
    FrozenHardGen(Family),
    HardGenerator(Family, u8),  // (family, tier)
    Item(Family, u8),
}

/// A board layout definition for campaign tracks.
#[derive(Clone, Debug)]
pub struct BoardLayout {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Vec<CellInit>>,
}

impl BoardLayout {
    /// Build a Board from this layout definition.
    pub fn build(&self) -> Board {
        let mut board = Board::new_empty(self.rows, self.cols);
        for r in 0..self.rows.min(self.cells.len()) {
            for c in 0..self.cols.min(self.cells[r].len()) {
                board.cells[r][c] = match &self.cells[r][c] {
                    CellInit::Empty => Cell::Empty,
                    CellInit::Frozen => Cell::Frozen(Piece::Regular(Item::new(Family::Wood, 1))),
                    CellInit::FrozenItem(fam, tier) => {
                        Cell::Frozen(Piece::Regular(Item::new(*fam, *tier)))
                    }
                    CellInit::FrozenBlueprint(fam) => Cell::Frozen(Piece::Blueprint(*fam)),
                    CellInit::FrozenHardGen(fam) => {
                        // Frozen hard gen: when thawed, becomes a hard generator
                        // We represent as frozen piece; engine handles special conversion
                        Cell::Frozen(Piece::Blueprint(*fam))
                    }
                    CellInit::HardGenerator(fam, tier) => Cell::HardGenerator {
                        family: *fam,
                        tier: *tier,
                        cooldown_remaining: 0,
                    },
                    CellInit::Item(fam, tier) => {
                        Cell::Piece(Piece::Regular(Item::new(*fam, *tier)))
                    }
                };
            }
        }
        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::MAX_TIER;

    fn test_board() -> Board {
        let mut board = Board::new_empty(3, 3);
        board.cells[0][0] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[0][1] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[0][2] = Cell::Piece(Piece::Regular(Item::new(Family::Stone, 1)));
        board.cells[1][0] = Cell::Frozen(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[2][2] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 1,
            cooldown_remaining: 0,
        };
        board
    }

    #[test]
    fn can_merge_non_adjacent_same_type() {
        let mut board = Board::new_empty(3, 3);
        board.cells[0][0] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[2][2] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        // Non-adjacent but same type — should merge
        assert!(board.can_merge((0, 0), (2, 2)));
    }

    #[test]
    fn cannot_merge_different_family() {
        let board = test_board();
        assert!(!board.can_merge((0, 0), (0, 2)));
    }

    #[test]
    fn can_merge_into_frozen() {
        let board = test_board();
        // (0,0) is free Wood T1, (1,0) is frozen Wood T1
        assert!(board.can_merge((0, 0), (1, 0)));
    }

    #[test]
    fn cannot_merge_from_frozen() {
        let board = test_board();
        // src must be free
        assert!(!board.can_merge((1, 0), (0, 0)));
    }

    #[test]
    fn do_merge_thaws_frozen() {
        let mut board = test_board();
        let result = board.do_merge((0, 0), (1, 0));
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.thawed);
        assert!(board.cells[0][0].is_empty());
        assert!(board.cells[1][0].is_piece());
        if let Cell::Piece(Piece::Regular(item)) = &board.cells[1][0] {
            assert_eq!(item.tier, 2);
            assert_eq!(item.family, Family::Wood);
        } else {
            panic!("Expected regular piece after merge");
        }
    }

    #[test]
    fn do_merge_regular() {
        let mut board = test_board();
        let result = board.do_merge((0, 0), (0, 1));
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(!result.thawed);
        assert!(board.cells[0][0].is_empty());
    }

    #[test]
    fn cannot_merge_self() {
        let board = test_board();
        assert!(!board.can_merge((0, 0), (0, 0)));
    }

    #[test]
    fn has_any_merge_finds_non_adjacent() {
        let mut board = Board::new_empty(3, 3);
        board.cells[0][0] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[2][2] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        assert!(board.has_any_merge());
    }

    #[test]
    fn has_any_merge_finds_frozen_target() {
        let board = test_board();
        assert!(board.has_any_merge());
    }

    #[test]
    fn has_any_merge_false_no_matches() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[0][1] = Cell::Piece(Piece::Regular(Item::new(Family::Stone, 1)));
        board.cells[1][0] = Cell::Piece(Piece::Regular(Item::new(Family::Metal, 1)));
        board.cells[1][1] = Cell::Piece(Piece::Regular(Item::new(Family::Cloth, 1)));
        assert!(!board.has_any_merge());
    }

    #[test]
    fn blueprint_merge() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::Piece(Piece::Blueprint(Family::Metal));
        board.cells[1][1] = Cell::Piece(Piece::Blueprint(Family::Metal));
        assert!(board.can_merge((0, 0), (1, 1)));
        let result = board.do_merge((0, 0), (1, 1));
        assert!(result.is_some());
    }

    #[test]
    fn board_layout_build() {
        let layout = BoardLayout {
            rows: 2,
            cols: 2,
            cells: vec![
                vec![CellInit::HardGenerator(Family::Wood, 1), CellInit::Empty],
                vec![
                    CellInit::FrozenItem(Family::Stone, 2),
                    CellInit::Item(Family::Metal, 1),
                ],
            ],
        };
        let board = layout.build();
        assert!(board.cells[0][0].is_hard_generator());
        assert!(board.cells[0][1].is_empty());
        assert!(board.cells[1][0].is_frozen());
        assert!(board.cells[1][1].is_piece());
    }

    #[test]
    fn frozen_count() {
        let board = test_board();
        assert_eq!(board.frozen_count(), 1);
    }

    #[test]
    fn thaw_adjacent() {
        let mut board = Board::new_empty(3, 3);
        board.cells[1][1] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[0][1] = Cell::Frozen(Piece::Regular(Item::new(Family::Stone, 1)));
        board.cells[1][0] = Cell::Frozen(Piece::Regular(Item::new(Family::Metal, 1)));
        let thawed = board.thaw_adjacent(1, 1);
        assert_eq!(thawed.len(), 2);
        assert!(board.cells[0][1].is_piece());
        assert!(board.cells[1][0].is_piece());
    }

    #[test]
    fn serde_roundtrip() {
        let board = test_board();
        let json = serde_json::to_string(&board).unwrap();
        let restored: Board = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.rows, board.rows);
        assert_eq!(restored.cols, board.cols);
    }
}
