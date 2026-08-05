use std::io::{self, Stdout};

use loom_engine::render::{Color, Style, Surface};
use loom_engine_term::TermSurface;

use crate::engine::{CellState, GameEngine, GameStatus};

/// Width of each rendered cell in terminal columns.
const CELL_W: u16 = 2;
/// How many columns the clue area is (right-aligned inside MAX_CLUE_W).
const MAX_CLUE_W: u16 = 6; // up to 3 digits + space each

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

/// Render the full picross board with clues, cursor, and status line.
///
/// The caller (`tui.rs`'s `render_state`) owns the frame's Clear/flush, so
/// this only draws into a mid-frame `TermSurface` and does not clear/flush
/// itself.
pub fn render(stdout: &mut Stdout, engine: &GameEngine, origin_x: u16, origin_y: u16) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_inner(&mut surface, engine, origin_x, origin_y);
    surface.done()
}

fn render_inner(surface: &mut dyn Surface, engine: &GameEngine, origin_x: u16, origin_y: u16) {
    let rows = engine.puzzle.rows;
    let _cols = engine.puzzle.cols;

    // ── Column clues (above the grid) ──────────────────────────────────────
    // Find max clue height for columns.
    let col_clue_h = engine.puzzle.col_clues.iter()
        .map(|c| c.len())
        .max()
        .unwrap_or(1) as u16;

    // Row clue area width
    let row_clue_w = MAX_CLUE_W;

    // Grid top-left corner
    let grid_x = origin_x + row_clue_w;
    let grid_y = origin_y + col_clue_h;

    // Draw column clues
    for (c, clues) in engine.puzzle.col_clues.iter().enumerate() {
        let cx = grid_x + c as u16 * CELL_W;
        let offset = col_clue_h as usize - clues.len();
        for (i, &clue) in clues.iter().enumerate() {
            let cy = origin_y + (offset + i) as u16;
            let s = if clue == 0 {
                " 0".to_string()
            } else {
                format!("{:2}", clue)
            };
            surface.print(cx, cy, &s, Style::default());
        }
    }

    // ── Row clues + grid rows ───────────────────────────────────────────────
    for (r, row_clues) in engine.puzzle.row_clues.iter().enumerate() {
        let ry = grid_y + r as u16;

        // Right-align the row clue within MAX_CLUE_W characters
        let clue_str: String = row_clues.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let padded = format!("{:>width$}", clue_str, width = row_clue_w as usize);
        surface.print(origin_x, ry, &padded, Style::default());

        // Grid cells
        for (c, &cell) in engine.grid[r].iter().enumerate() {
            let cx = grid_x + c as u16 * CELL_W;
            let is_cursor = engine.cursor == (r, c);

            let glyph = match cell {
                CellState::Unknown => "  ",
                CellState::Filled  => "██",
                CellState::Crossed => "╳╳",
            };

            if is_cursor {
                let style = Style { fg: Color::Black, bg: Color::Yellow, ..Default::default() };
                surface.print(cx, ry, glyph, style);
            } else if cell == CellState::Unknown {
                surface.print(cx, ry, "▒▒", fg(Color::DarkGrey));
            } else {
                surface.print(cx, ry, glyph, Style::default());
            }
        }
    }

    // ── Status line ────────────────────────────────────────────────────────
    let status_y = grid_y + rows as u16 + 1;

    match engine.status {
        GameStatus::Playing => {
            let pct = engine.completion_pct();
            let status = format!(
                "Mistakes: {}/{} │ Filled: {:.0}%",
                engine.mistakes,
                crate::engine::MAX_MISTAKES,
                pct
            );
            surface.print(origin_x, status_y, &status, Style::default());
        }
        GameStatus::Won => {
            surface.print(
                origin_x, status_y,
                "  *** PUZZLE SOLVED! *** Press Q to return to menu  ",
                fg(Color::Green),
            );
        }
        GameStatus::TooManyMistakes => {
            surface.print(
                origin_x, status_y,
                "  Too many mistakes! Press Q to try again  ",
                fg(Color::Red),
            );
        }
    }

    // ── Key bar ────────────────────────────────────────────────────────────
    let keybar_y = status_y + 1;
    surface.print(
        origin_x, keybar_y,
        "Arrows:move  Space/Enter:fill  X:cross  Q/Esc:quit",
        fg(Color::DarkGrey),
    );
}
