use crate::puzzle::{Difficulty, Puzzle};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Convert a compact string grid into a bool grid.
/// '#' = filled, any other char = empty.
fn from_str_grid(rows: &[&str]) -> Vec<Vec<bool>> {
    rows.iter()
        .map(|row| row.chars().map(|c| c == '#').collect())
        .collect()
}

// ── 5×5 Easy Puzzles ─────────────────────────────────────────────────────────

fn puzzle_heart() -> Puzzle {
    Puzzle::from_solution(
        "Heart",
        Difficulty::Easy,
        from_str_grid(&[
            ".##.##",
            "######",
            "######",
            ".####.",
            "..##..",
            "...#..",
        ]),
    )
}

fn puzzle_star() -> Puzzle {
    Puzzle::from_solution(
        "Star",
        Difficulty::Easy,
        from_str_grid(&[
            "..#..",
            "#####",
            ".###.",
            "##.##",
            "#...#",
        ]),
    )
}

fn puzzle_house() -> Puzzle {
    Puzzle::from_solution(
        "House",
        Difficulty::Easy,
        from_str_grid(&[
            "..#..",
            ".###.",
            "#####",
            "#.#.#",
            "#####",
        ]),
    )
}

fn puzzle_smiley() -> Puzzle {
    Puzzle::from_solution(
        "Smiley",
        Difficulty::Easy,
        from_str_grid(&[
            ".###.",
            "#...#",
            "#.#.#",  // eyes
            "#...#",
            ".#.#.",  // smile corners
            ".###.",
        ]),
    )
}

fn puzzle_diamond() -> Puzzle {
    Puzzle::from_solution(
        "Diamond",
        Difficulty::Easy,
        from_str_grid(&[
            "..#..",
            ".###.",
            "#####",
            ".###.",
            "..#..",
        ]),
    )
}

fn puzzle_cross() -> Puzzle {
    Puzzle::from_solution(
        "Cross",
        Difficulty::Easy,
        from_str_grid(&[
            "..#..",
            "..#..",
            "#####",
            "..#..",
            "..#..",
        ]),
    )
}

// ── 7×7 Medium Puzzles ───────────────────────────────────────────────────────

fn puzzle_tree() -> Puzzle {
    Puzzle::from_solution(
        "Tree",
        Difficulty::Medium,
        from_str_grid(&[
            "...#...",
            "..###..",
            ".#####.",
            "#######",
            "...#...",
            "..###..",
            "...#...",
        ]),
    )
}

fn puzzle_rocket() -> Puzzle {
    Puzzle::from_solution(
        "Rocket",
        Difficulty::Medium,
        from_str_grid(&[
            "...#...",
            "..###..",
            ".#####.",
            ".#.#.#.",
            ".#####.",
            "#.###.#",
            "#.....#",
        ]),
    )
}

fn puzzle_fish() -> Puzzle {
    Puzzle::from_solution(
        "Fish",
        Difficulty::Medium,
        from_str_grid(&[
            "#......",
            "##.###.",
            "##.####",
            "#######",
            "##.####",
            "##.###.",
            "#......",
        ]),
    )
}

fn puzzle_flag() -> Puzzle {
    Puzzle::from_solution(
        "Flag",
        Difficulty::Medium,
        from_str_grid(&[
            "#......",
            "####...",
            "#######",
            "####...",
            "#......",
            "#......",
            "#......",
        ]),
    )
}

fn puzzle_crown() -> Puzzle {
    Puzzle::from_solution(
        "Crown",
        Difficulty::Medium,
        from_str_grid(&[
            "#.#.#.#",
            "#######",
            "#######",
            ".#####.",
            ".#####.",
            "..###..",
            ".......",
        ]),
    )
}

fn puzzle_mushroom() -> Puzzle {
    Puzzle::from_solution(
        "Mushroom",
        Difficulty::Medium,
        from_str_grid(&[
            ".#####.",
            "#######",
            "#.#.#.#",
            "#######",
            "..###..",
            "..###..",
            "..###..",
        ]),
    )
}

// ── 10×10 Hard Puzzles ───────────────────────────────────────────────────────

fn puzzle_skull() -> Puzzle {
    Puzzle::from_solution(
        "Skull",
        Difficulty::Hard,
        from_str_grid(&[
            "..######..",
            ".########.",
            "##########",
            "#.####.###",
            "##########",
            ".########.",
            "..######..",
            "...####...",
            "..######..",
            "..#....#..",
        ]),
    )
}

fn puzzle_anchor() -> Puzzle {
    Puzzle::from_solution(
        "Anchor",
        Difficulty::Hard,
        from_str_grid(&[
            "....##....",
            "...####...",
            "....##....",
            "....##....",
            ".########.",
            "....##....",
            "....##....",
            "#...##...#",
            ".#######..",
            "..#####...",
        ]),
    )
}

fn puzzle_butterfly() -> Puzzle {
    Puzzle::from_solution(
        "Butterfly",
        Difficulty::Hard,
        from_str_grid(&[
            "#.......#.",
            "##.....##.",
            "###...###.",
            "####.####.",
            "####.####.",
            "###...###.",
            "##.....##.",
            "#.......#.",
            "..........",
            ".....#....",
        ]),
    )
}

fn puzzle_castle() -> Puzzle {
    Puzzle::from_solution(
        "Castle",
        Difficulty::Hard,
        from_str_grid(&[
            "#.#.#.#.#.",
            "##########",
            "##########",
            "#........#",
            "#.##..##.#",
            "#.##..##.#",
            "#........#",
            "####..####",
            "####..####",
            "####..####",
        ]),
    )
}

// ── Public Puzzle Library ────────────────────────────────────────────────────

/// All built-in puzzles in order (easy → medium → hard).
pub fn all_puzzles() -> Vec<Puzzle> {
    vec![
        puzzle_cross(),
        puzzle_diamond(),
        puzzle_star(),
        puzzle_house(),
        puzzle_smiley(),
        puzzle_heart(),
        puzzle_tree(),
        puzzle_flag(),
        puzzle_mushroom(),
        puzzle_fish(),
        puzzle_crown(),
        puzzle_rocket(),
        puzzle_skull(),
        puzzle_anchor(),
        puzzle_butterfly(),
        puzzle_castle(),
    ]
}

pub fn puzzle_count() -> usize {
    all_puzzles().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_15_puzzles() {
        assert!(all_puzzles().len() >= 15);
    }

    #[test]
    fn all_puzzles_valid_dimensions() {
        for p in all_puzzles() {
            assert!(p.rows > 0);
            assert!(p.cols > 0);
            assert_eq!(p.solution.len(), p.rows);
            for row in &p.solution {
                assert_eq!(row.len(), p.cols);
            }
        }
    }

    #[test]
    fn clues_match_solution() {
        use crate::puzzle::compute_clues;
        for p in all_puzzles() {
            for (r, row) in p.solution.iter().enumerate() {
                assert_eq!(p.row_clues[r], compute_clues(row), "row clue mismatch in {}", p.name);
            }
        }
    }
}
