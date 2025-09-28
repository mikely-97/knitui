// ./src/lib/game_board.rs
use crossterm::style::Color;
use crate::board_entity::BoardEntity;
use rand::prelude::*;

pub struct GameBoard{
    pub board: Vec<Vec<BoardEntity>>,
    pub height: u16,
    pub width: u16
}

impl GameBoard {
    pub fn make_random(
        height: u16, 
        width: u16,
        selected_palette: &Vec<Color>,
        obstacle_percentage: u16,
    ) -> Self {
        let mut rng = rand::rng();
        let mut board: Vec<Vec<BoardEntity>> = Vec::new();
        for _ in 0..height{
            let mut row: Vec<BoardEntity> = Vec::new();
            for _ in 0..width{
                if rng.random_range(0..=100) <= obstacle_percentage{
                    row.push(BoardEntity::Obstacle);
                }
                else {
                    row.push(BoardEntity::Thread(*selected_palette.choose(& mut rng).unwrap()));
                }
            }
            board.push(row);
        }
        Self { board: board, height: height, width: width }
    }
}



