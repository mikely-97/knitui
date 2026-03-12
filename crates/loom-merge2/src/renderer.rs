use std::io::{self, Stdout};

use crossterm::{
    cursor::MoveTo,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor},
    terminal::size as term_size,
    QueueableCommand,
};

use crate::blessings::{self, ALL_BLESSINGS};
use crate::board::Cell;
use crate::engine::{GameEngine, GameStatus};
use crate::glyphs::{self, cell_dims};
use crate::order::OrderType;

// ── Layout ────────────────────────────────────────────────────────────────

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
        let (cw, _) = cell_dims(scale);
        let board_w = (engine.board.cols * (cw + 1) + 1) as u16;
        let (term_w, _) = term_size().unwrap_or((80, 24));
        let order_panel_w = 24u16;

        if term_w >= board_w + 3 + order_panel_w {
            LayoutGeometry {
                layout: Layout::Horizontal,
                board_x: 1,
                board_y: 2,
                order_x: board_w + 3,
                order_y: 2,
                scale,
            }
        } else {
            // vertical: orders above board
            let order_rows = 10u16;
            LayoutGeometry {
                layout: Layout::Vertical,
                board_x: 1,
                board_y: 2 + order_rows,
                order_x: 1,
                order_y: 2,
                scale,
            }
        }
    }

    /// Y coordinate of the row below the board (for inventory strip).
    pub fn inventory_y(&self, engine: &GameEngine) -> u16 {
        let (_, ch) = cell_dims(self.scale);
        let board_h = (engine.board.rows * (ch + 1) + 1) as u16;
        self.board_y + board_h
    }

    /// Y coordinate of the key bar (bottom hint line).
    pub fn key_bar_y(&self, engine: &GameEngine) -> u16 {
        self.inventory_y(engine) + 1
    }
}

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

// ── HUD (score / energy / stars) ─────────────────────────────────────────

