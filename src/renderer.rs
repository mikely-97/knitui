#![allow(warnings)]

use std::io::{self, Write, Stdout};
use std::time::Instant;
use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Attribute, SetAttribute},
    terminal::{self, Clear, ClearType, BeginSynchronizedUpdate, EndSynchronizedUpdate},
    cursor::{MoveTo, Hide},
};
use crate::engine::{GameEngine, GameStatus, BonusState};
use crate::board_entity::{BoardEntity, Direction};
use crate::glyphs;

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

pub fn detect_layout(config_layout: &str, visible_patches: u16, board_height: u16, scale: u16) -> Layout {
    match config_layout {
        "horizontal" => Layout::Horizontal,
        "vertical" => Layout::Vertical,
        _ => {
            let sh = scale;
            let (_, term_height) = terminal::size().unwrap_or((80, 24));
            let yarn_h = visible_patches * sh + visible_patches.saturating_sub(1) * YARN_VGAP;
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

// ── Scaled rendering helpers ─────────────────────────────────────────────────

/// Render yarn patches into a region starting at (x0, y0), scaled with spacing.
/// `with_balloons`: if true, render balloon columns to the right of regular
/// yarn (used in vertical layout). If false, caller handles balloon rendering
/// separately (used in horizontal layout to avoid overlap).
pub fn render_yarn(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16, with_balloons: bool) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for offset in 0..(engine.yarn.visible_patches as usize) {
        let true_offset = (engine.yarn.visible_patches as usize) - offset;
        let row_y = y0 + (offset as u16) * (sh + YARN_VGAP);
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, row_y + sy))?;
            for (ci, column) in engine.yarn.board.iter().enumerate() {
                if ci > 0 {
                    for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
                }
                if true_offset <= column.len() {
                    let pos = column.len() - true_offset;
                    if scale > 1 {
                        let glyph_rows = glyphs::yarn_patch_glyph(column[pos].locked, scale);
                        stdout.queue(Print(glyph_rows[sy as usize].as_str().with(column[pos].color)))?;
                    } else {
                        for _ in 0..sw { stdout.queue(Print(&column[pos]))?; }
                    }
                } else {
                    for _ in 0..sw { stdout.queue(Print(' '))?; }
                }
            }
        }
    }

    // Render balloon columns to the right (vertical layout only)
    if with_balloons {
        render_balloon_columns(stdout, engine, x0, y0, scale)?;
    }

    Ok(())
}

/// Render balloon pseudo-columns at (x0, y0), to the right of regular yarn.
/// Uses compact height based on actual balloon content so patches are
/// visible right at y0, aligned to the bottom of the yarn area.
pub fn render_balloon_columns(stdout: &mut Stdout, engine: &GameEngine, yarn_x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    if engine.yarn.balloon_columns.is_empty() {
        return Ok(());
    }
    let sh = scale;
    let sw = scale * 2;
    let regular_w = engine.yarn.yarn_lines * sw
        + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let balloon_x0 = yarn_x0 + regular_w + COMP_GAP;

    // Single row of fixed slots, bottom-aligned with yarn
    let yarn_h = engine.yarn.visible_patches * (sh + YARN_VGAP) - YARN_VGAP;
    let y_start = y0 + yarn_h - sh;

    for sy in 0..sh {
        stdout.queue(MoveTo(balloon_x0, y_start + sy))?;
        for (ci, slot) in engine.yarn.balloon_columns.iter().enumerate() {
            if ci > 0 {
                for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
            }
            match slot {
                Some(patch) => {
                    if scale > 1 {
                        let glyph_rows = glyphs::yarn_patch_glyph(patch.locked, scale);
                        stdout.queue(Print(glyph_rows[sy as usize].as_str().with(patch.color)))?;
                    } else {
                        for _ in 0..sw { stdout.queue(Print(patch))?; }
                    }
                }
                None => {
                    for _ in 0..sw { stdout.queue(Print(' '))?; }
                }
            }
        }
    }
    Ok(())
}

