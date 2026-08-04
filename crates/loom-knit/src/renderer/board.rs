use loom_engine::render::{Attrs, Color, Style, Surface};
use crate::engine::{GameEngine, BonusState};
use crate::board_entity::BoardEntity;
use crate::glyphs;
use super::{YARN_HGAP, YARN_VGAP, THREAD_GAP, FlankSide};

/// Render yarn stitches into a region starting at (x0, y0), scaled with spacing.
/// `with_balloons`: if true, render balloon columns to the right of regular
/// yarn (used in vertical layout). If false, caller handles balloon rendering
/// separately (used in horizontal layout to avoid overlap).
pub fn render_yarn(surface: &mut dyn Surface, engine: &GameEngine, x0: u16, y0: u16, scale: u16, with_balloons: bool) {
    let sh = scale;
    let sw = scale * 2;
    for offset in 0..(engine.yarn.visible_stitches as usize) {
        let true_offset = (engine.yarn.visible_stitches as usize) - offset;
        let row_y = y0 + (offset as u16) * (sh + YARN_VGAP);
        for sy in 0..sh {
            let y = row_y + sy;
            let mut x = x0;
            for (ci, column) in engine.yarn.board.iter().enumerate() {
                if ci > 0 {
                    surface.print(x, y, &" ".repeat(YARN_HGAP as usize), Style::default());
                    x += YARN_HGAP;
                }
                if true_offset <= column.len() {
                    let pos = column.len() - true_offset;
                    let stitch = &column[pos];
                    let is_hint = engine.blessing_flags.match_hint
                        && engine.last_picked_color.is_some()
                        && engine.last_picked_color.unwrap() == stitch.color
                        && !stitch.locked;
                    let style = Style {
                        fg: stitch.color,
                        attrs: if is_hint { Attrs::REVERSE } else { Attrs::empty() },
                        ..Default::default()
                    };
                    if scale > 1 {
                        let glyph_rows = glyphs::yarn_patch_glyph(stitch.locked, scale);
                        surface.print(x, y, &glyph_rows[sy as usize], style);
                    } else {
                        surface.print(x, y, &stitch.to_string().repeat(sw as usize), style);
                    }
                } else {
                    surface.print(x, y, &" ".repeat(sw as usize), Style::default());
                }
                x += sw;
            }
        }
    }

    // Render balloon columns to the right (vertical layout only)
    if with_balloons {
        render_balloon_columns(surface, engine, x0, y0, scale);
    }
}

/// Render balloon pseudo-columns at (x0, y0), to the right of regular yarn.
/// Uses compact height based on actual balloon content so stitches are
/// visible right at y0, aligned to the bottom of the yarn area.
pub fn render_balloon_columns(surface: &mut dyn Surface, engine: &GameEngine, yarn_x0: u16, y0: u16, scale: u16) {
    if engine.yarn.balloon_columns.is_empty() {
        return;
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
        let y = y_start + sy;
        let mut x = balloon_x0;
        for (ci, slot) in engine.yarn.balloon_columns.iter().enumerate() {
            if ci > 0 {
                surface.print(x, y, &" ".repeat(YARN_HGAP as usize), Style::default());
                x += YARN_HGAP;
            }
            match slot {
                Some(stitch) => {
                    let style = Style { fg: stitch.color, ..Default::default() };
                    if scale > 1 {
                        let glyph_rows = glyphs::yarn_patch_glyph(stitch.locked, scale);
                        surface.print(x, y, &glyph_rows[sy as usize], style);
                    } else {
                        surface.print(x, y, &stitch.to_string().repeat(sw as usize), style);
                    }
                }
                None => {
                    surface.print(x, y, &" ".repeat(sw as usize), Style::default());
                }
            }
            x += sw;
        }
    }
}

/// Render a single flanking balloon cell (left or right of yarn).
/// Each flank is one patch wide (sw). Left shows patches lifted from the
/// leftmost yarn column, right shows patches from the rightmost.
/// balloon_columns[0] = left patches, balloon_columns[last] = right patches.
/// Shows dim ░ placeholders when balloons available but unused.
pub fn render_balloon_flank(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    x0: u16,
    y0: u16,
    scale: u16,
    side: FlankSide,
) {
    let sh = scale;
    let sw = scale * 2;
    let balloon_count = engine.bonuses.balloon_count as usize;

    // Left flank gets first left_count slots, right gets the rest
    let (start_idx, count) = match side {
        FlankSide::Left  => (0, balloon_count / 2),
        FlankSide::Right => (balloon_count / 2, (balloon_count + 1) / 2),
    };
    if count == 0 { return; }

    let show = engine.bonuses.balloons > 0 || !engine.yarn.balloon_columns.is_empty();
    if !show { return; }

    let slots = &engine.yarn.balloon_columns;

    // Bottom-align with yarn visible area
    let yarn_h = engine.yarn.visible_stitches * (sh + YARN_VGAP) - YARN_VGAP;
    let flank_h = count as u16 * (sh + YARN_VGAP) - YARN_VGAP;
    let y_start = y0 + yarn_h.saturating_sub(flank_h);

    for i in 0..count {
        let row_y = y_start + (i as u16) * (sh + YARN_VGAP);
        let slot_idx = start_idx + i;
        for sy in 0..sh {
            let y = row_y + sy;
            if slots.is_empty() {
                // Balloons available but unused — show placeholder
                let style = Style { fg: Color::DarkGrey, ..Default::default() };
                surface.print(x0, y, &"░".repeat(sw as usize), style);
            } else {
                match slots.get(slot_idx) {
                    Some(Some(stitch)) => {
                        let style = Style { fg: stitch.color, ..Default::default() };
                        if scale > 1 {
                            let glyph_rows = glyphs::yarn_patch_glyph(stitch.locked, scale);
                            surface.print(x0, y, &glyph_rows[sy as usize], style);
                        } else {
                            surface.print(x0, y, &stitch.to_string().repeat(sw as usize), style);
                        }
                    }
                    Some(None) | None => {
                        // Processed — empty space
                        surface.print(x0, y, &" ".repeat(sw as usize), Style::default());
                    }
                }
            }
        }
    }
}