pub fn render_hud(
    stdout: &mut Stdout,
    engine: &GameEngine,
    label: &str,
) -> io::Result<()> {
    // Row 0: game label
    stdout.queue(MoveTo(1, 0))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(label))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    // Row 1: Score  ⚡NN/NN [bar] +Xs  ★NN
    stdout.queue(MoveTo(1, 1))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("Score: {:>7}", engine.score)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    // Energy bar
    let e = &engine.energy;
    let bar_total = 10usize;
    let filled = if e.max > 0 {
        ((e.current as usize * bar_total) / e.max as usize).min(bar_total)
    } else {
        bar_total
    };
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_total - filled);
    let secs = e.secs_until_next();
    let regen_str = if e.is_full() {
        "  full ".to_string()
    } else {
        format!(" +{}s ", secs)
    };

    stdout.queue(Print("  "))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("⚡{}/{}", e.current, e.max)))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!(" [{}]", bar)))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(&regen_str))?;

    // Stars
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print(format!(" ★{}", engine.stars)))?;

    // Ad hint
    if engine.can_watch_ad() {
        let next_reward = crate::ad::reward_for_use(engine.ads_used, &engine.available_families);
        stdout.queue(SetForegroundColor(Color::Magenta))?;
        stdout.queue(Print(format!("  [A] {}", crate::ad::hud_label(&next_reward)
            .trim_start_matches("[AD] ")
            .trim_end_matches(" — press A"))))?;
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

/// Thin wrapper kept for backward compatibility with tui.rs during transition.
pub fn render_score(stdout: &mut Stdout, engine: &GameEngine) -> io::Result<()> {
    render_hud(stdout, engine, "Merge-2")
}

// ── Orders panel ─────────────────────────────────────────────────────────

pub fn render_orders(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) -> io::Result<()> {
    let x = geo.order_x;
    let mut y = geo.order_y;

    let story: Vec<_>  = engine.active_orders.iter()
        .filter(|o| matches!(o.order_type, OrderType::Story)).collect();
    let random: Vec<_> = engine.active_orders.iter()
        .filter(|o| matches!(o.order_type, OrderType::Random)).collect();
    let timed: Vec<_>  = engine.active_orders.iter()
        .filter(|o| matches!(o.order_type, OrderType::TimeLimited { .. })).collect();

    let panel_w = 22usize;
    let inner_w = panel_w - 2;

    let render_section = |stdout: &mut Stdout, label: &str, orders: &[&&crate::order::Order], y: &mut u16| -> io::Result<()> {
        if orders.is_empty() { return Ok(()); }

        stdout.queue(MoveTo(x, *y))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(SetAttribute(Attribute::Bold))?;
        stdout.queue(Print(label))?;
        stdout.queue(SetAttribute(Attribute::Reset))?;
        *y += 1;

        stdout.queue(MoveTo(x, *y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(format!("┌{}┐", "─".repeat(inner_w))))?;
        *y += 1;

        for order in orders.iter() {
            // Timed countdown
            if let OrderType::TimeLimited { ticks_remaining } = &order.order_type {
                stdout.queue(MoveTo(x, *y))?;
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                let secs = ticks_remaining / 5; // 200ms ticks
                stdout.queue(Print(format!("│ ⏱ {:>3}s{}", secs, " ".repeat(inner_w.saturating_sub(8)))))?;
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                *y += 1;
            }

            for req in &order.requirements {
                stdout.queue(MoveTo(x, *y))?;
                let done = req.delivered;
                let need = req.required;
                let name = req.family.tier_name(req.tier);
                let fam_color = glyphs::family_color(req.family);

                let progress = format!("[{}/{}]", done, need);
                let glyph = req.family.glyph(req.tier);
                let desc = format!("{}×{} {}", need - done, glyph, name);
                let line = format!(" {:<13}{:>5} ", desc, progress);
                let line = if line.chars().count() > inner_w {
                    line.chars().take(inner_w).collect::<String>()
                } else {
                    format!("{:<w$}", line, w = inner_w)
                };

                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                if done >= need {
                    stdout.queue(SetForegroundColor(Color::Green))?;
                } else {
                    stdout.queue(SetForegroundColor(fam_color))?;
                }
                stdout.queue(Print(&line))?;
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                *y += 1;
            }

            // Rewards preview
            let reward_str: String = order.rewards.iter().map(|r| {
                use crate::order::Reward;
                match r {
                    Reward::Score(n)       => format!("+{}pts ", n),
                    Reward::Energy(n)      => format!("+{}⚡ ", n),
                    Reward::Stars(n)       => format!("+{}★ ", n),
                    Reward::InventorySlot  => "+inv ".to_string(),
                    Reward::SpawnPiece(_)  => "+item ".to_string(),
                }
            }).collect();
            if !reward_str.is_empty() {
                stdout.queue(MoveTo(x, *y))?;
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                stdout.queue(SetForegroundColor(Color::Yellow))?;
                let rs_raw = format!(" {}", reward_str.trim());
                let rs = if rs_raw.chars().count() >= inner_w {
                    rs_raw.chars().take(inner_w).collect::<String>()
                } else {
                    format!("{:<w$}", rs_raw, w = inner_w)
                };
                stdout.queue(Print(format!("{:<w$}", rs, w = inner_w)))?;
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                *y += 1;
            }
        }

        stdout.queue(MoveTo(x, *y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(format!("└{}┘", "─".repeat(inner_w))))?;
        *y += 1;

        stdout.queue(ResetColor)?;
        Ok(())
    };

    if !story.is_empty() {
        render_section(stdout, "STORY ORDERS", &story.iter().collect::<Vec<_>>().as_slice(), &mut y)?;
    }
    if !random.is_empty() {
        render_section(stdout, "ORDERS      ", &random.iter().collect::<Vec<_>>().as_slice(), &mut y)?;
    }
    if !timed.is_empty() {
        render_section(stdout, "TIMED       ", &timed.iter().collect::<Vec<_>>().as_slice(), &mut y)?;
    }

    Ok(())
}

// ── Inventory strip ───────────────────────────────────────────────────────

pub fn render_inventory(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    selected_slot: Option<usize>,
) -> io::Result<()> {
    let y = geo.inventory_y(engine);
    let x = geo.board_x;

    stdout.queue(MoveTo(x, y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    let used = engine.inventory.used_count();
    let total = engine.inventory.slot_count();
    stdout.queue(Print(format!("Inv [{}/{}]: ", used, total)))?;

    for (i, slot) in engine.inventory.slots.iter().enumerate() {
        let is_sel = selected_slot == Some(i);
        if is_sel {
            stdout.queue(SetBackgroundColor(Color::Rgb { r: 0, g: 60, b: 0 }))?;
        }
        match slot {
            None => {
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("[   ]"))?;
            }
            Some(piece) => {
                let (label, color, _) = glyphs::cell_label(&Cell::Piece(piece.clone()));
                stdout.queue(SetForegroundColor(color))?;
                stdout.queue(Print(format!("[{:<3}]", label)))?;
            }
        }
        stdout.queue(ResetColor)?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(" "))?;
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Key bar ───────────────────────────────────────────────────────────────

pub fn render_key_bar(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) -> io::Result<()> {
    let y = geo.key_bar_y(engine);
    stdout.queue(MoveTo(1, y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;

    let hints = [
        ("↑↓←→", "Move"),
        ("Enter", "Select/Merge"),
        ("D", "Deliver"),
        ("S", "Store"),
        ("I", "Inventory"),
        ("A", "Ad"),
        ("H", "Help"),
        ("Q", "Quit"),
    ];
    let line: String = hints.iter()
        .map(|(k, v)| format!("{} {} ", k, v))
        .collect();
    stdout.queue(Print(&line))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Game over overlay ─────────────────────────────────────────────────────

pub fn render_game_over(
    stdout: &mut Stdout,
    status: &GameStatus,
    score: u32,
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let cx = term_w / 2;
    let cy = term_h / 2;
    let box_w = 26u16;
    let bx = cx.saturating_sub(box_w / 2);
    let by = cy.saturating_sub(3);

    let (title, color) = match status {
        GameStatus::Won    => ("MISSION COMPLETE!", Color::Green),
        GameStatus::Lost   => ("BOARD FULL!",       Color::Red),
        GameStatus::Stuck  => ("STUCK!",             Color::Yellow),
        GameStatus::Playing => return Ok(()),
    };

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    let msg = format!("{:^w$}", title, w = box_w as usize - 2);
    stdout.queue(Print(format!("║{}║", msg)))?;

    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(SetForegroundColor(color))?;
    let sc = format!("Score: {:>8}", score);
    stdout.queue(Print(format!("║{:^w$}║", sc, w = box_w as usize - 2)))?;

    stdout.queue(MoveTo(bx, by + 3))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("║{:^w$}║", "Enter: menu  Q: quit", w = box_w as usize - 2)))?;

    stdout.queue(MoveTo(bx, by + 4))?;
    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Help overlay ──────────────────────────────────────────────────────────

pub fn render_help(
    stdout: &mut Stdout,
    help_lines: &[(&str, &str)],
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 38u16;
    let box_h = help_lines.len() as u16 + 4;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(box_h / 2);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("║{:^w$}║", "HELP", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    for (i, (key, desc)) in help_lines.iter().enumerate() {
        let y = by + 3 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(Print(format!(" {:<12}", key)))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        let remaining = box_w as usize - 2 - 13;
        stdout.queue(Print(format!("{:<w$}", desc, w = remaining)))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let ey = by + 3 + help_lines.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;

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
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let cx = term_w / 2;
    let cy = term_h / 2;
    let box_w = 28u16;
    let bx = cx.saturating_sub(box_w / 2);
    let by = cy.saturating_sub((items.len() as u16 + 4) / 2);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "MERGE-2", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;

    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    for (i, item) in items.iter().enumerate() {
        let y = by + 3 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Black))?;
            stdout.queue(SetBackgroundColor(Color::Cyan))?;
            stdout.queue(Print(format!("║ ▶ {:<w$}║", item, w = box_w as usize - 5)))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(SetBackgroundColor(Color::Reset))?;
            stdout.queue(Print(format!("║   {:<w$}║", item, w = box_w as usize - 5)))?;
        }
        stdout.queue(ResetColor)?;
    }

    let ey = by + 3 + items.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;

    if let Some(msg) = flash {
        stdout.queue(MoveTo(bx, ey + 1))?;
        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(Print(format!(" {}", msg)))?;
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Campaign track select ─────────────────────────────────────────────────

pub fn render_campaign_select(
    stdout: &mut Stdout,
    tracks: &[&str],
    progress: &[String],
    selected: usize,
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 34u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub((tracks.len() as u16 + 4) / 2);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "SELECT TRACK", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    for (i, (name, prog)) in tracks.iter().zip(progress.iter()).enumerate() {
        let y = by + 3 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Black))?;
            stdout.queue(SetBackgroundColor(Color::Green))?;
            stdout.queue(Print(format!("║ ▶ {:<18}{:>8}║", name, prog)))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(SetBackgroundColor(Color::Reset))?;
            stdout.queue(Print(format!("║   {:<18}{:>8}║", name, prog)))?;
        }
        stdout.queue(ResetColor)?;
    }

    let ey = by + 3 + tracks.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::Green))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Campaign level intro ──────────────────────────────────────────────────

pub fn render_level_intro(
    stdout: &mut Stdout,
    lines: &[String],
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 44u16;
    let box_h = lines.len() as u16 + 4;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(box_h / 2);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "MISSION START", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    for (i, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(bx, by + 3 + i as u16))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(Print(format!("║ {:<w$}║", line, w = box_w as usize - 3)))?;
    }

    let ey = by + 3 + lines.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, ey + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("  Press Enter to begin"))?;

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Ad watching overlay ───────────────────────────────────────────────────

pub fn render_ad_overlay(
    stdout: &mut Stdout,
    quote: &str,
    elapsed_secs: u64,
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 44u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = term_h / 4;
    let ad_dur = 10u64;
    let remaining = ad_dur.saturating_sub(elapsed_secs);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Magenta))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "★  SPONSORED MESSAGE  ★", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    // Word-wrap the quote into the box
    let inner = box_w as usize - 4;
    let words: Vec<&str> = quote.split_whitespace().collect();
    let mut line = String::new();
    let mut rows: Vec<String> = Vec::new();
    for word in words {
        if line.len() + word.len() + 1 > inner {
            rows.push(line.clone());
            line = word.to_string();
        } else {
            if !line.is_empty() { line.push(' '); }
            line.push_str(word);
        }
    }
    rows.push(line);

    for (i, row) in rows.iter().enumerate() {
        stdout.queue(MoveTo(bx, by + 3 + i as u16))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        stdout.queue(Print(format!("║  {:<w$}  ║", row, w = inner)))?;
    }

    let ey = by + 3 + rows.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("║{:^w$}║", format!("{}s remaining…", remaining), w = box_w as usize - 2)))?;
    stdout.queue(MoveTo(bx, ey + 1))?;
    stdout.queue(SetForegroundColor(Color::Magenta))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Options screen ────────────────────────────────────────────────────────

