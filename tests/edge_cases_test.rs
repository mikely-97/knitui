// Integration tests for edge cases
use knitui::game_board::GameBoard;
use knitui::palette::{select_palette, ColorMode};
use knitui::spool::Spool;
use knitui::yarn::Yarn;
use knitui::color_counter::ColorCounter;
use crossterm::style::Color;
use std::collections::HashMap;

#[test]
fn test_empty_yarn_processing() {
    let counter = ColorCounter {
        color_hashmap: HashMap::new(),
    };
    let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);

    let mut spool = Spool {
        color: Color::Red,
        fill: 1,
        has_key: false,
    };

    yarn.process_one(&mut spool);

    assert_eq!(spool.fill, 1);
}

#[test]
fn test_single_color_board() {
    let palette = vec![Color::Cyan];
    let board = GameBoard::make_random(10, 10, &palette, 0, 1, 0, 0);

    let counter = board.count_spools();
    assert_eq!(counter.color_hashmap.len(), 1);
    assert!(counter.color_hashmap.contains_key(&Color::Cyan));
}

#[test]
fn test_large_board() {
    let palette = select_palette(ColorMode::Dark, 6);
    let board = GameBoard::make_random(20, 20, &palette, 15, 3, 0, 0);

    assert_eq!(board.height, 20);
    assert_eq!(board.width, 20);

    let counter = board.count_spools();
    let yarn = Yarn::make_from_color_counter(counter, 10, 8);

    assert_eq!(yarn.yarn_lines, 10);
}

#[test]
fn test_spool_processing_beyond_yarn_capacity() {
    let mut map = HashMap::new();
    map.insert(Color::Yellow, 2);

    let counter = ColorCounter { color_hashmap: map };
    let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);

    let mut spool = Spool {
        color: Color::Yellow,
        fill: 1,
        has_key: false,
    };

    for _ in 0..5 {
        yarn.process_one(&mut spool);
    }

    assert!(spool.fill <= 3);
}

#[test]
fn test_many_spools_same_color() {
    let mut map = HashMap::new();
    map.insert(Color::Green, 10);

    let counter = ColorCounter { color_hashmap: map };
    let mut yarn = Yarn::make_from_color_counter(counter, 3, 5);

    let mut spools = vec![];
    for _ in 0..10 {
        spools.push(Spool {
            color: Color::Green,
            fill: 1,
            has_key: false,
        });
    }

    yarn.process_sequence(&mut spools);

    for spool in &spools {
        assert_eq!(spool.fill, 2);
    }

    let remaining: usize = yarn.board.iter().map(|col| col.len()).sum();
    assert_eq!(remaining, 0);
}

#[test]
fn test_mixed_spool_colors() {
    // Deterministic yarn: col0=[Red,Blue,Red], col1=[Green,Blue,Green], col2=[Red]
    // (last element = top of stack, processed first)
    use knitui::yarn::Stitch;
    let mut yarn = Yarn {
        board: vec![
            vec![
                Stitch { color: Color::Red,  locked: false },
                Stitch { color: Color::Blue, locked: false },
                Stitch { color: Color::Red,  locked: false },
            ],
            vec![
                Stitch { color: Color::Green, locked: false },
                Stitch { color: Color::Blue,  locked: false },
                Stitch { color: Color::Green, locked: false },
            ],
            vec![
                Stitch { color: Color::Red, locked: false },
            ],
        ],
        yarn_lines: 3,
        visible_stitches: 5,
        balloon_columns: Vec::new(),
    };

    let mut spools = vec![
        Spool { color: Color::Red,   fill: 1, has_key: false },
        Spool { color: Color::Blue,  fill: 1, has_key: false },
        Spool { color: Color::Green, fill: 1, has_key: false },
        Spool { color: Color::Red,   fill: 1, has_key: false },
    ];

    yarn.process_sequence(&mut spools);

    // Each spool matches exactly once (left-to-right scan):
    // Red  → pops col0 top (Red)
    // Blue → pops col0 top (Blue, was under Red)
    // Green→ pops col1 top (Green)
    // Red  → pops col1 top (Blue? no — col0 top is now Red, pops col0)
    // Total removed: 4
    let remaining: usize = yarn.board.iter().map(|col| col.len()).sum();
    assert_eq!(remaining, 7 - 4);
    assert_eq!(spools[0].fill, 2);
    assert_eq!(spools[1].fill, 2);
    assert_eq!(spools[2].fill, 2);
    assert_eq!(spools[3].fill, 2);
}

#[test]
fn test_zero_spool_capacity() {
    let palette = vec![Color::White];
    let board = GameBoard::make_random(3, 3, &palette, 0, 0, 0, 0);

    let counter = board.count_spools();

    let total: u16 = counter.color_hashmap.values().sum();
    assert_eq!(total, 0);
}

#[test]
fn test_very_high_spool_capacity() {
    let palette = vec![Color::DarkRed];
    let board = GameBoard::make_random(2, 2, &palette, 0, 100, 0, 0);

    let counter = board.count_spools();

    assert_eq!(*counter.color_hashmap.get(&Color::DarkRed).unwrap(), 400);
}

#[test]
fn test_narrow_board() {
    let palette = vec![Color::Blue, Color::Red];
    let board = GameBoard::make_random(10, 1, &palette, 0, 2, 0, 0);

    assert_eq!(board.width, 1);
    assert_eq!(board.height, 10);
    assert_eq!(board.board[0].len(), 1);
}

#[test]
fn test_wide_board() {
    let palette = vec![Color::Cyan];
    let board = GameBoard::make_random(1, 15, &palette, 0, 1, 0, 0);

    assert_eq!(board.width, 15);
    assert_eq!(board.height, 1);
    assert_eq!(board.board[0].len(), 15);
}
