use std::io::{self, Stdout};

use crossterm::{
    cursor::MoveTo,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor},
    QueueableCommand,
};

use crate::board::Cell;
use crate::engine::{GameEngine, GameStatus};
use crate::glyphs;
use crate::order::OrderType;
use super::LayoutGeometry;

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

            // Follow-up indicator
            if order.follow_up.is_some() {
                stdout.queue(MoveTo(x, *y))?;
                stdout.queue(SetForegroundColor(Color::DarkGrey))?;
                stdout.queue(Print("│"))?;
                stdout.queue(SetForegroundColor(Color::Cyan))?;
                let fu_line = format!("{:<w$}", " → more", w = inner_w);
                stdout.queue(Print(fu_line))?;
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
        ("U", "Upgrade gen"),
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
    use crossterm::terminal::size as term_size;
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
