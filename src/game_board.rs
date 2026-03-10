// ./src/lib/game_board.rs
use crossterm::style::Color;
use crate::board_entity::BoardEntity;
use crate::color_counter::ColorCounter;
use rand::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct GameBoard {
    pub board: Vec<Vec<BoardEntity>>,
    pub height: u16,
    pub width: u16,
    pub knit_volume: u16,
}

impl GameBoard {
    pub fn make_random(
        height: u16,
        width: u16,
        selected_palette: &Vec<Color>,
        obstacle_percentage: u16,
        knit_volume: u16,
    ) -> Self {
        let mut rng = rand::rng();
        let mut board: Vec<Vec<BoardEntity>> = Vec::new();
        for _ in 0..height {
            let mut row: Vec<BoardEntity> = Vec::new();
            for _ in 0..width {
                if rng.random_range(0..=100) <= obstacle_percentage {
                    row.push(BoardEntity::Obstacle);
                } else {
                    row.push(BoardEntity::Thread(*selected_palette.choose(&mut rng).unwrap()));
                }
            }
            board.push(row);
        }
        Self { board, height, width, knit_volume }
    }

    /// Returns a ColorCounter representing the total yarn patches needed to
    /// complete the board: each Thread/KeyThread contributes `knit_volume`
    /// patches, and each Generator contributes `knit_volume` patches per
    /// entry in its queue.
    pub fn count_knits(&self) -> ColorCounter {
        let mut counter = HashMap::new();
        for row in &self.board {
            for cell in row {
                match cell {
                    BoardEntity::Thread(color) | BoardEntity::KeyThread(color) => {
                        counter
                            .entry(*color)
                            .and_modify(|e| *e += self.knit_volume)
                            .or_insert(self.knit_volume);
                    }
                    BoardEntity::Generator(data) => {
                        for color in &data.queue {
                            counter
                                .entry(*color)
                                .and_modify(|e| *e += self.knit_volume)
                                .or_insert(self.knit_volume);
                        }
                    }
                    BoardEntity::Obstacle | BoardEntity::Void | BoardEntity::DepletedGenerator => {}
                }
            }
        }
        ColorCounter { color_hashmap: counter }
    }

    /// Compute the set of Void cells reachable from the top row via
    /// orthogonal adjacency through other Void cells.  Row-0 Void cells
    /// are the seeds; any Void that can be reached from them is
    /// "surface-connected."  Top-row threads/obstacles are NOT seeds
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
    /// Threads / KeyThreads must pass `is_selectable`.
    /// Obstacles and depleted generators are never focusable.
    pub fn is_focusable(&self, row: usize, col: usize) -> bool {
        match &self.board[row][col] {
            BoardEntity::Thread(_) | BoardEntity::KeyThread(_) => self.is_selectable(row, col),
            BoardEntity::Obstacle | BoardEntity::DepletedGenerator => false,
            _ => true,
        }
    }

    /// Returns true if at least one Thread or KeyThread on the board is selectable.
    pub fn has_selectable_thread(&self) -> bool {
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
    ///   - it contains a Thread or KeyThread, AND
    ///   - it is in the top row (row 0), OR at least one orthogonal
    ///     neighbor is a *surface-connected* Void.
    ///
    /// This prevents tweezers-created isolated voids from granting
    /// selectability to buried threads.
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
            BoardEntity::Thread(_) | BoardEntity::KeyThread(_) => {}
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
        let board = GameBoard::make_random(5, 7, &palette, 0, 3);

        assert_eq!(board.height, 5);
        assert_eq!(board.width, 7);
        assert_eq!(board.board.len(), 5);
        assert_eq!(board.board[0].len(), 7);
    }

    #[test]
    fn test_game_board_no_obstacles() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(4, 4, &palette, 0, 2);

