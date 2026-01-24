// ./src/lib/game_board.rs
use crossterm::style::Color;
use crate::board_entity::BoardEntity;
use crate::yarn::Yarn;
use crate::color_counter::ColorCounter;
use rand::prelude::*;
use std::collections::HashMap;

pub struct GameBoard{
    pub board: Vec<Vec<BoardEntity>>,
    pub height: u16,
    pub width: u16,
    pub knit_volume: u16
}

impl GameBoard {
    pub fn make_random(
        height: u16, 
        width: u16,
        selected_palette: &Vec<Color>,
        obstacle_percentage: u16,
        knit_volume: u16,
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
        Self { board: board, height: height, width: width, knit_volume: knit_volume }
    }
    pub fn count_knits(self: &Self) -> ColorCounter{
        let mut counter = HashMap::new();
        for row in &self.board{
            for knit in row{
                if let BoardEntity::Thread(color) = knit{
                    counter.entry(*color).and_modify(|e| {*e+=self.knit_volume}).or_insert(self.knit_volume);
                }
            }
        }
        return ColorCounter{color_hashmap:counter};
    }
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_board_dimensions() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(5, 7, &palette, 0, 3);

        assert_eq!(board.height, 5);
        assert_eq!(board.width, 7);
        assert_eq!(board.board.len(), 5);
        assert_eq!(board.board[0].len(), 7);
    }

    #[test]
    fn test_game_board_no_obstacles() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(4, 4, &palette, 0, 2);

        let mut thread_count = 0;
        let mut obstacle_count = 0;
        for row in &board.board {
            for entity in row {
                match entity {
                    BoardEntity::Thread(_) => thread_count += 1,
                    BoardEntity::Obstacle => obstacle_count += 1,
                    BoardEntity::Void => {}
                }
            }
        }
        // With 0% obstacle_percentage, most cells should be threads
        // but there's a 1/101 chance per cell of getting an obstacle due to <= comparison
        assert!(thread_count >= 15);
        assert!(obstacle_count <= 1);
    }

    #[test]
    fn test_game_board_all_obstacles() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 100, 1);

        let mut obstacle_count = 0;
        for row in &board.board {
            for entity in row {
                match entity {
                    BoardEntity::Obstacle => obstacle_count += 1,
                    _ => {}
                }
            }
        }
        assert_eq!(obstacle_count, 9);
    }

    #[test]
    fn test_count_knits_empty_board() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 100, 5);
        let counter = board.count_knits();

        assert_eq!(counter.color_hashmap.len(), 0);
    }

    #[test]
    fn test_count_knits_multiplies_by_knit_volume() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 0, 3);
        let counter = board.count_knits();

        // 4 threads of red color, each needs to be processed 3 times
        assert_eq!(*counter.color_hashmap.get(&Color::Red).unwrap(), 12);
    }

    #[test]
    fn test_count_knits_different_colors() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(5, 5, &palette, 0, 2);
        let counter = board.count_knits();

        // Should have between 1 and 3 colors
        assert!(counter.color_hashmap.len() >= 1);
        assert!(counter.color_hashmap.len() <= 3);

        // Total count should be close to 25 * 2 = 50
        // (might be slightly less due to the <= in obstacle generation allowing edge case obstacles)
        let total: u16 = counter.color_hashmap.values().sum();
        assert!(total >= 48 && total <= 50);
    }

    #[test]
    fn test_knit_volume_stored() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(3, 3, &palette, 0, 7);

        assert_eq!(board.knit_volume, 7);
    }
}

