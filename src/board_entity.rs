// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

pub enum BoardEntity {
    Thread(Color),
    Obstacle,
    Void,
}

impl fmt::Display for BoardEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BoardEntity::Thread(color) => 'T'.with(*color),
                BoardEntity::Obstacle => 'X'.stylize(),
                BoardEntity::Void => ' '.stylize()
            }
        )
    }
}