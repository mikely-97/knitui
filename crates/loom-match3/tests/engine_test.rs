//! Integration tests for loom-match3 GameEngine.
//!
//! Tests live in `tests/` (auto-discovered by Cargo) so they can only
//! access the public API of the crate.

use clap::Parser;

use m3tui::board::{Board, Cell, CellContent, TileModifier};
use m3tui::bonuses::BonusState;
use m3tui::config::Config;
use m3tui::engine::{GameEngine, GamePhase, GameStatus};

use loom_engine::render::Color;

// ── Helpers ───────────────────────────────────────────────────────────────

fn default_config() -> Config {
    Config::parse_from::<[&str; 0], &str>([])
}

fn engine() -> GameEngine {
    GameEngine::new(&default_config())
}

fn config_with(move_limit: u32) -> Config {
    let mut c = default_config();
    c.move_limit = move_limit;
    c
}

// ── Sanity: dimensions and initial state ──────────────────────────────────

#[test]
fn board_initializes_with_correct_dimensions() {
    let e = engine();
    assert_eq!(e.board.height, 8);
    assert_eq!(e.board.width, 8);
    assert_eq!(e.board.cells.len(), 8);
    assert!(e.board.cells.iter().all(|row| row.len() == 8));
}

#[test]
fn game_status_starts_as_playing() {
    let mut e = engine();
    // A fresh board always has valid swaps (8×8, 6 colors, no modifiers by default)
    // so status must be Playing.
    let status = e.game_status();
    assert_eq!(status, GameStatus::Playing);
}

#[test]
fn phase_starts_as_player_input() {
    let e = engine();
    assert!(matches!(e.phase, GamePhase::PlayerInput));
}

// ── Sanity: valid swap → Resolving phase ──────────────────────────────────

#[test]
fn valid_swap_triggers_resolving_phase() {
    let mut e = engine();

    // Build a guaranteed 3-match in row 7 (bottom row).
    // Place Red at [7][0], [7][1], [7][3] and Blue at [7][2].
    // Swapping [7][2] and [7][3] gives three consecutive Reds → match.
    e.board.cells[7][0] = Cell::gem(Color::Red);
    e.board.cells[7][1] = Cell::gem(Color::Red);
    e.board.cells[7][2] = Cell::gem(Color::Blue);
    e.board.cells[7][3] = Cell::gem(Color::Red);
    // Prevent vertical match with row 6
    e.board.cells[6][0] = Cell::gem(Color::Blue);
    e.board.cells[6][1] = Cell::gem(Color::Blue);
    e.board.cells[6][2] = Cell::gem(Color::Red);
    e.board.cells[6][3] = Cell::gem(Color::Blue);
    // Row 5 to break any accidental vertical
    e.board.cells[5][0] = Cell::gem(Color::Green);
    e.board.cells[5][1] = Cell::gem(Color::Green);
    e.board.cells[5][2] = Cell::gem(Color::Blue);
    e.board.cells[5][3] = Cell::gem(Color::Green);

    // Select [7][2] (Blue), then move cursor to [7][3] (Red) and confirm swap.
    e.cursor_row = 7;
    e.cursor_col = 2;
    e.confirm_selection(); // select Blue at (7,2)
    e.cursor_col = 3;
    e.confirm_selection(); // attempt swap with Red at (7,3) → 3-Red match

    assert!(
        matches!(e.phase, GamePhase::Resolving { .. }),
        "Expected Resolving after a valid swap, got {:?}", e.phase
    );
}

// ── Move limit warning: remaining count ───────────────────────────────────

#[test]
fn move_limit_remaining_starts_at_full() {
    let e = GameEngine::new(&config_with(5));
    let remaining = e.move_limit.saturating_sub(e.moves_used);
    assert_eq!(remaining, 5);
}

#[test]
fn moves_used_increments_on_valid_swap() {
    let mut e = engine();

    // Set up the same forced match as above
    e.board.cells[7][0] = Cell::gem(Color::Red);
    e.board.cells[7][1] = Cell::gem(Color::Red);
    e.board.cells[7][2] = Cell::gem(Color::Blue);
    e.board.cells[7][3] = Cell::gem(Color::Red);
    e.board.cells[6][0] = Cell::gem(Color::Blue);
    e.board.cells[6][1] = Cell::gem(Color::Blue);
    e.board.cells[6][2] = Cell::gem(Color::Red);
    e.board.cells[6][3] = Cell::gem(Color::Blue);
    e.board.cells[5][0] = Cell::gem(Color::Green);
    e.board.cells[5][1] = Cell::gem(Color::Green);
    e.board.cells[5][2] = Cell::gem(Color::Blue);
    e.board.cells[5][3] = Cell::gem(Color::Green);

    let before = e.moves_used;
    e.cursor_row = 7;
    e.cursor_col = 2;
    e.confirm_selection();
    e.cursor_col = 3;
    e.confirm_selection();

    assert_eq!(e.moves_used, before + 1);
    let remaining = e.move_limit.saturating_sub(e.moves_used);
    assert_eq!(remaining, e.move_limit - before - 1);
}

