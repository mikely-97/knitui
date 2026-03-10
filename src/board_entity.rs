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

pub struct GeneratorData {
    /// Color displayed on the generator cell itself.
    pub color: Color,
    /// Which adjacent cell is the output cell.
    pub output_dir: Direction,
    /// Remaining threads to generate; front of vec = next to produce.
    pub queue: Vec<Color>,
}

pub enum BoardEntity {
    /// A normal selectable thread.
    Thread(Color),
    /// A thread that carries a key — unlocks a matching locked yarn patch.
    KeyThread(Color),
    /// Impassable obstacle; never becomes Void.
    Obstacle,
    /// Empty cell — makes orthogonal neighbors selectable.
    Void,
    /// Produces threads in its output cell until its queue is exhausted.
    Generator(GeneratorData),
    /// A depleted generator; acts like an Obstacle.
    DepletedGenerator,
}

impl fmt::Display for BoardEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BoardEntity::Thread(color)     => 'T'.with(*color),
                BoardEntity::KeyThread(color)  => 'K'.with(*color),
                BoardEntity::Obstacle          => 'X'.stylize(),
                BoardEntity::Void              => ' '.stylize(),
                BoardEntity::Generator(data)   => match data.output_dir {
                    Direction::Up    => '^',
                    Direction::Down  => 'V',
                    Direction::Left  => '<',
                    Direction::Right => '>',
                }.with(data.color),
                BoardEntity::DepletedGenerator => '#'.stylize(),
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
    fn test_board_entity_key_thread_creation() {
        let entity = BoardEntity::KeyThread(Color::Blue);
        match entity {
            BoardEntity::KeyThread(color) => assert_eq!(color, Color::Blue),
            _ => panic!("Expected KeyThread variant"),
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
    fn test_board_entity_generator_creation() {
        let data = GeneratorData {
            color: Color::Green,
            output_dir: Direction::Down,
            queue: vec![Color::Green, Color::Green],
        };
        let entity = BoardEntity::Generator(data);
        match entity {
            BoardEntity::Generator(d) => {
                assert_eq!(d.color, Color::Green);
                assert_eq!(d.output_dir, Direction::Down);
                assert_eq!(d.queue.len(), 2);
            }
            _ => panic!("Expected Generator variant"),
        }
    }

    #[test]
    fn test_board_entity_depleted_generator_creation() {
        let entity = BoardEntity::DepletedGenerator;
        match entity {
            BoardEntity::DepletedGenerator => {},
            _ => panic!("Expected DepletedGenerator variant"),
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
        let thread = BoardEntity::Thread(Color::Blue);
        let key    = BoardEntity::KeyThread(Color::Red);
        let obs    = BoardEntity::Obstacle;
        let void   = BoardEntity::Void;
        let dep    = BoardEntity::DepletedGenerator;
        let generator = BoardEntity::Generator(GeneratorData {
            color: Color::Cyan,
            output_dir: Direction::Right,
            queue: vec![],
        });

        // Verify Display doesn't panic for any variant
        let _ = format!("{}", thread);
        let _ = format!("{}", key);
        let _ = format!("{}", obs);
        let _ = format!("{}", void);
        let _ = format!("{}", dep);
        let _ = format!("{}", generator);
    }
}
