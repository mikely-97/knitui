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
