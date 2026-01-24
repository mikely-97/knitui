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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_entity_thread_creation() {
        let entity = BoardEntity::Thread(Color::Red);
        match entity {
            BoardEntity::Thread(color) => assert_eq!(color, Color::Red),
            _ => panic!("Expected Thread variant"),
        }
    }

    #[test]
    fn test_board_entity_obstacle_creation() {
        let entity = BoardEntity::Obstacle;
        match entity {
            BoardEntity::Obstacle => {},
            _ => panic!("Expected Obstacle variant"),
        }
    }

    #[test]
    fn test_board_entity_void_creation() {
        let entity = BoardEntity::Void;
        match entity {
            BoardEntity::Void => {},
            _ => panic!("Expected Void variant"),
        }
    }

    #[test]
    fn test_board_entity_display_format() {
        // Test that Display trait is implemented (will output styled characters)
        let thread = BoardEntity::Thread(Color::Blue);
        let obstacle = BoardEntity::Obstacle;
        let void = BoardEntity::Void;

        // Just verify that format! doesn't panic
        let _ = format!("{}", thread);
        let _ = format!("{}", obstacle);
        let _ = format!("{}", void);
    }
}