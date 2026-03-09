/// Integration tests for game flow — exercising GameEngine end-to-end.
use knitui::engine::GameEngine;
use knitui::config::Config;
use knitui::board_entity::{BoardEntity, Direction};
use knitui::game_board::GameBoard;
use knitui::palette::{select_palette, ColorMode};
use knitui::yarn::Yarn;
use knitui::active_threads::Thread;
use crossterm::style::Color;

fn make_config(
    board_height: u16, board_width: u16, color_number: u16,
    color_mode: &str, knit_volume: u16, obstacle_percentage: u16,
) -> Config {
    Config {
        board_height, board_width, color_number,
        color_mode: color_mode.into(),
        active_threads_limit: 7,
        knit_volume,
        yarn_lines: 4,
        obstacle_percentage,
        visible_patches: 6,
        generator_capacity: 3,
    }
}

// ── Engine creation ─────────────────────────────────────────────────────────

#[test]
fn test_engine_full_game_flow() {
    let config = make_config(4, 4, 3, "dark", 2, 5);
    let mut engine = GameEngine::new(&config);

    assert_eq!(engine.board.height, 4);
    assert_eq!(engine.board.width, 4);
    assert!(engine.active_threads.is_empty());
    assert!(!engine.is_won());

    // Pick up thread at (0,0) — top row is always selectable (if it's a Thread)
    // Move to a thread if (0,0) is an obstacle
    let mut picked = false;
    for col in 0..4u16 {
        engine.cursor_col = col;
        if engine.pick_up().is_ok() {
            picked = true;
            break;
        }
    }
    assert!(picked, "should be able to pick up at least one top-row thread");
    assert_eq!(engine.active_threads.len(), 1);

    // Process the thread
    engine.process_all_active();
    // With knit_volume=2, thread needs 2 yarn hits. After one round it may or may not be done.
    // Just verify no crash and state is consistent.
    assert!(engine.active_threads.len() <= 1);
}

#[test]
fn test_engine_board_and_yarn_consistency() {
    let config = make_config(3, 3, 1, "dark", 1, 0);
    let engine = GameEngine::new(&config);

    // With 1 color, 0 obstacles, 3x3 board, knit_volume=1: should have 9 yarn patches
    let total_patches: usize = engine.yarn.board.iter().map(|c| c.len()).sum();
    let counter = engine.board.count_knits();
    let expected: u16 = counter.color_hashmap.values().sum();
    assert_eq!(total_patches as u16, expected);
}

#[test]
fn test_engine_multiple_palettes() {
    for mode in ["dark", "bright", "colorblind"] {
        let config = make_config(3, 3, 4, mode, 2, 10);
        let engine = GameEngine::new(&config);
        // Just verify creation doesn't panic and board is populated
        assert_eq!(engine.board.height, 3);
        assert_eq!(engine.board.width, 3);
        let total_patches: usize = engine.yarn.board.iter().map(|c| c.len()).sum();
        assert!(total_patches > 0);
    }
}

#[test]
fn test_engine_high_obstacle_board() {
    // High obstacle % — engine should still produce a game (solvability retry loop)
    let config = make_config(5, 5, 2, "dark", 2, 80);
    let engine = GameEngine::new(&config);
    assert_eq!(engine.board.height, 5);
    // Board should exist even if mostly obstacles
    assert!(engine.yarn.board.len() > 0);
}

#[test]
fn test_engine_knit_volume_affects_yarn() {
    // Compare engines with different knit_volume on same-shape boards
    let config1 = make_config(3, 3, 1, "dark", 1, 0);
    let config3 = make_config(3, 3, 1, "dark", 3, 0);
    let e1 = GameEngine::new(&config1);
    let e3 = GameEngine::new(&config3);

    let patches1: usize = e1.yarn.board.iter().map(|c| c.len()).sum();
    let patches3: usize = e3.yarn.board.iter().map(|c| c.len()).sum();
    // With same number of threads, knit_volume=3 should have ~3x the patches
    assert!(patches3 > patches1);
}

// ── Engine actions ──────────────────────────────────────────────────────────

