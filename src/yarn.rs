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
        Self { board, yarn_lines, visible_patches }
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
        for column in &mut self.board {
            let Some(last) = column.last() else { continue };

            if last.locked {
                if last.color == thread.color && thread.has_key {
                    column.pop();
                    thread.knit_on();
                    thread.has_key = false;
                    break;
                }
                // Locked patch blocks the column — skip it entirely.
                continue;
            }

            if last.color == thread.color {
                column.pop();
                thread.knit_on();
                break;
            }
        }
    }

    pub fn process_sequence(&mut self, threads: &mut Vec<Thread>) {
        for thread in threads {
            self.process_one(thread);
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
    fn test_yarn_display_format() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);

        let counter = ColorCounter { color_hashmap: map };
        let yarn = Yarn::make_from_color_counter(counter, 2, 3);

        let _ = format!("{}", yarn);
    }
}
