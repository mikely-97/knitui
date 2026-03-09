// Integration tests for edge cases
use knitui::game_board::GameBoard;
use knitui::palette::{select_palette, ColorMode};
use knitui::active_threads::Thread;
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

    let mut thread = Thread {
        color: Color::Red,
        status: 1,
        has_key: false,
    };

    yarn.process_one(&mut thread);

    assert_eq!(thread.status, 1);
}

#[test]
fn test_single_color_board() {
    let palette = vec![Color::Cyan];
    let board = GameBoard::make_random(10, 10, &palette, 0, 1);

    let counter = board.count_knits();
    assert_eq!(counter.color_hashmap.len(), 1);
    assert!(counter.color_hashmap.contains_key(&Color::Cyan));
}

#[test]
fn test_large_board() {
    let palette = select_palette(ColorMode::Dark, 6);
    let board = GameBoard::make_random(20, 20, &palette, 15, 3);

    assert_eq!(board.height, 20);
    assert_eq!(board.width, 20);

    let counter = board.count_knits();
    let yarn = Yarn::make_from_color_counter(counter, 10, 8);

    assert_eq!(yarn.yarn_lines, 10);
}

#[test]
fn test_thread_processing_beyond_yarn_capacity() {
    let mut map = HashMap::new();
    map.insert(Color::Yellow, 2);

    let counter = ColorCounter { color_hashmap: map };
    let mut yarn = Yarn::make_from_color_counter(counter, 2, 3);

    let mut thread = Thread {
        color: Color::Yellow,
        status: 1,
        has_key: false,
    };

    for _ in 0..5 {
        yarn.process_one(&mut thread);
    }

    assert!(thread.status <= 3);
}

#[test]
fn test_many_threads_same_color() {
    let mut map = HashMap::new();
    map.insert(Color::Green, 10);

    let counter = ColorCounter { color_hashmap: map };
    let mut yarn = Yarn::make_from_color_counter(counter, 3, 5);

    let mut threads = vec![];
    for _ in 0..10 {
        threads.push(Thread {
            color: Color::Green,
            status: 1,
            has_key: false,
        });
    }

    yarn.process_sequence(&mut threads);

    for thread in &threads {
        assert_eq!(thread.status, 2);
    }

    let remaining: usize = yarn.board.iter().map(|col| col.len()).sum();
    assert_eq!(remaining, 0);
}

#[test]
fn test_mixed_thread_colors() {
    let mut map = HashMap::new();
    map.insert(Color::Red, 3);
    map.insert(Color::Blue, 3);
    map.insert(Color::Green, 3);

    let counter = ColorCounter { color_hashmap: map };
    let mut yarn = Yarn::make_from_color_counter(counter, 3, 5);

    let mut threads = vec![
        Thread { color: Color::Red,   status: 1, has_key: false },
        Thread { color: Color::Blue,  status: 1, has_key: false },
        Thread { color: Color::Green, status: 1, has_key: false },
        Thread { color: Color::Red,   status: 1, has_key: false },
    ];

    let initial_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
    yarn.process_sequence(&mut threads);

    let final_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
    let patches_removed = initial_patches - final_patches;

    assert!(patches_removed >= 3 && patches_removed <= 4);
}

#[test]
fn test_zero_knit_volume() {
    let palette = vec![Color::White];
    let board = GameBoard::make_random(3, 3, &palette, 0, 0);

    let counter = board.count_knits();

    let total: u16 = counter.color_hashmap.values().sum();
    assert_eq!(total, 0);
}

#[test]
fn test_very_high_knit_volume() {
    let palette = vec![Color::DarkRed];
    let board = GameBoard::make_random(2, 2, &palette, 0, 100);

    let counter = board.count_knits();

    assert_eq!(*counter.color_hashmap.get(&Color::DarkRed).unwrap(), 400);
}

#[test]
fn test_narrow_board() {
    let palette = vec![Color::Blue, Color::Red];
    let board = GameBoard::make_random(10, 1, &palette, 0, 2);

    assert_eq!(board.width, 1);
    assert_eq!(board.height, 10);
    assert_eq!(board.board[0].len(), 1);
}

#[test]
fn test_wide_board() {
    let palette = vec![Color::Cyan];
    let board = GameBoard::make_random(1, 15, &palette, 0, 1);

    assert_eq!(board.width, 15);
    assert_eq!(board.height, 1);
    assert_eq!(board.board[0].len(), 15);
}
