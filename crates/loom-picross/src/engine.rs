use crate::puzzle::Puzzle;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Unknown,
    Filled,
    Crossed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameStatus {
    Playing,
    Won,
    TooManyMistakes,
}

pub const MAX_MISTAKES: u32 = 5;

pub struct GameEngine {
    pub puzzle: Puzzle,
    /// Player's current marks on the grid.
    pub grid: Vec<Vec<CellState>>,
    /// (row, col) of the cursor.
    pub cursor: (usize, usize),
    pub status: GameStatus,
    /// Number of wrong fills made.
    pub mistakes: u32,
    /// Number of correctly filled cells.
    pub filled_count: u32,
}

impl GameEngine {
    pub fn new(puzzle: Puzzle) -> Self {
        let rows = puzzle.rows;
        let cols = puzzle.cols;
        let grid = vec![vec![CellState::Unknown; cols]; rows];
        GameEngine {
            puzzle,
            grid,
            cursor: (0, 0),
            status: GameStatus::Playing,
            mistakes: 0,
            filled_count: 0,
        }
    }

    /// Total cells that should be filled in the solution.
    pub fn total_to_fill(&self) -> u32 {
        self.puzzle
            .solution
            .iter()
            .flat_map(|r| r.iter())
            .filter(|&&b| b)
            .count() as u32
    }

    /// Completion percentage (0.0–100.0).
    pub fn completion_pct(&self) -> f32 {
        let total = self.total_to_fill();
        if total == 0 {
            return 100.0;
        }
        self.filled_count as f32 / total as f32 * 100.0
    }

    /// Fill or unfill the cell at the cursor.
    pub fn toggle_fill(&mut self) {
        if self.status != GameStatus::Playing {
            return;
        }
        let (r, c) = self.cursor;
        match self.grid[r][c] {
            CellState::Filled => {
                // Unfill: if it was correct, decrement filled_count
                if self.puzzle.solution[r][c] {
                    self.filled_count = self.filled_count.saturating_sub(1);
                }
                self.grid[r][c] = CellState::Unknown;
            }
            CellState::Unknown => {
                self.grid[r][c] = CellState::Filled;
                if self.puzzle.solution[r][c] {
                    self.filled_count += 1;
                } else {
                    self.mistakes += 1;
                    if self.mistakes >= MAX_MISTAKES {
                        self.status = GameStatus::TooManyMistakes;
                        return;
                    }
                }
                self.check_win();
            }
            CellState::Crossed => {
                // Clicking fill on a crossed cell: switch to filled
                self.grid[r][c] = CellState::Filled;
                if self.puzzle.solution[r][c] {
                    self.filled_count += 1;
                } else {
                    self.mistakes += 1;
                    if self.mistakes >= MAX_MISTAKES {
                        self.status = GameStatus::TooManyMistakes;
                        return;
                    }
                }
                self.check_win();
            }
        }
    }

    /// Cross out or uncross the cell at the cursor.
    pub fn toggle_cross(&mut self) {
        if self.status != GameStatus::Playing {
            return;
        }
        let (r, c) = self.cursor;
        match self.grid[r][c] {
            CellState::Crossed => {
                self.grid[r][c] = CellState::Unknown;
            }
            CellState::Unknown | CellState::Filled => {
                // If it was filled and correct, we lose the progress count
                if self.grid[r][c] == CellState::Filled && self.puzzle.solution[r][c] {
                    self.filled_count = self.filled_count.saturating_sub(1);
                }
                self.grid[r][c] = CellState::Crossed;
            }
        }
    }

    /// Move cursor by (dr, dc), clamped to the grid boundaries.
    pub fn move_cursor(&mut self, dr: i32, dc: i32) {
        let rows = self.puzzle.rows as i32;
        let cols = self.puzzle.cols as i32;
        let new_r = (self.cursor.0 as i32 + dr).clamp(0, rows - 1) as usize;
        let new_c = (self.cursor.1 as i32 + dc).clamp(0, cols - 1) as usize;
        self.cursor = (new_r, new_c);
    }

    /// Check whether the player has filled every correct cell.
    pub fn check_win(&mut self) {
        if self.filled_count >= self.total_to_fill() {
            self.status = GameStatus::Won;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Difficulty, Puzzle};

    fn tiny_puzzle() -> Puzzle {
        Puzzle::from_solution(
            "tiny",
            Difficulty::Easy,
            vec![vec![true, false], vec![false, true]],
        )
    }

    #[test]
    fn win_after_filling_correct_cells() {
        let mut e = GameEngine::new(tiny_puzzle());
        e.cursor = (0, 0);
        e.toggle_fill();
        assert_eq!(e.status, GameStatus::Playing);
        e.cursor = (1, 1);
        e.toggle_fill();
        assert_eq!(e.status, GameStatus::Won);
    }

    #[test]
    fn mistake_increments() {
        let mut e = GameEngine::new(tiny_puzzle());
        e.cursor = (0, 1); // wrong cell
        e.toggle_fill();
        assert_eq!(e.mistakes, 1);
    }

    #[test]
    fn too_many_mistakes_triggers_loss() {
        let p = Puzzle::from_solution(
            "loss",
            Difficulty::Easy,
            vec![vec![true, false, false, false, false, false]],
        );
        let mut e = GameEngine::new(p);
        for c in 1..=5 {
            e.cursor = (0, c);
            e.toggle_fill();
        }
        assert_eq!(e.status, GameStatus::TooManyMistakes);
    }
}
