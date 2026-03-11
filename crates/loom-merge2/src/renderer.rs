use std::io::{self, Stdout, Write};

use crossterm::{
    cursor::{Hide, MoveTo},
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor, Stylize},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

use crate::blessings::{self, ALL_BLESSINGS};
use crate::board::Cell;
use crate::engine::{GameEngine, GameStatus};
use crate::glyphs;

// ── Layout ────────────────────────────────────────────────────────────────

/// Cell inner dimensions (width in columns, height in rows) at a given scale.
fn cell_dims(scale: u16) -> (usize, usize) {
    let w = scale as usize * 2 + 2; // 4 at s1, 6 at s2, 8 at s3
    let h = scale as usize;          // 1 at s1, 2 at s2, 3 at s3
    (w, h)
}

#[derive(Clone, Copy, Debug)]
pub enum Layout { Horizontal, Vertical }

pub struct LayoutGeometry {
    pub layout: Layout,
    pub board_x: u16,
    pub board_y: u16,
    pub order_x: u16,
    pub order_y: u16,
    pub scale: u16,
}

impl LayoutGeometry {
    pub fn compute(engine: &GameEngine) -> Self {
        let scale = engine.scale;
        let (cw, _ch) = cell_dims(scale);
        let board_w = (engine.board.width * (cw + 1) + 1) as u16;

        let (term_w, _term_h) = crossterm::terminal::size().unwrap_or((80, 24));
        let order_panel_w = 22u16;

        if board_w + order_panel_w + 4 <= term_w {
            LayoutGeometry {
                layout: Layout::Horizontal,
                board_x: 2,
                board_y: 3,
                order_x: 2 + board_w + 2,
                order_y: 3,
                scale,
            }
        } else {
            let order_rows = engine.orders.len() as u16 + 4;
            LayoutGeometry {
                layout: Layout::Vertical,
                board_x: 2,
                board_y: 3 + order_rows,
                order_x: 2,
                order_y: 3,
                scale,
            }
        }
    }
}

// ── Board rendering ───────────────────────────────────────────────────────

/// Box-drawing character for a grid intersection.
fn grid_intersection(gr: usize, gc: usize, rows: usize, cols: usize) -> char {
    let up = gr > 0;
    let down = gr < rows;
    let left = gc > 0;
    let right = gc < cols;
    match (up, down, left, right) {
        (false, true,  false, true)  => '┌',
        (false, true,  true,  true)  => '┬',
        (false, true,  true,  false) => '┐',
        (true,  true,  false, true)  => '├',
        (true,  true,  true,  true)  => '┼',
        (true,  true,  true,  false) => '┤',
        (true,  false, false, true)  => '└',
        (true,  false, true,  true)  => '┴',
        (true,  false, true,  false) => '┘',
        _ => '+',
    }
}

/// Draw a full horizontal grid border row.
fn draw_h_border(
    stdout: &mut Stdout, bx: u16, y: u16,
    gr: usize, cols: usize, rows: usize, cw: usize,
) -> io::Result<()> {
    stdout.queue(MoveTo(bx, y))?;
    let mut line = String::with_capacity(cols * (cw + 1) + 1);
    for gc in 0..=cols {
        line.push(grid_intersection(gr, gc, rows, cols));
        if gc < cols {
            for _ in 0..cw { line.push('─'); }
        }
    }
    stdout.queue(Print(&line))?;
    Ok(())
}

