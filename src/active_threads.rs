// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub struct Thread {
    pub color: Color,
    pub status: u8,
}

impl fmt::Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.status {
                1 => '1'.with(self.color),
                2 => '2'.with(self.color),
                3 => '3'.with(self.color),
                _ => '?'.with(self.color),
            }
        )
    }
}


