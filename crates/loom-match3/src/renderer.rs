#![allow(warnings)]

use std::io::{self, Write, Stdout};

use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Color, SetForegroundColor, ResetColor, SetBackgroundColor},
    terminal::{self, Clear, ClearType},
    cursor::{MoveTo, Hide},
};

use crate::board::{Board, Cell, CellContent, SpecialPiece, TileModifier};
use crate::bonuses::{BonusInventory, BonusState};
use crate::engine::{GameEngine, GamePhase, GameStatus};
use crate::glyphs;

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
                let final_rows: Vec<String> = if let Some(ref modifier) = cell.modifier {
                    glyphs::modifier_overlay(modifier, scale)
                } else {
                    glyph_rows
                };

                // 3. Print the row_offset row with gem color (if any) and highlight styling.
                let row_str = final_rows
                    .get(row_offset_u)
                    .map(|s| s.as_str())
                    .unwrap_or("  ");

                if let Some(color) = cell_color {
                    stdout.queue(SetForegroundColor(color))?;
                }
                if is_selected {
                    stdout.queue(Print(row_str.negative()))?;
                } else if in_match {
                    stdout.queue(Print(row_str.bold()))?;
                } else {
                    stdout.queue(Print(row_str))?;
                }
                if cell_color.is_some() {
                    stdout.queue(ResetColor)?;
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

fn is_in_active_match(engine: &GameEngine, r: usize, c: usize) -> bool {
    if let GamePhase::Resolving { match_groups, .. } = &engine.phase {
        match_groups.iter().any(|g| g.cells.contains(&(r, c)))
    } else {
        false
    }
}

// ── render_hud ────────────────────────────────────────────────────────────

/// Render the HUD panel: score, moves, bonus inventory.
pub fn render_hud(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    objective_label: &str,
) -> io::Result<()> {
    let x = geo.hud_x;
    let mut y = geo.hud_y;

    let moves_left = engine.move_limit.saturating_sub(engine.moves_used);

    stdout.queue(MoveTo(x, y))?;
    stdout.queue(Print(format!("Score: {:>8}", engine.score)))?;
    y += 1;

    stdout.queue(MoveTo(x, y))?;
    stdout.queue(Print(format!("Moves:  {:>7}", moves_left)))?;
    y += 1;

    if !objective_label.is_empty() {
        stdout.queue(MoveTo(x, y))?;
        stdout.queue(Print(format!("Goal: {}", objective_label)))?;
        y += 1;
    }

    y += 1;

    stdout.queue(MoveTo(x, y))?;
    let hammer_str  = format!("[Z] Hammer x{}", engine.bonuses.hammer);
    let laser_str   = format!("[X] Laser  x{}", engine.bonuses.laser);
    let blaster_str = format!("[C] Blast  x{}", engine.bonuses.blaster);
    let warp_str    = format!("[V] Warp   x{}", engine.bonuses.warp);

    let dim_if_zero = |s: String, count: u16| {
        if count == 0 { format!("\x1b[2m{}\x1b[0m", s) } else { s }
    };

    stdout.queue(Print(dim_if_zero(hammer_str,  engine.bonuses.hammer)))?;
    stdout.queue(Print("  "))?;
    stdout.queue(Print(dim_if_zero(laser_str,   engine.bonuses.laser)))?;
    y += 1;
    stdout.queue(MoveTo(x, y))?;
    stdout.queue(Print(dim_if_zero(blaster_str, engine.bonuses.blaster)))?;
    stdout.queue(Print("  "))?;
    stdout.queue(Print(dim_if_zero(warp_str,    engine.bonuses.warp)))?;

    Ok(())
}

// ── render_key_bar ────────────────────────────────────────────────────────

/// Render the persistent key bar at the bottom of the terminal.
pub fn render_key_bar(stdout: &mut Stdout, bonus_state: &BonusState) -> io::Result<()> {
    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    stdout.queue(MoveTo(0, term_h - 1))?;

    let bar = match bonus_state {
        BonusState::HammerActive { .. } =>
            "Arrows Move  Enter Destroy  Esc Cancel".to_string(),
        BonusState::None =>
            "Arrows Move  Enter Select  H Help  Z Hammer  X Laser  C Blast  V Warp  Esc Menu  Q Quit".to_string(),
    };

    let padded = format!("{:<width$}", bar, width = term_w as usize);
    stdout.queue(Print(padded.negative()))?;

    Ok(())
}

// ── render_help ───────────────────────────────────────────────────────────

/// Full-screen help overlay. Any keypress will dismiss it.
pub fn render_help(stdout: &mut Stdout) -> io::Result<()> {
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let start_y = th / 4;
    let start_x = tw / 4;

    stdout.queue(MoveTo(start_x, start_y))?;
    stdout.queue(Print("═══════════ HELP ═══════════"))?;

    let lines = [
        "",
        "Movement:  ← → ↑ ↓     Move cursor",
        "Select:    Enter         Select gem / confirm swap",
        "Deselect:  Esc           Cancel selection / active bonus",
        "Quit:      Q             Exit game",
        "",
        "─── Bonuses ───",
        "[Z] 🔨 Hammer   Destroy one cell (requires targeting)",
        "[X] ══ Laser    Destroy entire row instantly",
        "[C] ‖‖ Blaster  Destroy entire column instantly",
        "[V] ⟳  Warp     Shuffle the whole board",
        "",
        "─── Special Pieces (created by large matches) ───",
        "══  Line Bomb   Match 4 in a line → destroys full row/column",
        "✦   Color Bomb  Match 5 in a line → destroys all of one color",
        "⊛   Area Bomb   L/T shape match → destroys 3×3 or 5×5 area",
        "",
        "         Press any key to close",
    ];

    for (i, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(start_x, start_y + 1 + i as u16))?;
        stdout.queue(Print(line))?;
    }

    Ok(())
}