/// Render the content of a single cell sub-row.
fn render_cell_content(
    stdout: &mut Stdout,
    engine: &GameEngine,
    r: usize, c: usize,
    sub_row: usize,
    cw: usize, ch: usize,
) -> io::Result<()> {
    let cell = &engine.board.cells[r][c];
    let mid = ch / 2;
    let is_cursor = r == engine.cursor_row && c == engine.cursor_col;
    let is_selected = engine.selected == Some((r, c));

    // Subtle background tint for cursor / selected cells
    if is_selected {
        stdout.queue(SetBackgroundColor(Color::Rgb { r: 0, g: 50, b: 0 }))?;
    } else if is_cursor {
        stdout.queue(SetBackgroundColor(Color::Rgb { r: 50, g: 50, b: 0 }))?;
    }

    match cell {
        Cell::Item(item) => {
            stdout.queue(SetForegroundColor(item.color))?;
            if is_cursor || is_selected {
                stdout.queue(SetAttribute(Attribute::Bold))?;
            }
            if sub_row == mid {
                let glyph = glyphs::tier_glyph(item.tier);
                stdout.queue(Print(format!("{:^w$}", glyph, w = cw)))?;
            } else {
                stdout.queue(Print(" ".repeat(cw)))?;
            }
        }
        Cell::Generator { color, charges, .. } => {
            let exhausted = matches!(charges, Some(0));
            if exhausted {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            } else {
                stdout.queue(SetForegroundColor(*color))?;
            }
            if is_cursor { stdout.queue(SetAttribute(Attribute::Bold))?; }
            if sub_row == mid {
                let label = match charges {
                    Some(n) => format!("G{}", n),
                    None => "G\u{221e}".to_string(),
                };
                stdout.queue(Print(format!("{:^w$}", label, w = cw)))?;
            } else {
                stdout.queue(Print(" ".repeat(cw)))?;
            }
        }
        Cell::Blocked => {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            stdout.queue(Print("\u{2591}".repeat(cw)))?;
        }
        Cell::Empty => {
            if sub_row == mid {
                stdout.queue(SetForegroundColor(Color::Rgb { r: 60, g: 60, b: 60 }))?;
                stdout.queue(Print(format!("{:^w$}", "\u{00b7}", w = cw)))?;
            } else {
                stdout.queue(Print(" ".repeat(cw)))?;
            }
        }
    }
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

/// Overdraw the border of a single cell in a highlight color.
fn highlight_cell_border(
    stdout: &mut Stdout,
    bx: u16, by: u16,
    r: usize, c: usize,
    cw: usize, ch: usize,
    rows: usize, cols: usize,
    color: Color,
) -> io::Result<()> {
    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;

    let left_x = bx + (c * (cw + 1)) as u16;
    let right_x = bx + ((c + 1) * (cw + 1)) as u16;
    let top_y = by + (r * (ch + 1)) as u16;
    let bot_y = by + ((r + 1) * (ch + 1)) as u16;

    // Top: tl ──── tr
    stdout.queue(MoveTo(left_x, top_y))?;
    stdout.queue(Print(grid_intersection(r, c, rows, cols)))?;
    stdout.queue(Print("─".repeat(cw)))?;
    stdout.queue(MoveTo(right_x, top_y))?;
    stdout.queue(Print(grid_intersection(r, c + 1, rows, cols)))?;

    // Bottom: bl ──── br
    stdout.queue(MoveTo(left_x, bot_y))?;
    stdout.queue(Print(grid_intersection(r + 1, c, rows, cols)))?;
    stdout.queue(Print("─".repeat(cw)))?;
    stdout.queue(MoveTo(right_x, bot_y))?;
    stdout.queue(Print(grid_intersection(r + 1, c + 1, rows, cols)))?;

    // Left and right vertical borders
    for sub in 0..ch {
        let y = top_y + 1 + sub as u16;
        stdout.queue(MoveTo(left_x, y))?;
        stdout.queue(Print("│"))?;
        stdout.queue(MoveTo(right_x, y))?;
        stdout.queue(Print("│"))?;
    }

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

pub fn render_board(stdout: &mut Stdout, engine: &GameEngine, geo: &LayoutGeometry) -> io::Result<()> {
    let scale = geo.scale;
    let (cw, ch) = cell_dims(scale);
    let rows = engine.board.height;
    let cols = engine.board.width;
    let bx = geo.board_x;
    let by = geo.board_y;

    // ── Draw grid ──
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    draw_h_border(stdout, bx, by, 0, cols, rows, cw)?;

    for r in 0..rows {
        for sub in 0..ch {
            let y = by + (r * (ch + 1)) as u16 + 1 + sub as u16;
            for c in 0..cols {
                let x = bx + (c * (cw + 1)) as u16;
                stdout.queue(MoveTo(x, y))?;
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                render_cell_content(stdout, engine, r, c, sub, cw, ch)?;
            }
            let x = bx + (cols * (cw + 1)) as u16;
            stdout.queue(MoveTo(x, y))?;
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            stdout.queue(Print("│"))?;
        }
        let sep_y = by + ((r + 1) * (ch + 1)) as u16;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        draw_h_border(stdout, bx, sep_y, r + 1, cols, rows, cw)?;
    }

    // ── Cursor highlight (yellow) ──
    highlight_cell_border(stdout, bx, by, engine.cursor_row, engine.cursor_col,
                          cw, ch, rows, cols, Color::Yellow)?;

    // ── Selected highlight (green, drawn last to take priority) ──
    if let Some((sr, sc)) = engine.selected {
        highlight_cell_border(stdout, bx, by, sr, sc,
                              cw, ch, rows, cols, Color::Green)?;
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Orders panel ──────────────────────────────────────────────────────────

pub fn render_orders(stdout: &mut Stdout, engine: &GameEngine, geo: &LayoutGeometry) -> io::Result<()> {
    let x = geo.order_x;
    let mut y = geo.order_y;

    stdout.queue(MoveTo(x, y))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print("┌─── Orders ───┐"))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    y += 1;

    for (i, order) in engine.orders.iter().enumerate() {
        let fulfilled = order.is_fulfilled();
        stdout.queue(MoveTo(x, y))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(Print(format!("│ Order {}:      │", i + 1)))?;
        y += 1;

        for oi in &order.items {
            stdout.queue(MoveTo(x, y))?;
            stdout.queue(Print("│  "))?;

            if fulfilled || oi.is_fulfilled() {
                stdout.queue(SetForegroundColor(Color::Green))?;
                stdout.queue(Print("✓ "))?;
            } else {
                stdout.queue(SetForegroundColor(oi.color))?;
            }

            let glyph = crate::item::Item::new(oi.color, oi.tier).glyph();
            stdout.queue(Print(format!("{} ×{}/{}", glyph, oi.delivered, oi.required)))?;

            // Pad to panel width
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print("     │"))?;
            y += 1;
        }
    }

    stdout.queue(MoveTo(x, y))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(Print("└───────────────┘"))?;

    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Score bar ─────────────────────────────────────────────────────────────

pub fn render_score(stdout: &mut Stdout, engine: &GameEngine) -> io::Result<()> {
    stdout.queue(MoveTo(1, 1))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("Score: {}", engine.score)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    // Show ad count if available
    if engine.ad_limit > 0 {
        stdout.queue(Print(format!("  Ads: {}/{}", engine.ads_used, engine.ad_limit)))?;
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Key bar ───────────────────────────────────────────────────────────────

pub fn render_key_bar(stdout: &mut Stdout, _engine: &GameEngine) -> io::Result<()> {
    let (_, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let y = term_h - 1;

    stdout.queue(MoveTo(0, y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;

    let keys = "←→↑↓ Move  Enter Select/Merge  D Deliver  A Ad  H Help  +/- Scale  Q Quit";
    stdout.queue(Print(keys))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Game over overlay ─────────────────────────────────────────────────────

pub fn render_game_over(stdout: &mut Stdout, status: &GameStatus, score: u32) -> io::Result<()> {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;
    let cy = term_h / 2;

    let (title, color) = match status {
        GameStatus::Won => ("ORDER COMPLETE!", Color::Green),
        GameStatus::Lost => ("BOARD FULL!", Color::Red),
        GameStatus::Stuck => ("STUCK!", Color::Yellow),
        GameStatus::Playing => return Ok(()),
    };

    // Box
    let box_w = 24u16;
    let bx = cx.saturating_sub(box_w / 2);
    let by = cy.saturating_sub(3);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", title, w = box_w as usize - 2)))?;

    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(Print(format!("║{:^w$}║", format!("Score: {}", score), w = box_w as usize - 2)))?;

    stdout.queue(MoveTo(bx, by + 3))?;
    stdout.queue(Print(format!("║{:^w$}║", "", w = box_w as usize - 2)))?;

    let prompt = match status {
        GameStatus::Won => "Enter: Continue",
        GameStatus::Stuck => "A: Watch Ad  Q: Quit",
        _ => "R: Retry  Q: Quit",
    };
    stdout.queue(MoveTo(bx, by + 4))?;
    stdout.queue(Print(format!("║{:^w$}║", prompt, w = box_w as usize - 2)))?;

    stdout.queue(MoveTo(bx, by + 5))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Help overlay ──────────────────────────────────────────────────────────

pub fn render_help(stdout: &mut Stdout, help_lines: &[(&str, &str)]) -> io::Result<()> {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;
    let box_w = 36u16;
    let box_h = help_lines.len() as u16 + 4;
    let bx = cx.saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(box_h / 2);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "MERGE-2 HELP", w = box_w as usize - 2)))?;

    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    for (i, (key, desc)) in help_lines.iter().enumerate() {
        stdout.queue(MoveTo(bx, by + 3 + i as u16))?;
        stdout.queue(Print(format!("║ {:>10} │ {:<w$}║", key, desc, w = box_w as usize - 16)))?;
    }

    stdout.queue(MoveTo(bx, by + 3 + help_lines.len() as u16))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Main menu ─────────────────────────────────────────────────────────────

pub fn render_main_menu(
    stdout: &mut Stdout,
    items: &[&str],
    selected: usize,
    flash: Option<&str>,
) -> io::Result<()> {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;

    // Title
    let title = "═══ MERGE-2 ═══";
    stdout.queue(MoveTo(cx.saturating_sub(title.len() as u16 / 2), 2))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(title))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    // Menu items
    for (i, item) in items.iter().enumerate() {
        let y = 5 + i as u16;
        stdout.queue(MoveTo(cx.saturating_sub(10), y))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(format!("▸ {}", item)))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print(format!("  {}", item)))?;
        }
    }

    // Flash message
    if let Some(msg) = flash {
        stdout.queue(MoveTo(cx.saturating_sub(msg.len() as u16 / 2), term_h - 3))?;
        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(Print(msg))?;
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Campaign select ───────────────────────────────────────────────────────

pub fn render_campaign_select(
    stdout: &mut Stdout,
    track_names: &[&str],
    progress: &[String],
    selected: usize,
) -> io::Result<()> {
    let (term_w, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;

    stdout.queue(MoveTo(cx.saturating_sub(10), 2))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print("Select Campaign Track"))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    for (i, name) in track_names.iter().enumerate() {
        let y = 5 + (i as u16 * 2);
        stdout.queue(MoveTo(cx.saturating_sub(15), y))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(format!("▸ {:<12} {}", name, progress[i])))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print(format!("  {:<12} {}", name, progress[i])))?;
        }
    }

    stdout.queue(MoveTo(cx.saturating_sub(10), 5 + (track_names.len() as u16 * 2) + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("Enter: Select  Esc: Back"))?;

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Level intro ───────────────────────────────────────────────────────────

pub fn render_level_intro(stdout: &mut Stdout, lines: &[String]) -> io::Result<()> {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;
    let cy = term_h / 2;
    let start_y = cy.saturating_sub(lines.len() as u16 / 2);

    for (i, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(cx.saturating_sub(line.len() as u16 / 2), start_y + i as u16))?;
        if i == 0 {
            stdout.queue(SetForegroundColor(Color::Cyan))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
        }
        stdout.queue(Print(line))?;
    }

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(cx.saturating_sub(10), start_y + lines.len() as u16 + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("Press Enter to start"))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Ad overlay ────────────────────────────────────────────────────────────

fn ad_boxed_line(stdout: &mut Stdout, x0: u16, y: u16, inner_w: usize, content: &str) -> io::Result<()> {
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("║"))?;
    let content_chars = content.chars().count();
    stdout.queue(Print(content))?;
    for _ in content_chars..inner_w {
        stdout.queue(Print(' '))?;
    }
    stdout.queue(Print("║"))?;
    Ok(())
}

fn ad_center_text(text: &str, width: usize) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }
    let padding = (width - text_len) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

fn ad_word_wrap(text: &str, max_width: usize) -> Vec<String> {
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
    lines
}

pub fn render_ad_overlay(stdout: &mut Stdout, quote: &str, elapsed_secs: u64) -> io::Result<()> {
    let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    let remaining = 15u64.saturating_sub(elapsed_secs);
    let progress = ((elapsed_secs as f64 / 15.0) * 100.0).min(100.0) as u16;
    let done = remaining == 0;

    // Box dimensions
    let box_w = 50u16.min(term_w.saturating_sub(4));
    let box_inner = (box_w - 2) as usize;

    let wrapped = ad_word_wrap(quote, box_inner);
    let box_h = 8 + wrapped.len() as u16;
    let x0 = (term_w.saturating_sub(box_w)) / 2;
    let y0 = (term_h.saturating_sub(box_h)) / 2;

    let mut y = y0;

    // Top border
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print("╔"))?;
    for _ in 0..box_inner { stdout.queue(Print("═"))?; }
    stdout.queue(Print("╗"))?;
    y += 1;

    // Empty line
    ad_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Header
    ad_boxed_line(stdout, x0, y, box_inner, &ad_center_text("🧩 FREE SPACE 🧩", box_inner))?;
    y += 1;

    // Empty line
    ad_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Quote lines (word-wrapped)
    stdout.queue(SetForegroundColor(Color::White))?;
    for line in &wrapped {
        ad_boxed_line(stdout, x0, y, box_inner, &ad_center_text(line, box_inner))?;
        y += 1;
    }

    // Empty line
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    ad_boxed_line(stdout, x0, y, box_inner, "")?;
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
    stdout.queue(SetForegroundColor(Color::Green))?;
    ad_boxed_line(stdout, x0, y, box_inner, &ad_center_text(&bar, box_inner))?;
    y += 1;

    // Countdown or close prompt
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    if done {
        let msg = "[ Press ESC to collect your reward ]";
        ad_boxed_line(stdout, x0, y, box_inner, &ad_center_text(msg, box_inner))?;
    } else {
        let msg = format!("[{}s remaining]", remaining);
        ad_boxed_line(stdout, x0, y, box_inner, &ad_center_text(&msg, box_inner))?;
    }
    y += 1;

    // Empty line
    ad_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Bottom border
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("╚"))?;
    for _ in 0..box_inner { stdout.queue(Print("═"))?; }
    stdout.queue(Print("╝"))?;

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Options screen ────────────────────────────────────────────────────────

pub fn render_options(
    stdout: &mut Stdout,
    settings: &loom_engine::settings::UserSettings,
    selected: usize,
) -> io::Result<()> {
    let (term_w, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;

    stdout.queue(MoveTo(cx.saturating_sub(5), 2))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print("Options"))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    let items = [
        format!("Scale: {}", settings.scale),
        format!("Color mode: {}", settings.color_mode),
    ];

    for (i, item) in items.iter().enumerate() {
        stdout.queue(MoveTo(cx.saturating_sub(12), 5 + i as u16 * 2))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(format!("◂ {} ▸", item)))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print(format!("  {}", item)))?;
        }
    }

    stdout.queue(MoveTo(cx.saturating_sub(12), 5 + items.len() as u16 * 2 + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("←→ Adjust  Esc: Back"))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Custom game ───────────────────────────────────────────────────────────

pub fn render_custom_game(
    stdout: &mut Stdout,
    config: &crate::config::Config,
    preset_name: &str,
    selected_field: usize,
) -> io::Result<()> {
    let (term_w, _) = crossterm::terminal::size().unwrap_or((80, 24));
    let cx = term_w / 2;

    stdout.queue(MoveTo(cx.saturating_sub(8), 2))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print("Custom Game"))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    let fields = [
        format!("Preset: {}", preset_name),
        format!("Height: {}", config.board_height),
        format!("Width: {}", config.board_width),
        format!("Colors: {}", config.color_count),
        format!("Generators: {}", config.generator_count),
        format!("Gen Charges: {}", if config.generator_charges == 0 { "∞".to_string() } else { config.generator_charges.to_string() }),
        format!("Gen Interval: {}", config.generator_interval),
        format!("Blocked: {}", config.blocked_cells),
        format!("Orders: {}", config.order_count),
        format!("Max Tier: {}", config.max_order_tier),
        format!("Ad Limit: {}", config.ad_limit),
    ];

    for (i, field) in fields.iter().enumerate() {
        stdout.queue(MoveTo(cx.saturating_sub(12), 4 + i as u16))?;
        if i == selected_field {
            stdout.queue(SetForegroundColor(Color::Yellow))?;
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(format!("◂ {} ▸", field)))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(Print(format!("  {}", field)))?;
        }
    }

    stdout.queue(MoveTo(cx.saturating_sub(12), 4 + fields.len() as u16 + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("←→ Adjust  Enter: Start  Esc: Back"))?;
    stdout.queue(ResetColor)?;

    Ok(())
}

