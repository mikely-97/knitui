#![allow(warnings)]

mod board;
mod panels;
pub use board::*;
pub use panels::*;

use std::io::{self, Write, Stdout};
use crossterm::{
    QueueableCommand,
    terminal::{self, Clear, ClearType, BeginSynchronizedUpdate, EndSynchronizedUpdate},
    cursor::{Hide},
};
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
            let sh = scale;
            let (_, term_height) = terminal::size().unwrap_or((80, 24));
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

// ── Rendering ─────────────────────────────────────────────────────────────────

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

    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    render_yarn(stdout, engine, 0, 0, scale, true)?;
    render_active_h(stdout, engine, 0, active_y, scale)?;
    render_board(stdout, engine, 0, board_y, scale)?;

    let board_h = 1 + engine.board.height * (sh + 1);
    let bonus_y = board_y + board_h + 1;
    render_bonus_display_h(stdout, engine, 0, bonus_y)?;

    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    render_keybar(stdout, engine, term_h.saturating_sub(1))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

fn draw_stuck_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) -> io::Result<()> {
    use crossterm::style::{Print, Stylize};
    use crossterm::cursor::MoveTo;
    if matches!(status, GameStatus::Stuck) && overlay_msg.is_none() {
        let b = &engine.bonuses;
        let s_txt = format!("[Z] \u{2702} x{}  ", b.scissors);
        let t_txt = format!("[X] \u{2295} x{}  ", b.tweezers);
        let b_txt = format!("[C] \u{25cb} x{}", b.balloons);
        stdout.queue(MoveTo(0, 0))?;
        if b.scissors == 0 { stdout.queue(Print(s_txt.as_str().dark_grey()))?; } else { stdout.queue(Print(&s_txt))?; }
        if b.tweezers == 0 { stdout.queue(Print(t_txt.as_str().dark_grey()))?; } else { stdout.queue(Print(&t_txt))?; }
        if b.balloons == 0 { stdout.queue(Print(b_txt.as_str().dark_grey()))?; } else { stdout.queue(Print(&b_txt))?; }
        if engine.can_watch_ad() { stdout.queue(Print("  [A] Watch ad"))?; }
        stdout.queue(MoveTo(0, 1))?;
        stdout.queue(Print("You're lost! R:Restart  M:Menu  Q:Quit"))?;
    } else {
        use crossterm::style::Print;
        use crossterm::cursor::MoveTo;
        let default_msg = match status {
            GameStatus::Stuck => "You're lost! R:Restart  M:Menu  Q:Quit",
            GameStatus::Won   => "You won! R:Restart  M:Menu  Q:Quit",
            _ => return Ok(()),
        };
        let message = overlay_msg.unwrap_or(default_msg);
        stdout.queue(MoveTo(0, 0))?;
        stdout.queue(Print(message))?;
    }
    Ok(())
}

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

    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    // Left balloon flank (deeper patches)
    if yarn_x > 0 {
        render_balloon_flank(stdout, engine, 0, 0, scale, FlankSide::Left)?;
    }

    // Yarn columns
    render_yarn(stdout, engine, yarn_x, 0, scale, false)?;

    // Right balloon flank (front patches)
    let right_flank_x = yarn_x + yarn_w + YARN_HGAP;
    if right_flank_x < active_x {
        render_balloon_flank(stdout, engine, right_flank_x, 0, scale, FlankSide::Right)?;
    }

    render_active_v(stdout, engine, active_x, 0, scale)?;
    render_board(stdout, engine, board_x, 0, scale)?;

    let board_w = 1 + engine.board.width * (sw + 1);
    let panel_x = board_x + board_w + 2;
    render_bonus_panel(stdout, engine, panel_x, 0)?;

    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    render_keybar(stdout, engine, term_h.saturating_sub(1))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

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
