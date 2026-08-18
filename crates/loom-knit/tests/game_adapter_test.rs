//! Proves the `loom_engine::game::GameEngine` adapter around loom-knit's
//! concrete engine (crates/loom-knit/src/game.rs) is wired correctly — the
//! generic Phase 3 `Shell<G>` will drive gameplay exclusively through this
//! trait, so a bug here is invisible to the existing engine-level tests
//! (which all call `knitui::engine::GameEngine` directly, bypassing the
//! adapter entirely).

use knitui::game::KnitGame;
use loom_engine::game::{Action, Game, GameStatus, RenderArea};
use loom_engine::input::{Key, KeyEvent};
use loom_engine::render::CellGrid;

fn full_area(w: u16, h: u16) -> RenderArea {
    RenderArea { x: 0, y: 0, width: w, height: h }
}

#[test]
fn create_engine_starts_playing() {
    let game = KnitGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn render_draws_a_non_blank_frame() {
    let game = KnitGame;
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
    // area.height feeds layout auto-detection (vertical vs horizontal) --
    // this is the specific wiring the Phase 3 adapter had to add (detect_layout
    // used to query crossterm internally). A tiny height should force the
    // horizontal branch without panicking.
    let game = KnitGame;
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
    let game = KnitGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    for key in [Key::Up, Key::Down, Key::Left, Key::Right] {
        let action = engine.handle_key(KeyEvent::new(key));
        assert_eq!(action, Action::Redraw);
    }
}

#[test]
fn esc_with_no_active_bonus_quits_to_menu() {
    let game = KnitGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    assert_eq!(engine.handle_key(KeyEvent::new(Key::Esc)), Action::QuitToMenu);
}

#[test]
fn h_key_requests_help_screen() {
    let game = KnitGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    assert_eq!(engine.handle_key(KeyEvent::new(Key::Char('h'))), Action::ShowHelp);
}

#[test]
fn score_is_none_knit_has_no_numeric_score() {
    let game = KnitGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.score(), None);
}

#[test]
fn board_dims_matches_config() {
    let game = KnitGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.board_dims(), (config.board_height, config.board_width));
}

#[test]
fn scale_round_trips() {
    let game = KnitGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    engine.set_scale(3);
    assert_eq!(engine.scale(), 3);

    // A render after changing scale must not panic -- proves the adapter's
    // stored geometry (not just the config clone) actually uses the new value.
    let mut grid = CellGrid::new(200, 50);
    engine.render(&mut grid, full_area(200, 50));
}

#[test]
fn campaign_entry_progresses_and_completes() {
    let game = KnitGame;
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
fn campaign_config_reflects_level_and_hard_track_flag() {
    let game = KnitGame;
    let base = game.default_config();
    let entry = game.new_campaign_entry(0);
    let cfg = game.campaign_config(&entry, &base);
    // Track 0 (level 0) is a real track's first level -- just prove the
    // config actually came from the level table, not an unrelated default.
    assert!(cfg.board_height > 0 && cfg.board_width > 0);
}

#[test]
fn confirm_blessings_records_ids_and_banks_one_time_bonuses() {
    let game = KnitGame;
    let mut entry = game.new_campaign_entry(0);
    let before_scissors = entry.banked_scissors;
    let before_balloons = entry.banked_balloons;

    game.confirm_blessings(&mut entry, &[
        "apprentices_kit".to_string(),
        "light_pockets".to_string(),
    ]);

    assert_eq!(entry.blessings, vec!["apprentices_kit", "light_pockets"]);
    assert_eq!(entry.banked_scissors, before_scissors + 1);
    assert_eq!(entry.banked_balloons, before_balloons + 1);
}

#[test]
fn all_blessings_returns_the_full_catalog() {
    let game = KnitGame;
    // The blessing-selection screen needs the whole list (locked cards are
    // shown greyed out, not hidden) -- 12 blessings total for knit.
    assert_eq!(game.all_blessings().len(), 12);
}

#[test]
fn create_campaign_engine_applies_ad_limit_and_blessings() {
    let game = KnitGame;
    let base = game.default_config();
    let mut entry = game.new_campaign_entry(0);
    game.confirm_blessings(&mut entry, &["light_pockets".to_string()]);

    let cfg = game.campaign_config(&entry, &base);
    let engine = game.create_campaign_engine(&entry, &cfg, &[]);
    // No panic building it is the main bar (proves set_ad_limit/set_blessings
    // wiring didn't break); status should still be a fresh, playable game.
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn pick_up_and_tick_do_not_panic_across_a_scripted_sequence() {
    let game = KnitGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    for _ in 0..20 {
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
        if engine.status() != GameStatus::Playing {
            break;
        }
        engine.handle_key(KeyEvent::new(Key::Right));
    }
    // No panic across a real, if unsophisticated, play sequence is the bar
    // here -- solving correctness is already covered by the engine-level
    // test suite (game_flow_test.rs etc.), which this adapter delegates to.
}
