#![allow(warnings)]

mod board;
mod panels;
pub use board::*;
pub use panels::*;

#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Stdout, Write};
#[cfg(not(target_arch = "wasm32"))]
use crossterm::terminal;
#[cfg(not(target_arch = "wasm32"))]
use loom_engine_term::TermSurface;
use loom_engine::render::{Color, Style, Surface};
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

pub fn detect_layout(config_layout: &str, visible_stitches: u16, board_height: u16, scale: u16) -> Layout {
    match config_layout {
        "horizontal" => Layout::Horizontal,
        "vertical" => Layout::Vertical,
        _ => {
            #[cfg(not(target_arch = "wasm32"))]
            let term_height = terminal::size().unwrap_or((80, 24)).1;
            #[cfg(target_arch = "wasm32")]
            let term_height = 24u16;

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

/// Render the vertical layout directly into a caller-owned `Surface` (no
/// frame lifecycle of its own -- the caller clears/blits). This is the
/// entry point non-terminal frontends (web, tests) should use instead of
/// [`render_vertical`], which additionally owns a `TermSurface` frame.
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

// ── Rendering ─────────────────────────────────────────────────────────────────
// The functions below own a terminal frame (TermSurface::begin/finish) and
// are native-only; non-terminal frontends should call
// `render_vertical_to_surface` (above) directly instead.

#[cfg(not(target_arch = "wasm32"))]
pub fn render_vertical(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
) -> io::Result<()> {
    let sh = scale;
    let yarn_h = engine.yarn.visible_stitches * sh
        + engine.yarn.visible_stitches.saturating_sub(1) * YARN_VGAP;
    let active_y = yarn_h + COMP_GAP;

    let mut surface = TermSurface::begin(stdout)?;

    render_yarn(&mut surface, engine, 0, 0, scale, true);
    render_active_h(&mut surface, engine, 0, active_y, scale);
    render_board(&mut surface, engine, 0, board_y, scale);

    let board_h = 1 + engine.board.height * (sh + 1);
    let bonus_y = board_y + board_h + 1;
    render_bonus_display_h(&mut surface, engine, 0, bonus_y);

    let (_, term_h) = surface.size();
    render_keybar(&mut surface, engine, term_h.saturating_sub(1));

    surface.finish()
}

#[cfg(not(target_arch = "wasm32"))]
fn draw_stuck_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    draw_stuck_overlay_inner(&mut surface, engine, status, overlay_msg);
    surface.done()
}

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

#[cfg(not(target_arch = "wasm32"))]
pub fn render_vertical_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) -> io::Result<()> {
    render_vertical(stdout, engine, board_y, scale)?;
    draw_stuck_overlay(stdout, engine, status, overlay_msg)?;
    stdout.flush()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_horizontal(
    stdout: &mut Stdout,
    engine: &GameEngine,
    yarn_x: u16,
    board_x: u16,
    scale: u16,
) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    let yarn_w = engine.yarn.yarn_lines * sw
        + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let active_x = board_x - COMP_GAP - sw;

    let mut surface = TermSurface::begin(stdout)?;

    // Left balloon flank (deeper patches)
    if yarn_x > 0 {
        render_balloon_flank(&mut surface, engine, 0, 0, scale, FlankSide::Left);
    }

    // Yarn columns
    render_yarn(&mut surface, engine, yarn_x, 0, scale, false);

    // Right balloon flank (front patches)
    let right_flank_x = yarn_x + yarn_w + YARN_HGAP;
    if right_flank_x < active_x {
        render_balloon_flank(&mut surface, engine, right_flank_x, 0, scale, FlankSide::Right);
    }

    render_active_v(&mut surface, engine, active_x, 0, scale);
    render_board(&mut surface, engine, board_x, 0, scale);

    let board_w = 1 + engine.board.width * (sw + 1);
    let panel_x = board_x + board_w + 2;
    render_bonus_panel(&mut surface, engine, panel_x, 0);

    let (_, term_h) = surface.size();
    render_keybar(&mut surface, engine, term_h.saturating_sub(1));

    surface.finish()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_horizontal_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    yarn_x: u16,
    board_x: u16,
    scale: u16,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) -> io::Result<()> {
    render_horizontal(stdout, engine, yarn_x, board_x, scale)?;
    draw_stuck_overlay(stdout, engine, status, overlay_msg)?;
    stdout.flush()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn do_render(
    stdout: &mut Stdout,
    engine: &GameEngine,
    layout: Layout,
    yarn_x: u16,
    board_x: u16,
    board_y: u16,
    scale: u16,
) -> io::Result<()> {
    match layout {
        Layout::Vertical => render_vertical(stdout, engine, board_y, scale),
        Layout::Horizontal => render_horizontal(stdout, engine, yarn_x, board_x, scale),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn do_render_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    layout: Layout,
    yarn_x: u16,
    board_x: u16,
    board_y: u16,
    scale: u16,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) -> io::Result<()> {
    match layout {
        Layout::Vertical => render_vertical_overlay(stdout, engine, board_y, scale, status, overlay_msg),
        Layout::Horizontal => render_horizontal_overlay(stdout, engine, yarn_x, board_x, scale, status, overlay_msg),
    }
}
