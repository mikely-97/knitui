// ./src/lib/game_board.rs
use crossterm::style::Color;
use crate::board_entity::{BoardEntity, Direction, ConveyorData};
use crate::color_counter::ColorCounter;
use rand::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct GameBoard {
    pub board: Vec<Vec<BoardEntity>>,
    pub height: u16,
    pub width: u16,
    pub spool_capacity: u16,
}

impl GameBoard {
    pub fn make_random(
        height: u16,
        width: u16,
        selected_palette: &Vec<Color>,
        obstacle_percentage: u16,
        spool_capacity: u16,
        conveyor_percentage: u16,
        conveyor_capacity: u16,
    ) -> Self {
        let mut rng = rand::rng();
        let h = height as usize;
        let w = width as usize;

        // Pass 1: place obstacles and spools
        let mut board: Vec<Vec<BoardEntity>> = Vec::new();
        for _ in 0..h {
            let mut row: Vec<BoardEntity> = Vec::new();
            for _ in 0..w {
                if rng.random_range(0..=100) <= obstacle_percentage {
                    row.push(BoardEntity::Obstacle);
                } else {
                    row.push(BoardEntity::Spool(*selected_palette.choose(&mut rng).unwrap()));
                }
            }
            board.push(row);
        }

        // Pass 2: convert some spools to conveyors
        if conveyor_percentage > 0 && conveyor_capacity > 0 {
            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];
            for r in 0..h {
                for c in 0..w {
                    if !matches!(board[r][c], BoardEntity::Spool(_)) {
                        continue;
                    }
                    if rng.random_range(0..=100) > conveyor_percentage {
                        continue;
                    }
                    // Collect valid output directions (adjacent Spool cells)
                    let valid_dirs: Vec<Direction> = directions.iter().copied().filter(|d| {
                        let (dr, dc) = d.offset();
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32
                            && matches!(board[nr as usize][nc as usize], BoardEntity::Spool(_))
                    }).collect();
                    if let Some(&dir) = valid_dirs.choose(&mut rng) {
                        let queue: Vec<Color> = (0..conveyor_capacity)
                            .map(|_| *selected_palette.choose(&mut rng).unwrap())
                            .collect();
                        let color = queue[0];
                        board[r][c] = BoardEntity::Conveyor(ConveyorData {
                            color,
                            output_dir: dir,
                            queue,
                        });
                    }
                }
            }
        }