pub fn render_options(
    stdout: &mut Stdout,
    settings: &loom_engine::settings::UserSettings,
    selected: usize,
) -> io::Result<()> {
    let (term_w, _) = term_size().unwrap_or((80, 24));
    let cx = term_w / 2;
    let box_w = 32u16;
    let bx = cx.saturating_sub(box_w / 2);

    stdout.queue(MoveTo(bx, 3))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, 4))?;
    stdout.queue(Print(format!("║{:^w$}║", "OPTIONS", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, 5))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    let rows: &[(&str, String)] = &[
        ("Color mode", settings.color_mode.clone()),
        ("Scale",      settings.scale.to_string()),
    ];
    for (i, (name, val)) in rows.iter().enumerate() {
        let y = 6 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Black))?;
            stdout.queue(SetBackgroundColor(Color::Cyan))?;
            stdout.queue(Print(format!("║ ▶ {:<14} {:>9}║", name, val)))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(SetBackgroundColor(Color::Reset))?;
            stdout.queue(Print(format!("║   {:<14} {:>9}║", name, val)))?;
        }
        stdout.queue(ResetColor)?;
    }

    stdout.queue(MoveTo(bx, 6 + rows.len() as u16))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, 7 + rows.len() as u16))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("  ←→ Adjust  Esc: Back"))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Custom game ───────────────────────────────────────────────────────────

