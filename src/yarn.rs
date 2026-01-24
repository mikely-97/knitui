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
    color: Color
}

impl fmt::Display for Patch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            '▦'.with(self.color)
        )
    }
}

pub struct Yarn {
    pub board: Vec<Vec<Patch>>,
    pub yarn_lines: u16,
    pub visible_patches: u16,
}

impl Yarn {
    pub fn make_from_color_counter(counter: ColorCounter, yarn_lines: u16, visible_patches: u16,) -> Self{
        // init the board itself
        let mut board: Vec<Vec<Patch>> = Vec::new();
        for _ in 0..yarn_lines{
            let mut row: Vec<Patch> = Vec::new();
            board.push(row);
        }
        // get the queue of patches
        let shuffled_queue: Vec<Color> = counter.get_shuffled_queue();
        // actually fill the board
        for color in shuffled_queue.iter(){
            let column_number = rand::random::<u16>()%yarn_lines;
            board[(column_number as usize)].push(Patch { color: *color });
        }
        Self { board: board, yarn_lines: yarn_lines, visible_patches: visible_patches }
    }

    pub fn process_one(&mut self, thread: &mut Thread){
        for column in &mut self.board {
            let closure = |x: &mut Patch| x.color == thread.color;
            if let Some(_) = column.pop_if(closure){
                thread.knit_on();
                break;
            }
        }
    }

    pub fn process_sequence(&mut self, threads: &mut Vec<Thread>){
        for thread in threads{
            self.process_one(thread);
        }
    }

}

impl fmt::Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for offset in 0..(self.visible_patches as usize){
            let true_offset: usize = (self.visible_patches as usize)-offset;
            for column in &self.board{
                if !(true_offset > column.len()){
                    let pos_to_print = column.len() - true_offset;
                    write!(
                        f,
                        "{}",
                        column[pos_to_print]
                    )?;
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

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let yarn = Yarn::make_from_color_counter(counter, 2, 3);

        // Count total patches
        let total_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(total_patches, 5);
    }

    #[test]
    fn test_yarn_dimensions() {
        let mut map = HashMap::new();
        map.insert(Color::Blue, 10);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let yarn = Yarn::make_from_color_counter(counter, 4, 6);

        assert_eq!(yarn.board.len(), 4); // Should have 4 columns
        assert_eq!(yarn.yarn_lines, 4);
        assert_eq!(yarn.visible_patches, 6);
    }

    #[test]
    fn test_process_one_removes_matching_patch() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut thread = Thread {
            color: Color::Red,
            status: 1,
        };

        yarn.process_one(&mut thread);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total - 1);
        assert_eq!(thread.status, 2); // Status should be incremented
    }

    #[test]
    fn test_process_one_no_matching_color() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 2);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut thread = Thread {
            color: Color::Blue, // Different color
            status: 1,
        };

        yarn.process_one(&mut thread);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total); // No change
        assert_eq!(thread.status, 1); // Status unchanged
    }

    #[test]
    fn test_process_sequence_multiple_threads() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 5);
        map.insert(Color::Blue, 5);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let mut yarn = Yarn::make_from_color_counter(counter, 2, 5);

        let mut threads = vec![
            Thread { color: Color::Red, status: 1 },
            Thread { color: Color::Blue, status: 1 },
            Thread { color: Color::Red, status: 1 },
        ];

        yarn.process_sequence(&mut threads);

        // All threads should have incremented status (enough patches available)
        assert_eq!(threads[0].status, 2);
        assert_eq!(threads[1].status, 2);
        assert_eq!(threads[2].status, 2);
    }

    #[test]
    fn test_process_sequence_empty_threads() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 5);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);
        let initial_total: usize = yarn.board.iter().map(|col| col.len()).sum();

        let mut threads: Vec<Thread> = vec![];
        yarn.process_sequence(&mut threads);

        let final_total: usize = yarn.board.iter().map(|col| col.len()).sum();
        assert_eq!(final_total, initial_total); // No change
    }

    #[test]
    fn test_yarn_display_format() {
        let mut map = HashMap::new();
        map.insert(Color::Red, 3);

        let counter = ColorCounter {
            color_hashmap: map,
        };

        let yarn = Yarn::make_from_color_counter(counter, 2, 3);

        // Just verify that format! doesn't panic
        let _ = format!("{}", yarn);
    }
}
