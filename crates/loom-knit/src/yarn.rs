use std::fmt;

use crossterm::style::{Color, Stylize};

use crate::color_counter::ColorCounter;
use crate::spool::Spool;

#[derive(Clone)]
pub struct Stitch {
    pub color: Color,
    /// A locked stitch blocks its entire column until cleared by a matching KeySpool.
    pub locked: bool,
}

impl fmt::Display for Stitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = if self.locked { '▣' } else { '▦' };
        write!(f, "{}", ch.with(self.color))
    }
}

pub struct Yarn {
    pub board: Vec<Vec<Stitch>>,
    pub yarn_lines: u16,
    pub visible_stitches: u16,
    pub balloon_columns: Vec<Option<Stitch>>,
}

impl Yarn {
    pub fn make_from_color_counter(counter: ColorCounter, yarn_lines: u16, visible_stitches: u16) -> Self {
        let mut board: Vec<Vec<Stitch>> = Vec::new();
        for _ in 0..yarn_lines {
            board.push(Vec::new());
        }
        let shuffled_queue: Vec<Color> = counter.get_shuffled_queue();
        for color in shuffled_queue.iter() {
            let column_number = rand::random::<u16>() % yarn_lines;
            board[column_number as usize].push(Stitch { color: *color, locked: false });
        }
        Self { board, yarn_lines, visible_stitches, balloon_columns: Vec::new() }
    }

    /// Process one spool against the yarn.
    ///
    /// Scans columns left-to-right; the first column whose last stitch matches
    /// `spool.color` is consumed (popped) and the spool advances one stage.
    ///
    /// Lock rules:
    /// - A locked stitch at the end of a column blocks that entire column.
    /// - A locked stitch is only clearable by a spool with `has_key == true`
    ///   and a matching color. Clearing it consumes the key.
    pub fn process_one(&mut self, spool: &mut Spool) {
        // Check regular columns first
        for column in &mut self.board {
            let Some(last) = column.last() else { continue };

            if last.locked {
                if last.color == spool.color && spool.has_key {
                    column.pop();
                    spool.wind();
                    spool.has_key = false;
                    return;
                }
                // Locked stitch blocks the column — skip it entirely.
                continue;
            }

            if last.color == spool.color {
                column.pop();
                spool.wind();
                return;
            }
        }

        // Then check balloon slots (fixed positions, not a queue)
        for slot in &mut self.balloon_columns {
            if let Some(stitch) = slot {
                if stitch.color == spool.color {
                    *slot = None;
                    spool.wind();
                    return;
                }
            }
        }
    }

    /// Clear balloon slots once all stitches have been processed.
    pub fn cleanup_balloon_columns(&mut self) {
        if !self.balloon_columns.is_empty()
            && self.balloon_columns.iter().all(|s| s.is_none())
        {
            self.balloon_columns.clear();
        }
    }

    pub fn process_sequence(&mut self, spools: &mut Vec<Spool>) {
        for spool in spools {
            self.process_one(spool);
        }
    }

    /// Deep-scan all yarn columns (and balloon columns) for a matching stitch.
    /// Unlike process_one, this ignores queue order — it searches ALL stitches
    /// in each column, not just the front.
    ///
    /// Uses BFS across columns: checks depth 0 (front) of ALL columns first,
    /// then depth 1 of all columns, etc. This ensures the visually closest
    /// match across all columns is consumed first, rather than exhausting
    /// one column before checking the next.
    /// Locked stitches are skipped entirely.
    pub fn deep_scan_process(&mut self, spool: &mut Spool) {
        let max_len = self.board.iter()
            .map(|c| c.len()).max().unwrap_or(0);

        for depth in 0..max_len {
            // Check regular columns at this depth
            for col_idx in 0..self.board.len() {
                let col = &self.board[col_idx];
                if col.len() <= depth { continue; }
                let pos = col.len() - 1 - depth;
                if !col[pos].locked && col[pos].color == spool.color {
                    self.board[col_idx].remove(pos);
                    spool.wind();
                    return;
                }
            }
            // Balloon slots are flat (exposed stitches) — check at depth 0 only
            if depth == 0 {
                for slot in &mut self.balloon_columns {
                    if let Some(stitch) = slot {
                        if !stitch.locked && stitch.color == spool.color {
                            *slot = None;
                            spool.wind();
                            return;
                        }
                    }
                }
            }
        }
    }
}

impl fmt::Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for offset in 0..(self.visible_stitches as usize) {
            let true_offset: usize = (self.visible_stitches as usize) - offset;
            for column in &self.board {
                if !(true_offset > column.len()) {
                    let pos_to_print = column.len() - true_offset;
                    write!(f, "{}", column[pos_to_print])?;
                }
            }
            write!(f, "{}", "\n\r")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_yarn_creation_from_empty_counter() {
        let counter = ColorCounter {
            color_hashmap: HashMap::new(),
        };

        let yarn = Yarn::make_from_color_counter(counter, 3, 5);

        assert_eq!(yarn.yarn_lines, 3);
        assert_eq!(yarn.visible_stitches, 5);
        assert_eq!(yarn.board.len(), 3);
    }