pub fn render_custom_game(
    stdout: &mut Stdout,
    config: &crate::config::Config,
    preset_name: &str,
    selected: usize,
) -> io::Result<()> {
    let (term_w, _) = term_size().unwrap_or((80, 24));
    let box_w = 36u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);

    stdout.queue(MoveTo(bx, 2))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, 3))?;
    stdout.queue(Print(format!("║{:^w$}║", "CUSTOM GAME", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, 4))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    let fields: &[(&str, String)] = &[
        ("Preset",          preset_name.to_string()),
        ("Rows",            config.board_rows.to_string()),
        ("Cols",            config.board_cols.to_string()),
        ("Scale",           config.scale.to_string()),
        ("Energy max",      config.energy_max.to_string()),
        ("Regen (secs)",    config.energy_regen_secs.to_string()),
        ("Gen cost",        config.generator_cost.to_string()),
        ("Families",        config.family_count.to_string()),
        ("Orders",          config.random_order_count.to_string()),
        ("Max order tier",  config.max_order_tier.to_string()),
        ("Inventory slots", config.inventory_slots.to_string()),
    ];

    for (i, (name, val)) in fields.iter().enumerate() {
        let y = 5 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        if i == selected {
            stdout.queue(SetForegroundColor(Color::Black))?;
            stdout.queue(SetBackgroundColor(Color::Cyan))?;
            stdout.queue(Print(format!("║ ▶ {:<18} {:>11}║", name, val)))?;
        } else {
            stdout.queue(SetForegroundColor(Color::White))?;
            stdout.queue(SetBackgroundColor(Color::Reset))?;
            stdout.queue(Print(format!("║   {:<18} {:>11}║", name, val)))?;
        }
        stdout.queue(ResetColor)?;
    }

    let ey = 5 + fields.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, ey + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("  ↑↓ Select  ←→ Adjust  Enter Start  Esc Back"))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

