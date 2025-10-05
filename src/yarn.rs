use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;
use std::collections::HashMap;
use crate::color_counter::ColorCounter;

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
}

impl fmt::Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for offset in 1..=(self.visible_patches as usize){
            for column in &self.board{
                if !(offset > column.len()){
                    let pos_to_print = column.len() - offset;
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