/// Render a single flanking balloon cell (left or right of yarn).
/// Each flank is one patch wide (sw). Left shows patches lifted from the
/// leftmost yarn column, right shows patches from the rightmost.
/// balloon_columns[0] = left patches, balloon_columns[last] = right patches.
/// Shows dim ░ placeholders when balloons available but unused.
pub fn render_balloon_flank(
    stdout: &mut Stdout,
    engine: &GameEngine,
    x0: u16,
    y0: u16,
    scale: u16,
    side: FlankSide,
) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    let balloon_count = engine.bonuses.balloon_count as usize;

    // Left flank gets first left_count slots, right gets the rest
    let (start_idx, count) = match side {
        FlankSide::Left  => (0, balloon_count / 2),
        FlankSide::Right => (balloon_count / 2, (balloon_count + 1) / 2),
    };
    if count == 0 { return Ok(()); }

    let show = engine.bonuses.balloons > 0 || !engine.yarn.balloon_columns.is_empty();
    if !show { return Ok(()); }

    let slots = &engine.yarn.balloon_columns;

    // Bottom-align with yarn visible area
    let yarn_h = engine.yarn.visible_patches * (sh + YARN_VGAP) - YARN_VGAP;
    let flank_h = count as u16 * (sh + YARN_VGAP) - YARN_VGAP;
    let y_start = y0 + yarn_h.saturating_sub(flank_h);

    for i in 0..count {
        let row_y = y_start + (i as u16) * (sh + YARN_VGAP);
        let slot_idx = start_idx + i;
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, row_y + sy))?;
            if slots.is_empty() {
                // Balloons available but unused — show placeholder
                for _ in 0..sw { stdout.queue(Print("░".dark_grey()))?; }
            } else {
                match slots.get(slot_idx) {
                    Some(Some(patch)) => {
                        if scale > 1 {
                            let glyph_rows = glyphs::yarn_patch_glyph(patch.locked, scale);
                            stdout.queue(Print(glyph_rows[sy as usize].as_str().with(patch.color)))?;
                        } else {
                            for _ in 0..sw { stdout.queue(Print(patch))?; }
                        }
                    }
                    Some(None) => {
                        // Processed — empty space
                        for _ in 0..sw { stdout.queue(Print(' '))?; }
                    }
                    None => {
                        for _ in 0..sw { stdout.queue(Print(' '))?; }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Render active threads horizontally (one row, scaled) starting at (x0, y0).
pub fn render_active_h(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for sy in 0..sh {
        stdout.queue(MoveTo(x0, y0 + sy))?;
        for (i, thread) in engine.active_threads.iter().enumerate() {
            if i > 0 {
                for _ in 0..THREAD_GAP { stdout.queue(Print(' '))?; }
            }
            for _ in 0..sw { stdout.queue(Print(thread))?; }
        }
    }
    Ok(())
}

/// Render active threads vertically (one column, scaled) starting at (x0, y0).
pub fn render_active_v(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for (i, thread) in engine.active_threads.iter().enumerate() {
        let ty = y0 + (i as u16) * (sh + THREAD_GAP);
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, ty + sy))?;
            for _ in 0..sw { stdout.queue(Print(thread))?; }
        }
    }
    Ok(())
}

/// Draw a horizontal border line for the board grid.
/// kind: 0=top (┌┬┐), 1=middle (├┼┤), 2=bottom (└┴┘)
pub fn draw_hline(stdout: &mut Stdout, x0: u16, y: u16, cols: usize, sw: u16, kind: u8) -> io::Result<()> {
    stdout.queue(MoveTo(x0, y))?;
    let (left, fill, cross, right) = match kind {
        0 => ('┌', '─', '┬', '┐'),
        2 => ('└', '─', '┴', '┘'),
        _ => ('├', '─', '┼', '┤'),
    };
    stdout.queue(Print(left))?;
    for c in 0..cols {
        for _ in 0..sw { stdout.queue(Print(fill))?; }
        if c < cols - 1 { stdout.queue(Print(cross))?; }
    }
    stdout.queue(Print(right))?;
    Ok(())
}