// ── Blessing selection ────────────────────────────────────────────────────

pub fn render_blessing_selection(
    stdout: &mut Stdout,
    cursor: usize,
    chosen: &[usize],
    completed_tracks: usize,
) -> io::Result<()> {
    let (term_w, _) = term_size().unwrap_or((80, 24));
    let box_w = 50u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);

    stdout.queue(MoveTo(bx, 1))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, 2))?;
    stdout.queue(Print(format!("║{:^w$}║", "SELECT BLESSINGS", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, 3))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    for (i, blessing) in ALL_BLESSINGS.iter().enumerate() {
        let y = 4 + i as u16;
        let is_cursor  = i == cursor;
        let is_chosen  = chosen.contains(&i);
        let is_locked  = !blessings::is_unlocked(blessing, completed_tracks);
        let req_tracks = blessings::tracks_required(blessing.tier);

        stdout.queue(MoveTo(bx, y))?;
        let (fg, bg) = if is_cursor && !is_locked {
            (Color::Black, Color::Yellow)
        } else if is_chosen {
            (Color::Green, Color::Reset)
        } else if is_locked {
            (Color::DarkGrey, Color::Reset)
        } else {
            (Color::White, Color::Reset)
        };
        stdout.queue(SetForegroundColor(fg))?;
        stdout.queue(SetBackgroundColor(bg))?;
        let tier_s = format!("[{}]", blessing.tier.label());
        let lock_s = if is_locked { "*" } else if is_chosen { "v" } else { " " };
        stdout.queue(Print(format!("║{} {:<4} {:<26} {:>14}║",
            lock_s, tier_s, blessing.name,
            format!("{} tracks", req_tracks))))?;
        stdout.queue(ResetColor)?;
    }

    let ey = 4 + ALL_BLESSINGS.len() as u16;
    stdout.queue(MoveTo(bx, ey))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, ey + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("  ↑↓ Move  Enter Toggle  Space Start  Esc Back"))?;
    stdout.queue(ResetColor)?;
    Ok(())
}
