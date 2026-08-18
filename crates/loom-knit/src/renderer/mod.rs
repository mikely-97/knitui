#![allow(dead_code)]

mod board;
mod panels;
pub use board::*;
pub use panels::*;

use loom_engine::render::{Color, Style, Surface};
use crate::config::Config;
use crate::engine::{GameEngine, GameStatus};

// ── Spacing constants ────────────────────────────────────────────────────────
pub const YARN_HGAP: u16 = 2;   // horizontal gap between yarn columns
pub const YARN_VGAP: u16 = 1;   // vertical gap between yarn rows (< YARN_HGAP)
pub const THREAD_GAP: u16 = 1;  // gap between active threads
pub const COMP_GAP: u16 = 3;    // gap between components (> all inner gaps)

#[derive(Clone, Copy)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy)]
pub enum FlankSide { Left, Right }

pub fn detect_layout(config_layout: &str, visible_stitches: u16, board_height: u16, scale: u16, term_height: u16) -> Layout {
    match config_layout {
        "horizontal" => Layout::Horizontal,
        "vertical" => Layout::Vertical,
        _ => {
            let sh = scale;
            let yarn_h = visible_stitches * sh + visible_stitches.saturating_sub(1) * YARN_VGAP;
            let board_h = 1 + board_height * (sh + 1);
            let vertical_height = yarn_h + COMP_GAP + sh + COMP_GAP + board_h;
            if vertical_height + 2 > term_height {
                Layout::Horizontal
            } else {
                Layout::Vertical
            }
        }
    }
}

/// Compute layout + component-origin geometry for a config and terminal
/// height. Portable (no crossterm) — the caller owns the term-height query
/// (native: `crossterm::terminal::size()`; Surface-backed callers such as
/// the `GameEngine` trait adapter: `RenderArea::height`). Returns
/// `(layout, yarn_x, board_x, board_y)`.
pub fn compute_geometry(config: &Config, term_height: u16) -> (Layout, u16, u16, u16) {
    let scale = config.scale;
    let sh = scale;
    let sw = scale * 2;

    let layout = detect_layout(
        &config.layout, config.visible_stitches, config.board_height, scale, term_height,
    );

    let yarn_h = config.visible_stitches * sh
        + config.visible_stitches.saturating_sub(1) * YARN_VGAP;
    let board_y: u16 = yarn_h + COMP_GAP + sh + COMP_GAP;

    let yarn_w = config.yarn_lines * sw
        + config.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let has_flanks = config.balloons > 0 && config.balloon_count > 0;
    let (yarn_x, board_x) = if has_flanks {
        let has_left  = config.balloon_count / 2 > 0;
        let has_right = (config.balloon_count + 1) / 2 > 0;
        let left_w  = if has_left  { sw } else { 0 };
        let right_w = if has_right { sw } else { 0 };
        let left_gap  = if has_left  { YARN_HGAP } else { 0 };
        let right_gap = if has_right { YARN_HGAP } else { 0 };
        let yx = left_w + left_gap;
        let bx = yx + yarn_w + right_gap + right_w + COMP_GAP + sw + COMP_GAP;
        (yx, bx)
    } else {
        (0u16, yarn_w + COMP_GAP + sw + COMP_GAP)
    };

    (layout, yarn_x, board_x, board_y)
}

/// Render the vertical layout directly into a caller-owned `Surface` (no
/// frame lifecycle of its own -- the caller clears/blits). This is the
/// entry point every frontend goes through now (native rendering via
/// `Shell<KnitGame>` wraps a `TermSurface` around this call; web via
/// `WasmSurface`).
pub fn render_vertical_to_surface(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
) {
    let sh = scale;
    let yarn_h = engine.yarn.visible_stitches * sh
        + engine.yarn.visible_stitches.saturating_sub(1) * YARN_VGAP;
    let active_y = yarn_h + COMP_GAP;

    render_yarn(surface, engine, 0, 0, scale, true);
    render_active_h(surface, engine, 0, active_y, scale);
    render_board(surface, engine, 0, board_y, scale);

    let board_h = 1 + engine.board.height * (sh + 1);
    let bonus_y = board_y + board_h + 1;
    render_bonus_display_h(surface, engine, 0, bonus_y);

    let (_, term_h) = surface.size();
    render_keybar(surface, engine, term_h.saturating_sub(1));
}