/// Render the game board with box borders and bracket cursor markers.
pub fn render_board(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    let rows = engine.board.height as usize;
    let cols = engine.board.width as usize;
    let cur_r = engine.cursor_row as usize;
    let cur_c = engine.cursor_col as usize;

    let tweezers = matches!(engine.bonus_state, BonusState::TweezersActive { .. });
    let (open_bracket, close_bracket) = if tweezers { ('{', '}') } else { ('[', ']') };

    // Top border
    draw_hline(stdout, x0, y0, cols, sw, 0)?;

    for (row_idx, thread_row) in engine.board.board.iter().enumerate() {
        let content_y = y0 + 1 + (row_idx as u16) * (sh + 1);
        let is_cur_row = row_idx == cur_r;

        if scale > 1 {
            for (col_idx, cell) in thread_row.iter().enumerate() {
                let is_cursor = is_cur_row && col_idx == cur_c;
                let is_after_cursor = is_cur_row && col_idx > 0 && col_idx - 1 == cur_c;
                let glyph_rows = match &engine.board.board[row_idx][col_idx] {
                    BoardEntity::Thread(_) => glyphs::entity_glyph_thread(scale),
                    BoardEntity::KeyThread(_) => glyphs::entity_glyph_key_thread(scale),
                    BoardEntity::Obstacle => glyphs::entity_glyph_obstacle(scale),
                    BoardEntity::Generator(data) => glyphs::entity_glyph_generator(data.output_dir, scale),
                    BoardEntity::DepletedGenerator => glyphs::entity_glyph_depleted(scale),
                    BoardEntity::Void => glyphs::entity_glyph_void(scale),
                };
                let color = match &engine.board.board[row_idx][col_idx] {
                    BoardEntity::Thread(c) | BoardEntity::KeyThread(c) => Some(*c),
                    BoardEntity::Generator(data) => Some(data.color),
                    _ => None,
                };
                for (sy_idx, glyph_row) in glyph_rows.iter().enumerate() {
                    let cell_x = x0 + 1 + (col_idx as u16) * (sw + 1);
                    let cell_y = content_y + sy_idx as u16;
                    // Left border (only on sy_idx == 0 column position is handled by cell_x offset)
                    stdout.queue(MoveTo(x0 + (col_idx as u16) * (sw + 1), cell_y))?;
                    if is_cursor {
                        stdout.queue(Print(open_bracket.bold().white()))?;
                    } else if is_after_cursor {
                        stdout.queue(Print(close_bracket.bold().white()))?;
                    } else {
                        stdout.queue(Print('│'))?;
                    }
                    // Cell content
                    stdout.queue(MoveTo(cell_x, cell_y))?;
                    if is_cursor {
                        let chars: Vec<char> = glyph_row.chars().collect();
                        stdout.queue(Print(open_bracket.bold().white()))?;
                        let inner: String = chars[1..chars.len()-1].iter().collect();
                        match color {
                            Some(col) => { stdout.queue(Print(inner.with(col)))?; }
                            None => { stdout.queue(Print(&inner))?; }
                        }
                        stdout.queue(Print(close_bracket.bold().white()))?;
                    } else {
                        match color {
                            Some(col) => { stdout.queue(Print(glyph_row.as_str().with(col)))?; }
                            None => { stdout.queue(Print(glyph_row.as_str()))?; }
                        }
                    }
                }
            }
            // Right border for glyph path
            for sy_idx in 0..sh {
                let cell_y = content_y + sy_idx;
                stdout.queue(MoveTo(x0 + (cols as u16) * (sw + 1), cell_y))?;
                if is_cur_row && cols - 1 == cur_c {
                    stdout.queue(Print(close_bracket.bold().white()))?;
                } else {
                    stdout.queue(Print('│'))?;
                }
            }
        } else {
            for sy in 0..sh {
                stdout.queue(MoveTo(x0, content_y + sy))?;
                for (col_idx, cell) in thread_row.iter().enumerate() {
                    let is_cursor = is_cur_row && col_idx == cur_c;
                    let is_after_cursor = is_cur_row && col_idx > 0 && col_idx - 1 == cur_c;

                    // Left border: bright brackets for cursor edges, normal │ otherwise
                    if is_cursor {
                        stdout.queue(Print(open_bracket.bold().white()))?;
                    } else if is_after_cursor {
                        stdout.queue(Print(close_bracket.bold().white()))?;
                    } else {
                        stdout.queue(Print('│'))?;
                    }

                    // Cell content: inverted colors for cursor cell
                    if is_cursor {
                        stdout.queue(SetAttribute(Attribute::Reverse))?;
                        for _ in 0..sw { stdout.queue(Print(cell))?; }
                        stdout.queue(SetAttribute(Attribute::Reset))?;
                    } else {
                        for _ in 0..sw { stdout.queue(Print(cell))?; }
                    }
                }
                // Right border
                if is_cur_row && cols - 1 == cur_c {
                    stdout.queue(Print(close_bracket.bold().white()))?;
                } else {
                    stdout.queue(Print('│'))?;
                }
            }
        }

        let line_y = content_y + sh;
        if row_idx < rows - 1 {
            draw_hline(stdout, x0, line_y, cols, sw, 1)?;
        } else {
            draw_hline(stdout, x0, line_y, cols, sw, 2)?;
        }
    }

    Ok(())
}

