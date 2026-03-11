use crossterm::style::Color;
use crate::board::{Board, Orientation, SpecialPiece};

// ── Public types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchGroup {
    /// All cell positions in this group, sorted.
    pub cells: Vec<(usize, usize)>,
    /// Color of every gem in the group (all same color by construction).
    pub color: Color,
    /// Special piece to place at the swap destination, if any.
    pub create_special: Option<SpecialPiece>,
}

// ── Main entry point ──────────────────────────────────────────────────────

/// Find all match groups (connected same-color runs of ≥3) on the board.
/// Pure function — does not mutate anything.
pub fn find_matches(board: &Board) -> Vec<MatchGroup> {
    let h = board.height;
    let w = board.width;

    // h_run[r][c] = true if cell is part of a horizontal run of 3+
    let mut h_run = vec![vec![false; w]; h];
    for r in 0..h {
        let mut c = 0;
        while c < w {
            let Some(color) = board.cells[r][c].color() else { c += 1; continue; };
            let mut len = 1;
            while c + len < w && board.cells[r][c + len].color() == Some(color) {
                len += 1;
            }
            if len >= 3 {
                for i in 0..len { h_run[r][c + i] = true; }
            }
            c += len;
        }
    }

    // v_run[r][c] = true if cell is part of a vertical run of 3+
    let mut v_run = vec![vec![false; w]; h];
    for c in 0..w {
        let mut r = 0;
        while r < h {
            let Some(color) = board.cells[r][c].color() else { r += 1; continue; };
            let mut len = 1;
            while r + len < h && board.cells[r + len][c].color() == Some(color) {
                len += 1;
            }
            if len >= 3 {
                for i in 0..len { v_run[r + i][c] = true; }
            }
            r += len;
        }
    }

    // BFS connected components of matched cells, same-color only.
    let mut visited = vec![vec![false; w]; h];
    let mut groups = Vec::new();

    for r0 in 0..h {
        for c0 in 0..w {
            if (h_run[r0][c0] || v_run[r0][c0]) && !visited[r0][c0] {
                let start_color = board.cells[r0][c0].color().unwrap();
                let mut cells: Vec<(usize, usize)> = Vec::new();
                let mut queue = vec![(r0, c0)];
                visited[r0][c0] = true;

                while let Some((r, c)) = queue.pop() {
                    cells.push((r, c));
                    for (dr, dc) in [(-1i32, 0), (1, 0), (0, -1i32), (0, 1)] {
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                            let (nr, nc) = (nr as usize, nc as usize);
                            if (h_run[nr][nc] || v_run[nr][nc])
                                && !visited[nr][nc]
                                // Critical: only merge cells of the same color
                                && board.cells[nr][nc].color() == Some(start_color)
                            {
                                visited[nr][nc] = true;
                                queue.push((nr, nc));
                            }
                        }
                    }
                }

                cells.sort_unstable();
                let create_special = classify(&cells, &h_run, &v_run);
                groups.push(MatchGroup { cells, color: start_color, create_special });
            }
        }
    }

    groups
}

// ── Shape classification ──────────────────────────────────────────────────

