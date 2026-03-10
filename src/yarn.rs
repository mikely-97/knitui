use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;
use std::collections::HashMap;
use crate::color_counter::ColorCounter;
use crate::active_threads::Thread;

use rand::prelude::*;

pub struct Patch {
    pub color: Color,
    /// A locked patch blocks its entire column until cleared by a matching KeyThread.
    pub locked: bool,
}

impl fmt::Display for Patch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ch = if self.locked { '▣' } else { '▦' };
        write!(f, "{}", ch.with(self.color))
    }
}

pub struct Yarn {
    pub board: Vec<Vec<Patch>>,
    pub yarn_lines: u16,
    pub visible_patches: u16,
    pub balloon_columns: Vec<Option<Patch>>,
}

impl Yarn {
    pub fn make_from_color_counter(counter: ColorCounter, yarn_lines: u16, visible_patches: u16) -> Self {
        let mut board: Vec<Vec<Patch>> = Vec::new();
        for _ in 0..yarn_lines {
            board.push(Vec::new());
        }
        let shuffled_queue: Vec<Color> = counter.get_shuffled_queue();
        for color in shuffled_queue.iter() {
            let column_number = rand::random::<u16>() % yarn_lines;
            board[column_number as usize].push(Patch { color: *color, locked: false });
        }
        Self { board, yarn_lines, visible_patches, balloon_columns: Vec::new() }
    }

    /// Process one thread against the yarn.
    ///
    /// Scans columns left-to-right; the first column whose last patch matches
    /// `thread.color` is consumed (popped) and the thread advances one stage.
    ///
    /// Lock rules:
    /// - A locked patch at the end of a column blocks that entire column.
    /// - A locked patch is only clearable by a thread with `has_key == true`
    ///   and a matching color. Clearing it consumes the key.
    pub fn process_one(&mut self, thread: &mut Thread) {
        // Check regular columns first
        for column in &mut self.board {
            let Some(last) = column.last() else { continue };

            if last.locked {
                if last.color == thread.color && thread.has_key {
                    column.pop();
                    thread.knit_on();
                    thread.has_key = false;
                    return;
                }
                // Locked patch blocks the column — skip it entirely.
                continue;
            }

            if last.color == thread.color {
                column.pop();
                thread.knit_on();
                return;
            }
        }

        // Then check balloon slots (fixed positions, not a queue)
        for slot in &mut self.balloon_columns {
            if let Some(patch) = slot {
                if patch.color == thread.color {
                    *slot = None;
                    thread.knit_on();
                    return;
                }
            }
        }
    }

    /// Clear balloon slots once all patches have been processed.
    pub fn cleanup_balloon_columns(&mut self) {
        if !self.balloon_columns.is_empty()
            && self.balloon_columns.iter().all(|s| s.is_none())
        {
            self.balloon_columns.clear();
        }
    }

    pub fn process_sequence(&mut self, threads: &mut Vec<Thread>) {
        for thread in threads {
            self.process_one(thread);
        }
    }