pub fn render_help(stdout: &mut Stdout) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let lines = [
        "",
        "                    ═══ HELP ═══",
        "",
        "  Movement:   ← → ↑ ↓   Move cursor",
        "  Pick up:    Enter       Pick up thread at cursor",
        "  Menu:       Esc         Return to main menu",
        "  Restart:    R           New game (from game-over)",
        "  Help:       H           Show this screen",
        "",
        "  ─── Bonuses ───",
        "  [Z] ✂ Scissors    Auto-knit thread by deep-scanning yarn",
        "  [X] ⊹ Tweezers    Pick any thread from the board",
        "  [C] ⊛ Balloons    Lift front patches, expose patches behind",
        "  [A] ⊟ Watch ad    Watch a fake ad for +1 scissors",
        "",
        "              Press any key to close",
    ];

    for (i, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(0, i as u16))?;
        stdout.queue(Print(line))?;
    }

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

pub fn render_keybar(stdout: &mut Stdout, engine: &GameEngine, y: u16) -> io::Result<()> {
    stdout.queue(MoveTo(0, y))?;
    let (term_w, _) = terminal::size().unwrap_or((80, 24));
    for _ in 0..term_w { stdout.queue(Print(' '))?; }
    stdout.queue(MoveTo(0, y))?;

    stdout.queue(Print("←→↑↓ ".dark_grey()))?;
    stdout.queue(Print("Move  ".white()))?;
    stdout.queue(Print("Enter ".dark_grey()))?;
    stdout.queue(Print("Pick  ".white()))?;
    stdout.queue(Print("H ".dark_grey()))?;
    stdout.queue(Print("Help  ".white()))?;

    if engine.bonuses.scissors > 0 {
        stdout.queue(Print("Z ".dark_grey()))?;
        stdout.queue(Print(format!("✂x{} ", engine.bonuses.scissors).white()))?;
    } else {
        stdout.queue(Print("Z ✂x0 ".dark_grey()))?;
    }
    if engine.bonuses.tweezers > 0 {
        stdout.queue(Print("X ".dark_grey()))?;
        stdout.queue(Print(format!("⊹x{} ", engine.bonuses.tweezers).white()))?;
    } else {
        stdout.queue(Print("X ⊹x0 ".dark_grey()))?;
    }
    if engine.bonuses.balloons > 0 {
        stdout.queue(Print("C ".dark_grey()))?;
        stdout.queue(Print(format!("⊛x{} ", engine.bonuses.balloons).white()))?;
    } else {
        stdout.queue(Print("C ⊛x0 ".dark_grey()))?;
    }

    stdout.queue(Print("A ".dark_grey()))?;
    stdout.queue(Print("Ad ".white()))?;

    stdout.queue(Print("Esc ".dark_grey()))?;
    stdout.queue(Print("Menu".white()))?;
    Ok(())
}

