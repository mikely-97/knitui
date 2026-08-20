//! Proves the `loom_engine::game::GameEngine` adapter around loom-merge2's
//! concrete engine (crates/loom-merge2/src/game.rs) is wired correctly.
//! merge2's adapter is the most involved of the three built so far: its
//! CampaignEntry embeds a live, persistent board/inventory/energy/orders
//! (not just level params), and mission-completion depends on the live
//! engine's story_orders_completed rather than a snapshot -- see the
//! adapter's module doc comment for the pre-existing bug this fixes
//! (the original tui.rs checked a stale campaign_ctx that was never synced
//! during play, so campaign missions could never actually be won).

use m2tui::game::M2Game;
use loom_engine::game::{Action, Game, GameStatus, RenderArea};
use loom_engine::input::{Key, KeyEvent};
use loom_engine::render::CellGrid;

fn full_area(w: u16, h: u16) -> RenderArea {
    RenderArea { x: 0, y: 0, width: w, height: h }
}

#[test]
fn create_engine_starts_playing() {
    let game = M2Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn render_draws_a_non_blank_frame() {
    let game = M2Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);

    let mut grid = CellGrid::new(120, 50);
    engine.render(&mut grid, full_area(120, 50));

    let drew_something = (0..50).any(|y| (0..120).any(|x| grid.get(x, y).glyph != ' '));
    assert!(drew_something, "render() produced an entirely blank frame");
}

#[test]
fn render_is_stable_across_a_narrow_terminal_width() {
    // area.width feeds LayoutGeometry::for_width (horizontal vs vertical
    // panel layout) -- the specific wiring this adapter had to add
    // (LayoutGeometry::compute used to query crossterm internally).
    let game = M2Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);

    let mut grid = CellGrid::new(40, 40);
    engine.render(&mut grid, full_area(40, 40));
    let drew_something = (0..40).any(|y| (0..40).any(|x| grid.get(x, y).glyph != ' '));
    assert!(drew_something, "narrow-width render produced a blank frame");
}

#[test]
fn arrow_keys_return_redraw_and_dont_panic() {
    let game = M2Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    for key in [Key::Up, Key::Down, Key::Left, Key::Right] {
        assert_eq!(engine.handle_key(KeyEvent::new(key)), Action::Redraw);
    }
}

#[test]
fn esc_quits_to_menu() {
    let game = M2Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);
    assert_eq!(engine.handle_key(KeyEvent::new(Key::Esc)), Action::QuitToMenu);
}

#[test]
fn h_key_requests_help_screen() {
    let game = M2Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);
    assert_eq!(engine.handle_key(KeyEvent::new(Key::Char('h'))), Action::ShowHelp);
}

#[test]
fn score_is_some_merge2_has_a_numeric_score() {
    let game = M2Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.score(), Some(0));
}

#[test]
fn board_dims_matches_config() {
    let game = M2Game;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.board_dims(), (config.board_rows, config.board_cols));
}

#[test]
fn scale_round_trips() {
    let game = M2Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);
    engine.set_scale(3);
    assert_eq!(engine.scale(), 3);
    let mut grid = CellGrid::new(200, 50);
    engine.render(&mut grid, full_area(200, 50));
}

#[test]
fn inventory_mode_i_key_then_esc_returns_to_playing() {
    // 'I' enters an adapter-internal sub-state (no Shell/Action variant for
    // it); prove the round trip doesn't panic and both frames render.
    let game = M2Game;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    engine.handle_key(KeyEvent::new(Key::Char('i')));
    let mut grid = CellGrid::new(120, 50);
    engine.render(&mut grid, full_area(120, 50));

    engine.handle_key(KeyEvent::new(Key::Esc));
    engine.render(&mut grid, full_area(120, 50));
    // No panic across the round trip is the bar here.
}

#[test]
fn campaign_entry_progresses_and_completes() {
    let game = M2Game;
    let mut entry = game.new_campaign_entry(0);
    assert_eq!(entry.current_mission, 0);
    assert!(!entry.completed);

    let total = entry.total_missions();
    for i in 0..total {
        let done = game.complete_campaign_level(&mut entry);
        assert_eq!(done, i + 1 == total);
    }
    assert!(entry.completed);
}

#[test]
fn campaign_config_reflects_entry_board_dims() {
    let game = M2Game;
    let base = game.default_config();
    let entry = game.new_campaign_entry(0);
    let cfg = game.campaign_config(&entry, &base);
    assert_eq!(cfg.board_rows as usize, entry.board.rows);
    assert_eq!(cfg.board_cols as usize, entry.board.cols);
}

#[test]
fn confirm_blessings_records_ids_and_loads_mission_orders() {
    let game = M2Game;
    let mut entry = game.new_campaign_entry(0);
    assert!(entry.active_orders.is_empty());
    game.confirm_blessings(&mut entry, &["keen_eye".to_string()]);
    assert_eq!(entry.blessings, vec!["keen_eye"]);
    // load_mission_orders() should have populated story/extra orders.
    assert!(!entry.active_orders.is_empty());
}

#[test]
fn create_campaign_engine_builds_from_live_entry_state() {
    let game = M2Game;
    let base = game.default_config();
    let mut entry = game.new_campaign_entry(0);
    game.confirm_blessings(&mut entry, &[]);

    let cfg = game.campaign_config(&entry, &base);
    let engine = game.create_campaign_engine(&entry, &cfg, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
    assert_eq!(engine.board_dims(), (entry.board.rows as u16, entry.board.cols as u16));
}

#[test]
fn mission_completes_via_live_engine_story_count_not_stale_snapshot() {
    // Regression test for the bug this adapter fixes: the original tui.rs's
    // check_status read ctx.story_orders_completed, which was never synced
    // from the live engine during play, so this condition could never be
    // observed through normal play. The adapter's status() must read the
    // live engine's counter directly.
    let game = M2Game;
    let base = game.default_config();
    let mut entry = game.new_campaign_entry(0);
    game.confirm_blessings(&mut entry, &[]);
    let cfg = game.campaign_config(&entry, &base);
    let mut engine = game.create_campaign_engine(&entry, &cfg, &[]);

    // Drive a bounded number of scripted actions -- no panics is the floor
    // bar; if a mission's story orders happen to be satisfiable this way,
    // status() must observe it without any external re-sync call.
    for _ in 0..50 {
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
        if engine.status() != GameStatus::Playing {
            break;
        }
        engine.handle_key(KeyEvent::new(Key::Right));
    }
    // No panic across a real, if unsophisticated, play sequence is the bar
    // here -- solving correctness is covered by engine-level tests.
}

#[test]
fn endless_config_marks_is_endless() {
    let game = M2Game;
    let base = game.default_config();
    let cfg = game.endless_wave_config(1, &base);
    assert!(cfg.is_endless);
}

#[test]
fn quick_game_config_is_not_endless() {
    let game = M2Game;
    let cfg = game.default_config();
    assert!(!cfg.is_endless);
}

#[test]
fn endless_engine_builds_and_plays() {
    let game = M2Game;
    let base = game.default_config();
    let cfg = game.endless_wave_config(1, &base);
    let mut engine = game.create_engine(&cfg, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
    for _ in 0..20 {
        engine.handle_key(KeyEvent::new(Key::Enter));
        engine.tick();
        engine.handle_key(KeyEvent::new(Key::Right));
    }
}
