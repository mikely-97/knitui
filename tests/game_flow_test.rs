/// Integration tests for game flow — exercising GameEngine end-to-end.
use knitui::engine::{GameEngine, BonusInventory, BonusState};
use knitui::config::Config;
use knitui::board_entity::{BoardEntity, Direction};
use knitui::game_board::GameBoard;
use knitui::palette::{select_palette, ColorMode};
use knitui::yarn::Yarn;
use knitui::spool::Spool;
use crossterm::style::Color;

fn make_config(
    board_height: u16, board_width: u16, color_number: u16,
    color_mode: &str, spool_capacity: u16, obstacle_percentage: u16,
) -> Config {
    Config {
        board_height, board_width, color_number,
        color_mode: color_mode.into(),
        spool_limit: 7,
        spool_capacity,
        yarn_lines: 4,
        obstacle_percentage,
        visible_stitches: 6,
        conveyor_capacity: 3,
        conveyor_percentage: 0,
        layout: "auto".into(),
        scale: 1,
        scissors: 0, tweezers: 0, balloons: 0,
        scissors_spools: 1, balloon_count: 2,
        ad_file: None,
        max_solutions: None,
    }
}

// ── Engine creation ─────────────────────────────────────────────────────────

#[test]
fn test_engine_full_game_flow() {
    let config = make_config(4, 4, 3, "dark", 2, 5);
    let mut engine = GameEngine::new(&config);

    assert_eq!(engine.board.height, 4);
    assert_eq!(engine.board.width, 4);
    assert!(engine.held_spools.is_empty());
    assert!(!engine.is_won());

    // Pick up spool at (0,0) — top row is always selectable (if it's a Spool)
    // Move to a spool if (0,0) is an obstacle
    let mut picked = false;
    for col in 0..4u16 {
        engine.cursor_col = col;
        if engine.pick_up().is_ok() {
            picked = true;
            break;
        }
    }
    assert!(picked, "should be able to pick up at least one top-row spool");
    assert_eq!(engine.held_spools.len(), 1);

    // Process the spool
    engine.process_all_active();
    // With spool_capacity=2, spool needs 2 yarn hits. After one round it may or may not be done.
    // Just verify no crash and state is consistent.
    assert!(engine.held_spools.len() <= 1);
}

#[test]
fn test_engine_board_and_yarn_consistency() {
    let config = make_config(3, 3, 1, "dark", 1, 0);
    let engine = GameEngine::new(&config);

    // With 1 color, 0 obstacles, 3x3 board, spool_capacity=1: should have 9 yarn stitches
    let total_stitches: usize = engine.yarn.board.iter().map(|c| c.len()).sum();
    let counter = engine.board.count_spools();
    let expected: u16 = counter.color_hashmap.values().sum();
    assert_eq!(total_stitches as u16, expected);
}

#[test]
fn test_engine_multiple_palettes() {
    for mode in ["dark", "bright", "colorblind"] {
        let config = make_config(3, 3, 4, mode, 2, 10);
        let engine = GameEngine::new(&config);
        // Just verify creation doesn't panic and board is populated
        assert_eq!(engine.board.height, 3);
        assert_eq!(engine.board.width, 3);
        let total_stitches: usize = engine.yarn.board.iter().map(|c| c.len()).sum();
        assert!(total_stitches > 0);
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
fn test_engine_spool_capacity_affects_yarn() {
    // Compare engines with different spool_capacity on same-shape boards
    let config1 = make_config(3, 3, 1, "dark", 1, 0);
    let config3 = make_config(3, 3, 1, "dark", 3, 0);
    let e1 = GameEngine::new(&config1);
    let e3 = GameEngine::new(&config3);

    let stitches1: usize = e1.yarn.board.iter().map(|c| c.len()).sum();
    let stitches3: usize = e3.yarn.board.iter().map(|c| c.len()).sum();
    // With same number of spools, spool_capacity=3 should have ~3x the stitches
    assert!(stitches3 > stitches1);
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
    // Use an all-Void board so every cell is focusable (Void is not a spool).
    let void_row = || vec![BoardEntity::Void, BoardEntity::Void, BoardEntity::Void, BoardEntity::Void];
    let mut engine = GameEngine {
        board: GameBoard {
            board: vec![void_row(), void_row(), void_row(), void_row()],
            height: 4, width: 4, spool_capacity: 2,
        },
        yarn: Yarn { board: vec![vec![], vec![], vec![], vec![]], yarn_lines: 4, visible_stitches: 6, balloon_columns: Vec::new() },
        held_spools: vec![],
        cursor_row: 0, cursor_col: 0,
        spool_capacity: 2, spool_limit: 7,
        bonuses: BonusInventory {
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2,
        },
        bonus_state: BonusState::None,
        ad_limit: None,
        ads_used: 0,
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
    // Use a deterministic board: row 0 selectable spools, rest Void.
    let void_row = || vec![BoardEntity::Void, BoardEntity::Void, BoardEntity::Void, BoardEntity::Void];
    let mut engine = GameEngine {
        board: GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue),
                     BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue)],
                void_row(), void_row(), void_row(),
            ],
            height: 4, width: 4, spool_capacity: 2,
        },
        yarn: Yarn { board: vec![vec![], vec![], vec![], vec![]], yarn_lines: 4, visible_stitches: 6, balloon_columns: Vec::new() },
        held_spools: vec![],
        cursor_row: 0, cursor_col: 0,
        spool_capacity: 2, spool_limit: 7,
        bonuses: BonusInventory {
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2,
        },
        bonus_state: BonusState::None,
        ad_limit: None,
        ads_used: 0,
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
    assert_eq!(restored.held_spools.len(), engine.held_spools.len());
    assert_eq!(restored.board.height, engine.board.height);
    assert_eq!(restored.spool_capacity, engine.spool_capacity);
}

