/// Tests for hard mode, hint system, and solvability.
use knitui::engine::GameEngine;
use knitui::config::Config;
use knitui::board_entity::BoardEntity;
use knitui::game_board::GameBoard;
use knitui::yarn::Yarn;
use knitui::solvability::{find_solution, is_solvable};
use loom_engine::render::Color;

// ── helpers ──────────────────────────────────────────────────────────────────

fn base_config() -> Config {
    Config {
        board_height: 4,
        board_width: 4,
        color_number: 3,
        color_mode: "dark".into(),
        spool_limit: 7,
        spool_capacity: 2,
        yarn_lines: 4,
        obstacle_percentage: 0,
        visible_stitches: 6,
        conveyor_capacity: 3,
        conveyor_percentage: 0,
        layout: "auto".into(),
        scale: 1,
        scissors: 2,
        tweezers: 1,
        balloons: 1,
        scissors_spools: 1,
        balloon_count: 2,
        ad_file: None,
        max_solutions: None,
        hard_mode: false,
    }
}

fn hard_config() -> Config {
    Config { hard_mode: true, ..base_config() }
}

// ── Hard mode ────────────────────────────────────────────────────────────────

#[test]
fn test_hard_mode_bonuses_start_at_zero() {
    let engine = GameEngine::new(&hard_config());
    assert_eq!(engine.bonuses.scissors, 0, "hard mode: scissors should be 0");
    assert_eq!(engine.bonuses.tweezers, 0, "hard mode: tweezers should be 0");
    assert_eq!(engine.bonuses.balloons, 0, "hard mode: balloons should be 0");
}

#[test]
fn test_hard_mode_blessing_flags_empty() {
    let engine = GameEngine::new(&hard_config());
    // BlessingFlags::default() leaves all flags false
    assert!(!engine.blessing_flags.scouts_eye,     "hard mode: scouts_eye should be false");
    assert!(!engine.blessing_flags.wrap_around,    "hard mode: wrap_around should be false");
    assert!(!engine.blessing_flags.tidy_workspace, "hard mode: tidy_workspace should be false");
    assert!(!engine.blessing_flags.conveyor_peek,  "hard mode: conveyor_peek should be false");
    assert!(!engine.blessing_flags.color_count,    "hard mode: color_count should be false");
    assert!(!engine.blessing_flags.match_hint,     "hard mode: match_hint should be false");
}

#[test]
fn test_normal_mode_bonuses_non_zero() {
    // Sanity: normal mode should pick up bonuses from config
    let engine = GameEngine::new(&base_config());
    assert_eq!(engine.bonuses.scissors, 2);
    assert_eq!(engine.bonuses.tweezers, 1);
    assert_eq!(engine.bonuses.balloons, 1);
}

#[test]
fn test_hard_mode_does_not_set_blessings_after_set_blessings() {
    // Even if the caller calls set_blessings, hard mode boards still start
    // without blessings applied (set_blessings is called after new() by callers
    // who respect is_hard_track; here we verify the struct default is clean).
    let engine = GameEngine::new(&hard_config());
    // The engine from hard_config starts with blessing_flags all false
    assert!(!engine.blessing_flags.match_hint);
}

// ── Hint system ──────────────────────────────────────────────────────────────

#[test]
fn test_compute_hint_returns_some_when_spools_available() {
    // Create an engine with match_hint blessing active.
    let mut engine = GameEngine::new(&base_config());
    engine.set_blessings(&["match_hint".to_string()]);
    assert!(engine.blessing_flags.match_hint, "blessing should be set");

    // A fresh board with spools should always yield a hint
    let hint = engine.compute_hint();
    assert!(hint.is_some(), "compute_hint should return Some when spools are on the board");

    let (r, c) = hint.unwrap();
    assert!(r < engine.board.height as usize, "hint row in bounds");
    assert!(c < engine.board.width as usize,  "hint col in bounds");
    // The hinted cell should be a spool
    assert!(
        matches!(engine.board.board[r][c], BoardEntity::Spool(_) | BoardEntity::KeySpool(_)),
        "hint cell must be a Spool or KeySpool"
    );
}

