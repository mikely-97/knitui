#![allow(warnings)]

use std::io::{self, Write, Stdout};

use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Color, SetForegroundColor, ResetColor, SetBackgroundColor, Attribute, SetAttribute},
    terminal::{self, Clear, ClearType},
    cursor::{MoveTo, Hide},
};

use crate::engine::GameEngine;
use crate::bonuses::BonusState;

mod board;
mod panels;

pub use board::render_board;
pub use panels::{
    render_hud,
    render_key_bar,
    render_help,
    render_game_over,
    render_main_menu,
    render_options,
    render_blessing_selection,
    render_celebration,
    render_level_summary,
};

// ── Layout constants ──────────────────────────────────────────────────────

pub const CELL_GAP: u16 = 1;   // gap between cells (columns) in cells * scale*2
pub const COMP_GAP: u16 = 3;   // gap between components (board vs HUD panel)

#[derive(Clone, Copy, Debug)]
pub enum Layout { Vertical, Horizontal }

/// Decide vertical vs horizontal based on terminal height.
pub fn detect_layout(board_height: usize, board_width: usize, scale: u16) -> Layout {
    let sh = scale;
    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    let board_h = board_height as u16 * (sh + CELL_GAP);
    let hud_h = 6u16;
    if board_h + hud_h + 4 <= term_h {
        Layout::Vertical
    } else {
        Layout::Horizontal
    }
}

// ── LayoutGeometry ────────────────────────────────────────────────────────

pub struct LayoutGeometry {
    pub layout:  Layout,
    pub board_x: u16,
    pub board_y: u16,
    pub hud_x:   u16,
    pub hud_y:   u16,
    pub scale:   u16,
}

impl LayoutGeometry {
    pub fn compute(board_height: usize, board_width: usize, scale: u16) -> Self {
        let layout = detect_layout(board_height, board_width, scale);
        let sh = scale;
        let sw = scale * 2;
        let cell_w = sw + CELL_GAP;
        let board_render_w = board_width as u16 * cell_w;

        match layout {
            Layout::Vertical => Self {
                layout,
                board_x: 2,
                board_y: 6,
                hud_x: 2,
                hud_y: 0,
                scale,
            },
            Layout::Horizontal => Self {
                layout,
                board_x: 20,
                board_y: 1,
                hud_x: 0,
                hud_y: 1,
                scale,
            },
        }
    }
}

// ── do_render ─────────────────────────────────────────────────────────────

/// Full frame render during Playing state.
pub fn do_render(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    objective_label: &str,
) -> io::Result<()> {
    // BeginSynchronizedUpdate / EndSynchronizedUpdate not available in crossterm 0.27
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    render_hud(stdout, engine, geo, objective_label)?;
    render_board(stdout, engine, geo)?;
    render_key_bar(stdout, &engine.bonus_state)?;

    stdout.flush()?;
    Ok(())
}
