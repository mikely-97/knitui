use std::io::{self, Stdout, Write};
use std::time::Instant;
use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Attribute, SetAttribute},
    terminal::{self, Clear, ClearType, BeginSynchronizedUpdate, EndSynchronizedUpdate},
    cursor::{MoveTo, Hide},
};
use crate::blessings::{self, ALL_BLESSINGS};
use crate::engine::GameEngine;
use crate::board_entity::BoardEntity;

pub fn render_help(stdout: &mut Stdout, engine: &GameEngine) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    use crossterm::style::{SetForegroundColor, ResetColor};
    use crossterm::style::Color;

    let (term_w, _) = terminal::size().unwrap_or((80, 24));
    let box_w = 52u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);

    // Title box
    stdout.queue(MoveTo(bx, 1))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, 2))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("║{:^w$}║", "═══ KNITUI HELP ═══", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, 3))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    // Two-column keybindings
    let keys: &[(&str, &str)] = &[
        ("← → ↑ ↓",  "Move cursor"),
        ("Enter",     "Pick up spool at cursor"),
        ("H",         "Show this help screen"),
        ("Esc",       "Return to main menu"),
        ("R",         "Restart (from game-over)"),
        ("Z  ✂",      "Scissors: auto-wind spool"),
        ("X  ⊹",      "Tweezers: pick any spool"),
        ("C  ⊛",      "Balloons: lift front patches"),
        ("A",         "Watch ad for +1 scissors"),
        ("?",         "Hint (Scout's Eye blessing)"),
    ];

    let col1_w = 14usize;
    let inner = box_w as usize - 2;
    for (i, (key, desc)) in keys.iter().enumerate() {
        let y = 4 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(Print(format!(" {:<w$}", key, w = col1_w)))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        let remaining = inner - 1 - col1_w - 1;
        stdout.queue(Print(format!("{:<w$}", desc, w = remaining)))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let sep_y = 4 + keys.len() as u16;
    stdout.queue(MoveTo(bx, sep_y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    // Active blessings section
    stdout.queue(MoveTo(bx, sep_y + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("{:^w$}", "Active Blessings", w = inner)))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;

    use crate::blessings::ALL_BLESSINGS;
    let flags = &engine.blessing_flags;
    let active: Vec<(&str, &str)> = ALL_BLESSINGS.iter()
        .filter(|b| match b.id {
            "scouts_eye"      => flags.scouts_eye,
            "wrap_around"     => flags.wrap_around,
            "tidy_workspace"  => flags.tidy_workspace,
            "conveyor_peek"   => flags.conveyor_peek,
            "color_count"     => flags.color_count,
            "match_hint"      => flags.match_hint,
            _                 => false,
        })
        .map(|b| (b.name, b.description))
        .collect();

    if active.is_empty() {
        stdout.queue(MoveTo(bx, sep_y + 2))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(format!("║{:^w$}║", "none", w = inner)))?;
    }
    for (i, (name, desc)) in active.iter().enumerate() {
        let y = sep_y + 2 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(Print(format!(" {:<w$}", name, w = col1_w)))?;
        stdout.queue(SetForegroundColor(Color::White))?;
        let remaining = inner - 1 - col1_w - 1;
        stdout.queue(Print(format!("{:<w$}", desc, w = remaining)))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let bless_rows = active.len().max(1) as u16;
    let sep2_y = sep_y + 2 + bless_rows;
    stdout.queue(MoveTo(bx, sep2_y))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    // Bonus inventory
    stdout.queue(MoveTo(bx, sep2_y + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("{:^w$}", "Bonus Inventory", w = inner)))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print("║"))?;

    let bonuses = [
        ("✂ Scissors", engine.bonuses.scissors),
        ("⊹ Tweezers", engine.bonuses.tweezers),
        ("⊛ Balloons", engine.bonuses.balloons),
    ];
    for (i, (name, count)) in bonuses.iter().enumerate() {
        let y = sep2_y + 2 + i as u16;
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        if *count > 0 {
            stdout.queue(SetForegroundColor(Color::White))?;
        } else {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        }
        stdout.queue(Print(format!("  {:<w$}x{}", name, count, w = col1_w + 1)))?;
        let remaining = inner - 2 - col1_w - 1 - 2;
        stdout.queue(Print(format!("{:>w$}", "", w = remaining)))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
    }

    let end_y = sep2_y + 2 + bonuses.len() as u16;
    stdout.queue(MoveTo(bx, end_y))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╚{}╝", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, end_y + 1))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("{:^w$}", "Press any key to close", w = box_w as usize)))?;
    stdout.queue(ResetColor)?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

/// Render a celebration sweep overlay on the board.
/// `tick` ranges 0..20; column = (tick/2) % cols lights up with ✦.
pub fn render_celebration(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_x: u16,
    board_y: u16,
    scale: u16,
    tick: u8,
) -> io::Result<()> {
    use crossterm::style::{SetForegroundColor, ResetColor, Color};
    let sw = scale * 2;
    let sh = scale;
    let cols = engine.board.width;
    let rows = engine.board.height;
    let lit_col = (tick / 2) as usize % cols as usize;
    let color = if (tick / 2) % 2 == 0 { Color::Yellow } else { Color::Green };

    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    for row in 0..rows {
        for sy in 0..sh {
            let y = board_y + (row as u16) * (sh + 1) + 1 + sy;
            let x = board_x + 1 + (lit_col as u16) * (sw + 1);
            stdout.queue(MoveTo(x, y))?;
            for _ in 0..sw {
                stdout.queue(Print('✦'))?;
            }
        }
    }
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(ResetColor)?;
    Ok(())
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

/// Render the endless mode game-over screen (shown when stuck).
pub fn render_endless_gameover(
    stdout: &mut Stdout,
    wave: usize,
    best_wave: usize,
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let start_y = term_h / 2 - 3;

    let title = "═══ ENDLESS MODE ═══";
    let tx = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(tx, start_y))?;
    stdout.queue(Print(title))?;

    let wave_str = format!("You reached wave {}", wave);
    let wx = (term_w.saturating_sub(wave_str.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(wx, start_y + 2))?;
    stdout.queue(Print(&wave_str))?;

    if wave >= best_wave {
        let record_str = "New record!";
        let rx = (term_w.saturating_sub(record_str.chars().count() as u16)) / 2;
        stdout.queue(MoveTo(rx, start_y + 3))?;
        stdout.queue(Print(record_str))?;
    } else {
        let best_str = format!("Best: wave {}", best_wave);
        let bx = (term_w.saturating_sub(best_str.chars().count() as u16)) / 2;
        stdout.queue(MoveTo(bx, start_y + 3))?;
        stdout.queue(Print(best_str.dark_grey()))?;
    }

    let hint = "R:Play Again  M:Menu  Q:Quit";
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
    // Held spool counter
    let held = engine.held_spools.len() as u16;
    let limit = engine.spool_limit as u16;
    stdout.queue(Print("  "))?;
    let counter_str = format!("⊞ {}/{}", held, limit);
    if held >= limit.saturating_sub(1) {
        stdout.queue(Print(counter_str.red()))?;
    } else if held >= limit.saturating_sub(2) {
        stdout.queue(Print(counter_str.yellow()))?;
    } else {
        stdout.queue(Print(counter_str.white()))?;
    }
    // Color count blessing: show remaining spools per color
    if engine.blessing_flags.color_count {
        stdout.queue(MoveTo(x, y + 1))?;
        render_color_counts(stdout, engine)?;
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
    // Held spool counter
    let held = engine.held_spools.len() as u16;
    let limit = engine.spool_limit as u16;
    let mut row = bonuses.len() as u16;
    stdout.queue(MoveTo(x, y + row))?;
    let counter_str = format!("⊞ {}/{}", held, limit);
    if held >= limit.saturating_sub(1) {
        stdout.queue(Print(counter_str.red()))?;
    } else if held >= limit.saturating_sub(2) {
        stdout.queue(Print(counter_str.yellow()))?;
    } else {
        stdout.queue(Print(counter_str.white()))?;
    }
    // Color count blessing: show remaining spools per color
    if engine.blessing_flags.color_count {
        row += 1;
        stdout.queue(MoveTo(x, y + row))?;
        render_color_counts(stdout, engine)?;
    }
    Ok(())
}

fn render_color_counts(stdout: &mut Stdout, engine: &GameEngine) -> io::Result<()> {
    use std::collections::HashMap;
    let mut counts: HashMap<crossterm::style::Color, u16> = HashMap::new();
    for row in &engine.board.board {
        for cell in row {
            match cell {
                BoardEntity::Spool(c) | BoardEntity::KeySpool(c) => {
                    *counts.entry(*c).or_insert(0) += 1;
                }
                _ => {}
            }
        }
    }
    if counts.is_empty() {
        return Ok(());
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    for (i, (color, count)) in pairs.iter().enumerate() {
        if i > 0 { stdout.queue(Print(" "))?; }
        stdout.queue(Print(format!("{}", count).with(*color)))?;
    }
    Ok(())
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

// ── Blessing selection screen ─────────────────────────────────────────────

const CARD_W: usize = 17;  // inner width of card
const CARD_H: usize = 11;  // total rows per card (border + 5 art + name + tier + desc + border)
const CARD_COLS: usize = 3;

pub fn render_blessing_selection(
    stdout: &mut Stdout,
    cursor: usize,
    chosen: &[usize],
    completed_tracks: usize,
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let total_blessings = ALL_BLESSINGS.len();
    let rows = (total_blessings + CARD_COLS - 1) / CARD_COLS;

    // Title
    let title = "═══ CHOOSE 3 BLESSINGS ═══";
    let title_x = term_w.saturating_sub(title.len() as u16) / 2;
    stdout.queue(MoveTo(title_x, 0))?;
    stdout.queue(Print(title))?;

    // Grid origin
    let grid_w = (CARD_W + 3) * CARD_COLS + 1; // card + gap
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

        // Pick border chars + color
        let (tl, tr, bl, br, hz, vt) = if is_chosen {
            ('╔', '╗', '╚', '╝', '═', '║')
        } else if is_cursor {
            ('┌', '┐', '└', '┘', '─', '│')
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

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}
