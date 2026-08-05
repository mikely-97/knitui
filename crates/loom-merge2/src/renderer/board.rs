#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Stdout};

use loom_engine::render::{Attrs, Color, Style, Surface};
#[cfg(not(target_arch = "wasm32"))]
use loom_engine_term::TermSurface;

use loom_engine::anim::AnimKind;
use crate::board::Cell;
use crate::engine::GameEngine;
use crate::glyphs::{self, cell_dims};
use super::LayoutGeometry;

// ── Grid helpers ──────────────────────────────────────────────────────────

fn grid_char(up: bool, down: bool, left: bool, right: bool) -> char {
    match (up, down, left, right) {
        (false, true,  false, true ) => '┌',
        (false, true,  true,  true ) => '┬',
        (false, true,  true,  false) => '┐',
        (true,  true,  false, true ) => '├',
        (true,  true,  true,  true ) => '┼',
        (true,  true,  true,  false) => '┤',
        (true,  false, false, true ) => '└',
        (true,  false, true,  true ) => '┴',
        (true,  false, true,  false) => '┘',
        _ => '+',
    }
}

fn render_grid_row(
    surface: &mut dyn Surface,
    bx: u16, by: u16,
    gr: usize, cw: usize,
    rows: usize, cols: usize,
) {
    let mut line = String::new();
    for gc in 0..=cols {
        let up = gr > 0;
        let down = gr < rows;
        let left = gc > 0;
        let right = gc < cols;
        line.push(grid_char(up, down, left, right));
        if gc < cols {
            for _ in 0..cw { line.push('─'); }
        }
    }
    surface.print(bx, by, &line, Style::default());
}

// ── Cell rendering ────────────────────────────────────────────────────────

/// Render one cell's content at explicit (x, y). Always emits exactly `cw`
/// characters, so callers can advance by `cw` unconditionally.
fn render_cell_content(
    surface: &mut dyn Surface,
    x: u16, y: u16,
    engine: &GameEngine,
    r: usize, c: usize,
    sub_row: usize,
    cw: usize, ch: usize,
) {
    let cell = &engine.board.cells[r][c];
    let mid = if ch > 1 { ch / 2 } else { 0 };
    let is_cursor   = r == engine.cursor_row && c == engine.cursor_col;
    let is_selected = engine.selected == Some((r, c));
    let is_hint     = engine.hint_pair
        .map_or(false, |(a, b)| a == (r, c) || b == (r, c));

    let centered = |label: &str| -> String {
        let chars = label.chars().count();
        let pad = cw.saturating_sub(chars);
        format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))
    };

    // Animation overlay: render burst frame and return early (skip normal content)
    if sub_row == mid {
        if let Some(anim) = engine.anim_cells.get((r, c)) {
            match (anim.kind, anim.frame) {
                (AnimKind::Dissolve, 3) => {
                    surface.print(x, y, &centered("██"), Style { fg: Color::White, attrs: Attrs::BOLD, ..Default::default() });
                    return;
                }
                (AnimKind::Dissolve, 2) => {
                    surface.print(x, y, &centered("✦ "), Style { fg: Color::Yellow, ..Default::default() });
                    return;
                }
                (AnimKind::Dissolve, 1) => {
                    surface.print(x, y, &centered("· "), Style { fg: Color::DarkGrey, ..Default::default() });
                    return;
                }
                (AnimKind::Rise, 3) => {
                    surface.print(x, y, &centered("✦ "), Style { fg: Color::Cyan, ..Default::default() });
                    return;
                }
                (AnimKind::Rise, 2) => {
                    surface.print(x, y, &centered("★ "), Style { fg: Color::Yellow, ..Default::default() });
                    return;
                }
                (AnimKind::Rise, 1) => {
                    // Frame 1: render actual item with bold/bright to show it just appeared
                    let (label, color, _) = glyphs::cell_label(cell);
                    surface.print(x, y, &centered(&label), Style { fg: color, attrs: Attrs::BOLD, ..Default::default() });
                    return;
                }
                _ => {} // frame 0 or unexpected: fall through to normal render
            }
        }
    } else if engine.anim_cells.contains_key(&(r, c)) {
        // Non-mid rows during animation: blank them out
        surface.print(x, y, &" ".repeat(cw), Style::default());
        return;
    }

    // Background tint
    let bg = if is_selected {
        Color::Rgb { r: 0, g: 60, b: 0 }
    } else if is_cursor {
        Color::Rgb { r: 60, g: 60, b: 0 }
    } else if is_hint {
        Color::Rgb { r: 0, g: 0, b: 70 }
    } else {
        Color::Reset
    };

    if sub_row == mid {
        let (label, color, bold) = glyphs::cell_label(cell);
        let is_frozen = matches!(cell, Cell::Frozen(_));

        let mut attrs = Attrs::empty();
        if bold || is_selected || is_cursor {
            attrs |= Attrs::BOLD;
        }
        if is_frozen {
            attrs |= Attrs::DIM;
        }

        surface.print(x, y, &centered(&label), Style { fg: color, bg, attrs });
    } else {
        // Secondary rows: show cooldown counter for generators
        let show_cd = match cell {
            Cell::HardGenerator { cooldown_remaining: cd, .. } |
            Cell::SoftGenerator { cooldown_remaining: cd, .. } if *cd > 0 => {
                Some(*cd)
            }
            _ => None,
        };
        if sub_row == 0 {
            if let Some(cd) = show_cd {
                let s = format!("{:^w$}", cd, w = cw);
                surface.print(x, y, &s, Style { fg: Color::DarkGrey, bg, ..Default::default() });
                return;
            }
        }
        surface.print(x, y, &" ".repeat(cw), Style { bg, ..Default::default() });
    }
}

// ── Board ─────────────────────────────────────────────────────────────────

/// Only ever called from inside tui.rs's centralized Clear/flush dispatcher,
/// so this draws into a mid-frame `TermSurface` (no clear/flush of its own).
#[cfg(not(target_arch = "wasm32"))]
pub fn render_board(stdout: &mut Stdout, engine: &GameEngine, geo: &LayoutGeometry) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_board_inner(&mut surface, engine, geo);
    surface.done()
}

pub fn render_board_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) {
    let scale = geo.scale;
    let (cw, ch) = cell_dims(scale);
    let rows = engine.board.rows;
    let cols = engine.board.cols;
    let bx = geo.board_x;
    let by = geo.board_y;

    for gr in 0..=rows {
        let y = by + (gr * (ch + 1)) as u16;
        render_grid_row(surface, bx, y, gr, cw, rows, cols);

        if gr < rows {
            for sr in 0..ch {
                let y2 = y + 1 + sr as u16;
                let mut x = bx;
                surface.print(x, y2, "│", Style::default());
                x += 1;
                for c in 0..cols {
                    render_cell_content(surface, x, y2, engine, gr, c, sr, cw, ch);
                    x += cw as u16;
                    surface.print(x, y2, "│", Style::default());
                    x += 1;
                }
            }
        }
    }
}