// ── Blessing selection ────────────────────────────────────────────────────

const CARD_W: usize = 17;
const CARD_H: usize = 11;
const CARD_COLS: usize = 3;

pub fn render_blessing_selection(
    stdout: &mut Stdout,
    cursor: usize,
    chosen: &[usize],
    completed_tracks: usize,
) -> io::Result<()> {
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, _term_h) = terminal::size().unwrap_or((80, 24));
    let total_blessings = ALL_BLESSINGS.len();
    let rows = (total_blessings + CARD_COLS - 1) / CARD_COLS;

    // Title
    let title = "═══ CHOOSE 3 BLESSINGS ═══";
    let title_x = term_w.saturating_sub(title.len() as u16) / 2;
    stdout.queue(MoveTo(title_x, 0))?;
    stdout.queue(Print(title))?;

    // Grid origin
    let grid_w = (CARD_W + 3) * CARD_COLS + 1;
    let grid_x = (term_w as usize).saturating_sub(grid_w) / 2;
    let grid_y = 2u16;

    for idx in 0..total_blessings {
        let b = &ALL_BLESSINGS[idx];
        let row = idx / CARD_COLS;
        let col = idx % CARD_COLS;
        let x = grid_x + col * (CARD_W + 3);
        let y = grid_y + (row as u16) * (CARD_H as u16 + 1);

        let is_cursor = idx == cursor;
        let is_chosen = chosen.contains(&idx);
        let unlocked = blessings::is_unlocked(b, completed_tracks);

        let (tl, tr, bl, br, hz, vt) = if is_chosen {
            ('╔', '╗', '╚', '╝', '═', '║')
        } else {
            ('┌', '┐', '└', '┘', '─', '│')
        };

        // Top border
        let top = format!("{}{}{}", tl, hz.to_string().repeat(CARD_W), tr);
        stdout.queue(MoveTo(x as u16, y))?;
        if is_chosen {
            stdout.queue(Print(top.clone().green().to_string()))?;
        } else if is_cursor {
            stdout.queue(Print(top.clone().yellow().to_string()))?;
        } else {
            stdout.queue(Print(&top))?;
        }

        // Art lines (5 lines)
        for (ai, art_line) in b.ascii_art.iter().enumerate() {
            let padded = format!("{:^w$}", art_line, w = CARD_W);
            let line = format!("{}{}{}", vt, padded, vt);
            stdout.queue(MoveTo(x as u16, y + 1 + ai as u16))?;
            if !unlocked {
                stdout.queue(Print(line.dark_grey().to_string()))?;
            } else if is_chosen {
                stdout.queue(Print(line.green().to_string()))?;
            } else if is_cursor {
                stdout.queue(Print(line.yellow().to_string()))?;
            } else {
                stdout.queue(Print(&line))?;
            }
        }

        // Name line
        let name_str = format!("{:^w$}", b.name, w = CARD_W);
        let name_line = format!("{}{}{}", vt, name_str, vt);
        stdout.queue(MoveTo(x as u16, y + 6))?;
        if !unlocked {
            stdout.queue(Print(name_line.dark_grey().to_string()))?;
        } else if is_chosen {
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(name_line.green().to_string()))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else if is_cursor {
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(name_line.yellow().to_string()))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        } else {
            stdout.queue(SetAttribute(Attribute::Bold))?;
            stdout.queue(Print(&name_line))?;
            stdout.queue(SetAttribute(Attribute::Reset))?;
        }

        // Tier line
        let tier_label = if unlocked {
            format!("{:^w$}", format!("─ {} Tier ─", b.tier.label()), w = CARD_W)
        } else {
            let needed = blessings::tracks_required(b.tier);
            format!("{:^w$}", format!("Locked ({}+ tracks)", needed), w = CARD_W)
        };
        let tier_line = format!("{}{}{}", vt, tier_label, vt);
        stdout.queue(MoveTo(x as u16, y + 7))?;
        if !unlocked {
            stdout.queue(Print(tier_line.dark_grey().to_string()))?;
        } else {
            stdout.queue(Print(&tier_line))?;
        }

        // Description line
        let desc = format!("{:^w$}", b.description, w = CARD_W);
        let desc_line = format!("{}{}{}", vt, desc, vt);
        stdout.queue(MoveTo(x as u16, y + 8))?;
        if !unlocked {
            stdout.queue(Print(desc_line.dark_grey().to_string()))?;
        } else {
            stdout.queue(Print(&desc_line))?;
        }

        // Bottom border
        let bot = format!("{}{}{}", bl, hz.to_string().repeat(CARD_W), br);
        stdout.queue(MoveTo(x as u16, y + 9))?;
        if is_chosen {
            stdout.queue(Print(bot.green().to_string()))?;
        } else if is_cursor {
            stdout.queue(Print(bot.yellow().to_string()))?;
        } else {
            stdout.queue(Print(&bot))?;
        }

        // Selection marker
        if is_chosen {
            let marker = format!("{:^w$}", "★ SELECTED", w = CARD_W + 2);
            stdout.queue(MoveTo(x as u16, y + 10))?;
            stdout.queue(Print(marker.green().to_string()))?;
        }
    }

    // Status bar
    let status_y = grid_y + (rows as u16) * (CARD_H as u16 + 1) + 1;
    let status = format!(
        "Selected: {}/3    ↑↓←→ Navigate  Enter/Space: Toggle  {}  Esc: Back",
        chosen.len(),
        if chosen.len() == 3 { "C: Confirm" } else { "" },
    );
    let sx = (term_w as usize).saturating_sub(status.len()) / 2;
    stdout.queue(MoveTo(sx as u16, status_y))?;
    if chosen.len() == 3 {
        stdout.queue(Print(status.green().to_string()))?;
    } else {
        stdout.queue(Print(status.dark_grey().to_string()))?;
    }

    stdout.flush()
}