/// Render held spools horizontally (one row, scaled) starting at (x0, y0).
pub fn render_active_h(surface: &mut dyn Surface, engine: &GameEngine, x0: u16, y0: u16, scale: u16) {
    let sh = scale;
    let sw = scale * 2;
    for sy in 0..sh {
        let y = y0 + sy;
        let mut x = x0;
        for (i, spool) in engine.held_spools.iter().enumerate() {
            if i > 0 {
                surface.print(x, y, &" ".repeat(THREAD_GAP as usize), Style::default());
                x += THREAD_GAP;
            }
            let style = Style { fg: spool.color, ..Default::default() };
            surface.print(x, y, &spool.to_string().repeat(sw as usize), style);
            x += sw;
        }
    }
}

/// Render held spools vertically (one column, scaled) starting at (x0, y0).
pub fn render_active_v(surface: &mut dyn Surface, engine: &GameEngine, x0: u16, y0: u16, scale: u16) {
    let sh = scale;
    let sw = scale * 2;
    for (i, spool) in engine.held_spools.iter().enumerate() {
        let ty = y0 + (i as u16) * (sh + THREAD_GAP);
        let style = Style { fg: spool.color, ..Default::default() };
        for sy in 0..sh {
            surface.print(x0, ty + sy, &spool.to_string().repeat(sw as usize), style);
        }
    }
}

/// Draw a horizontal border line for the board grid.
/// kind: 0=top (┌┬┐), 1=middle (├┼┤), 2=bottom (└┴┘)
pub fn draw_hline(surface: &mut dyn Surface, x0: u16, y: u16, cols: usize, sw: u16, kind: u8) {
    let (left, fill, cross, right) = match kind {
        0 => ('┌', '─', '┬', '┐'),
        2 => ('└', '─', '┴', '┘'),
        _ => ('├', '─', '┼', '┤'),
    };
    let mut line = String::new();
    line.push(left);
    for c in 0..cols {
        for _ in 0..sw { line.push(fill); }
        if c < cols - 1 { line.push(cross); }
    }
    line.push(right);
    surface.print(x0, y, &line, Style::default());
}