#[test]
fn out_of_moves_when_moves_used_equals_limit() {
    let mut e = GameEngine::new(&config_with(5));
    e.moves_used = 5;
    assert_eq!(e.game_status(), GameStatus::OutOfMoves);
}

#[test]
fn playing_when_moves_remain() {
    let mut e = GameEngine::new(&config_with(5));
    e.moves_used = 3;
    // 2 moves left; as long as there's a valid swap the engine reports Playing.
    let status = e.game_status();
    assert!(
        status == GameStatus::Playing || status == GameStatus::Stuck,
        "Expected Playing or Stuck, got {:?}", status
    );
}

// ── Combo tracking: cascade_depth → combo_display_ticks ───────────────────

#[test]
fn combo_display_ticks_set_on_cascade() {
    let mut e = engine();

    // Force a cascade: put a 3-Red match on the board, enter Refilling phase,
    // so that the refill finds new matches and increments cascade_depth.
    // Approach: manually set cascade_depth = 2 and put a match on the board,
    // then enter Refilling — the engine will detect the match, set combo_display_ticks = 30.

    // Ensure combo_display_ticks starts at 0.
    assert_eq!(e.combo_display_ticks, 0);

    // Place a guaranteed 3-Red horizontal match and isolate it.
    e.board.cells[7][0] = Cell::gem(Color::Red);
    e.board.cells[7][1] = Cell::gem(Color::Red);
    e.board.cells[7][2] = Cell::gem(Color::Red);
    e.board.cells[6][0] = Cell::gem(Color::Blue);
    e.board.cells[6][1] = Cell::gem(Color::Blue);
    e.board.cells[6][2] = Cell::gem(Color::Blue);
    e.board.cells[5][0] = Cell::gem(Color::Green);
    e.board.cells[5][1] = Cell::gem(Color::Green);
    e.board.cells[5][2] = Cell::gem(Color::Green);

    // Manually prime cascade_depth to 1 (so next cascade brings it to 2).
    e.cascade_depth = 1;
    e.phase = GamePhase::Refilling;

    // tick() in Refilling: refills board, finds the Red match → cascade_depth becomes 2 → sets combo_display_ticks = 30.
    e.tick();

    assert_eq!(e.combo_display_ticks, 30,
        "combo_display_ticks should be 30 when cascade_depth reaches 2");
    assert!(e.cascade_depth >= 2);
}

#[test]
fn combo_display_ticks_decrements_each_tick() {
    let mut e = engine();
    e.combo_display_ticks = 10;
    e.phase = GamePhase::PlayerInput;
    e.tick();
    assert_eq!(e.combo_display_ticks, 9);
}

// ── Ice tiles: two-hit mechanic ────────────────────────────────────────────

#[test]
fn ice_first_hit_does_not_clear_gem() {
    let mut e = engine();

    // Place a Red gem with Ice { hp: 2 } at (4, 4).
    e.board.cells[4][4] = Cell {
        content: CellContent::Gem { color: Color::Red, special: None },
        modifier: Some(TileModifier::Ice { hp: 2 }),
    };

    // Use Hammer (direct hit = one damage).
    e.bonuses.hammer = 1;
    e.cursor_row = 4;
    e.cursor_col = 4;
    e.activate_hammer();
    e.cursor_row = 4;
    e.cursor_col = 4;
    e.confirm_hammer();

    // After one direct hit: hp goes 2 → 1; cell content cleared by hammer,
    // but modifier should be gone at hp=1? Let's check actual behaviour:
    // damage_modifier: hp=2 → new_hp=1 → modifier stays; then confirm_hammer
    // sets content = Empty regardless.
    // So after first hammer: content=Empty, modifier=Some(Ice{hp:1}).
    // The gem IS gone (hammer always clears content), but modifier is reduced.
    assert!(
        matches!(e.board.cells[4][4].modifier, Some(TileModifier::Ice { hp: 1 })),
        "Ice hp should be 1 after one direct hit, got: {:?}", e.board.cells[4][4].modifier
    );
}

