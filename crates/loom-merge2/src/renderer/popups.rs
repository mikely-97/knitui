use std::io::{self, Stdout};

use crossterm::{
    cursor::MoveTo,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor, ResetColor},
    terminal::size as term_size,
    QueueableCommand,
};

use crate::blessings::{self, ALL_BLESSINGS};

// ── Help overlay ──────────────────────────────────────────────────────────

pub fn render_help(
    stdout: &mut Stdout,
    help_lines: &[(&str, &str)],
    engine: Option<&crate::engine::GameEngine>,
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 46u16;
    let col1_w = 14usize;
    let inner = box_w as usize - 2;
    // Derive active blessing names from engine's blessing_flags (if engine provided)
    let bless_active: Vec<(&str, &str)> = if let Some(eng) = engine {
        let flags = &eng.blessing_flags;
        crate::blessings::ALL_BLESSINGS.iter()
            .filter(|b| match b.id {
                "energy_saver"    => flags.energy_saver,
                "quick_regen"     => flags.quick_regen,
                "keen_eye"        => flags.keen_eye,
                "bigger_pockets"  => flags.bigger_pockets,
                "thaw_aura"       => flags.thaw_aura,
                "lucky_orders"    => flags.lucky_orders,
                "chain_merge"     => flags.chain_merge,
                "tier_boost"      => flags.tier_boost,
                "generator_surge" => flags.generator_surge,
                "double_deliver"  => flags.double_deliver,
                "soft_gen_master" => flags.soft_gen_master,
                "deep_thaw"       => flags.deep_thaw,
                _ => false,
            })
            .map(|b| (b.name, b.description))
            .collect()
    } else {
        vec![]
    };
    let bless_rows = bless_active.len().max(1) as u16;
    // inventory items in engine
    let inv_count = engine.map(|e| e.inventory.slot_count()).unwrap_or(0) as u16;
    let box_h = help_lines.len() as u16 + 4 + 2 + bless_rows + 2 + inv_count.max(1) + 3;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(box_h / 2);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("║{:^w$}║", "MERGE-2 HELP", w = inner)))?;
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
        stdout.queue(Print(format!(" {:<w$}", key, w = col1_w)))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        let remaining = inner.saturating_sub(1 + col1_w + 1);
        stdout.queue(Print(format!("{:<w$}", desc, w = remaining)))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let sep_y = by + 3 + help_lines.len() as u16;
    stdout.queue(MoveTo(bx, sep_y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    // Active blessings
    stdout.queue(MoveTo(bx, sep_y + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("{:^w$}", "Active Blessings", w = inner)))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;

    if bless_active.is_empty() {
        stdout.queue(MoveTo(bx, sep_y + 2))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(format!("║{:^w$}║", "none", w = inner)))?;
    }
    for (i, (name, desc)) in bless_active.iter().enumerate() {
        let y = sep_y + 2 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(Print(format!(" {:<w$}", name, w = col1_w)))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        let remaining = inner.saturating_sub(1 + col1_w + 1);
        stdout.queue(Print(format!("{:<w$}", desc, w = remaining)))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let sep2_y = sep_y + 2 + bless_rows;
    stdout.queue(MoveTo(bx, sep2_y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    // Bonus inventory
    stdout.queue(MoveTo(bx, sep2_y + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("{:^w$}", "Inventory", w = inner)))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;

    let slots = engine.map(|e| e.inventory.slot_count()).unwrap_or(0);
    if slots == 0 {
        stdout.queue(MoveTo(bx, sep2_y + 2))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(format!("║{:^w$}║", "empty", w = inner)))?;
    }
    for i in 0..slots {
        let y = sep2_y + 2 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        if let Some(piece) = engine.and_then(|e| e.inventory.peek(i)) {
            stdout.queue(SetForegroundColor(Color::White))?;
            let piece_name = match piece {
                crate::item::Piece::Regular(item) => format!("{} T{}", item.family.name(), item.tier),
                crate::item::Piece::Blueprint(fam) => format!("Blueprint({})", fam.name()),
            };
            let label = format!("  Slot {}: {}", i + 1, piece_name);
            stdout.queue(Print(format!("{:<w$}", label, w = inner)))?;
        } else {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
            stdout.queue(Print(format!("{:<w$}", format!("  Slot {}: (empty)", i + 1), w = inner)))?;
        }
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let end_y = sep2_y + 2 + slots.max(1) as u16;
    stdout.queue(MoveTo(bx, end_y))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, end_y + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("{:^w$}", "Press any key to close", w = box_w as usize)))?;

    stdout.queue(ResetColor)?;
    Ok(())
}

/// Render a celebration sweep overlay on the merge board.
pub fn render_celebration(
    stdout: &mut Stdout,
    engine: &crate::engine::GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) -> io::Result<()> {
    let (cw, ch) = crate::glyphs::cell_dims(geo.scale);
    let cols = engine.board.cols;
    let rows = engine.board.rows;
    let lit_col = (tick / 2) as usize % cols.max(1);
    let color = if (tick / 2) % 2 == 0 { Color::Yellow } else { Color::Green };

    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    for row in 0..rows {
        for sy in 0..ch {
            let y = geo.board_y + (row as u16) * (ch as u16 + 1) + 1 + sy as u16;
            let x = geo.board_x + 1 + (lit_col as u16) * (cw as u16 + 1);
            stdout.queue(MoveTo(x, y))?;
            for _ in 0..cw {
                stdout.queue(Print('✦'))?;
            }
        }
    }
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;
    Ok(())
}

/// Render the mission-complete score summary screen.
pub fn render_mission_summary(
    stdout: &mut Stdout,
    ctx: &crate::campaign::CampaignState,
) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 44u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = term_h / 4;
    let inner = box_w as usize - 2;

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "MISSION COMPLETE!", w = inner)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    let track = crate::campaign_levels::TRACK_NAMES.get(ctx.track_idx).copied().unwrap_or("Unknown");
    stdout.queue(MoveTo(bx, by + 3))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    stdout.queue(Print(format!("║ {:<w$}║",
        format!("{} — Mission {}/{}", track, ctx.current_mission + 1, ctx.total_missions()),
        w = inner - 1)))?;

    stdout.queue(MoveTo(bx, by + 4))?;
    stdout.queue(Print(format!("║ {:<w$}║",
        format!("Score: {}  Stars: ★{}", ctx.score, ctx.stars),
        w = inner - 1)))?;

    stdout.queue(MoveTo(bx, by + 5))?;
    stdout.queue(Print(format!("║ {:<w$}║",
        format!("Total merges: {}  Thawed: {}", ctx.total_merges, ctx.cells_thawed),
        w = inner - 1)))?;

    stdout.queue(MoveTo(bx, by + 6))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 7))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("║{:^w$}║", "Inventory Carried Over", w = inner)))?;

    let slots = ctx.inventory.slot_count();
    let mut row_off = 8u16;
    let mut any = false;
    for i in 0..slots {
        if let Some(piece) = ctx.inventory.peek(i) {
            stdout.queue(MoveTo(bx, by + row_off))?;
            stdout.queue(SetForegroundColor(Color::White))?;
            let piece_name = match piece {
                crate::item::Piece::Regular(item) => format!("{} T{}", item.family.name(), item.tier),
                crate::item::Piece::Blueprint(fam) => format!("Blueprint({})", fam.name()),
            };
            let s = format!("  {}", piece_name);
            stdout.queue(Print(format!("║{:<w$}║", s, w = inner)))?;
            row_off += 1;
            any = true;
        }
    }
    if !any {
        stdout.queue(MoveTo(bx, by + row_off))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(format!("║{:^w$}║", "none", w = inner)))?;
        row_off += 1;
    }

    stdout.queue(MoveTo(bx, by + row_off))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + row_off + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("{:^w$}", "Press Enter to continue", w = box_w as usize)))?;
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

// ── Inventory expansion popup ─────────────────────────────────────────────

pub fn render_inv_expansion_popup(stdout: &mut Stdout) -> io::Result<()> {
    let (term_w, term_h) = term_size().unwrap_or((80, 24));
    let box_w = 32u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(3);

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 1))?;
    let title = format!("{:^w$}", "INVENTORY EXPANDED!", w = box_w as usize - 2);
    stdout.queue(Print(format!("║{}║", title)))?;

    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    let msg = format!("{:^w$}", "+1 inventory slot unlocked", w = box_w as usize - 2);
    stdout.queue(Print(format!("║{}║", msg)))?;

    stdout.queue(MoveTo(bx, by + 3))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    let hint = format!("{:^w$}", "Y Accept  N Decline", w = box_w as usize - 2);
    stdout.queue(Print(format!("║{}║", hint)))?;

    stdout.queue(MoveTo(bx, by + 4))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(ResetColor)?;
    Ok(())
}
