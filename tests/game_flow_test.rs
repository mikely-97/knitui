// Integration test for game flow
use knitui::game_board::GameBoard;
use knitui::palette::{select_palette, ColorMode};
use knitui::active_threads::Thread;
use knitui::yarn::Yarn;
use crossterm::style::Color;

#[test]
fn test_full_game_flow() {
    let palette = select_palette(ColorMode::Dark, 3);
    assert_eq!(palette.len(), 3);

    let board = GameBoard::make_random(4, 4, &palette, 20, 2);
    assert_eq!(board.height, 4);
    assert_eq!(board.width, 4);

    let counter = board.count_knits();
    let mut yarn = Yarn::make_from_color_counter(counter, 3, 5);
    assert_eq!(yarn.yarn_lines, 3);

    let mut threads = vec![
        Thread { color: palette[0], status: 1, has_key: false },
        Thread { color: palette[1], status: 1, has_key: false },
    ];

    yarn.process_sequence(&mut threads);

    for thread in &threads {
        assert!(thread.status >= 1);
    }
}

#[test]
fn test_board_and_yarn_consistency() {
    let palette = vec![Color::Red];
    let board = GameBoard::make_random(3, 3, &palette, 0, 1);

    let counter = board.count_knits();
    let red_count = *counter.color_hashmap.get(&Color::Red).unwrap_or(&0);
    assert!(red_count >= 8 && red_count <= 9);

    let yarn = Yarn::make_from_color_counter(counter, 3, 5);
    let total_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
    assert_eq!(total_patches as u16, red_count);
}

#[test]
fn test_complete_thread_processing_workflow() {
    let palette = vec![Color::Blue];
    let board = GameBoard::make_random(2, 2, &palette, 0, 3);

    let counter = board.count_knits();
    let mut yarn = Yarn::make_from_color_counter(counter, 2, 4);

    let mut thread = Thread {
        color: Color::Blue,
        status: 1,
        has_key: false,
    };

    let initial_patches: usize = yarn.board.iter().map(|col| col.len()).sum();

    yarn.process_one(&mut thread);
    assert_eq!(thread.status, 2);

    yarn.process_one(&mut thread);
    assert_eq!(thread.status, 3);

    yarn.process_one(&mut thread);
    assert_eq!(thread.status, 4);

    let final_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
    assert_eq!(initial_patches - final_patches, 3);
}

#[test]
fn test_multiple_palettes_work() {
    for mode in [ColorMode::Dark, ColorMode::Bright, ColorMode::Colorblind] {
        let palette = select_palette(mode, 4);
        let board = GameBoard::make_random(3, 3, &palette, 10, 2);
        let counter = board.count_knits();
        let yarn = Yarn::make_from_color_counter(counter, 3, 4);

        assert_eq!(yarn.yarn_lines, 3);
    }
}

#[test]
fn test_high_obstacle_board_still_works() {
    let palette = vec![Color::Green, Color::Yellow];
    let board = GameBoard::make_random(5, 5, &palette, 80, 2);

    let counter = board.count_knits();
    let yarn = Yarn::make_from_color_counter(counter, 2, 3);

    assert!(yarn.board.len() > 0);
}

#[test]
fn test_knit_volume_affects_total_yarn_patches() {
    use knitui::board_entity::BoardEntity;

    // Use two identical deterministic boards differing only in knit_volume so
    // that thread-count variance from random generation can't affect the ratio.
    let make_board = |knit_volume: u16| GameBoard {
        board: vec![
            vec![BoardEntity::Thread(Color::Magenta), BoardEntity::Thread(Color::Magenta)],
            vec![BoardEntity::Thread(Color::Magenta), BoardEntity::Thread(Color::Magenta)],
        ],
        height: 2,
        width: 2,
        knit_volume,
    };

    let board1 = make_board(1);
    let board3 = make_board(3);

    let total1: u16 = board1.count_knits().color_hashmap.values().sum();
    let total3: u16 = board3.count_knits().color_hashmap.values().sum();

    // 4 threads × 1 = 4,  4 threads × 3 = 12, ratio exactly 3.
    assert_eq!(total1, 4);
    assert_eq!(total3, 12);
    assert_eq!(total3 / total1, 3);
}
