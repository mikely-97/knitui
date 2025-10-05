// ./src/lib/game_board.rs
use crossterm::style::Color;
use crate::board_entity::BoardEntity;
use crate::yarn::Yarn;
use rand::prelude::*;
use std::collections::HashMap;

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
    pub fn count_knits(self: &Self) -> HashMap<&Color, u16>{
        let mut counter = HashMap::new();
        for row in &self.board{
            for knit in row{
                if let BoardEntity::Thread(color) = knit{
                    counter.entry(color).and_modify(|e| {*e+=1}).or_insert(1);
                }
            }
        }
        return counter;
    }
    }