#[test]
fn ice_second_hit_removes_modifier() {
    let mut e = engine();

    // Place a gem with Ice { hp: 2 } at (4, 4).
    e.board.cells[4][4] = Cell {
        content: CellContent::Gem { color: Color::Red, special: None },
        modifier: Some(TileModifier::Ice { hp: 2 }),
    };

    // First hammer hit
    e.bonuses.hammer = 1;
    e.cursor_row = 4;
    e.cursor_col = 4;
    e.activate_hammer();
    e.confirm_hammer();

    // Restore the gem content for the second hit (the ice is still there at hp:1).
    e.board.cells[4][4].content = CellContent::Gem { color: Color::Red, special: None };
    e.phase = GamePhase::PlayerInput;

    // Second hammer hit
    e.bonuses.hammer = 1;
    e.cursor_row = 4;
    e.cursor_col = 4;
    e.activate_hammer();
    e.confirm_hammer();

    // After second direct hit: hp 1 → 0 → modifier removed.
    assert!(
        e.board.cells[4][4].modifier.is_none(),
        "Ice modifier should be gone after second hit, got: {:?}", e.board.cells[4][4].modifier
    );
}

#[test]
fn ice_tile_placed_by_board_has_hp_2() {
    // Board::make_random with ice_tile_pct=100 should produce Ice{hp:2} cells.
    let palette = vec![Color::Red, Color::Blue, Color::Green, Color::Yellow];
    let b = Board::make_random(4, 4, &palette, 0, 100);
    let ice_cells: Vec<_> = b.cells.iter().flatten()
        .filter(|c| matches!(c.modifier, Some(TileModifier::Ice { hp: 2 })))
        .collect();
    assert!(!ice_cells.is_empty(), "Board with ice_tile_pct=100 should have Ice cells");
}

// ── Color bomb ────────────────────────────────────────────────────────────

#[test]
fn color_bomb_activate_enters_targeting_mode() {
    let mut e = engine();
    e.bonuses.color_bomb = 1;
    e.activate_color_bomb();
    assert!(matches!(e.bonus_state, BonusState::ColorBombActive { .. }));
    assert_eq!(e.bonuses.color_bomb, 0);
}

#[test]
fn color_bomb_confirm_removes_all_matching_color() {
    let mut e = engine();

    // Fill board with Blue, then plant exactly 3 Red gems.
    for r in 0..e.board.height {
        for c in 0..e.board.width {
            e.board.cells[r][c] = Cell::gem(Color::Blue);
        }
    }
    e.board.cells[0][0] = Cell::gem(Color::Red);
    e.board.cells[2][3] = Cell::gem(Color::Red);
    e.board.cells[5][7] = Cell::gem(Color::Red);

    // Activate color bomb, aim at a Red cell.
    e.bonuses.color_bomb = 1;
    e.activate_color_bomb();
    e.cursor_row = 0;
    e.cursor_col = 0; // Red cell
    e.confirm_color_bomb();

    // All Red cells should now be Empty.
    assert_eq!(e.board.cells[0][0].content, CellContent::Empty, "(0,0) Red should be cleared");
    assert_eq!(e.board.cells[2][3].content, CellContent::Empty, "(2,3) Red should be cleared");
    assert_eq!(e.board.cells[5][7].content, CellContent::Empty, "(5,7) Red should be cleared");

    // Blue cells must be untouched.
    assert!(
        matches!(e.board.cells[0][1].content, CellContent::Gem { color: Color::Blue, .. }),
        "Blue cell should be untouched"
    );

    // Phase transitions to Falling after color bomb.
    assert!(matches!(e.phase, GamePhase::Falling),
        "Phase should be Falling after color bomb, got {:?}", e.phase);
}

#[test]
fn color_bomb_at_zero_does_nothing() {
    let mut e = engine();
    e.bonuses.color_bomb = 0;
    e.activate_color_bomb();
    assert_eq!(e.bonus_state, BonusState::None);
}

#[test]
fn color_bomb_cancel_refunds_charge() {
    let mut e = engine();
    e.bonuses.color_bomb = 1;
    e.activate_color_bomb();
    e.cancel_bonus();
    assert_eq!(e.bonuses.color_bomb, 1);
    assert_eq!(e.bonus_state, BonusState::None);
}
