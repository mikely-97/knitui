// ./src/lib/game_board.rs
use crossterm::style::Color;
use crate::board_entity::BoardEntity;

pub fn make_game_board() -> Vec<Vec<BoardEntity>>{
    // TODO: assert the horizontal size is the same everywhere
    vec![
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Obstacle,
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
        vec![
        BoardEntity::Thread(Color::Red),
        BoardEntity::Thread(Color::Magenta),
        BoardEntity::Thread(Color::Blue),
        BoardEntity::Thread(Color::Yellow),
        BoardEntity::Thread(Color::Green),
        ],
    ]
}


