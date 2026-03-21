use std::io::{self, Stdout};

use crossterm::{
    cursor::MoveTo,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor},
    QueueableCommand,
};

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
    stdout: &mut Stdout,
    bx: u16, by: u16,
    gr: usize, cw: usize,
    rows: usize, cols: usize,
) -> io::Result<()> {
    stdout.queue(MoveTo(bx, by))?;
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
    stdout.queue(Print(&line))?;
    Ok(())
}

// ── Cell rendering ────────────────────────────────────────────────────────

fn render_cell_content(
    stdout: &mut Stdout,
    engine: &GameEngine,
    r: usize, c: usize,
    sub_row: usize,
    cw: usize, ch: usize,
) -> io::Result<()> {
    let cell = &engine.board.cells[r][c];
    let mid = if ch > 1 { ch / 2 } else { 0 };
    let is_cursor   = r == engine.cursor_row && c == engine.cursor_col;
    let is_selected = engine.selected == Some((r, c));
    let is_hint     = engine.hint_pair
        .map_or(false, |(a, b)| a == (r, c) || b == (r, c));

    // Animation overlay: render burst frame and return early (skip normal content)
    if sub_row == mid {
        if let Some(anim) = engine.anim_cells.get((r, c)) {
            match (anim.kind, anim.frame) {
                (AnimKind::Dissolve, 3) => {
                    stdout.queue(SetForegroundColor(Color::White))?;
                    stdout.queue(SetAttribute(Attribute::Bold))?;
                    let label = "██";
                    let pad = cw.saturating_sub(label.chars().count());
                    stdout.queue(Print(format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))))?;
                    stdout.queue(SetAttribute(Attribute::Reset))?;
                    stdout.queue(ResetColor)?;
                    return Ok(());
                }
                (AnimKind::Dissolve, 2) => {
                    stdout.queue(SetForegroundColor(Color::Yellow))?;
                    let label = "✦ ";
                    let pad = cw.saturating_sub(label.chars().count());
                    stdout.queue(Print(format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))))?;
                    stdout.queue(ResetColor)?;
                    return Ok(());
                }
                (AnimKind::Dissolve, 1) => {
                    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                    let label = "· ";
                    let pad = cw.saturating_sub(label.chars().count());
                    stdout.queue(Print(format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))))?;
                    stdout.queue(ResetColor)?;
                    return Ok(());
                }
                (AnimKind::Rise, 3) => {
                    stdout.queue(SetForegroundColor(Color::Cyan))?;
                    let label = "✦ ";
                    let pad = cw.saturating_sub(label.chars().count());
                    stdout.queue(Print(format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))))?;
                    stdout.queue(ResetColor)?;
                    return Ok(());
                }
                (AnimKind::Rise, 2) => {
                    stdout.queue(SetForegroundColor(Color::Yellow))?;
                    let label = "★ ";
                    let pad = cw.saturating_sub(label.chars().count());
                    stdout.queue(Print(format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))))?;
                    stdout.queue(ResetColor)?;
                    return Ok(());
                }
                (AnimKind::Rise, 1) => {
                    // Frame 1: render actual item with bold/bright to show it just appeared
                    let (label, color, _) = glyphs::cell_label(cell);
                    stdout.queue(SetForegroundColor(color))?;
                    stdout.queue(SetAttribute(Attribute::Bold))?;
                    let chars = label.chars().count();
                    let pad = cw.saturating_sub(chars);
                    stdout.queue(Print(format!("{}{}{}", " ".repeat(pad / 2), label, " ".repeat(pad - pad / 2))))?;
                    stdout.queue(SetAttribute(Attribute::Reset))?;
                    stdout.queue(ResetColor)?;
                    return Ok(());
                }
                _ => {} // frame 0 or unexpected: fall through to normal render
            }
        }
    } else if engine.anim_cells.contains_key(&(r, c)) {
        // Non-mid rows during animation: blank them out
        stdout.queue(Print(" ".repeat(cw)))?;
        return Ok(());
    }

    // Background tint
    if is_selected {
        stdout.queue(SetBackgroundColor(Color::Rgb { r: 0, g: 60, b: 0 }))?;
    } else if is_cursor {
        stdout.queue(SetBackgroundColor(Color::Rgb { r: 60, g: 60, b: 0 }))?;
    } else if is_hint {
        stdout.queue(SetBackgroundColor(Color::Rgb { r: 0, g: 0, b: 70 }))?;
    }

    if sub_row == mid {
        let (label, color, bold) = glyphs::cell_label(cell);
        let is_frozen = matches!(cell, Cell::Frozen(_));

        stdout.queue(SetForegroundColor(color))?;
        if bold || is_selected || is_cursor {
            stdout.queue(SetAttribute(Attribute::Bold))?;
        }
        if is_frozen {
            stdout.queue(SetAttribute(Attribute::Dim))?;
        }

        // Center label in `cw` columns
        let chars = label.chars().count();
        let pad = cw.saturating_sub(chars);
        let pad_l = pad / 2;
        let pad_r = pad - pad_l;
        stdout.queue(Print(format!("{}{}{}", " ".repeat(pad_l), label, " ".repeat(pad_r))))?;
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
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                let s = format!("{:^w$}", cd, w = cw);
                stdout.queue(Print(&s))?;
                stdout.queue(ResetColor)?;
                return Ok(());
            }
        }
        stdout.queue(Print(" ".repeat(cw)))?;
    }

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Board ─────────────────────────────────────────────────────────────────

pub fn render_board(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) -> io::Result<()> {
    let scale = geo.scale;
    let (cw, ch) = cell_dims(scale);
    let rows = engine.board.rows;
    let cols = engine.board.cols;
    let bx = geo.board_x;
    let by = geo.board_y;

    for gr in 0..=rows {
        let y = by + (gr * (ch + 1)) as u16;
        render_grid_row(stdout, bx, y, gr, cw, rows, cols)?;

        if gr < rows {
            for sr in 0..ch {
                let y2 = y + 1 + sr as u16;
                stdout.queue(MoveTo(bx, y2))?;
                stdout.queue(Print("│"))?;
                for c in 0..cols {
                    render_cell_content(stdout, engine, gr, c, sr, cw, ch)?;
                    stdout.queue(Print("│"))?;
                }
            }
        }
    }

    stdout.queue(ResetColor)?;
    Ok(())
}
