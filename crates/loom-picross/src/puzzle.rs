/// A single nonogram/picross puzzle.
#[derive(Clone, Debug)]
pub struct Puzzle {
    pub rows: usize,
    pub cols: usize,
    /// true = filled cell in the solution
    pub solution: Vec<Vec<bool>>,
    /// clues for each row (consecutive run lengths)
    pub row_clues: Vec<Vec<u8>>,
    /// clues for each column (consecutive run lengths)
    pub col_clues: Vec<Vec<u8>>,
    pub name: &'static str,
    pub difficulty: Difficulty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Puzzle {
    /// Build a Puzzle from a 2-D solution grid, computing clues automatically.
    pub fn from_solution(
        name: &'static str,
        difficulty: Difficulty,
        solution: Vec<Vec<bool>>,
    ) -> Self {
        let rows = solution.len();
        let cols = if rows > 0 { solution[0].len() } else { 0 };

        let row_clues: Vec<Vec<u8>> = solution.iter().map(|r| compute_clues(r)).collect();
        let col_clues: Vec<Vec<u8>> = (0..cols)
            .map(|c| {
                let col: Vec<bool> = (0..rows).map(|r| solution[r][c]).collect();
                compute_clues(&col)
            })
            .collect();

        Puzzle { rows, cols, solution, row_clues, col_clues, name, difficulty }
    }
}

/// Count consecutive runs of `true` in a slice.
pub fn compute_clues(line: &[bool]) -> Vec<u8> {
    let mut clues = Vec::new();
    let mut run = 0u8;
    for &cell in line {
        if cell {
            run += 1;
        } else if run > 0 {
            clues.push(run);
            run = 0;
        }
    }
    if run > 0 {
        clues.push(run);
    }
    if clues.is_empty() {
        clues.push(0);
    }
    clues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clue_empty_row() {
        assert_eq!(compute_clues(&[false, false, false]), vec![0]);
    }

    #[test]
    fn clue_full_row() {
        assert_eq!(compute_clues(&[true, true, true]), vec![3]);
    }

    #[test]
    fn clue_two_runs() {
        assert_eq!(compute_clues(&[true, false, true, true]), vec![1, 2]);
    }

    #[test]
    fn puzzle_from_solution_builds_clues() {
        let sol = vec![
            vec![true, false, true],
            vec![false, true, false],
        ];
        let p = Puzzle::from_solution("test", Difficulty::Easy, sol);
        assert_eq!(p.row_clues[0], vec![1, 1]);
        assert_eq!(p.row_clues[1], vec![1]);
        assert_eq!(p.col_clues[0], vec![1]);
        assert_eq!(p.col_clues[1], vec![1]);
        assert_eq!(p.col_clues[2], vec![1]);
    }
}