        let mut thread_count = 0;
        let mut obstacle_count = 0;
        for row in &board.board {
            for entity in row {
                match entity {
                    BoardEntity::Thread(_) => thread_count += 1,
                    BoardEntity::Obstacle  => obstacle_count += 1,
                    _ => {}
                }
            }
        }
        assert!(thread_count >= 15);
        assert!(obstacle_count <= 1);
    }

    #[test]
    fn test_game_board_all_obstacles() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 100, 1);

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
    fn test_count_knits_empty_board() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 100, 5);
        let counter = board.count_knits();

        assert_eq!(counter.color_hashmap.len(), 0);
    }

    #[test]
    fn test_count_knits_multiplies_by_knit_volume() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 0, 3);
        let counter = board.count_knits();

        // 4 threads of red * 3 knit_volume = 12
        assert_eq!(*counter.color_hashmap.get(&Color::Red).unwrap(), 12);
    }

    #[test]
    fn test_count_knits_different_colors() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(5, 5, &palette, 0, 2);
        let counter = board.count_knits();

        assert!(counter.color_hashmap.len() >= 1);
        assert!(counter.color_hashmap.len() <= 3);

        let total: u16 = counter.color_hashmap.values().sum();
        assert!(total >= 48 && total <= 50);
    }

    #[test]
    fn test_knit_volume_stored() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 0, 7);
        assert_eq!(board.knit_volume, 7);
    }

    #[test]
    fn test_is_selectable_top_row_thread() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 0, 1);
        // Top-row threads are always selectable.
        for c in 0..3 {
            if matches!(board.board[0][c], BoardEntity::Thread(_)) {
                assert!(board.is_selectable(0, c));
            }
        }
    }

    #[test]
    fn test_is_selectable_obstacle_never() {
        let palette = vec![Color::Red];
        // 100% obstacles
        let board = GameBoard::make_random(3, 3, &palette, 100, 1);
        for r in 0..3 {
            for c in 0..3 {
                assert!(!board.is_selectable(r, c));
            }
        }
    }

    #[test]
    fn test_is_selectable_void_neighbor() {
        use crate::board_entity::BoardEntity;
        // Build a manual board: row 0 = Void, row 1 = Thread
        // The Thread at (1,0) borders a Void above → selectable.
        let mut board = GameBoard {
            board: vec![
                vec![BoardEntity::Void],
                vec![BoardEntity::Thread(Color::Red)],
                vec![BoardEntity::Thread(Color::Blue)],
            ],
            height: 3,
            width: 1,
            knit_volume: 1,
        };
        assert!(board.is_selectable(1, 0));  // borders Void above
        assert!(!board.is_selectable(2, 0)); // no Void neighbor
    }

    #[test]
    fn test_is_focusable_buried_thread() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Thread(Color::Red)],
                vec![BoardEntity::Thread(Color::Blue)],
                vec![BoardEntity::Thread(Color::Green)],
            ],
            height: 3,
            width: 1,
            knit_volume: 1,
        };
        assert!(board.is_focusable(0, 0));   // top row → selectable → focusable
        assert!(!board.is_focusable(1, 0));  // buried thread → not focusable
        assert!(!board.is_focusable(2, 0));  // buried thread → not focusable
    }

    #[test]
    fn test_is_focusable_non_thread_types() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle, BoardEntity::Void],
            ],
            height: 1,
            width: 2,
            knit_volume: 1,
        };
        assert!(!board.is_focusable(0, 0)); // obstacle → NOT focusable
        assert!(board.is_focusable(0, 1));  // void → focusable
    }

    #[test]
    fn has_selectable_thread_true_when_exposed() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Thread(Color::Red), BoardEntity::Void],
                vec![BoardEntity::Thread(Color::Blue), BoardEntity::Obstacle],
            ],
            height: 2, width: 2, knit_volume: 1,
        };
        assert!(board.has_selectable_thread());
    }

    #[test]
    fn has_selectable_thread_false_when_all_buried() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle, BoardEntity::Obstacle],
                vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
                vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
            ],
            height: 3, width: 2, knit_volume: 1,
        };
        assert!(!board.has_selectable_thread());
    }

    #[test]
    fn has_selectable_thread_false_when_no_threads() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Void, BoardEntity::Obstacle],
            ],
            height: 1, width: 2, knit_volume: 1,
        };
        assert!(!board.has_selectable_thread());
    }

    #[test]
    fn test_isolated_void_does_not_grant_selectability() {
        // An isolated void (not connected to top row) should NOT make
        // neighboring threads selectable.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Thread(Color::Red)],   // row 0: thread, no void
                vec![BoardEntity::Thread(Color::Blue)],  // row 1: thread
                vec![BoardEntity::Void],                 // row 2: void, but not connected to top
                vec![BoardEntity::Thread(Color::Green)], // row 3: has void above but isolated
            ],
            height: 4,
            width: 1,
            knit_volume: 1,
        };
        // Row 0 is selectable (top row)
        assert!(board.is_selectable(0, 0));
        // Row 1 has no surface-connected void neighbor
        assert!(!board.is_selectable(1, 0));
        // Row 2 is void, not a thread
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
                vec![BoardEntity::Thread(Color::Green)], // row 2: has connected void above
            ],
            height: 3,
            width: 1,
            knit_volume: 1,
        };
        // Row 2 borders void at (1,0), which connects to (0,0) → selectable
        assert!(board.is_selectable(2, 0));
    }

    #[test]
    fn test_surface_connected_voids_computation() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Void,                BoardEntity::Thread(Color::Red)],
                vec![BoardEntity::Void,                BoardEntity::Thread(Color::Blue)],
                vec![BoardEntity::Thread(Color::Green), BoardEntity::Void],
            ],
            height: 3,
            width: 2,
            knit_volume: 1,
        };
        let connected = board.surface_connected_voids();
        // (0,0) is a void in row 0 → connected
        assert!(connected.contains(&(0, 0)));
        // (1,0) is void adjacent to (0,0) → connected
        assert!(connected.contains(&(1, 0)));
        // (2,1) is void but not connected to any top-row void → NOT connected
        assert!(!connected.contains(&(2, 1)));
    }
}