        // Pass 3: revert any conveyor whose output cell is itself a conveyor.
        // This can happen when Pass 2 converts cell A (pointing at B) and then
        // also converts B to a conveyor — leaving A permanently blocked.
        let mut to_revert: Vec<(usize, usize)> = Vec::new();
        for r in 0..h {
            for c in 0..w {
                if let BoardEntity::Conveyor(ref data) = board[r][c] {
                    let (dr, dc) = data.output_dir.offset();
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                        if matches!(board[nr as usize][nc as usize], BoardEntity::Conveyor(_)) {
                            to_revert.push((r, c));
                        }
                    }
                }
            }
        }
        for (r, c) in to_revert {
            board[r][c] = BoardEntity::Spool(*selected_palette.choose(&mut rng).unwrap());
        }

        Self { board, height, width, spool_capacity }
    }

    /// Returns a ColorCounter representing the total yarn stitches needed to
    /// complete the board: each Spool/KeySpool contributes `spool_capacity`
    /// stitches, and each Conveyor contributes `spool_capacity` stitches per
    /// entry in its queue.
    pub fn count_spools(&self) -> ColorCounter {
        let mut counter = HashMap::new();
        for row in &self.board {
            for cell in row {
                match cell {
                    BoardEntity::Spool(color) | BoardEntity::KeySpool(color) => {
                        counter
                            .entry(*color)
                            .and_modify(|e| *e += self.spool_capacity)
                            .or_insert(self.spool_capacity);
                    }
                    BoardEntity::Conveyor(data) => {
                        for color in &data.queue {
                            counter
                                .entry(*color)
                                .and_modify(|e| *e += self.spool_capacity)
                                .or_insert(self.spool_capacity);
                        }
                    }
                    BoardEntity::Obstacle | BoardEntity::Void | BoardEntity::EmptyConveyor => {}
                }
            }
        }
        ColorCounter { color_hashmap: counter }
    }

    /// Compute the set of Void cells reachable from the top row via
    /// orthogonal adjacency through other Void cells.  Row-0 Void cells
    /// are the seeds; any Void that can be reached from them is
    /// "surface-connected."  Top-row spools/obstacles are NOT seeds
    /// (only actual Void cells propagate connectivity).
    pub fn surface_connected_voids(&self) -> HashSet<(usize, usize)> {
        let h = self.height as usize;
        let w = self.width as usize;
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Seed: all Void cells in row 0
        for c in 0..w {
            if matches!(self.board[0][c], BoardEntity::Void) {
                visited.insert((0, c));
                queue.push_back((0, c));
            }
        }

        while let Some((r, c)) = queue.pop_front() {
            for (nr, nc) in Self::neighbors(r, c, h, w) {
                if !visited.contains(&(nr, nc))
                    && matches!(self.board[nr][nc], BoardEntity::Void)
                {
                    visited.insert((nr, nc));
                    queue.push_back((nr, nc));
                }
            }
        }

        visited
    }

    fn neighbors(r: usize, c: usize, h: usize, w: usize) -> Vec<(usize, usize)> {
        let mut n = Vec::with_capacity(4);
        if r > 0     { n.push((r - 1, c)); }
        if r + 1 < h { n.push((r + 1, c)); }
        if c > 0     { n.push((r, c - 1)); }
        if c + 1 < w { n.push((r, c + 1)); }
        n
    }

    /// A cell is focusable (cursor can land on it) when it is actionable.
    /// Spools / KeySpools must pass `is_selectable`.
    /// Obstacles and empty conveyors are never focusable.
    pub fn is_focusable(&self, row: usize, col: usize) -> bool {
        match &self.board[row][col] {
            BoardEntity::Spool(_) | BoardEntity::KeySpool(_) => self.is_selectable(row, col),
            BoardEntity::Void => true,
            _ => false,
        }
    }

    /// Returns true if at least one Spool or KeySpool on the board is selectable.
    pub fn has_selectable_spool(&self) -> bool {
        let connected = self.surface_connected_voids();
        for row in 0..self.height as usize {
            for col in 0..self.width as usize {
                if self.is_selectable_with(row, col, &connected) {
                    return true;
                }
            }
        }
        false
    }

    /// A cell is selectable when:
    ///   - it contains a Spool or KeySpool, AND
    ///   - it is in the top row (row 0), OR at least one orthogonal
    ///     neighbor is a *surface-connected* Void.
    ///
    /// This prevents tweezers-created isolated voids from granting
    /// selectability to buried spools.
    pub fn is_selectable(&self, row: usize, col: usize) -> bool {
        let connected = self.surface_connected_voids();
        self.is_selectable_with(row, col, &connected)
    }

    /// Like `is_selectable` but accepts a pre-computed connectivity set
    /// to avoid redundant BFS when checking many cells.
    pub fn is_selectable_with(
        &self,
        row: usize,
        col: usize,
        connected_voids: &HashSet<(usize, usize)>,
    ) -> bool {
        match &self.board[row][col] {
            BoardEntity::Spool(_) | BoardEntity::KeySpool(_) => {}
            _ => return false,
        }
        if row == 0 {
            return true;
        }
        let h = self.height as usize;
        let w = self.width as usize;

        for (nr, nc) in Self::neighbors(row, col, h, w) {
            if connected_voids.contains(&(nr, nc)) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_board_dimensions() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(5, 7, &palette, 0, 3, 0, 0);

        assert_eq!(board.height, 5);
        assert_eq!(board.width, 7);
        assert_eq!(board.board.len(), 5);
        assert_eq!(board.board[0].len(), 7);
    }

    #[test]
    fn test_game_board_no_obstacles() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(4, 4, &palette, 0, 2, 0, 0);

        let mut spool_count = 0;
        let mut obstacle_count = 0;
        for row in &board.board {
            for entity in row {
                match entity {
                    BoardEntity::Spool(_) => spool_count += 1,
                    BoardEntity::Obstacle  => obstacle_count += 1,
                    _ => {}
                }
            }
        }
        assert!(spool_count >= 15);
        assert!(obstacle_count <= 1);
    }

    #[test]
    fn test_game_board_all_obstacles() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 100, 1, 0, 0);

        let mut obstacle_count = 0;
        for row in &board.board {
            for entity in row {
                match entity {
                    BoardEntity::Obstacle => obstacle_count += 1,
                    _ => {}
                }
            }
        }
        assert_eq!(obstacle_count, 9);
    }

    #[test]
    fn test_count_spools_empty_board() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 100, 5, 0, 0);
        let counter = board.count_spools();

        assert_eq!(counter.color_hashmap.len(), 0);
    }

    #[test]
    fn test_count_spools_multiplies_by_spool_capacity() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 0, 3, 0, 0);
        let counter = board.count_spools();

        // 4 spools of red * 3 spool_capacity = 12
        assert_eq!(*counter.color_hashmap.get(&Color::Red).unwrap(), 12);
    }

    #[test]
    fn test_count_spools_different_colors() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(5, 5, &palette, 0, 2, 0, 0);
        let counter = board.count_spools();

        assert!(counter.color_hashmap.len() >= 1);
        assert!(counter.color_hashmap.len() <= 3);

        let total: u16 = counter.color_hashmap.values().sum();
        assert!(total >= 48 && total <= 50);
    }

    #[test]
    fn test_spool_capacity_stored() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 0, 7, 0, 0);
        assert_eq!(board.spool_capacity, 7);
    }

    #[test]
    fn test_is_selectable_top_row_spool() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 0, 1, 0, 0);
        // Top-row spools are always selectable.
        for c in 0..3 {
            if matches!(board.board[0][c], BoardEntity::Spool(_)) {
                assert!(board.is_selectable(0, c));
            }
        }
    }

    #[test]
    fn test_is_selectable_obstacle_never() {
        let palette = vec![Color::Red];
        // 100% obstacles
        let board = GameBoard::make_random(3, 3, &palette, 100, 1, 0, 0);
        for r in 0..3 {
            for c in 0..3 {
                assert!(!board.is_selectable(r, c));
            }
        }
    }

    #[test]
    fn test_is_selectable_void_neighbor() {
        use crate::board_entity::BoardEntity;
        // Build a manual board: row 0 = Void, row 1 = Spool
        // The Spool at (1,0) borders a Void above → selectable.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Void],
                vec![BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Spool(Color::Blue)],
            ],
            height: 3,
            width: 1,
            spool_capacity: 1,
        };
        assert!(board.is_selectable(1, 0));  // borders Void above
        assert!(!board.is_selectable(2, 0)); // no Void neighbor
    }

    #[test]
    fn test_is_focusable_buried_spool() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Spool(Color::Blue)],
                vec![BoardEntity::Spool(Color::Green)],
            ],
            height: 3,
            width: 1,
            spool_capacity: 1,
        };
        assert!(board.is_focusable(0, 0));   // top row → selectable → focusable
        assert!(!board.is_focusable(1, 0));  // buried spool → not focusable
        assert!(!board.is_focusable(2, 0));  // buried spool → not focusable
    }

    #[test]
    fn test_is_focusable_non_spool_types() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle, BoardEntity::Void],
            ],
            height: 1,
            width: 2,
            spool_capacity: 1,
        };
        assert!(!board.is_focusable(0, 0)); // obstacle → NOT focusable
        assert!(board.is_focusable(0, 1));  // void → focusable
    }

    #[test]
    fn test_is_focusable_conveyor_not_focusable() {
        use crate::board_entity::Direction;
        let data = ConveyorData {
            color: Color::Red,
            output_dir: Direction::Right,
            queue: vec![Color::Red],
        };
        let board = GameBoard {
            board: vec![vec![
                BoardEntity::Conveyor(data),
                BoardEntity::EmptyConveyor,
                BoardEntity::Spool(Color::Blue),
            ]],
            height: 1,
            width: 3,
            spool_capacity: 1,
        };
        assert!(!board.is_focusable(0, 0)); // active conveyor → NOT focusable
        assert!(!board.is_focusable(0, 1)); // empty conveyor → NOT focusable
        assert!(board.is_focusable(0, 2));  // top-row spool → focusable
    }

    #[test]
    fn has_selectable_spool_true_when_exposed() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red), BoardEntity::Void],
                vec![BoardEntity::Spool(Color::Blue), BoardEntity::Obstacle],
            ],
            height: 2, width: 2, spool_capacity: 1,
        };
        assert!(board.has_selectable_spool());
    }

    #[test]
    fn has_selectable_spool_false_when_all_buried() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle, BoardEntity::Obstacle],
                vec![BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue)],
                vec![BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue)],
            ],
            height: 3, width: 2, spool_capacity: 1,
        };
        assert!(!board.has_selectable_spool());
    }

    #[test]
    fn has_selectable_spool_false_when_no_spools() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Void, BoardEntity::Obstacle],
            ],
            height: 1, width: 2, spool_capacity: 1,
        };
        assert!(!board.has_selectable_spool());
    }

    #[test]
    fn test_isolated_void_does_not_grant_selectability() {
        // An isolated void (not connected to top row) should NOT make
        // neighboring spools selectable.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red)],   // row 0: spool, no void
                vec![BoardEntity::Spool(Color::Blue)],  // row 1: spool
                vec![BoardEntity::Void],                 // row 2: void, but not connected to top
                vec![BoardEntity::Spool(Color::Green)], // row 3: has void above but isolated
            ],
            height: 4,
            width: 1,
            spool_capacity: 1,
        };
        // Row 0 is selectable (top row)
        assert!(board.is_selectable(0, 0));
        // Row 1 has no surface-connected void neighbor
        assert!(!board.is_selectable(1, 0));
        // Row 2 is void, not a spool
        assert!(!board.is_selectable(2, 0));
        // Row 3 borders a void at (2,0), but that void is NOT connected to top → not selectable
        assert!(!board.is_selectable(3, 0));
    }

    #[test]
    fn test_connected_void_chain_grants_selectability() {
        // A chain of voids from top row should still grant selectability.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Void],                 // row 0: void (seed)
                vec![BoardEntity::Void],                 // row 1: void connected to seed
                vec![BoardEntity::Spool(Color::Green)], // row 2: has connected void above
            ],
            height: 3,
            width: 1,
            spool_capacity: 1,
        };
        // Row 2 borders void at (1,0), which connects to (0,0) → selectable
        assert!(board.is_selectable(2, 0));
    }

    #[test]
    fn test_surface_connected_voids_computation() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Void,                BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Void,                BoardEntity::Spool(Color::Blue)],
                vec![BoardEntity::Spool(Color::Green), BoardEntity::Void],
            ],
            height: 3,
            width: 2,
            spool_capacity: 1,
        };
        let connected = board.surface_connected_voids();
        // (0,0) is a void in row 0 → connected
        assert!(connected.contains(&(0, 0)));
        // (1,0) is void adjacent to (0,0) → connected
        assert!(connected.contains(&(1, 0)));
        // (2,1) is void but not connected to any top-row void → NOT connected
        assert!(!connected.contains(&(2, 1)));
    }

    #[test]
    fn test_conveyors_spawn_with_high_percentage() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        // 100% conveyor chance on a large board → should produce at least one
        let board = GameBoard::make_random(6, 6, &palette, 0, 1, 100, 3);
        let gen_count = board.board.iter().flatten().filter(|e| {
            matches!(e, BoardEntity::Conveyor(_))
        }).count();
        assert!(gen_count > 0, "expected conveyors on board with 100% conveyor_percentage");
    }

    #[test]
    fn test_conveyor_output_points_to_valid_cell() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(8, 8, &palette, 0, 1, 50, 2);
        let h = board.height as i32;
        let w = board.width as i32;
        for r in 0..board.height as usize {
            for c in 0..board.width as usize {
                if let BoardEntity::Conveyor(ref data) = board.board[r][c] {
                    let (dr, dc) = data.output_dir.offset();
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    assert!(nr >= 0 && nr < h && nc >= 0 && nc < w,
                        "conveyor at ({r},{c}) output direction points out of bounds");
                }
            }
        }
    }

    #[test]
    fn test_no_conveyors_when_percentage_zero() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(5, 5, &palette, 0, 1, 0, 3);
        let gen_count = board.board.iter().flatten().filter(|e| {
            matches!(e, BoardEntity::Conveyor(_))
        }).count();
        assert_eq!(gen_count, 0);
    }
}
