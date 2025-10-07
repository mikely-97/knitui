// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub struct Thread {
    pub color: Color,
    pub status: u16,
}

impl Thread{
    pub fn knit_on(&mut self){
        self.status += 1;
    }
}

impl fmt::Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.status {
                1 => '0'.with(self.color),
                2 => '1'.with(self.color),
                3 => '2'.with(self.color),
                _ => '?'.with(self.color),
            }
        )
    }
}


