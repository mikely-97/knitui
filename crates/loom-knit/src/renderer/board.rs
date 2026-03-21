use std::io::{self, Stdout};
use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Attribute, SetAttribute},
    cursor::MoveTo,
};
use crate::engine::{GameEngine, BonusState};
use crate::board_entity::BoardEntity;
use crate::glyphs;
use super::{YARN_HGAP, YARN_VGAP, THREAD_GAP, FlankSide};

/// Render yarn stitches into a region starting at (x0, y0), scaled with spacing.
/// `with_balloons`: if true, render balloon columns to the right of regular
/// yarn (used in vertical layout). If false, caller handles balloon rendering
/// separately (used in horizontal layout to avoid overlap).
pub fn render_yarn(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16, with_balloons: bool) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for offset in 0..(engine.yarn.visible_stitches as usize) {
        let true_offset = (engine.yarn.visible_stitches as usize) - offset;
        let row_y = y0 + (offset as u16) * (sh + YARN_VGAP);
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, row_y + sy))?;
            for (ci, column) in engine.yarn.board.iter().enumerate() {
                if ci > 0 {
                    for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
                }
                if true_offset <= column.len() {
                    let pos = column.len() - true_offset;
                    let stitch = &column[pos];
                    let is_hint = engine.blessing_flags.match_hint
                        && engine.last_picked_color.is_some()
                        && engine.last_picked_color.unwrap() == stitch.color
                        && !stitch.locked;
                    if scale > 1 {
                        let glyph_rows = glyphs::yarn_patch_glyph(stitch.locked, scale);
                        if is_hint {
                            stdout.queue(SetAttribute(Attribute::Reverse))?;
                            stdout.queue(Print(glyph_rows[sy as usize].as_str().with(stitch.color)))?;
                            stdout.queue(SetAttribute(Attribute::Reset))?;
                        } else {
                            stdout.queue(Print(glyph_rows[sy as usize].as_str().with(stitch.color)))?;
                        }
                    } else if is_hint {
                        stdout.queue(SetAttribute(Attribute::Reverse))?;
                        for _ in 0..sw { stdout.queue(Print(stitch.to_string().as_str().with(stitch.color)))?; }
                        stdout.queue(SetAttribute(Attribute::Reset))?;
                    } else {
                        for _ in 0..sw { stdout.queue(Print(stitch))?; }
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
/// Uses compact height based on actual balloon content so stitches are
/// visible right at y0, aligned to the bottom of the yarn area.
pub fn render_balloon_columns(stdout: &mut Stdout, engine: &GameEngine, yarn_x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    if engine.yarn.balloon_columns.is_empty() {
        return Ok(());
    }
    let sh = scale;
    let sw = scale * 2;
    let regular_w = engine.yarn.yarn_lines * sw
        + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
    use super::COMP_GAP;
    let balloon_x0 = yarn_x0 + regular_w + COMP_GAP;

    // Single row of fixed slots, bottom-aligned with yarn
    let yarn_h = engine.yarn.visible_stitches * (sh + YARN_VGAP) - YARN_VGAP;
    let y_start = y0 + yarn_h - sh;

    for sy in 0..sh {
        stdout.queue(MoveTo(balloon_x0, y_start + sy))?;
        for (ci, slot) in engine.yarn.balloon_columns.iter().enumerate() {
            if ci > 0 {
                for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
            }
            match slot {
                Some(stitch) => {
                    if scale > 1 {
                        let glyph_rows = glyphs::yarn_patch_glyph(stitch.locked, scale);
                        stdout.queue(Print(glyph_rows[sy as usize].as_str().with(stitch.color)))?;
                    } else {
                        for _ in 0..sw { stdout.queue(Print(stitch))?; }
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
    let yarn_h = engine.yarn.visible_stitches * (sh + YARN_VGAP) - YARN_VGAP;
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
                    Some(Some(stitch)) => {
                        if scale > 1 {
                            let glyph_rows = glyphs::yarn_patch_glyph(stitch.locked, scale);
                            stdout.queue(Print(glyph_rows[sy as usize].as_str().with(stitch.color)))?;
                        } else {
                            for _ in 0..sw { stdout.queue(Print(stitch))?; }
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

/// Render held spools horizontally (one row, scaled) starting at (x0, y0).
pub fn render_active_h(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for sy in 0..sh {
        stdout.queue(MoveTo(x0, y0 + sy))?;
        for (i, spool) in engine.held_spools.iter().enumerate() {
            if i > 0 {
                for _ in 0..THREAD_GAP { stdout.queue(Print(' '))?; }
            }
            for _ in 0..sw { stdout.queue(Print(spool))?; }
        }
    }
    Ok(())
}

/// Render held spools vertically (one column, scaled) starting at (x0, y0).
pub fn render_active_v(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for (i, spool) in engine.held_spools.iter().enumerate() {
        let ty = y0 + (i as u16) * (sh + THREAD_GAP);
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, ty + sy))?;
            for _ in 0..sw { stdout.queue(Print(spool))?; }
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

    for (row_idx, board_row) in engine.board.board.iter().enumerate() {
        let content_y = y0 + 1 + (row_idx as u16) * (sh + 1);
        let is_cur_row = row_idx == cur_r;

        if scale > 1 {
            for (col_idx, cell) in board_row.iter().enumerate() {
                let is_cursor = is_cur_row && col_idx == cur_c;
                let is_after_cursor = is_cur_row && col_idx > 0 && col_idx - 1 == cur_c;
                let glyph_rows = match &engine.board.board[row_idx][col_idx] {
                    BoardEntity::Spool(_) => glyphs::entity_glyph_thread(scale),
                    BoardEntity::KeySpool(_) => glyphs::entity_glyph_key_thread(scale),
                    BoardEntity::Obstacle => glyphs::entity_glyph_obstacle(scale),
                    BoardEntity::Conveyor(data) => glyphs::entity_glyph_generator(data.output_dir, scale),
                    BoardEntity::EmptyConveyor => glyphs::entity_glyph_depleted(scale),
                    BoardEntity::Void => glyphs::entity_glyph_void(scale),
                };
                let color = match &engine.board.board[row_idx][col_idx] {
                    BoardEntity::Spool(c) | BoardEntity::KeySpool(c) => Some(*c),
                    BoardEntity::Conveyor(data) => Some(data.color),
                    _ => None,
                };
                let anim_frame = engine.anim_cells.get((row_idx, col_idx));
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
                    } else if let Some(anim) = anim_frame {
                        use crossterm::style::SetForegroundColor;
                        let (ch, anim_color) = match anim.frame {
                            3 => ('█', crossterm::style::Color::White),
                            2 => ('✦', crossterm::style::Color::Yellow),
                            1 => ('·', crossterm::style::Color::DarkGrey),
                            _ => (' ', crossterm::style::Color::Reset),
                        };
                        stdout.queue(SetForegroundColor(anim_color))?;
                        // Fill the full cell width with the burst glyph
                        let cell_str: String = std::iter::repeat(ch).take(sw as usize).collect();
                        stdout.queue(Print(cell_str))?;
                        stdout.queue(SetAttribute(Attribute::Reset))?;
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
                for (col_idx, cell) in board_row.iter().enumerate() {
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

                    // Cell content: inverted colors for cursor cell; bright-white bg for hint cell
                    let is_hint = engine.hint_ticks > 0
                        && engine.hint_cell == Some((row_idx, col_idx));
                    let anim_frame = engine.anim_cells.get((row_idx, col_idx));
                    if is_cursor {
                        stdout.queue(SetAttribute(Attribute::Reverse))?;
                        for _ in 0..sw { stdout.queue(Print(cell))?; }
                        stdout.queue(SetAttribute(Attribute::Reset))?;
                    } else if let Some(anim) = anim_frame {
                        use crossterm::style::{SetForegroundColor};
                        let (ch, color) = match anim.frame {
                            3 => ('█', crossterm::style::Color::White),
                            2 => ('✦', crossterm::style::Color::Yellow),
                            1 => ('·', crossterm::style::Color::DarkGrey),
                            _ => (' ', crossterm::style::Color::Reset),
                        };
                        stdout.queue(SetForegroundColor(color))?;
                        for _ in 0..sw { stdout.queue(Print(ch))?; }
                        stdout.queue(SetAttribute(Attribute::Reset))?;
                    } else if is_hint {
                        use crossterm::style::{SetBackgroundColor, SetForegroundColor};
                        stdout.queue(SetBackgroundColor(crossterm::style::Color::White))?;
                        stdout.queue(SetForegroundColor(crossterm::style::Color::Black))?;
                        for _ in 0..sw { stdout.queue(Print('*'))?; }
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
