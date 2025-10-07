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
            if let _ = column.pop_if(closure){
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