/// Render the game board with box borders and bracket cursor markers.
pub fn render_board(surface: &mut dyn Surface, engine: &GameEngine, x0: u16, y0: u16, scale: u16) {
    let sh = scale;
    let sw = scale * 2;
    let rows = engine.board.height as usize;
    let cols = engine.board.width as usize;
    let cur_r = engine.cursor_row as usize;
    let cur_c = engine.cursor_col as usize;

    let tweezers = matches!(engine.bonus_state, BonusState::TweezersActive { .. });
    let (open_bracket, close_bracket) = if tweezers { ('{', '}') } else { ('[', ']') };
    let bracket_style = Style { fg: Color::White, attrs: Attrs::BOLD, ..Default::default() };

    // Top border
    draw_hline(surface, x0, y0, cols, sw, 0);

    for (row_idx, board_row) in engine.board.board.iter().enumerate() {
        let content_y = y0 + 1 + (row_idx as u16) * (sh + 1);
        let is_cur_row = row_idx == cur_r;

        if scale > 1 {
            for (col_idx, _) in board_row.iter().enumerate() {
                let is_cursor = is_cur_row && col_idx == cur_c;
                let is_after_cursor = is_cur_row && col_idx > 0 && col_idx - 1 == cur_c;
                let entity = &engine.board.board[row_idx][col_idx];
                let glyph_rows = match entity {
                    BoardEntity::Spool(_) => glyphs::entity_glyph_thread(scale),
                    BoardEntity::KeySpool(_) => glyphs::entity_glyph_key_thread(scale),
                    BoardEntity::Obstacle => glyphs::entity_glyph_obstacle(scale),
                    BoardEntity::Conveyor(data) => glyphs::entity_glyph_generator(data.output_dir, scale),
                    BoardEntity::EmptyConveyor => glyphs::entity_glyph_depleted(scale),
                    BoardEntity::Void => glyphs::entity_glyph_void(scale),
                };
                let color = entity.color();
                let anim_frame = engine.anim_cells.get((row_idx, col_idx));
                for (sy_idx, glyph_row) in glyph_rows.iter().enumerate() {
                    let cell_x = x0 + 1 + (col_idx as u16) * (sw + 1);
                    let cell_y = content_y + sy_idx as u16;
                    let border_x = x0 + (col_idx as u16) * (sw + 1);
                    // Left border
                    if is_cursor {
                        surface.print(border_x, cell_y, &open_bracket.to_string(), bracket_style);
                    } else if is_after_cursor {
                        surface.print(border_x, cell_y, &close_bracket.to_string(), bracket_style);
                    } else {
                        surface.print(border_x, cell_y, "│", Style::default());
                    }
                    // Cell content
                    if is_cursor {
                        let chars: Vec<char> = glyph_row.chars().collect();
                        let inner: String = chars[1..chars.len()-1].iter().collect();
                        let inner_style = Style { fg: color.unwrap_or(Color::Reset), ..Default::default() };
                        let mut x = cell_x;
                        surface.print(x, cell_y, &open_bracket.to_string(), bracket_style);
                        x += 1;
                        surface.print(x, cell_y, &inner, inner_style);
                        x += inner.chars().count() as u16;
                        surface.print(x, cell_y, &close_bracket.to_string(), bracket_style);
                    } else if let Some(anim) = anim_frame {
                        let (ch, anim_color) = match anim.frame {
                            3 => ('█', Color::White),
                            2 => ('✦', Color::Yellow),
                            1 => ('·', Color::DarkGrey),
                            _ => (' ', Color::Reset),
                        };
                        // Fill the full cell width with the burst glyph
                        let cell_str: String = std::iter::repeat(ch).take(sw as usize).collect();
                        surface.print(cell_x, cell_y, &cell_str, Style { fg: anim_color, ..Default::default() });
                    } else {
                        let style = Style { fg: color.unwrap_or(Color::Reset), ..Default::default() };
                        surface.print(cell_x, cell_y, glyph_row, style);
                    }
                }
            }
            // Right border for glyph path
            for sy_idx in 0..sh {
                let cell_y = content_y + sy_idx;
                let border_x = x0 + (cols as u16) * (sw + 1);
                if is_cur_row && cols - 1 == cur_c {
                    surface.print(border_x, cell_y, &close_bracket.to_string(), bracket_style);
                } else {
                    surface.print(border_x, cell_y, "│", Style::default());
                }
            }
        } else {
            for sy in 0..sh {
                let y = content_y + sy;
                let mut x = x0;
                for (col_idx, cell) in board_row.iter().enumerate() {
                    let is_cursor = is_cur_row && col_idx == cur_c;
                    let is_after_cursor = is_cur_row && col_idx > 0 && col_idx - 1 == cur_c;

                    // Left border: bright brackets for cursor edges, normal │ otherwise
                    if is_cursor {
                        surface.print(x, y, &open_bracket.to_string(), bracket_style);
                    } else if is_after_cursor {
                        surface.print(x, y, &close_bracket.to_string(), bracket_style);
                    } else {
                        surface.print(x, y, "│", Style::default());
                    }
                    x += 1;

                    // Cell content: inverted colors for cursor cell; bright-white bg for hint cell
                    let is_hint = engine.hint_ticks > 0
                        && engine.hint_cell == Some((row_idx, col_idx));
                    let anim_frame = engine.anim_cells.get((row_idx, col_idx));
                    if is_cursor {
                        let style = Style {
                            fg: cell.color().unwrap_or(Color::Reset),
                            attrs: Attrs::REVERSE,
                            ..Default::default()
                        };
                        surface.print(x, y, &cell.to_string().repeat(sw as usize), style);
                    } else if let Some(anim) = anim_frame {
                        let (ch, color) = match anim.frame {
                            3 => ('█', Color::White),
                            2 => ('✦', Color::Yellow),
                            1 => ('·', Color::DarkGrey),
                            _ => (' ', Color::Reset),
                        };
                        surface.print(x, y, &ch.to_string().repeat(sw as usize), Style { fg: color, ..Default::default() });
                    } else if is_hint {
                        let style = Style { fg: Color::Black, bg: Color::White, ..Default::default() };
                        surface.print(x, y, &"*".repeat(sw as usize), style);
                    } else {
                        let style = Style { fg: cell.color().unwrap_or(Color::Reset), ..Default::default() };
                        surface.print(x, y, &cell.to_string().repeat(sw as usize), style);
                    }
                    x += sw;
                }
                // Right border
                if is_cur_row && cols - 1 == cur_c {
                    surface.print(x, y, &close_bracket.to_string(), bracket_style);
                } else {
                    surface.print(x, y, "│", Style::default());
                }
            }
        }

        let line_y = content_y + sh;
        if row_idx < rows - 1 {
            draw_hline(surface, x0, line_y, cols, sw, 1);
        } else {
            draw_hline(surface, x0, line_y, cols, sw, 2);
        }
    }
}