/// Render the main menu screen.
pub fn render_main_menu(stdout: &mut Stdout, selected: usize, flash: Option<&str>) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let items = ["Quick Game", "Custom Game", "Campaign", "Endless", "Options", "Quit"];
    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let start_y = term_h / 2 - (items.len() as u16 + 4) / 2;

    // Title
    let title = "═══ KNITUI ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(title_x, start_y))?;
    stdout.queue(Print(title))?;

    // Menu items
    for (i, item) in items.iter().enumerate() {
        let y = start_y + 2 + i as u16;
        let prefix = if i == selected { "> " } else { "  " };
        let line = format!("{}{}", prefix, item);
        let x = (term_w.saturating_sub(line.chars().count() as u16 + 4)) / 2;
        stdout.queue(MoveTo(x, y))?;
        if i == selected {
            stdout.queue(SetAttribute(Attribute::Reverse))?;
            stdout.queue(Print(&line))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(Print(&line))?;
        }
    }

    // Flash message
    if let Some(msg) = flash {
        let flash_y = start_y + 2 + items.len() as u16 + 1;
        let flash_x = (term_w.saturating_sub(msg.chars().count() as u16)) / 2;
        stdout.queue(MoveTo(flash_x, flash_y))?;
        stdout.queue(Print(msg.dark_grey()))?;
    }

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

