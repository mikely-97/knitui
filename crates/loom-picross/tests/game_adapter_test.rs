//! Proves the `loom_engine::game::GameEngine` adapter around loom-picross's
//! concrete engine (crates/loom-picross/src/game.rs) is wired correctly.
//! Picross's campaign is deliberately sequential-only (a real, disclosed
//! design decision made 2026-08-21) -- the original tui.rs's free
//! puzzle-jumping via a dedicated LevelSelect screen has no equivalent in
//! Shell's generic campaign flow.

use pictui::game::PicrossGame;
use loom_engine::campaign::CampaignEntry;
use loom_engine::game::{Action, Game, GameStatus, RenderArea};
use loom_engine::input::{Key, KeyEvent};
use loom_engine::render::CellGrid;

fn full_area(w: u16, h: u16) -> RenderArea {
    RenderArea { x: 0, y: 0, width: w, height: h }
}

#[test]
fn create_engine_starts_playing() {
    let game = PicrossGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn render_draws_a_non_blank_frame() {
    let game = PicrossGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);

    let mut grid = CellGrid::new(80, 40);
    engine.render(&mut grid, full_area(80, 40));

    let drew_something = (0..40).any(|y| (0..80).any(|x| grid.get(x, y).glyph != ' '));
    assert!(drew_something, "render() produced an entirely blank frame");
}

#[test]
fn arrow_keys_return_redraw_and_dont_panic() {
    let game = PicrossGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);

    for key in [Key::Up, Key::Down, Key::Left, Key::Right] {
        assert_eq!(engine.handle_key(KeyEvent::new(key)), Action::Redraw);
    }
}

#[test]
fn esc_quits_to_menu() {
    let game = PicrossGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);
    assert_eq!(engine.handle_key(KeyEvent::new(Key::Esc)), Action::QuitToMenu);
}

#[test]
fn h_key_requests_help_screen() {
    let game = PicrossGame;
    let config = game.default_config();
    let mut engine = game.create_engine(&config, &[]);
    assert_eq!(engine.handle_key(KeyEvent::new(Key::Char('h'))), Action::ShowHelp);
}

#[test]
fn help_screen_renders_a_non_blank_frame() {
    let game = PicrossGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    let mut grid = CellGrid::new(80, 40);
    engine.render_help(&mut grid);
    let drew_something = (0..40).any(|y| (0..80).any(|x| grid.get(x, y).glyph != ' '));
    assert!(drew_something, "render_help() produced an entirely blank frame");
}

#[test]
fn score_is_none_picross_has_no_numeric_score() {
    let game = PicrossGame;
    let config = game.default_config();
    let engine = game.create_engine(&config, &[]);
    assert_eq!(engine.score(), None);
}

#[test]
fn board_dims_matches_the_actual_puzzle_not_the_hardcoded_config_default() {
    // Config::board_width/height used to be hardcoded to 10 regardless of
    // the real puzzle -- board_dims() must reflect the actual engine.
    let game = PicrossGame;
    let entry = game.new_campaign_entry(2); // Hard track -- likely non-10x10
    let base = game.default_config();
    let cfg = game.campaign_config(&entry, &base);
    let engine = game.create_campaign_engine(&entry, &cfg, &[]);
    assert_eq!(engine.board_dims(), (cfg.board_rows as u16, cfg.board_cols as u16));
}

#[test]
fn campaign_entry_progresses_sequentially_and_completes() {
    let game = PicrossGame;
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
fn campaign_config_reflects_the_selected_puzzles_real_dimensions() {
    let game = PicrossGame;
    let base = game.default_config();
    let entry = game.new_campaign_entry(0);
    let cfg = game.campaign_config(&entry, &base);
    assert!(cfg.board_rows > 0 && cfg.board_cols > 0);
}

#[test]
fn create_campaign_engine_builds_the_looked_up_puzzle() {
    let game = PicrossGame;
    let base = game.default_config();
    let entry = game.new_campaign_entry(0);
    let cfg = game.campaign_config(&entry, &base);
    let engine = game.create_campaign_engine(&entry, &cfg, &[]);
    assert_eq!(engine.status(), GameStatus::Playing);
}

#[test]
fn campaign_advances_to_the_next_puzzles_real_dimensions() {
    // Sequential model check: after completing level 0, campaign_config for
    // the (now-advanced) entry must reflect level 1's puzzle, not level 0's.
    let game = PicrossGame;
    let mut entry = game.new_campaign_entry(0);
    let base = game.default_config();
    let cfg0 = game.campaign_config(&entry, &base);

    game.complete_campaign_level(&mut entry);
    assert_eq!(entry.current_level, 1);
    let cfg1 = game.campaign_config(&entry, &base);
    // Not asserting the dims differ (two puzzles could coincidentally
    // match) -- just that building an engine for the new level works.
    let engine = game.create_campaign_engine(&entry, &cfg1, &[]);
    assert_eq!(engine.board_dims(), (cfg1.board_rows as u16, cfg1.board_cols as u16));
    let _ = cfg0;
}

#[test]
fn needs_blessing_selection_is_false_picross_has_no_blessings() {
    use loom_engine::campaign::CampaignEntry;
    let game = PicrossGame;
    let entry = game.new_campaign_entry(0);
    assert!(!entry.needs_blessing_selection());
    assert!(game.all_blessings().is_empty());
}

#[test]
fn main_menu_items_is_campaign_and_quit_only() {
    use loom_engine::game::MenuItem;
    let game = PicrossGame;
    assert_eq!(game.main_menu_items(), &[MenuItem::Campaign, MenuItem::Quit]);
}

#[test]
fn pick_up_and_tick_do_not_panic_across_a_scripted_sequence() {
    let game = PicrossGame;
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
}
