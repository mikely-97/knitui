use std::io::{self, Stdout};

use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Color, SetForegroundColor, ResetColor, SetBackgroundColor},
    cursor::MoveTo,
};

use crate::board::{CellContent, SpecialPiece, TileModifier};
use crate::engine::{GameEngine, GamePhase};
use crate::glyphs;

use super::LayoutGeometry;
use super::CELL_GAP;

// ── render_board ──────────────────────────────────────────────────────────

/// Render the game board to stdout.
pub fn render_board(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) -> io::Result<()> {
    let scale = geo.scale;
    let sh = scale;
    let sw = scale * 2;
    let gap = CELL_GAP;

    // Determine which cells are part of an in-progress bounce
    let bounce_cells: std::collections::HashSet<(usize, usize)> =
        if let GamePhase::Bouncing { .. } = &engine.phase {
            if let Some((a, b)) = engine.pending_swap_preview() {
                [a, b].iter().copied().collect()
            } else {
                Default::default()
            }
        } else {
            Default::default()
        };

    for r in 0..engine.board.height {
        for row_offset in 0..sh {
            let y = geo.board_y + r as u16 * (sh + gap) + row_offset;

            for c in 0..engine.board.width {
                let cell = &engine.board.cells[r][c];
                let x = geo.board_x + c as u16 * (sw + gap);
                stdout.queue(MoveTo(x, y))?;

                let row_offset_u = row_offset as usize;
                let is_cursor = engine.cursor_row == r && engine.cursor_col == c;
                let is_selected = engine.selected == Some((r, c));
                let is_bouncing = bounce_cells.contains(&(r, c));
                let in_match = is_in_active_match(engine, r, c);

                // 1. Determine base glyph rows and cell color.
                //    cell_color is only used when there is no modifier (modifiers fully
                //    replace the visual, so the underlying gem color is invisible).
                let (glyph_rows, cell_color): (Vec<String>, Option<Color>) = if is_bouncing {
                    (glyphs::bounce_glyph(scale), None)
                } else {
                    match &cell.content {
                        CellContent::Empty => (glyphs::empty_glyph(scale), None),
                        CellContent::Gem { color, special: None } => {
                            let color = *color;
                            let rows = glyphs::gem_glyph(scale);
                            (rows, if cell.modifier.is_none() { Some(color) } else { None })
                        }
                        CellContent::Gem { color, special: Some(sp) } => {
                            let color = *color;
                            let rows = glyphs::special_glyph(sp, scale);
                            (rows, if cell.modifier.is_none() { Some(color) } else { None })
                        }
                    }
                };

                // 2. Modifier overlay fully replaces the gem glyph (Stone, Ice, Crate, Locked).
                //    For Ice cells we keep the gem glyph but tint it cyan so the gem color shows.
                let is_ice = matches!(cell.modifier, Some(TileModifier::Ice { .. }));
                let final_rows: Vec<String> = if let Some(ref modifier) = cell.modifier {
                    if is_ice {
                        // Show the gem glyph but we'll tint with cyan below
                        glyph_rows
                    } else {
                        glyphs::modifier_overlay(modifier, scale)
                    }
                } else {
                    glyph_rows
                };

                // 3. Print the row_offset row with gem color (if any) and highlight styling.
                //    If there is an active burst animation on this cell, render that instead.
                if let Some(anim) = engine.anim_cells.get((r, c)) {
                    let (anim_glyph, anim_color) = match anim.frame {
                        3 => ("██", Color::White),
                        2 => ("✦ ", Color::Yellow),
                        1 => ("· ", Color::DarkGrey),
                        _ => ("  ", Color::Black),
                    };
                    stdout.queue(SetForegroundColor(anim_color))?;
                    stdout.queue(Print(anim_glyph))?;
                    stdout.queue(ResetColor)?;
                } else {
                    let row_str = final_rows
                        .get(row_offset_u)
                        .map(|s| s.as_str())
                        .unwrap_or("  ");

                    // Ice cells: show gem with cyan foreground tint; other cells use gem color
                    let effective_color = if is_ice {
                        Some(Color::Cyan)
                    } else {
                        cell_color
                    };

                    if let Some(color) = effective_color {
                        stdout.queue(SetForegroundColor(color))?;
                    }
                    if is_selected {
                        stdout.queue(Print(row_str.negative()))?;
                    } else if in_match {
                        stdout.queue(Print(row_str.bold()))?;
                    } else if is_ice {
                        // Wrap ice cell content in brackets for visual indicator
                        stdout.queue(Print(format!("{}", row_str)))?;
                    } else {
                        stdout.queue(Print(row_str))?;
                    }
                    if effective_color.is_some() {
                        stdout.queue(ResetColor)?;
                    }
                }

                // Gap between cells
                if c < engine.board.width - 1 {
                    stdout.queue(Print(" ".repeat(gap as usize)))?;
                }
            }

            // Overlay cursor brackets after all cells in this scanline
            if engine.cursor_row == r {
                let c = engine.cursor_col;
                let cx = geo.board_x + c as u16 * (sw + gap);
                // Left bracket — use the gap column before the cursor cell
                if cx > 0 {
                    stdout.queue(MoveTo(cx - 1, y))?;
                    stdout.queue(Print("[".white().bold()))?;
                }
                // Right bracket — use the gap column after the cursor cell
                stdout.queue(MoveTo(cx + sw, y))?;
                stdout.queue(Print("]".white().bold()))?;
            }
        }
    }

    Ok(())
}

pub fn is_in_active_match(engine: &GameEngine, r: usize, c: usize) -> bool {
    if let GamePhase::Resolving { match_groups, .. } = &engine.phase {
        match_groups.iter().any(|g| g.cells.contains(&(r, c)))
    } else {
        false
    }
}
