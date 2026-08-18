//! Proves the `loom_engine::game::GameEngine` adapter around loom-match3's
//! concrete engine (crates/loom-match3/src/game.rs) is wired correctly — the
//! generic Phase 3/4 `Shell<G>` drives gameplay exclusively through this
//! trait, so a bug here is invisible to the existing engine-level tests
//! (which all call `m3tui::engine::GameEngine` directly, bypassing the
//! adapter entirely).

use m3tui::game::M3Game;
use loom_engine::game::{Action, Game, GameStatus, RenderArea};
use loom_engine::input::{Key, KeyEvent};
use loom_engine::render::CellGrid;

fn full_area(w: u16, h: u16) -> RenderArea {
    RenderArea { x: 0, y: 0, width: w, height: h }
}

#[test]
fn create_engine_starts_playing() {
    let game = M3Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn render_draws_a_non_blank_frame() {
    let game = M3Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);

    let mut grid = CellGrid::new(120, 50);
    engine.render(&mut grid, full_area(120, 50));

    let drew_something = (0..50).any(|y| {
        (0..120).any(|x| grid.get(x, y).glyph != ' ')
    });
    assert!(drew_something, "render() produced an entirely blank frame");
}

#[test]
fn render_is_stable_across_a_small_terminal_height() {
    // area.height feeds LayoutGeometry::for_height (vertical vs horizontal)
    // -- the specific wiring this adapter had to add (detect_layout used to
    // query crossterm/hardcode 24 internally). A tiny height should force
    // the horizontal branch without panicking.
    let game = M3Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);

    let mut grid = CellGrid::new(200, 10);
    engine.render(&mut grid, full_area(200, 10));
    let drew_something = (0..10).any(|y| {
        (0..200).any(|x| grid.get(x, y).glyph != ' ')
    });
    assert!(drew_something, "horizontal-layout render produced a blank frame");
}

#[test]
fn arrow_keys_return_redraw_and_dont_panic() {
    let game = M3Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    for key in [Key::Up, Key::Down, Key::Left, Key::Right] {
        let action = engine.handle_key(KeyEvent::new(key));
        assert_eq!(action, Action::Redraw);
    }
}

#[test]
fn esc_with_no_active_bonus_quits_to_menu() {
    let game = M3Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    assert_eq!(engine.handle_key(KeyEvent::new(Key::Esc)), Action::QuitToMenu);
}

#[test]
fn h_key_requests_help_screen() {
    let game = M3Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    assert_eq!(engine.handle_key(KeyEvent::new(Key::Char('h'))), Action::ShowHelp);
}

#[test]
fn q_key_quits_to_menu() {
    let game = M3Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    assert_eq!(engine.handle_key(KeyEvent::new(Key::Char('q'))), Action::QuitToMenu);
}

#[test]
fn score_is_some_match3_has_a_numeric_score() {
    let game = M3Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.score(), Some(0));
}

#[test]
fn board_dims_matches_config() {
    let game = M3Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.board_dims(), (config.board_height, config.board_width));
}

#[test]
fn scale_round_trips() {
    let game = M3Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    engine.set_scale(3);
    assert_eq!(engine.scale(), 3);

    let mut grid = CellGrid::new(200, 50);
    engine.render(&mut grid, full_area(200, 50));
}

#[test]
fn campaign_entry_progresses_and_completes() {
    let game = M3Game;
    let mut entry = game.new_campaign_entry(0);
    assert_eq!(entry.current_level, 0);
    assert!(!entry.completed);

    let total = entry.total_levels();
    for i in 0..total {
        let done = game.complete_campaign_level(&mut entry);
        assert_eq!(done, i + 1 == total);
    }
    assert!(entry.completed);
}

#[test]
fn campaign_config_reflects_level() {
    let game = M3Game;
    let base = game.default_config();
    let entry = game.new_campaign_entry(0);
    let cfg = game.campaign_config(&entry, &base);
    assert!(cfg.board_height > 0 && cfg.board_width > 0);
}

#[test]
fn all_blessings_returns_the_full_catalog() {
    let game = M3Game;
    // The blessing-selection screen needs the whole list (locked cards are
    // shown greyed out, not hidden), and match3's campaign entry always
    // routes through it (see needs_blessing_selection's unconditional true).
    assert!(!game.all_blessings().is_empty());
}

#[test]
fn confirm_blessings_records_ids() {
    let game = M3Game;
    let mut entry = game.new_campaign_entry(0);
    game.confirm_blessings(&mut entry, &["extra_moves".to_string()]);
    assert_eq!(entry.blessings, vec!["extra_moves"]);
}

#[test]
fn create_campaign_engine_applies_blessings_and_objective() {
    let game = M3Game;
    let base = game.default_config();
    let mut entry = game.new_campaign_entry(0);
    game.confirm_blessings(&mut entry, &["extra_moves".to_string()]);

    let cfg = game.campaign_config(&entry, &base);
    let engine = game.create_campaign_engine(&entry, &cfg, &[]);
    // No panic building it is the main bar; status should be a fresh,
    // playable game (objective not met on move zero).
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn campaign_run_never_reports_stuck_or_lost_only_won_or_playing() {
    // Preserves a real quirk of the original tui.rs: campaign mode only
    // ever checks the level objective, never Stuck/OutOfMoves. Drive a
    // scripted sequence and confirm status is always Playing or Won.
    let game = M3Game;
    let base = game.default_config();
    let entry = game.new_campaign_entry(0);
    let cfg = game.campaign_config(&entry, &base);
    let mut engine = game.create_campaign_engine(&entry, &cfg, &[]);

    for _ in 0..40 {
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
        engine.handle_key(KeyEvent::new(Key::Right));
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
        match engine.status() {
            GameStatus::Playing | GameStatus::Won { .. } => {}
            other => panic!("campaign run must never report {other:?}"),
        }
    }
}

#[test]
fn endless_config_marks_is_endless() {
    let game = M3Game;
    let base = game.default_config();
    let cfg = game.endless_wave_config(1, &base);
    assert!(cfg.is_endless);
}

#[test]
fn quick_game_config_is_not_endless() {
    let game = M3Game;
    let cfg = game.default_config();
    assert!(!cfg.is_endless);
}

#[test]
fn pick_up_and_tick_do_not_panic_across_a_scripted_sequence() {
    let game = M3Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    for _ in 0..30 {
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
        if engine.status() != GameStatus::Playing {
            break;
        }
        engine.handle_key(KeyEvent::new(Key::Right));
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
    }
    // No panic across a real, if unsophisticated, play sequence is the bar
    // here -- solving correctness is already covered by the engine-level
    // test suite, which this adapter delegates to.
}