/// Render the custom game configuration screen.
pub fn render_custom_game(
    stdout: &mut Stdout,
    preset_name: &str,
    fields: &[(&str, u16)],
    selected_field: usize,
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let total_lines = 4 + fields.len() as u16 + 2; // title + preset + gap + fields + gap + hint
    let start_y = term_h / 2 - total_lines / 2;

    // Title
    let title = "═══ CUSTOM GAME ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(title_x, start_y))?;
    stdout.queue(Print(title))?;

    // Preset selector
    let preset_line = format!("Preset: ← [{}] →", preset_name);
    let preset_x = (term_w.saturating_sub(preset_line.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(preset_x, start_y + 2))?;
    if selected_field == 0 {
        stdout.queue(SetAttribute(Attribute::Reverse))?;
        stdout.queue(Print(&preset_line))?;
        stdout.queue(SetAttribute(Attribute::Reset))?;
    } else {
        stdout.queue(Print(&preset_line))?;
    }

    // Fields (selected_field 1..=fields.len() maps to fields[0..])
    let col_x = (term_w.saturating_sub(30)) / 2;
    for (i, (name, value)) in fields.iter().enumerate() {
        let y = start_y + 4 + i as u16;
        let prefix = if i + 1 == selected_field { "> " } else { "  " };
        let line = format!("{}{:<20}{:>4}", prefix, name, value);
        stdout.queue(MoveTo(col_x, y))?;
        if i + 1 == selected_field {
            stdout.queue(SetAttribute(Attribute::Reverse))?;
            stdout.queue(Print(&line))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(Print(&line))?;
        }
    }

    // Hint line
    let hint = "↑↓ Navigate  ←→ Adjust  Enter: Start  Esc: Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 4 + fields.len() as u16 + 1;
    stdout.queue(MoveTo(hint_x, hint_y))?;
    stdout.queue(Print(hint.dark_grey()))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

/// Render the options screen (scale, color mode).
pub fn render_options(
    stdout: &mut Stdout,
    selected: usize,
    scale: u16,
    color_mode: &str,
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let start_y = term_h / 2 - 5;

    // Title
    let title = "═══ OPTIONS ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(title_x, start_y))?;
    stdout.queue(Print(title))?;

    let fields: [(&str, String); 2] = [
        ("Scale", format!("← {} →", scale)),
        ("Color Mode", format!("← {} →", color_mode)),
    ];

    let col_x = (term_w.saturating_sub(34)) / 2;
    for (i, (name, value)) in fields.iter().enumerate() {
        let y = start_y + 2 + i as u16;
        let prefix = if i == selected { "> " } else { "  " };
        let line = format!("{}{:<16}{}", prefix, name, value);
        stdout.queue(MoveTo(col_x, y))?;
        if i == selected {
            stdout.queue(SetAttribute(Attribute::Reverse))?;
            stdout.queue(Print(&line))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(Print(&line))?;
        }
    }

    // Hint line
    let hint = "←→ Adjust  Esc: Save & Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 2 + fields.len() as u16 + 1;
    stdout.queue(MoveTo(hint_x, hint_y))?;
    stdout.queue(Print(hint.dark_grey()))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

/// Render the campaign track selection screen.
pub fn render_campaign_select(
    stdout: &mut Stdout,
    selected: usize,
    track_names: &[&str],
    track_sizes: &[usize],
    progress_labels: &[String],
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let total_lines = 4 + track_names.len() as u16 + 2;
    let start_y = term_h / 2 - total_lines / 2;

    let title = "═══ SELECT CAMPAIGN ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(title_x, start_y))?;
    stdout.queue(Print(title))?;

    let col_x = (term_w.saturating_sub(44)) / 2;
    for (i, name) in track_names.iter().enumerate() {
        let y = start_y + 2 + i as u16;
        let prefix = if i == selected { "> " } else { "  " };
        let progress = &progress_labels[i];
        let size_str = format!("({} levels)", track_sizes[i]);
        let line = if progress.is_empty() {
            format!("{}{:<10}{}", prefix, name, size_str)
        } else {
            format!("{}{:<10}{}    {}", prefix, name, size_str, progress)
        };
        stdout.queue(MoveTo(col_x, y))?;
        if i == selected {
            stdout.queue(SetAttribute(Attribute::Reverse))?;
            stdout.queue(Print(&line))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(Print(&line))?;
        }
    }

    let hint = "↑↓ Select  Enter: Start  Esc: Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 2 + track_names.len() as u16 + 1;
    stdout.queue(MoveTo(hint_x, hint_y))?;
    stdout.queue(Print(hint.dark_grey()))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

/// Render a brief level intro card before starting a campaign level.
pub fn render_level_intro(
    stdout: &mut Stdout,
    track_name: &str,
    level_num: usize,
    total_levels: usize,
    board_h: u16,
    board_w: u16,
    colors: u16,
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let start_y = term_h / 2 - 3;

    let title = format!("═══ {} CAMPAIGN ═══", track_name.to_uppercase());
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(title_x, start_y))?;
    stdout.queue(Print(&title))?;

    let level_str = format!("Level {}/{}", level_num, total_levels);
    let lx = (term_w.saturating_sub(level_str.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(lx, start_y + 2))?;
    stdout.queue(Print(&level_str))?;

    let desc = format!("{}x{} board, {} colors", board_w, board_h, colors);
    let dx = (term_w.saturating_sub(desc.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(dx, start_y + 3))?;
    stdout.queue(Print(desc.dark_grey()))?;

    let hint = "Press Enter to start";
    let hx = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(hx, start_y + 5))?;
    stdout.queue(Print(hint.dark_grey()))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

pub fn render_bonus_display_h(stdout: &mut Stdout, engine: &GameEngine, x: u16, y: u16) -> io::Result<()> {
    stdout.queue(MoveTo(x, y))?;
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        if i > 0 { stdout.queue(Print("  "))?; }
        if *count > 0 {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).white()))?;
        } else {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).dark_grey()))?;
        }
    }
    Ok(())
}

pub fn render_bonus_panel(stdout: &mut Stdout, engine: &GameEngine, x: u16, y: u16) -> io::Result<()> {
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        stdout.queue(MoveTo(x, y + i as u16))?;
        if *count > 0 {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).white()))?;
        } else {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).dark_grey()))?;
        }
    }
    Ok(())
}

// ── Rendering ─────────────────────────────────────────────────────────────────