fn classify(
    cells: &[(usize, usize)],
    h_run: &[Vec<bool>],
    v_run: &[Vec<bool>],
) -> Option<SpecialPiece> {
    let n = cells.len();

    // Any cell that appears in BOTH h_run and v_run is an intersection (L or T corner/joint).
    let has_intersection = cells.iter().any(|(r, c)| h_run[*r][*c] && v_run[*r][*c]);

    if has_intersection {
        // L or T shape. Larger = bigger area bomb.
        if n >= 6 {
            Some(SpecialPiece::AreaBomb { radius: 2 })
        } else {
            Some(SpecialPiece::AreaBomb { radius: 1 })
        }
    } else if n >= 5 {
        Some(SpecialPiece::ColorBomb)
    } else if n == 4 {
        let rows: std::collections::HashSet<usize> = cells.iter().map(|(r, _)| *r).collect();
        let orientation = if rows.len() == 1 { Orientation::Horizontal } else { Orientation::Vertical };
        Some(SpecialPiece::LineBomb(orientation))
    } else {
        None // n == 3: basic match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;
    use crate::board::{Board, Cell, CellContent};

    // Helper: build a Board from a color grid (no modifiers).
    fn board(grid: &[&[Color]]) -> Board {
        let cells: Vec<Vec<Cell>> = grid
            .iter()
            .map(|row| row.iter().map(|&c| Cell::gem(c)).collect())
            .collect();
        Board { height: cells.len(), width: cells[0].len(), cells }
    }

    const R: Color = Color::Red;
    const B: Color = Color::Blue;
    const G: Color = Color::Green;
    const Y: Color = Color::Yellow;

    // ── No matches ─────────────────────────────────────────────────────────

    #[test]
    fn checkerboard_no_matches() {
        let b = board(&[
            &[R, B, R],
            &[B, R, B],
            &[R, B, R],
        ]);
        assert_eq!(find_matches(&b).len(), 0);
    }

    // ── Basic 3-in-line ────────────────────────────────────────────────────

    #[test]
    fn horizontal_3_match_row0() {
        let b = board(&[
            &[R, R, R],
            &[B, G, B],
            &[G, B, G],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].cells.len(), 3);
        assert!(gs[0].create_special.is_none());
    }

    #[test]
    fn vertical_3_match_col0() {
        let b = board(&[
            &[R, B],
            &[R, G],
            &[R, B],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].cells.len(), 3);
        assert!(gs[0].create_special.is_none());
    }

    // ── Two separate matches ────────────────────────────────────────────────

    #[test]
    fn two_separate_horizontal_matches() {
        let b = board(&[
            &[R, R, R, B, B],
            &[G, R, B, R, G],
        ]);
        // Only one match: row 0 cols 0-2
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].cells.len(), 3);
    }

    #[test]
    fn two_non_adjacent_matches_counted_separately() {
        //  R R R  G G G   (two separate horizontal matches, no adjacency)
        let b = board(&[
            &[R, R, R, B, G, G, G],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 2);
    }

    // ── 4-in-line → LineBomb ────────────────────────────────────────────────

    #[test]
    fn horizontal_4_creates_horizontal_line_bomb() {
        let b = board(&[
            &[R, R, R, R, B],
            &[B, G, B, G, R],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].cells.len(), 4);
        assert!(matches!(
            gs[0].create_special,
            Some(SpecialPiece::LineBomb(Orientation::Horizontal))
        ));
    }

    #[test]
    fn vertical_4_creates_vertical_line_bomb() {
        let b = board(&[
            &[R, B],
            &[R, G],
            &[R, B],
            &[R, G],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert!(matches!(
            gs[0].create_special,
            Some(SpecialPiece::LineBomb(Orientation::Vertical))
        ));
    }

    // ── 5-in-line → ColorBomb ──────────────────────────────────────────────

    #[test]
    fn horizontal_5_creates_color_bomb() {
        let b = board(&[
            &[R, R, R, R, R],
            &[B, G, B, G, B],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert!(matches!(gs[0].create_special, Some(SpecialPiece::ColorBomb)));
    }

    #[test]
    fn vertical_5_creates_color_bomb() {
        let b = board(&[
            &[R, B],
            &[R, G],
            &[R, B],
            &[R, G],
            &[R, B],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert!(matches!(gs[0].create_special, Some(SpecialPiece::ColorBomb)));
    }

    // ── L / T shape → AreaBomb ─────────────────────────────────────────────

    #[test]
    fn l_shape_creates_area_bomb_r1() {
        // col 0: R R R (vertical), row 0: R R R (horizontal). L at (0,0).
        // Non-Red cells are varied so they form no 3-in-line runs.
        //  R R R B
        //  R G Y B
        //  R B G Y
        let b = board(&[
            &[R, R, R, B],
            &[R, G, Y, B],
            &[R, B, G, Y],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert_eq!(gs[0].cells.len(), 5); // 3 horiz + 3 vert - 1 shared
        assert!(matches!(gs[0].create_special, Some(SpecialPiece::AreaBomb { radius: 1 })));
    }

    #[test]
    fn large_l_shape_creates_area_bomb_r2() {
        // col 0: 4 reds vertical, row 0: 4 reds horizontal → 7 cells, L/T shape
        // Non-Red cells are varied so they form no 3-in-line runs.
        //  R R R R B
        //  R G Y B G
        //  R B G Y B
        //  R Y B G Y
        let b = board(&[
            &[R, R, R, R, B],
            &[R, G, Y, B, G],
            &[R, B, G, Y, B],
            &[R, Y, B, G, Y],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 1);
        assert!(matches!(gs[0].create_special, Some(SpecialPiece::AreaBomb { radius: 2 })));
    }

    // ── Colors don't bleed across runs ─────────────────────────────────────

    #[test]
    fn adjacent_different_color_runs_stay_separate() {
        //  R R R
        //  B B B   (adjacent but different colors → 2 separate groups)
        let b = board(&[
            &[R, R, R],
            &[B, B, B],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs.len(), 2);
    }

    // ── Group color field ────────────────────────────────────────────────────

    #[test]
    fn group_color_matches_board_gems() {
        let b = board(&[
            &[R, R, R],
            &[B, G, B],
        ]);
        let gs = find_matches(&b);
        assert_eq!(gs[0].color, R);
    }
}