/// Render the horizontal layout directly into a caller-owned `Surface` (see
/// [`render_vertical_to_surface`] for the rationale).
pub fn render_horizontal_to_surface(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    yarn_x: u16,
    board_x: u16,
    scale: u16,
) {
    let sw = scale * 2;
    let yarn_w = engine.yarn.yarn_lines * sw
        + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let active_x = board_x - COMP_GAP - sw;

    if yarn_x > 0 {
        render_balloon_flank(surface, engine, 0, 0, scale, FlankSide::Left);
    }

    render_yarn(surface, engine, yarn_x, 0, scale, false);

    let right_flank_x = yarn_x + yarn_w + YARN_HGAP;
    if right_flank_x < active_x {
        render_balloon_flank(surface, engine, right_flank_x, 0, scale, FlankSide::Right);
    }

    render_active_v(surface, engine, active_x, 0, scale);
    render_board(surface, engine, board_x, 0, scale);

    let board_w = 1 + engine.board.width * (sw + 1);
    let panel_x = board_x + board_w + 2;
    render_bonus_panel(surface, engine, panel_x, 0);

    let (_, term_h) = surface.size();
    render_keybar(surface, engine, term_h.saturating_sub(1));
}

/// Draw the stuck/won overlay message (and, when stuck, the bonus-usage
/// hint row) into a caller-owned `Surface`. Shared by the terminal overlay
/// wrappers below and by the portable `GameEngine` trait adapter.
pub fn draw_overlay_to_surface(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) {
    draw_stuck_overlay_inner(surface, engine, status, overlay_msg);
}

// ── Rendering ─────────────────────────────────────────────────────────────────
// loom-knit is now driven entirely through loom_engine::shell::Shell<KnitGame>
// (Phase 3), which only calls the Surface-only entry points above. The
// Stdout-wrapping functions that used to be called directly from tui.rs's
// hand-rolled loop (render_vertical/render_horizontal/do_render/... and
// their _overlay variants) were deleted here as dead code once that loop
// was replaced -- see git history if a future frontend needs a
// Stdout-owning entry point again.

fn draw_stuck_overlay_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) {
    if matches!(status, GameStatus::Stuck) && overlay_msg.is_none() {
        let b = &engine.bonuses;
        let s_txt = format!("[Z] \u{2702} x{}  ", b.scissors);
        let t_txt = format!("[X] \u{2295} x{}  ", b.tweezers);
        let c_txt = format!("[C] \u{25cb} x{}", b.balloons);
        let grey = Style { fg: Color::DarkGrey, ..Default::default() };

        let mut x = 0u16;
        let s_style = if b.scissors == 0 { grey } else { Style::default() };
        surface.print(x, 0, &s_txt, s_style);
        x += s_txt.chars().count() as u16;

        let t_style = if b.tweezers == 0 { grey } else { Style::default() };
        surface.print(x, 0, &t_txt, t_style);
        x += t_txt.chars().count() as u16;

        let c_style = if b.balloons == 0 { grey } else { Style::default() };
        surface.print(x, 0, &c_txt, c_style);
        x += c_txt.chars().count() as u16;

        if engine.can_watch_ad() {
            surface.print(x, 0, "  [A] Watch ad", Style::default());
        }
        surface.print(0, 1, "You're lost! R:Restart  M:Menu  Q:Quit", Style::default());
    } else {
        let default_msg = match status {
            GameStatus::Stuck => "You're lost! R:Restart  M:Menu  Q:Quit",
            GameStatus::Won   => "You won! R:Restart  M:Menu  Q:Quit",
            _ => return,
        };
        let message = overlay_msg.unwrap_or(default_msg);
        surface.print(0, 0, message, Style::default());
    }
}