    #[test]
    fn test_yarn_creation_with_colors() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 5);

        let counter = ColorCounter { color_hashmap: map };
        let yarn = Yarn::make_from_color_counter(counter, 2, 3);

        let total_stitches: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(total_stitches, 5);
    }

    #[test]
    fn test_yarn_dimensions() {
        let mut map = HashMap::new();
        map.insert(Color::Blue, 10);

        let counter = ColorCounter { color_hashmap: map };
        let yarn = Yarn::make_from_color_counter(counter, 4, 6);

        assert_eq!(yarn.board.len(), 4);
        assert_eq!(yarn.yarn_lines, 4);
        assert_eq!(yarn.visible_stitches, 6);
    }

    #[test]
    fn test_process_one_removes_matching_stitch() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);

        let counter = ColorCounter { color_hashmap: map };
        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut spool = Spool { color: Color::Red, fill: 1, has_key: false };
        yarn.process_one(&mut spool);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total - 1);
        assert_eq!(spool.fill, 2);
    }

    #[test]
    fn test_process_one_no_matching_color() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 2);

        let counter = ColorCounter { color_hashmap: map };
        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut spool = Spool { color: Color::Blue, fill: 1, has_key: false };
        yarn.process_one(&mut spool);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total);
        assert_eq!(spool.fill, 1);
    }

    #[test]
    fn test_process_one_locked_stitch_no_key() {
        // A locked stitch should block the column even when color matches.
        let mut yarn = Yarn {
            board: vec![vec![Stitch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Red, fill: 1, has_key: false };
        yarn.process_one(&mut spool);

        // Should not have processed: stitch still there, fill unchanged.
        assert_eq!(yarn.board[0].len(), 1);
        assert_eq!(spool.fill, 1);
    }

    #[test]
    fn test_process_one_locked_stitch_with_key() {
        // A key spool should clear a locked stitch of matching color.
        let mut yarn = Yarn {
            board: vec![vec![Stitch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Red, fill: 1, has_key: true };
        yarn.process_one(&mut spool);

        assert_eq!(yarn.board[0].len(), 0);
        assert_eq!(spool.fill, 2);
        assert!(!spool.has_key); // key consumed
    }

    #[test]
    fn test_process_one_locked_stitch_wrong_color_with_key() {
        // Key doesn't help if colors don't match.
        let mut yarn = Yarn {
            board: vec![vec![Stitch { color: Color::Blue, locked: true }]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Red, fill: 1, has_key: true };
        yarn.process_one(&mut spool);

        assert_eq!(yarn.board[0].len(), 1);
        assert_eq!(spool.fill, 1);
        assert!(spool.has_key); // key NOT consumed on wrong color
    }

    #[test]
    fn test_process_sequence_multiple_spools() {
        // Use a deterministic board to avoid the flakiness of random distribution.
        // Col 0: bottom=[Red, Blue, Red]=top  → spool[0] pops Red, spool[1] pops Blue, spool[2] pops Red
        let mut yarn = Yarn {
            board: vec![
                vec![
                    Stitch { color: Color::Red,  locked: false }, // bottom
                    Stitch { color: Color::Blue, locked: false },
                    Stitch { color: Color::Red,  locked: false }, // top (last)
                ],
                vec![],
            ],
            yarn_lines: 2,
            visible_stitches: 5,
            balloon_columns: Vec::new(),
        };

        let mut spools = vec![
            Spool { color: Color::Red,  fill: 1, has_key: false },
            Spool { color: Color::Blue, fill: 1, has_key: false },
            Spool { color: Color::Red,  fill: 1, has_key: false },
        ];

        yarn.process_sequence(&mut spools);

        assert_eq!(spools[0].fill, 2);
        assert_eq!(spools[1].fill, 2);
        assert_eq!(spools[2].fill, 2);
    }

    #[test]
    fn test_process_sequence_empty_spools() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 5);

        let counter = ColorCounter { color_hashmap: map };
        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut spools: Vec<Spool> = vec![];
        yarn.process_sequence(&mut spools);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total);
    }

    #[test]
    fn test_deep_scan_process_finds_match_behind_front() {
        // Col 0: [Blue(bottom), Red(top)] — front is Red, but spool is Blue
        // Deep scan should find Blue behind Red and remove it
        let mut yarn = Yarn {
            board: vec![vec![
                Stitch { color: Color::Blue, locked: false },  // bottom (index 0)
                Stitch { color: Color::Red, locked: false },   // top (index 1, front)
            ]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Blue, fill: 1, has_key: false };
        yarn.deep_scan_process(&mut spool);

        assert_eq!(spool.fill, 2); // wound once
        assert_eq!(yarn.board[0].len(), 1); // Blue removed, Red remains
        assert_eq!(yarn.board[0][0].color, Color::Red); // Red is still there
    }

    #[test]
    fn test_deep_scan_process_no_match() {
        let mut yarn = Yarn {
            board: vec![vec![
                Stitch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Green, fill: 1, has_key: false };
        yarn.deep_scan_process(&mut spool);

        assert_eq!(spool.fill, 1); // no change
        assert_eq!(yarn.board[0].len(), 1);
    }

    #[test]
    fn test_deep_scan_checks_balloon_columns() {
        let mut yarn = Yarn {
            board: vec![vec![
                Stitch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: vec![
                Some(Stitch { color: Color::Blue, locked: false }),
            ],
        };

        let mut spool = Spool { color: Color::Blue, fill: 1, has_key: false };
        yarn.deep_scan_process(&mut spool);

        assert_eq!(spool.fill, 2);
        assert!(yarn.balloon_columns[0].is_none());
    }

    #[test]
    fn test_deep_scan_bfs_prefers_front_across_columns() {
        // Col 0: [Blue(bottom), Red(top)]  — Blue at depth 1
        // Col 1: [Red(bottom), Blue(top)]  — Blue at depth 0 (front)
        // BFS should pick the Blue from col 1 (depth 0) before col 0 (depth 1).
        let mut yarn = Yarn {
            board: vec![
                vec![
                    Stitch { color: Color::Blue, locked: false },  // depth 1
                    Stitch { color: Color::Red, locked: false },   // depth 0 (front)
                ],
                vec![
                    Stitch { color: Color::Red, locked: false },   // depth 1
                    Stitch { color: Color::Blue, locked: false },  // depth 0 (front)
                ],
            ],
            yarn_lines: 2,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Blue, fill: 1, has_key: false };
        yarn.deep_scan_process(&mut spool);

        assert_eq!(spool.fill, 2);
        // Col 1 front (Blue) should have been consumed, not col 0 deep (Blue)
        assert_eq!(yarn.board[0].len(), 2); // col 0 untouched
        assert_eq!(yarn.board[1].len(), 1); // col 1 had its front removed
        assert_eq!(yarn.board[1][0].color, Color::Red); // Red remains at bottom
    }

    #[test]
    fn test_deep_scan_bfs_skips_locked_at_front() {
        // Col 0: [Blue(locked, front)] — locked, should be skipped
        // Col 1: [Blue(unlocked, deep), Red(front)]
        // BFS depth 0: col 0 locked → skip, col 1 front is Red → no match
        // BFS depth 1: col 1 deep is Blue → match
        let mut yarn = Yarn {
            board: vec![
                vec![
                    Stitch { color: Color::Blue, locked: true },   // depth 0, locked
                ],
                vec![
                    Stitch { color: Color::Blue, locked: false },  // depth 1
                    Stitch { color: Color::Red, locked: false },   // depth 0 (front)
                ],
            ],
            yarn_lines: 2,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };

        let mut spool = Spool { color: Color::Blue, fill: 1, has_key: false };
        yarn.deep_scan_process(&mut spool);

        assert_eq!(spool.fill, 2);
        assert_eq!(yarn.board[0].len(), 1); // col 0 untouched (locked)
        assert_eq!(yarn.board[1].len(), 1); // col 1 deep Blue removed
        assert_eq!(yarn.board[1][0].color, Color::Red); // Red remains at front
    }

    #[test]
    fn test_yarn_display_format() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);

        let counter = ColorCounter { color_hashmap: map };
        let yarn = Yarn::make_from_color_counter(counter, 2, 3);

        let _ = format!("{}", yarn);
    }

    #[test]
    fn test_process_one_checks_balloon_columns() {
        let mut yarn = Yarn {
            board: vec![vec![
                Stitch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: vec![
                Some(Stitch { color: Color::Blue, locked: false }),
            ],
        };

        let mut spool = Spool { color: Color::Blue, fill: 1, has_key: false };
        yarn.process_one(&mut spool);

        // Should match against balloon slot, not regular column
        assert_eq!(spool.fill, 2);
        assert!(yarn.balloon_columns[0].is_none());
        assert_eq!(yarn.board[0].len(), 1); // regular column unchanged
    }

    #[test]
    fn test_process_one_prefers_regular_over_balloon() {
        let mut yarn = Yarn {
            board: vec![vec![
                Stitch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: vec![
                Some(Stitch { color: Color::Red, locked: false }),
            ],
        };

        let mut spool = Spool { color: Color::Red, fill: 1, has_key: false };
        yarn.process_one(&mut spool);

        // Regular columns checked first
        assert_eq!(spool.fill, 2);
        assert_eq!(yarn.board[0].len(), 0); // regular consumed
        assert!(yarn.balloon_columns[0].is_some()); // balloon untouched
    }
}
