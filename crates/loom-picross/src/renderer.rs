use std::io::{self, Write, Stdout};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::engine::{CellState, GameEngine, GameStatus};

/// Width of each rendered cell in terminal columns.
const CELL_W: u16 = 2;
/// How many columns the clue area is (right-aligned inside MAX_CLUE_W).
const MAX_CLUE_W: u16 = 6; // up to 3 digits + space each

/// Render the full picross board with clues, cursor, and status line.
pub fn render(stdout: &mut Stdout, engine: &GameEngine, origin_x: u16, origin_y: u16) -> io::Result<()> {
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
            stdout.queue(MoveTo(cx, cy))?;
            stdout.queue(Print(s))?;
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
        stdout.queue(MoveTo(origin_x, ry))?;
        stdout.queue(Print(&padded))?;

        // Grid cells
        for (c, &cell) in engine.grid[r].iter().enumerate() {
            let cx = grid_x + c as u16 * CELL_W;
            let is_cursor = engine.cursor == (r, c);

            if is_cursor {
                stdout.queue(SetBackgroundColor(Color::Yellow))?;
                stdout.queue(SetForegroundColor(Color::Black))?;
            }

            let glyph = match cell {
                CellState::Unknown => "  ",
                CellState::Filled  => "██",
                CellState::Crossed => "╳╳",
            };

            // Border around unknown cells
            if cell == CellState::Unknown && !is_cursor {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(MoveTo(cx, ry))?;
                stdout.queue(Print("▒▒"))?;
                stdout.queue(ResetColor)?;
            } else {
                stdout.queue(MoveTo(cx, ry))?;
                stdout.queue(Print(glyph))?;
                stdout.queue(ResetColor)?;
            }
        }
    }

    // ── Status line ────────────────────────────────────────────────────────
    let status_y = grid_y + rows as u16 + 1;
    stdout.queue(MoveTo(origin_x, status_y))?;

    match engine.status {
        GameStatus::Playing => {
            let pct = engine.completion_pct();
            let status = format!(
                "Mistakes: {}/{} │ Filled: {:.0}%",
                engine.mistakes,
                crate::engine::MAX_MISTAKES,
                pct
            );
            stdout.queue(Print(status))?;
        }
        GameStatus::Won => {
            stdout.queue(SetForegroundColor(Color::Green))?;
            stdout.queue(Print("  *** PUZZLE SOLVED! *** Press Q to return to menu  "))?;
            stdout.queue(ResetColor)?;
        }
        GameStatus::TooManyMistakes => {
            stdout.queue(SetForegroundColor(Color::Red))?;
            stdout.queue(Print("  Too many mistakes! Press Q to try again  "))?;
            stdout.queue(ResetColor)?;
        }
    }

    // ── Key bar ────────────────────────────────────────────────────────────
    let keybar_y = status_y + 1;
    stdout.queue(MoveTo(origin_x, keybar_y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("Arrows:move  Space/Enter:fill  X:cross  Q/Esc:quit"))?;
    stdout.queue(ResetColor)?;

    stdout.flush()
}