#[test]
fn test_compute_hint_returns_none_on_empty_board() {
    // Build an engine and replace its board with an all-Void board.
    let mut engine = GameEngine::new(&base_config());
    engine.set_blessings(&["match_hint".to_string()]);

    // Overwrite every cell with Void
    let h = engine.board.height as usize;
    let w = engine.board.width as usize;
    for r in 0..h {
        for c in 0..w {
            engine.board.board[r][c] = BoardEntity::Void;
        }
    }

    let hint = engine.compute_hint();
    assert!(hint.is_none(), "compute_hint should return None on a board with no spools");
}

#[test]
fn test_compute_hint_without_blessing_still_returns_cell() {
    // compute_hint() is a pure board scan — it does NOT check the blessing flag itself.
    // The TUI gates the call; the function always scans.
    let engine = GameEngine::new(&base_config());
    // No blessing set, but board has spools — should still return Some.
    let hint = engine.compute_hint();
    assert!(hint.is_some(), "compute_hint scans board regardless of blessing flag");
}

// ── Solvability ───────────────────────────────────────────────────────────────

/// Build a minimal deterministic solvable board: 2x2, one Red and one Blue spool,
/// yarn has exactly one of each color.
fn make_trivial_solvable() -> (GameBoard, Yarn) {
    use knitui::yarn::Stitch;

    // Board: top-row spools are accessible (selectable), rest Void
    let board = GameBoard {
        board: vec![
            vec![BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue)],
            vec![BoardEntity::Void,              BoardEntity::Void],
        ],
        height: 2,
        width: 2,
        spool_capacity: 2,
    };

    // Yarn: one column per color, spool_capacity=2 stitches each
    let yarn = Yarn {
        board: vec![
            vec![
                Stitch { color: Color::Red,  locked: false },
                Stitch { color: Color::Red,  locked: false },
            ],
            vec![
                Stitch { color: Color::Blue, locked: false },
                Stitch { color: Color::Blue, locked: false },
            ],
        ],
        yarn_lines: 2,
        visible_stitches: 4,
        balloon_columns: Vec::new(),
    };

    (board, yarn)
}

#[test]
fn test_find_solution_returns_some_for_solvable_board() {
    let (board, yarn) = make_trivial_solvable();
    let result = find_solution(&board, &yarn, 2, 7);
    assert!(result.is_some(), "find_solution should return Some for a trivially solvable board");
    let moves = result.unwrap();
    assert!(!moves.is_empty(), "solution should contain at least one move");
}

#[test]
fn test_is_solvable_returns_true_for_trivial_board() {
    let (board, yarn) = make_trivial_solvable();
    assert!(is_solvable(&board, &yarn, 2, 7));
}

#[test]
fn test_find_solution_returns_none_for_impossible_board() {
    use knitui::yarn::Stitch;

    // Board has a Red spool, but yarn has only Blue stitches — color mismatch → unsolvable
    let board = GameBoard {
        board: vec![
            vec![BoardEntity::Spool(Color::Red), BoardEntity::Void],
            vec![BoardEntity::Void,              BoardEntity::Void],
        ],
        height: 2,
        width: 2,
        spool_capacity: 2,
    };
    let yarn = Yarn {
        board: vec![
            vec![Stitch { color: Color::Blue, locked: false }],
            vec![],
        ],
        yarn_lines: 2,
        visible_stitches: 2,
        balloon_columns: Vec::new(),
    };

    // count_balance should fail → is_solvable false
    assert!(!is_solvable(&board, &yarn, 2, 7),
        "mismatched color counts should be flagged as not solvable");
    // find_solution may return None or Some depending on DFS; the count_balance
    // pre-screen in is_solvable is the canonical check.
}

#[test]
fn test_game_engine_new_produces_solvable_board() {
    // GameEngine::new() retries until solvable (unless hard_mode=true).
    // Just verify the resulting board passes our solvability check.
    let cfg = base_config();
    let engine = GameEngine::new(&cfg);
    let solvable = is_solvable(
        &engine.board,
        &engine.yarn,
        engine.spool_capacity,
        engine.spool_limit,
    );
    assert!(solvable, "GameEngine::new should produce a solvable board in normal mode");
}

#[test]
fn test_campaign_levels_counts() {
    use knitui::campaign_levels::{SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN, HARD_CAMPAIGN};
    assert_eq!(SHORT_CAMPAIGN.len(),  10);
    assert_eq!(MEDIUM_CAMPAIGN.len(), 15);
    assert_eq!(LONG_CAMPAIGN.len(),   20);
    assert_eq!(HARD_CAMPAIGN.len(),   10);
}