    /// Deep-scan all yarn columns (and balloon columns) for a matching patch.
    /// Unlike process_one, this ignores queue order — it searches ALL patches
    /// in each column, not just the front.
    ///
    /// Uses BFS across columns: checks depth 0 (front) of ALL columns first,
    /// then depth 1 of all columns, etc. This ensures the visually closest
    /// match across all columns is consumed first, rather than exhausting
    /// one column before checking the next.
    /// Locked patches are skipped entirely.
    pub fn deep_scan_process(&mut self, thread: &mut Thread) {
        let max_len = self.board.iter()
            .map(|c| c.len()).max().unwrap_or(0);

        for depth in 0..max_len {
            // Check regular columns at this depth
            for col_idx in 0..self.board.len() {
                let col = &self.board[col_idx];
                if col.len() <= depth { continue; }
                let pos = col.len() - 1 - depth;
                if !col[pos].locked && col[pos].color == thread.color {
                    self.board[col_idx].remove(pos);
                    thread.knit_on();
                    return;
                }
            }
            // Balloon slots are flat (exposed patches) — check at depth 0 only
            if depth == 0 {
                for slot in &mut self.balloon_columns {
                    if let Some(patch) = slot {
                        if !patch.locked && patch.color == thread.color {
                            *slot = None;
                            thread.knit_on();
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
        for offset in 0..(self.visible_patches as usize) {
            let true_offset: usize = (self.visible_patches as usize) - offset;
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
        assert_eq!(yarn.visible_patches, 5);
        assert_eq!(yarn.board.len(), 3);
    }

    #[test]
    fn test_yarn_creation_with_colors() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 5);

        let counter = ColorCounter { color_hashmap: map };
        let yarn = Yarn::make_from_color_counter(counter, 2, 3);

        let total_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(total_patches, 5);
    }

    #[test]
    fn test_yarn_dimensions() {
        let mut map = HashMap::new();
        map.insert(Color::Blue, 10);

        let counter = ColorCounter { color_hashmap: map };
        let yarn = Yarn::make_from_color_counter(counter, 4, 6);

        assert_eq!(yarn.board.len(), 4);
        assert_eq!(yarn.yarn_lines, 4);
        assert_eq!(yarn.visible_patches, 6);
    }

    #[test]
    fn test_process_one_removes_matching_patch() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);

        let counter = ColorCounter { color_hashmap: map };
        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut thread = Thread { color: Color::Red, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total - 1);
        assert_eq!(thread.status, 2);
    }

    #[test]
    fn test_process_one_no_matching_color() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 2);

        let counter = ColorCounter { color_hashmap: map };
        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total);
        assert_eq!(thread.status, 1);
    }

    #[test]
    fn test_process_one_locked_patch_no_key() {
        // A locked patch should block the column even when color matches.
        let mut yarn = Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Red, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        // Should not have processed: patch still there, status unchanged.
        assert_eq!(yarn.board[0].len(), 1);
        assert_eq!(thread.status, 1);
    }

    #[test]
    fn test_process_one_locked_patch_with_key() {
        // A key thread should clear a locked patch of matching color.
        let mut yarn = Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Red, status: 1, has_key: true };
        yarn.process_one(&mut thread);

        assert_eq!(yarn.board[0].len(), 0);
        assert_eq!(thread.status, 2);
        assert!(!thread.has_key); // key consumed
    }

    #[test]
    fn test_process_one_locked_patch_wrong_color_with_key() {
        // Key doesn't help if colors don't match.
        let mut yarn = Yarn {
            board: vec![vec![Patch { color: Color::Blue, locked: true }]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Red, status: 1, has_key: true };
        yarn.process_one(&mut thread);

        assert_eq!(yarn.board[0].len(), 1);
        assert_eq!(thread.status, 1);
        assert!(thread.has_key); // key NOT consumed on wrong color
    }

    #[test]
    fn test_process_sequence_multiple_threads() {
        // Use a deterministic board to avoid the flakiness of random distribution.
        // Col 0: bottom=[Red, Blue, Red]=top  → thread[0] pops Red, thread[1] pops Blue, thread[2] pops Red
        let mut yarn = Yarn {
            board: vec![
                vec![
                    Patch { color: Color::Red,  locked: false }, // bottom
                    Patch { color: Color::Blue, locked: false },
                    Patch { color: Color::Red,  locked: false }, // top (last)
                ],
                vec![],
            ],
            yarn_lines: 2,
            visible_patches: 5,
            balloon_columns: Vec::new(),
        };

        let mut threads = vec![
            Thread { color: Color::Red,  status: 1, has_key: false },
            Thread { color: Color::Blue, status: 1, has_key: false },
            Thread { color: Color::Red,  status: 1, has_key: false },
        ];

        yarn.process_sequence(&mut threads);

        assert_eq!(threads[0].status, 2);
        assert_eq!(threads[1].status, 2);
        assert_eq!(threads[2].status, 2);
    }

    #[test]
    fn test_process_sequence_empty_threads() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 5);

        let counter = ColorCounter { color_hashmap: map };
        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut threads: Vec<Thread> = vec![];
        yarn.process_sequence(&mut threads);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total);
    }

    #[test]
    fn test_deep_scan_process_finds_match_behind_front() {
        // Col 0: [Blue(bottom), Red(top)] — front is Red, but thread is Blue
        // Deep scan should find Blue behind Red and remove it
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Blue, locked: false },  // bottom (index 0)
                Patch { color: Color::Red, locked: false },   // top (index 1, front)
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 2); // knitted once
        assert_eq!(yarn.board[0].len(), 1); // Blue removed, Red remains
        assert_eq!(yarn.board[0][0].color, Color::Red); // Red is still there
    }

    #[test]
    fn test_deep_scan_process_no_match() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Green, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 1); // no change
        assert_eq!(yarn.board[0].len(), 1);
    }

    #[test]
    fn test_deep_scan_checks_balloon_columns() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: vec![
                Some(Patch { color: Color::Blue, locked: false }),
            ],
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 2);
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
                    Patch { color: Color::Blue, locked: false },  // depth 1
                    Patch { color: Color::Red, locked: false },   // depth 0 (front)
                ],
                vec![
                    Patch { color: Color::Red, locked: false },   // depth 1
                    Patch { color: Color::Blue, locked: false },  // depth 0 (front)
                ],
            ],
            yarn_lines: 2,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 2);
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
                    Patch { color: Color::Blue, locked: true },   // depth 0, locked
                ],
                vec![
                    Patch { color: Color::Blue, locked: false },  // depth 1
                    Patch { color: Color::Red, locked: false },   // depth 0 (front)
                ],
            ],
            yarn_lines: 2,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 2);
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
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: vec![
                Some(Patch { color: Color::Blue, locked: false }),
            ],
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        // Should match against balloon slot, not regular column
        assert_eq!(thread.status, 2);
        assert!(yarn.balloon_columns[0].is_none());
        assert_eq!(yarn.board[0].len(), 1); // regular column unchanged
    }

    #[test]
    fn test_process_one_prefers_regular_over_balloon() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: vec![
                Some(Patch { color: Color::Red, locked: false }),
            ],
        };

        let mut thread = Thread { color: Color::Red, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        // Regular columns checked first
        assert_eq!(thread.status, 2);
        assert_eq!(yarn.board[0].len(), 0); // regular consumed
        assert!(yarn.balloon_columns[0].is_some()); // balloon untouched
    }
}