// ── Background processing (no manual trigger needed) ────────────────────────

#[test]
fn test_auto_processing_after_pick_up() {
    // Simulate the TUI loop's auto-processing: after pick_up, calling
    // process_one_active on each tick drains spools without any manual trigger.
    let config = make_config(3, 3, 1, "dark", 1, 0);
    let mut engine = GameEngine::new(&config);

    // Pick up a spool from the top row
    let mut picked = false;
    for col in 0..3u16 {
        engine.cursor_col = col;
        if engine.pick_up().is_ok() { picked = true; break; }
    }
    assert!(picked);
    assert_eq!(engine.held_spools.len(), 1);

    // Simulate background ticks — spool should be processed and removed
    // without any explicit "start processing" step.
    let mut ticks = 0;
    while !engine.held_spools.is_empty() && ticks < 20 {
        engine.process_one_active();
        ticks += 1;
    }
    assert!(engine.held_spools.is_empty(),
        "spool should auto-drain via background ticks");
}

#[test]
fn test_input_during_processing() {
    // Verify that cursor movement works while spools are being processed —
    // the engine does not block actions during processing.
    let void_row = || vec![BoardEntity::Void, BoardEntity::Void, BoardEntity::Void, BoardEntity::Void];
    let mut engine = GameEngine {
        board: GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue),
                     BoardEntity::Void, BoardEntity::Void],
                void_row(), void_row(), void_row(),
            ],
            height: 4, width: 4, spool_capacity: 2,
        },
        yarn: Yarn {
            board: vec![
                vec![knitui::yarn::Stitch { color: Color::Red, locked: false },
                     knitui::yarn::Stitch { color: Color::Blue, locked: false }],
                vec![knitui::yarn::Stitch { color: Color::Red, locked: false },
                     knitui::yarn::Stitch { color: Color::Blue, locked: false }],
                vec![], vec![],
            ],
            yarn_lines: 4, visible_stitches: 6,
            balloon_columns: Vec::new(),
        },
        held_spools: vec![],
        cursor_row: 0, cursor_col: 0,
        spool_capacity: 2, spool_limit: 7,
        bonuses: BonusInventory {
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2,
        },
        bonus_state: BonusState::None,
        ad_limit: None,
        ads_used: 0,
    };

    // Pick up a spool
    engine.pick_up().unwrap();
    assert_eq!(engine.held_spools.len(), 1);

    // Simulate one background tick
    engine.process_one_active();

    // Move cursor while spool is still processing (fill 2 <= spool_capacity 2)
    assert!(engine.move_cursor(Direction::Right).is_ok());
    assert_eq!(engine.cursor_col, 1);

    // Pick up another spool while first is still active
    engine.pick_up().unwrap();
    assert_eq!(engine.held_spools.len(), 2);
}

// ── Preserved direct-module tests (still valid for module-level coverage) ───

#[test]
fn test_spool_capacity_affects_total_yarn_stitches() {
    // Deterministic boards — no randomness
    let make_board = |spool_capacity: u16| GameBoard {
        board: vec![
            vec![BoardEntity::Spool(Color::Magenta), BoardEntity::Spool(Color::Magenta)],
            vec![BoardEntity::Spool(Color::Magenta), BoardEntity::Spool(Color::Magenta)],
        ],
        height: 2, width: 2, spool_capacity,
    };
    let total1: u16 = make_board(1).count_spools().color_hashmap.values().sum();
    let total3: u16 = make_board(3).count_spools().color_hashmap.values().sum();
    assert_eq!(total1, 4);
    assert_eq!(total3, 12);
}
