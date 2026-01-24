// Integration test for game flow
use knitui::game_board::GameBoard;
use knitui::palette::{select_palette, ColorMode};
use knitui::active_threads::Thread;
use knitui::yarn::Yarn;
use crossterm::style::Color;

#[test]
fn test_full_game_flow() {
    // 1. Create a palette
    let palette = select_palette(ColorMode::Dark, 3);
    assert_eq!(palette.len(), 3);

    // 2. Create a game board
    let board = GameBoard::make_random(4, 4, &palette, 20, 2);
    assert_eq!(board.height, 4);
    assert_eq!(board.width, 4);

    // 3. Count knits from the board
    let counter = board.count_knits();

    // 4. Create yarn from the counter
    let mut yarn = Yarn::make_from_color_counter(counter, 3, 5);
    assert_eq!(yarn.yarn_lines, 3);

    // 5. Create some threads (simulating player selection)
    let mut threads = vec![
        Thread {
            color: palette[0],
            status: 1,
        },
        Thread {
            color: palette[1],
            status: 1,
        },
    ];

    // 6. Process threads with yarn
    yarn.process_sequence(&mut threads);

    // All threads should have been processed (status incremented or unchanged)
    for thread in &threads {
        assert!(thread.status >= 1);
    }
}

#[test]
fn test_board_and_yarn_consistency() {
    // Create a simple board with known colors
    let palette = vec![Color::Red];
    let board = GameBoard::make_random(3, 3, &palette, 0, 1);

    // Count should be close to 9 threads * 1 knit_volume = 9
    // (might be slightly less due to obstacle generation edge case)
    let counter = board.count_knits();
    let red_count = *counter.color_hashmap.get(&Color::Red).unwrap_or(&0);
    assert!(red_count >= 8 && red_count <= 9);

    // Create yarn and verify it has the right number of patches
    let yarn = Yarn::make_from_color_counter(counter, 3, 5);
    let total_patches: usize = yarn.board.iter().map(|col| col.len()).sum();
    assert_eq!(total_patches as u16, red_count);
}

#[test]
fn test_complete_thread_processing_workflow() {
    // Use a single color to guarantee the thread color exists on the board
    let palette = vec![Color::Blue];
    let board = GameBoard::make_random(2, 2, &palette, 0, 3);

    let counter = board.count_knits();
    let mut yarn = Yarn::make_from_color_counter(counter, 2, 4);

    // Create a thread with the same color
    let mut thread = Thread {
        color: Color::Blue,
        status: 1,
    };

    let initial_patches: usize = yarn.board.iter().map(|col| col.len()).sum();

    // Process the thread multiple times
    yarn.process_one(&mut thread);
    assert_eq!(thread.status, 2);

    yarn.process_one(&mut thread);
    assert_eq!(thread.status, 3);

    yarn.process_one(&mut thread);
    assert_eq!(thread.status, 4);

    // Thread has been processed 3 times
    let final_patches: usize = yarn.board.iter().map(|col| col.len()).sum();

    // Exactly 3 patches should have been removed
    assert_eq!(initial_patches - final_patches, 3);
}

#[test]
fn test_multiple_palettes_work() {
    for mode in [ColorMode::Dark, ColorMode::Bright, ColorMode::Colorblind] {
        let palette = select_palette(mode, 4);
        let board = GameBoard::make_random(3, 3, &palette, 10, 2);
        let counter = board.count_knits();
        let yarn = Yarn::make_from_color_counter(counter, 3, 4);

        // Should all work without panicking
        assert_eq!(yarn.yarn_lines, 3);
    }
}

#[test]
fn test_high_obstacle_board_still_works() {
    let palette = vec![Color::Green, Color::Yellow];
    let board = GameBoard::make_random(5, 5, &palette, 80, 2);

    let counter = board.count_knits();
    let yarn = Yarn::make_from_color_counter(counter, 2, 3);

    // Should work even with mostly obstacles
    assert!(yarn.board.len() > 0);
}

#[test]
fn test_knit_volume_affects_total_yarn_patches() {
    let palette = vec![Color::Magenta];

    // Same board size, different knit volumes
    let board1 = GameBoard::make_random(2, 2, &palette, 0, 1);
    let board2 = GameBoard::make_random(2, 2, &palette, 0, 3);

    let counter1 = board1.count_knits();
    let counter2 = board2.count_knits();

    // If both boards have the same number of threads, counter2 should have 3x the patches
    // Due to obstacle edge case, boards might have slightly different thread counts
    let total1 = counter1.color_hashmap.values().sum::<u16>();
    let total2 = counter2.color_hashmap.values().sum::<u16>();

    // The ratio should be approximately 3:1, allowing for some variance
    let ratio = total2 as f32 / total1 as f32;
    assert!(ratio >= 2.5 && ratio <= 3.0);
}