#[test]
fn test_engine_pick_exposes_neighbors() {
    let config = make_config(3, 3, 1, "dark", 1, 0);
    let mut engine = GameEngine::new(&config);

    // Pick up (0,0) — should succeed (top row)
    engine.cursor_row = 0; engine.cursor_col = 0;
    assert!(engine.pick_up().is_ok());
    // (0,0) is now Void
    assert!(matches!(engine.board.board[0][0], BoardEntity::Void));
    // (1,0) should now be selectable (Void neighbor above)
    assert!(engine.board.is_selectable(1, 0));
}

#[test]
fn test_engine_cursor_traversal() {
    // Use an all-Void board so every cell is focusable (Void is not a knit).
    let void_row = || vec![BoardEntity::Void, BoardEntity::Void, BoardEntity::Void, BoardEntity::Void];
    let mut engine = GameEngine {
        board: GameBoard {
            board: vec![void_row(), void_row(), void_row(), void_row()],
            height: 4, width: 4, knit_volume: 2,
        },
        yarn: Yarn { board: vec![vec![], vec![], vec![], vec![]], yarn_lines: 4, visible_patches: 6 },
        active_threads: vec![],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 2, active_threads_limit: 7,
    };

    // Traverse to bottom-right corner
    for _ in 0..3 { engine.move_cursor(Direction::Right).unwrap(); }
    for _ in 0..3 { engine.move_cursor(Direction::Down).unwrap(); }
    assert_eq!(engine.cursor_row, 3);
    assert_eq!(engine.cursor_col, 3);

    // Further right/down should fail
    assert!(engine.move_cursor(Direction::Right).is_err());
    assert!(engine.move_cursor(Direction::Down).is_err());

    // Traverse back to origin
    for _ in 0..3 { engine.move_cursor(Direction::Left).unwrap(); }
    for _ in 0..3 { engine.move_cursor(Direction::Up).unwrap(); }
    assert_eq!(engine.cursor_row, 0);
    assert_eq!(engine.cursor_col, 0);
}

#[test]
fn test_engine_json_roundtrip_preserves_game_state() {
    // Use a deterministic board: row 0 selectable threads, rest Void.
    let void_row = || vec![BoardEntity::Void, BoardEntity::Void, BoardEntity::Void, BoardEntity::Void];
    let mut engine = GameEngine {
        board: GameBoard {
            board: vec![
                vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue),
                     BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
                void_row(), void_row(), void_row(),
            ],
            height: 4, width: 4, knit_volume: 2,
        },
        yarn: Yarn { board: vec![vec![], vec![], vec![], vec![]], yarn_lines: 4, visible_patches: 6 },
        active_threads: vec![],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 2, active_threads_limit: 7,
    };

    // Make some moves
    engine.move_cursor(Direction::Right).unwrap();
    engine.move_cursor(Direction::Down).unwrap();
    let _ = engine.pick_up(); // may or may not succeed depending on board

    // Serialize and deserialize
    let json = engine.to_json();
    let restored = GameEngine::from_json(&json).unwrap();

    assert_eq!(restored.cursor_row, engine.cursor_row);
    assert_eq!(restored.cursor_col, engine.cursor_col);
    assert_eq!(restored.active_threads.len(), engine.active_threads.len());
    assert_eq!(restored.board.height, engine.board.height);
    assert_eq!(restored.knit_volume, engine.knit_volume);
}

// ── Preserved direct-module tests (still valid for module-level coverage) ───

#[test]
fn test_knit_volume_affects_total_yarn_patches() {
    // Deterministic boards — no randomness
    let make_board = |knit_volume: u16| GameBoard {
        board: vec![
            vec![BoardEntity::Thread(Color::Magenta), BoardEntity::Thread(Color::Magenta)],
            vec![BoardEntity::Thread(Color::Magenta), BoardEntity::Thread(Color::Magenta)],
        ],
        height: 2, width: 2, knit_volume,
    };
    let total1: u16 = make_board(1).count_knits().color_hashmap.values().sum();
    let total3: u16 = make_board(3).count_knits().color_hashmap.values().sum();
    assert_eq!(total1, 4);
    assert_eq!(total3, 12);
}