// ── render_game_over ──────────────────────────────────────────────────────

/// Game-over / won overlay.
pub fn render_game_over(
    stdout: &mut Stdout,
    status: &GameStatus,
    score: u32,
) -> io::Result<()> {
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let cx = tw / 2 - 10;
    let cy = th / 2 - 3;

    let title = match status {
        GameStatus::Won        => "  ★  YOU WIN!  ★  ",
        GameStatus::OutOfMoves => "  OUT OF MOVES  ",
        GameStatus::Stuck      => "  BOARD STUCK  ",
        GameStatus::Playing    => return Ok(()),
    };

    stdout.queue(MoveTo(cx, cy))?;
    stdout.queue(Print(format!("╔══════════════════╗")))?;
    stdout.queue(MoveTo(cx, cy + 1))?;
    stdout.queue(Print(format!("║{:^20}║", title)))?;
    stdout.queue(MoveTo(cx, cy + 2))?;
    stdout.queue(Print(format!("║  Score: {:>10} ║", score)))?;
    stdout.queue(MoveTo(cx, cy + 3))?;
    stdout.queue(Print(format!("║                    ║")))?;
    stdout.queue(MoveTo(cx, cy + 4))?;
    stdout.queue(Print(format!("║ R Retry  Q Quit    ║")))?;
    stdout.queue(MoveTo(cx, cy + 5))?;
    stdout.queue(Print(format!("╚══════════════════╝")))?;

    Ok(())
}

// ── render_main_menu ──────────────────────────────────────────────────────

/// Main menu screen.
pub fn render_main_menu(
    stdout: &mut Stdout,
    selected: usize,
    flash: Option<&str>,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let cx = tw / 2 - 12;
    let cy = th / 4;

    stdout.queue(MoveTo(cx, cy))?;
    stdout.queue(Print("  ╔══════════════════════╗"))?;
    stdout.queue(MoveTo(cx, cy + 1))?;
    stdout.queue(Print("  ║    m3tui  Match-3    ║"))?;
    stdout.queue(MoveTo(cx, cy + 2))?;
    stdout.queue(Print("  ╚══════════════════════╝"))?;

    const ITEMS: &[&str] = &[
        "Quick Game",
        "Custom Game",
        "Campaign",
        "Endless",
        "Options",
        "Quit",
    ];

    for (i, item) in ITEMS.iter().enumerate() {
        stdout.queue(MoveTo(cx + 2, cy + 4 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {:}", item).negative()))?;
        } else {
            stdout.queue(Print(format!("  {:}", item)))?;
        }
    }

    if let Some(msg) = flash {
        stdout.queue(MoveTo(cx, cy + 4 + ITEMS.len() as u16 + 1))?;
        stdout.queue(Print(format!("  {}", msg)))?;
    }

    Ok(())
}

// ── render_options ────────────────────────────────────────────────────────

/// Options screen (scale + color mode).
pub fn render_options(
    stdout: &mut Stdout,
    selected: usize,
    scale: u16,
    color_mode: &str,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let cx = tw / 2 - 12;
    let cy = th / 4;

    stdout.queue(MoveTo(cx, cy))?;
    stdout.queue(Print("  OPTIONS"))?;

    let fields = [
        format!("Scale:      {}", scale),
        format!("Color Mode: {}", color_mode),
        "Back".to_string(),
    ];

    for (i, field) in fields.iter().enumerate() {
        stdout.queue(MoveTo(cx, cy + 2 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {}", field).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", field)))?;
        }
    }

    stdout.queue(MoveTo(cx, cy + 2 + fields.len() as u16 + 1))?;
    stdout.queue(Print("  ← → change value   Esc back"))?;

    Ok(())
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