pub fn render_vertical(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
) -> io::Result<()> {
    let sh = scale;
    let yarn_h = engine.yarn.visible_patches * sh
        + engine.yarn.visible_patches.saturating_sub(1) * YARN_VGAP;
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

pub fn render_vertical_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
    status: &GameStatus,
    overlay_msg: Option<&str>,
) -> io::Result<()> {
    render_vertical(stdout, engine, board_y, scale)?;
    let default_msg = match status {
        GameStatus::Stuck => "You're lost! R:Restart  M:Menu  Q:Quit",
        GameStatus::Won   => "You won! R:Restart  M:Menu  Q:Quit",
        _ => return Ok(()),
    };
    let message = overlay_msg.unwrap_or(default_msg);
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Print(message))?;
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
    let default_msg = match status {
        GameStatus::Stuck => "You're lost! R:Restart  M:Menu  Q:Quit",
        GameStatus::Won   => "You won! R:Restart  M:Menu  Q:Quit",
        _ => return Ok(()),
    };
    let message = overlay_msg.unwrap_or(default_msg);
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Print(message))?;
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

/// Render the pseudo-ad full-screen overlay.
pub fn render_ad_overlay(
    stdout: &mut Stdout,
    quote: &str,
    started_at: &Instant,
    ad_duration_secs: u64,
) -> io::Result<()> {
    let elapsed = started_at.elapsed().as_secs();
    let remaining = ad_duration_secs.saturating_sub(elapsed);
    let progress = if ad_duration_secs > 0 {
        ((elapsed as f64 / ad_duration_secs as f64) * 100.0).min(100.0) as u16
    } else {
        100
    };
    let done = remaining == 0;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));

    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Clear(ClearType::All))?;

    // Box dimensions
    let box_w = 50u16.min(term_w.saturating_sub(4));
    let box_inner = (box_w - 2) as usize;

    let wrapped = word_wrap(quote, box_inner);
    let box_h = 8 + wrapped.len() as u16;
    let x0 = (term_w.saturating_sub(box_w)) / 2;
    let y0 = (term_h.saturating_sub(box_h)) / 2;

    let mut y = y0;

    // Top border
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("╔"))?;
    for _ in 0..box_inner { stdout.queue(Print("═"))?; }
    stdout.queue(Print("╗"))?;
    y += 1;

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Header
    print_boxed_line(stdout, x0, y, box_inner, &center_text("✂ FREE SCISSORS ✂", box_inner))?;
    y += 1;

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Quote lines
    for line in &wrapped {
        print_boxed_line(stdout, x0, y, box_inner, &center_text(line, box_inner))?;
        y += 1;
    }

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Progress bar
    let bar_width = box_inner.saturating_sub(8);
    let filled = (bar_width as u16 * progress / 100) as usize;
    let empty = bar_width - filled;
    let bar = format!(
        "{}{}  {:>3}%",
        "█".repeat(filled),
        "░".repeat(empty),
        progress
    );
    print_boxed_line(stdout, x0, y, box_inner, &center_text(&bar, box_inner))?;
    y += 1;

    // Countdown or close prompt
    if done {
        let msg = "[ Press ESC to collect your reward ]";
        print_boxed_line(stdout, x0, y, box_inner, &center_text(msg, box_inner))?;
    } else {
        let msg = format!("[{}s remaining]", remaining);
        print_boxed_line(stdout, x0, y, box_inner, &center_text(&msg, box_inner))?;
    }
    y += 1;

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Bottom border
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("╚"))?;
    for _ in 0..box_inner { stdout.queue(Print("═"))?; }
    stdout.queue(Print("╝"))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

fn print_boxed_line(stdout: &mut Stdout, x0: u16, y: u16, inner_w: usize, content: &str) -> io::Result<()> {
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("║"))?;
    let content_chars: usize = content.chars().count();
    stdout.queue(Print(content))?;
    for _ in content_chars..inner_w {
        stdout.queue(Print(' '))?;
    }
    stdout.queue(Print("║"))?;
    Ok(())
}

fn center_text(text: &str, width: usize) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }
    let padding = (width - text_len) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
