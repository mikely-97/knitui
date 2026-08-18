#![allow(dead_code)]

#[cfg(not(target_arch = "wasm32"))]
use crossterm::terminal;
use loom_engine::render::Surface;

use crate::engine::GameEngine;

mod board;
mod panels;

pub use board::render_board;
pub use panels::{
    render_hud, render_key_bar,
    render_help_to_surface, render_celebration_to_surface, render_game_over_to_surface,
};

// ── Layout constants ──────────────────────────────────────────────────────

pub const CELL_GAP: u16 = 1;   // gap between cells (columns) in cells * scale*2
pub const COMP_GAP: u16 = 3;   // gap between components (board vs HUD panel)

#[derive(Clone, Copy, Debug)]
pub enum Layout { Vertical, Horizontal }

/// Decide vertical vs horizontal for a given render-target height. Portable
/// (no crossterm/wasm branching) — the caller owns the height query (native:
/// `crossterm::terminal::size()`; the `GameEngine` trait adapter:
/// `RenderArea::height`; wasm's `web.rs`: its own fixed canvas-row count).
pub fn detect_layout(board_height: usize, _board_width: usize, scale: u16, term_height: u16) -> Layout {
    let sh = scale;
    let board_h = board_height as u16 * (sh + CELL_GAP);
    let hud_h = 6u16;
    if board_h + hud_h + 4 <= term_height {
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
    /// Native/wasm convenience: resolves the render-target height itself
    /// (real terminal size on native, a fixed row count on wasm). Used by
    /// `web.rs`'s wasm build.
    pub fn compute(board_height: usize, board_width: usize, scale: u16) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let term_h = terminal::size().unwrap_or((80, 24)).1;
        #[cfg(target_arch = "wasm32")]
        let term_h = 24u16;
        Self::for_height(board_height, board_width, scale, term_h)
    }

    /// Portable: caller supplies the render-target height explicitly (the
    /// `GameEngine` trait adapter passes `RenderArea::height`; headless
    /// tests pass a `CellGrid`'s height). No crossterm/wasm branching.
    pub fn for_height(board_height: usize, board_width: usize, scale: u16, term_height: u16) -> Self {
        let layout = detect_layout(board_height, board_width, scale, term_height);

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

// ── do_render_to_surface ──────────────────────────────────────────────────
// match3 is now driven entirely through loom_engine::shell::Shell<M3Game>
// (Phase 4, native) or web.rs (wasm) -- both go through this Surface-only
// entry point. The old Stdout-owning `do_render` wrapper (and the matching
// Stdout wrappers for help/celebration/game-over/main-menu/options/
// blessing-selection/level-summary in panels.rs) were deleted as dead code
// once tui.rs's hand-rolled loop was replaced -- see git history if a
// future frontend needs a Stdout-owning entry point again.

/// Render directly into a caller-owned `Surface`, no frame lifecycle of its
/// own.
pub fn do_render_to_surface(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    objective_label: &str,
) {
    render_hud(surface, engine, geo, objective_label);
    render_board(surface, engine, geo);
    render_key_bar(surface, &engine.bonus_state);
}
