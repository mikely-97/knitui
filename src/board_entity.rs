// ./src/lib/board_entity.rs

use crossterm::style::{
    Color,
    Stylize
};

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns (row_delta, col_delta) for this direction.
    pub fn offset(self) -> (i32, i32) {
        match self {
            Direction::Up    => (-1,  0),
            Direction::Down  => ( 1,  0),
            Direction::Left  => ( 0, -1),
            Direction::Right => ( 0,  1),
        }
    }
}

pub struct ConveyorData {
    /// Color displayed on the conveyor cell itself.
    pub color: Color,
    /// Which adjacent cell is the output cell.
    pub output_dir: Direction,
    /// Remaining spools to generate; front of vec = next to produce.
    pub queue: Vec<Color>,
}

pub enum BoardEntity {
    /// A normal selectable spool.
    Spool(Color),
    /// A spool that carries a key — unlocks a matching locked yarn stitch.
    KeySpool(Color),
    /// Impassable obstacle; never becomes Void.
    Obstacle,
    /// Empty cell — makes orthogonal neighbors selectable.
    Void,
    /// Produces spools in its output cell until its queue is exhausted.
    Conveyor(ConveyorData),
    /// A depleted conveyor; acts like an Obstacle.
    EmptyConveyor,
}

impl fmt::Display for BoardEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BoardEntity::Spool(color)     => 'T'.with(*color),
                BoardEntity::KeySpool(color)  => 'K'.with(*color),
                BoardEntity::Obstacle         => 'X'.stylize(),
                BoardEntity::Void             => ' '.stylize(),
                BoardEntity::Conveyor(data)   => match data.output_dir {
                    Direction::Up    => '^',
                    Direction::Down  => 'V',
                    Direction::Left  => '<',
                    Direction::Right => '>',
                }.with(data.color),
                BoardEntity::EmptyConveyor    => '#'.stylize(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_entity_spool_creation() {
        let entity = BoardEntity::Spool(Color::Red);
        match entity {
            BoardEntity::Spool(color) => assert_eq!(color, Color::Red),
            _ => panic!("Expected Spool variant"),
        }
    }

    #[test]
    fn test_board_entity_key_spool_creation() {
        let entity = BoardEntity::KeySpool(Color::Blue);
        match entity {
            BoardEntity::KeySpool(color) => assert_eq!(color, Color::Blue),
            _ => panic!("Expected KeySpool variant"),
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
    fn test_board_entity_conveyor_creation() {
        let data = ConveyorData {
            color: Color::Green,
            output_dir: Direction::Down,
            queue: vec![Color::Green, Color::Green],
        };
        let entity = BoardEntity::Conveyor(data);
        match entity {
            BoardEntity::Conveyor(d) => {
                assert_eq!(d.color, Color::Green);
                assert_eq!(d.output_dir, Direction::Down);
                assert_eq!(d.queue.len(), 2);
            }
            _ => panic!("Expected Conveyor variant"),
        }
    }

    #[test]
    fn test_board_entity_empty_conveyor_creation() {
        let entity = BoardEntity::EmptyConveyor;
        match entity {
            BoardEntity::EmptyConveyor => {},
            _ => panic!("Expected EmptyConveyor variant"),
        }
    }

    #[test]
    fn test_direction_offsets() {
        assert_eq!(Direction::Up.offset(),    (-1,  0));
        assert_eq!(Direction::Down.offset(),  ( 1,  0));
        assert_eq!(Direction::Left.offset(),  ( 0, -1));
        assert_eq!(Direction::Right.offset(), ( 0,  1));
    }

    #[test]
    fn test_board_entity_display_format() {
        let spool   = BoardEntity::Spool(Color::Blue);
        let key     = BoardEntity::KeySpool(Color::Red);
        let obs     = BoardEntity::Obstacle;
        let void    = BoardEntity::Void;
        let dep     = BoardEntity::EmptyConveyor;
        let conveyor = BoardEntity::Conveyor(ConveyorData {
            color: Color::Cyan,
            output_dir: Direction::Right,
            queue: vec![],
        });

        // Verify Display doesn't panic for any variant
        let _ = format!("{}", spool);
        let _ = format!("{}", key);
        let _ = format!("{}", obs);
        let _ = format!("{}", void);
        let _ = format!("{}", dep);
        let _ = format!("{}", conveyor);
    }
}
