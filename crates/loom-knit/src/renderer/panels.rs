use std::io::{self, Stdout};
use std::time::Instant;
use loom_engine::render::{Attrs, Color, Style, Surface};
use loom_engine_term::TermSurface;
use crate::blessings::{self, ALL_BLESSINGS};
use crate::engine::GameEngine;
use crate::board_entity::BoardEntity;

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

fn bold() -> Style {
    Style { attrs: Attrs::BOLD, ..Default::default() }
}

pub fn render_help(stdout: &mut Stdout, engine: &GameEngine) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_help_inner(&mut surface, engine);
    surface.finish()
}

fn render_help_inner(surface: &mut dyn Surface, engine: &GameEngine) {
    let (term_w, _) = surface.size();
    let box_w = 52u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let cyan = fg(Color::Cyan);
    let grey = fg(Color::DarkGrey);
    let yellow = fg(Color::Yellow);
    let white = fg(Color::White);
    let green = fg(Color::Green);

    // Title box
    surface.print(bx, 1, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), cyan);
    surface.print(bx, 2, &format!("║{:^w$}║", "═══ KNITUI HELP ═══", w = box_w as usize - 2), bold());
    surface.print(bx, 3, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), grey);

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
        let mut x = bx;
        surface.print(x, y, "║", grey); x += 1;
        let key_field = format!(" {:<w$}", key, w = col1_w);
        let key_w = key_field.chars().count() as u16;
        surface.print(x, y, &key_field, yellow); x += key_w;
        let remaining = inner - 1 - col1_w - 1;
        let desc_field = format!("{:<w$}", desc, w = remaining);
        let desc_w = desc_field.chars().count() as u16;
        surface.print(x, y, &desc_field, white); x += desc_w;
        surface.print(x, y, "║", grey);
    }

    let sep_y = 4 + keys.len() as u16;
    surface.print(bx, sep_y, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), grey);

    // Active blessings section
    surface.print(bx, sep_y + 1, "║", grey);
    surface.print(bx + 1, sep_y + 1, &format!("{:^w$}", "Active Blessings", w = inner), cyan);
    surface.print(bx + 1 + inner as u16, sep_y + 1, "║", grey);

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
        surface.print(bx, sep_y + 2, &format!("║{:^w$}║", "none", w = inner), grey);
    }
    for (i, (name, desc)) in active.iter().enumerate() {
        let y = sep_y + 2 + i as u16;
        let mut x = bx;
        surface.print(x, y, "║", grey); x += 1;
        let name_field = format!(" {:<w$}", name, w = col1_w);
        let name_w = name_field.chars().count() as u16;
        surface.print(x, y, &name_field, green); x += name_w;
        let remaining = inner - 1 - col1_w - 1;
        let desc_field = format!("{:<w$}", desc, w = remaining);
        let desc_w = desc_field.chars().count() as u16;
        surface.print(x, y, &desc_field, white); x += desc_w;
        surface.print(x, y, "║", grey);
    }

    let bless_rows = active.len().max(1) as u16;
    let sep2_y = sep_y + 2 + bless_rows;
    surface.print(bx, sep2_y, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), grey);

    // Bonus inventory
    surface.print(bx, sep2_y + 1, "║", grey);
    surface.print(bx + 1, sep2_y + 1, &format!("{:^w$}", "Bonus Inventory", w = inner), cyan);
    surface.print(bx + 1 + inner as u16, sep2_y + 1, "║", grey);

    let bonuses = [
        ("✂ Scissors", engine.bonuses.scissors),
        ("⊹ Tweezers", engine.bonuses.tweezers),
        ("⊛ Balloons", engine.bonuses.balloons),
    ];
    for (i, (name, count)) in bonuses.iter().enumerate() {
        let y = sep2_y + 2 + i as u16;
        let mut x = bx;
        surface.print(x, y, "║", grey); x += 1;
        let count_style = if *count > 0 { white } else { grey };
        let name_field = format!("  {:<w$}x{}", name, count, w = col1_w + 1);
        let name_w = name_field.chars().count() as u16;
        surface.print(x, y, &name_field, count_style); x += name_w;
        let remaining = inner - 2 - col1_w - 1 - 2;
        let pad = format!("{:>w$}", "", w = remaining);
        let pad_w = pad.chars().count() as u16;
        surface.print(x, y, &pad, count_style); x += pad_w;
        surface.print(x, y, "║", grey);
    }

    let end_y = sep2_y + 2 + bonuses.len() as u16;
    surface.print(bx, end_y, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), cyan);
    surface.print(bx, end_y + 1, &format!("{:^w$}", "Press any key to close", w = box_w as usize), grey);
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
    let mut surface = TermSurface::new(stdout);
    render_celebration_inner(&mut surface, engine, board_x, board_y, scale, tick);
    surface.done()
}

fn render_celebration_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    board_x: u16,
    board_y: u16,
    scale: u16,
    tick: u8,
) {
    let sw = scale * 2;
    let sh = scale;
    let cols = engine.board.width;
    let rows = engine.board.height;
    let lit_col = (tick / 2) as usize % cols as usize;
    let color = if (tick / 2) % 2 == 0 { Color::Yellow } else { Color::Green };
    let style = Style { fg: color, attrs: Attrs::BOLD, ..Default::default() };

    for row in 0..rows {
        for sy in 0..sh {
            let y = board_y + (row as u16) * (sh + 1) + 1 + sy;
            let x = board_x + 1 + (lit_col as u16) * (sw + 1);
            surface.print(x, y, &"✦".repeat(sw as usize), style);
        }
    }
}

pub fn render_keybar(surface: &mut dyn Surface, engine: &GameEngine, y: u16) {
    let (term_w, _) = surface.size();
    surface.print(0, y, &" ".repeat(term_w as usize), Style::default());

    let grey = fg(Color::DarkGrey);
    let white = fg(Color::White);

    let mut segments: Vec<(String, Style)> = vec![
        ("←→↑↓ ".to_string(), grey),
        ("Move  ".to_string(), white),
        ("Enter ".to_string(), grey),
        ("Pick  ".to_string(), white),
        ("H ".to_string(), grey),
        ("Help  ".to_string(), white),
    ];

    if engine.bonuses.scissors > 0 {
        segments.push(("Z ".to_string(), grey));
        segments.push((format!("✂x{} ", engine.bonuses.scissors), white));
    } else {
        segments.push(("Z ✂x0 ".to_string(), grey));
    }
    if engine.bonuses.tweezers > 0 {
        segments.push(("X ".to_string(), grey));
        segments.push((format!("⊹x{} ", engine.bonuses.tweezers), white));
    } else {
        segments.push(("X ⊹x0 ".to_string(), grey));
    }
    if engine.bonuses.balloons > 0 {
        segments.push(("C ".to_string(), grey));
        segments.push((format!("⊛x{} ", engine.bonuses.balloons), white));
    } else {
        segments.push(("C ⊛x0 ".to_string(), grey));
    }

    segments.push(("A ".to_string(), grey));
    segments.push(("Ad ".to_string(), white));
    segments.push(("Esc ".to_string(), grey));
    segments.push(("Menu".to_string(), white));

    let mut x = 0u16;
    for (text, style) in &segments {
        surface.print(x, y, text, *style);
        x += text.chars().count() as u16;
    }
}

/// Render the main menu screen.
pub fn render_main_menu(stdout: &mut Stdout, selected: usize, flash: Option<&str>) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_main_menu_inner(&mut surface, selected, flash);
    surface.finish()
}

fn render_main_menu_inner(surface: &mut dyn Surface, selected: usize, flash: Option<&str>) {
    let items = ["Quick Game", "Custom Game", "Campaign", "Endless", "Options", "Quit"];
    let (term_w, term_h) = surface.size();
    let start_y = term_h / 2 - (items.len() as u16 + 4) / 2;

    // Title
    let title = "═══ KNITUI ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, title, Style::default());

    // Menu items
    for (i, item) in items.iter().enumerate() {
        let y = start_y + 2 + i as u16;
        let prefix = if i == selected { "> " } else { "  " };
        let line = format!("{}{}", prefix, item);
        let x = (term_w.saturating_sub(line.chars().count() as u16 + 4)) / 2;
        let style = if i == selected { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
        surface.print(x, y, &line, style);
    }

    // Flash message
    if let Some(msg) = flash {
        let flash_y = start_y + 2 + items.len() as u16 + 1;
        let flash_x = (term_w.saturating_sub(msg.chars().count() as u16)) / 2;
        surface.print(flash_x, flash_y, msg, fg(Color::DarkGrey));
    }
}

/// Render the custom game configuration screen.
pub fn render_custom_game(
    stdout: &mut Stdout,
    preset_name: &str,
    fields: &[(&str, u16)],
    selected_field: usize,
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_custom_game_inner(&mut surface, preset_name, fields, selected_field);
    surface.finish()
}

fn render_custom_game_inner(
    surface: &mut dyn Surface,
    preset_name: &str,
    fields: &[(&str, u16)],
    selected_field: usize,
) {
    let (term_w, term_h) = surface.size();
    let total_lines = 4 + fields.len() as u16 + 2; // title + preset + gap + fields + gap + hint
    let start_y = term_h / 2 - total_lines / 2;

    // Title
    let title = "═══ CUSTOM GAME ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, title, Style::default());

    // Preset selector
    let preset_line = format!("Preset: ← [{}] →", preset_name);
    let preset_x = (term_w.saturating_sub(preset_line.chars().count() as u16)) / 2;
    let preset_style = if selected_field == 0 { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
    surface.print(preset_x, start_y + 2, &preset_line, preset_style);

    // Fields (selected_field 1..=fields.len() maps to fields[0..])
    let col_x = (term_w.saturating_sub(30)) / 2;
    for (i, (name, value)) in fields.iter().enumerate() {
        let y = start_y + 4 + i as u16;
        let prefix = if i + 1 == selected_field { "> " } else { "  " };
        let line = format!("{}{:<20}{:>4}", prefix, name, value);
        let style = if i + 1 == selected_field { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
        surface.print(col_x, y, &line, style);
    }

    // Hint line
    let hint = "↑↓ Navigate  ←→ Adjust  Enter: Start  Esc: Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 4 + fields.len() as u16 + 1;
    surface.print(hint_x, hint_y, hint, fg(Color::DarkGrey));
}

/// Render the options screen (scale, color mode).
pub fn render_options(
    stdout: &mut Stdout,
    selected: usize,
    scale: u16,
    color_mode: &str,
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_options_inner(&mut surface, selected, scale, color_mode);
    surface.finish()
}

fn render_options_inner(surface: &mut dyn Surface, selected: usize, scale: u16, color_mode: &str) {
    let (term_w, term_h) = surface.size();
    let start_y = term_h / 2 - 5;

    // Title
    let title = "═══ OPTIONS ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, title, Style::default());

    let fields: [(&str, String); 2] = [
        ("Scale", format!("← {} →", scale)),
        ("Color Mode", format!("← {} →", color_mode)),
    ];

    let col_x = (term_w.saturating_sub(34)) / 2;
    for (i, (name, value)) in fields.iter().enumerate() {
        let y = start_y + 2 + i as u16;
        let prefix = if i == selected { "> " } else { "  " };
        let line = format!("{}{:<16}{}", prefix, name, value);
        let style = if i == selected { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
        surface.print(col_x, y, &line, style);
    }

    // Hint line
    let hint = "←→ Adjust  Esc: Save & Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 2 + fields.len() as u16 + 1;
    surface.print(hint_x, hint_y, hint, fg(Color::DarkGrey));
}

/// Render the campaign track selection screen.
pub fn render_campaign_select(
    stdout: &mut Stdout,
    selected: usize,
    track_names: &[&str],
    track_sizes: &[usize],
    progress_labels: &[String],
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_campaign_select_inner(&mut surface, selected, track_names, track_sizes, progress_labels);
    surface.finish()
}

fn render_campaign_select_inner(
    surface: &mut dyn Surface,
    selected: usize,
    track_names: &[&str],
    track_sizes: &[usize],
    progress_labels: &[String],
) {
    let (term_w, term_h) = surface.size();
    let total_lines = 4 + track_names.len() as u16 + 2;
    let start_y = term_h / 2 - total_lines / 2;

    let title = "═══ SELECT CAMPAIGN ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, title, Style::default());

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
        let style = if i == selected { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
        surface.print(col_x, y, &line, style);
    }

    let hint = "↑↓ Select  Enter: Start  Esc: Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 2 + track_names.len() as u16 + 1;
    surface.print(hint_x, hint_y, hint, fg(Color::DarkGrey));
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
    let mut surface = TermSurface::begin(stdout)?;
    render_level_intro_inner(&mut surface, track_name, level_num, total_levels, board_h, board_w, colors);
    surface.finish()
}

fn render_level_intro_inner(
    surface: &mut dyn Surface,
    track_name: &str,
    level_num: usize,
    total_levels: usize,
    board_h: u16,
    board_w: u16,
    colors: u16,
) {
    let (term_w, term_h) = surface.size();
    let start_y = term_h / 2 - 3;

    let title = format!("═══ {} CAMPAIGN ═══", track_name.to_uppercase());
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, &title, Style::default());

    let level_str = format!("Level {}/{}", level_num, total_levels);
    let lx = (term_w.saturating_sub(level_str.chars().count() as u16)) / 2;
    surface.print(lx, start_y + 2, &level_str, Style::default());

    let desc = format!("{}x{} board, {} colors", board_w, board_h, colors);
    let dx = (term_w.saturating_sub(desc.chars().count() as u16)) / 2;
    surface.print(dx, start_y + 3, &desc, fg(Color::DarkGrey));

    let hint = "Press Enter to start";
    let hx = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    surface.print(hx, start_y + 5, hint, fg(Color::DarkGrey));
}

/// Render the endless mode game-over screen (shown when stuck).
pub fn render_endless_gameover(
    stdout: &mut Stdout,
    wave: usize,
    best_wave: usize,
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_endless_gameover_inner(&mut surface, wave, best_wave);
    surface.finish()
}

fn render_endless_gameover_inner(surface: &mut dyn Surface, wave: usize, best_wave: usize) {
    let (term_w, term_h) = surface.size();
    let start_y = term_h / 2 - 3;

    let title = "═══ ENDLESS MODE ═══";
    let tx = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(tx, start_y, title, Style::default());

    let wave_str = format!("You reached wave {}", wave);
    let wx = (term_w.saturating_sub(wave_str.chars().count() as u16)) / 2;
    surface.print(wx, start_y + 2, &wave_str, Style::default());

    if wave >= best_wave {
        let record_str = "New record!";
        let rx = (term_w.saturating_sub(record_str.chars().count() as u16)) / 2;
        surface.print(rx, start_y + 3, record_str, Style::default());
    } else {
        let best_str = format!("Best: wave {}", best_wave);
        let bx = (term_w.saturating_sub(best_str.chars().count() as u16)) / 2;
        surface.print(bx, start_y + 3, &best_str, fg(Color::DarkGrey));
    }

    let hint = "R:Play Again  M:Menu  Q:Quit";
    let hx = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    surface.print(hx, start_y + 5, hint, fg(Color::DarkGrey));
}

pub fn render_bonus_display_h(surface: &mut dyn Surface, engine: &GameEngine, x: u16, y: u16) {
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    let mut cx = x;
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        if i > 0 {
            surface.print(cx, y, "  ", Style::default());
            cx += 2;
        }
        let text = format!("[{}] {} x{}", key, icon, count);
        let style = if *count > 0 { fg(Color::White) } else { fg(Color::DarkGrey) };
        surface.print(cx, y, &text, style);
        cx += text.chars().count() as u16;
    }
    // Held spool counter
    let held = engine.held_spools.len() as u16;
    let limit = engine.spool_limit as u16;
    surface.print(cx, y, "  ", Style::default());
    cx += 2;
    let counter_str = format!("⊞ {}/{}", held, limit);
    let counter_style = if held >= limit.saturating_sub(1) {
        fg(Color::Red)
    } else if held >= limit.saturating_sub(2) {
        fg(Color::Yellow)
    } else {
        fg(Color::White)
    };
    surface.print(cx, y, &counter_str, counter_style);
    // Color count blessing: show remaining spools per color
    if engine.blessing_flags.color_count {
        render_color_counts(surface, engine, x, y + 1);
    }
}

pub fn render_bonus_panel(surface: &mut dyn Surface, engine: &GameEngine, x: u16, y: u16) {
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        let text = format!("[{}] {} x{}", key, icon, count);
        let style = if *count > 0 { fg(Color::White) } else { fg(Color::DarkGrey) };
        surface.print(x, y + i as u16, &text, style);
    }
    // Held spool counter
    let held = engine.held_spools.len() as u16;
    let limit = engine.spool_limit as u16;
    let mut row = bonuses.len() as u16;
    let counter_str = format!("⊞ {}/{}", held, limit);
    let counter_style = if held >= limit.saturating_sub(1) {
        fg(Color::Red)
    } else if held >= limit.saturating_sub(2) {
        fg(Color::Yellow)
    } else {
        fg(Color::White)
    };
    surface.print(x, y + row, &counter_str, counter_style);
    // Color count blessing: show remaining spools per color
    if engine.blessing_flags.color_count {
        row += 1;
        render_color_counts(surface, engine, x, y + row);
    }
}

fn render_color_counts(surface: &mut dyn Surface, engine: &GameEngine, x: u16, y: u16) {
    use std::collections::HashMap;
    let mut counts: HashMap<Color, u16> = HashMap::new();
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
        return;
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
    let mut cx = x;
    for (i, (color, count)) in pairs.iter().enumerate() {
        if i > 0 {
            surface.print(cx, y, " ", Style::default());
            cx += 1;
        }
        let text = format!("{}", count);
        surface.print(cx, y, &text, fg(*color));
        cx += text.chars().count() as u16;
    }
}

/// Render the pseudo-ad full-screen overlay.
pub fn render_ad_overlay(
    stdout: &mut Stdout,
    quote: &str,
    started_at: &Instant,
    ad_duration_secs: u64,
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_ad_overlay_inner(&mut surface, quote, started_at, ad_duration_secs);
    surface.finish()
}

fn render_ad_overlay_inner(
    surface: &mut dyn Surface,
    quote: &str,
    started_at: &Instant,
    ad_duration_secs: u64,
) {
    let elapsed = started_at.elapsed().as_secs();
    let remaining = ad_duration_secs.saturating_sub(elapsed);
    let progress = if ad_duration_secs > 0 {
        ((elapsed as f64 / ad_duration_secs as f64) * 100.0).min(100.0) as u16
    } else {
        100
    };
    let done = remaining == 0;

    let (term_w, term_h) = surface.size();

    // Box dimensions
    let box_w = 50u16.min(term_w.saturating_sub(4));
    let box_inner = (box_w - 2) as usize;

    let wrapped = word_wrap(quote, box_inner);
    let box_h = 8 + wrapped.len() as u16;
    let x0 = (term_w.saturating_sub(box_w)) / 2;
    let y0 = (term_h.saturating_sub(box_h)) / 2;

    let mut y = y0;

    // Top border
    let mut top = String::from("╔");
    for _ in 0..box_inner { top.push('═'); }
    top.push('╗');
    surface.print(x0, y, &top, Style::default());
    y += 1;

    // Empty line
    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

    // Header
    print_boxed_line(surface, x0, y, box_inner, &center_text("✂ FREE SCISSORS ✂", box_inner));
    y += 1;

    // Empty line
    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

    // Quote lines
    for line in &wrapped {
        print_boxed_line(surface, x0, y, box_inner, &center_text(line, box_inner));
        y += 1;
    }

    // Empty line
    print_boxed_line(surface, x0, y, box_inner, "");
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
    print_boxed_line(surface, x0, y, box_inner, &center_text(&bar, box_inner));
    y += 1;

    // Countdown or close prompt
    if done {
        let msg = "[ Press ESC to collect your reward ]";
        print_boxed_line(surface, x0, y, box_inner, &center_text(msg, box_inner));
    } else {
        let msg = format!("[{}s remaining]", remaining);
        print_boxed_line(surface, x0, y, box_inner, &center_text(&msg, box_inner));
    }
    y += 1;

    // Empty line
    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

    // Bottom border
    let mut bottom = String::from("╚");
    for _ in 0..box_inner { bottom.push('═'); }
    bottom.push('╝');
    surface.print(x0, y, &bottom, Style::default());
}

fn print_boxed_line(surface: &mut dyn Surface, x0: u16, y: u16, inner_w: usize, content: &str) {
    let content_chars: usize = content.chars().count();
    let mut line = String::from("║");
    line.push_str(content);
    for _ in content_chars..inner_w {
        line.push(' ');
    }
    line.push('║');
    surface.print(x0, y, &line, Style::default());
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
    let mut surface = TermSurface::begin(stdout)?;
    render_blessing_selection_inner(&mut surface, cursor, chosen, completed_tracks);
    surface.finish()
}

fn render_blessing_selection_inner(
    surface: &mut dyn Surface,
    cursor: usize,
    chosen: &[usize],
    completed_tracks: usize,
) {
    let (term_w, _term_h) = surface.size();
    let total_blessings = ALL_BLESSINGS.len();
    let rows = (total_blessings + CARD_COLS - 1) / CARD_COLS;

    // Title
    let title = "═══ CHOOSE 3 BLESSINGS ═══";
    let title_x = term_w.saturating_sub(title.len() as u16) / 2;
    surface.print(title_x, 0, title, Style::default());

    // Grid origin
    let grid_w = (CARD_W + 3) * CARD_COLS + 1; // card + gap
    let grid_x = (term_w as usize).saturating_sub(grid_w) / 2;
    let grid_y = 2u16;

    let green = fg(Color::Green);
    let yellow = fg(Color::Yellow);
    let grey = fg(Color::DarkGrey);
    let green_bold = Style { fg: Color::Green, attrs: Attrs::BOLD, ..Default::default() };
    let yellow_bold = Style { fg: Color::Yellow, attrs: Attrs::BOLD, ..Default::default() };
    let bold = Style { attrs: Attrs::BOLD, ..Default::default() };

    for idx in 0..total_blessings {
        let b = &ALL_BLESSINGS[idx];
        let row = idx / CARD_COLS;
        let col = idx % CARD_COLS;
        let x = grid_x + col * (CARD_W + 3);
        let y = grid_y + (row as u16) * (CARD_H as u16 + 1);

        let is_cursor = idx == cursor;
        let is_chosen = chosen.contains(&idx);
        let unlocked = blessings::is_unlocked(b, completed_tracks);

        // Pick border chars
        let (tl, tr, bl, br, hz, vt) = if is_chosen {
            ('╔', '╗', '╚', '╝', '═', '║')
        } else {
            ('┌', '┐', '└', '┘', '─', '│')
        };

        // Top border
        let top = format!("{}{}{}", tl, hz.to_string().repeat(CARD_W), tr);
        let border_style = if is_chosen { green } else if is_cursor { yellow } else { Style::default() };
        surface.print(x as u16, y, &top, border_style);

        // Art lines (5 lines)
        for (ai, art_line) in b.ascii_art.iter().enumerate() {
            let padded = format!("{:^w$}", art_line, w = CARD_W);
            let line = format!("{}{}{}", vt, padded, vt);
            let style = if !unlocked { grey } else if is_chosen { green } else if is_cursor { yellow } else { Style::default() };
            surface.print(x as u16, y + 1 + ai as u16, &line, style);
        }

        // Name line
        let name_str = format!("{:^w$}", b.name, w = CARD_W);
        let name_line = format!("{}{}{}", vt, name_str, vt);
        let name_style = if !unlocked { grey } else if is_chosen { green_bold } else if is_cursor { yellow_bold } else { bold };
        surface.print(x as u16, y + 6, &name_line, name_style);

        // Tier line
        let tier_label = if unlocked {
            format!("{:^w$}", format!("─ {} Tier ─", b.tier.label()), w = CARD_W)
        } else {
            let needed = blessings::tracks_required(b.tier);
            format!("{:^w$}", format!("Locked ({}+ tracks)", needed), w = CARD_W)
        };
        let tier_line = format!("{}{}{}", vt, tier_label, vt);
        let tier_style = if !unlocked { grey } else { Style::default() };
        surface.print(x as u16, y + 7, &tier_line, tier_style);

        // Description line
        let desc = format!("{:^w$}", b.description, w = CARD_W);
        let desc_line = format!("{}{}{}", vt, desc, vt);
        let desc_style = if !unlocked { grey } else { Style::default() };
        surface.print(x as u16, y + 8, &desc_line, desc_style);

        // Bottom border
        let bot = format!("{}{}{}", bl, hz.to_string().repeat(CARD_W), br);
        surface.print(x as u16, y + 9, &bot, border_style);

        // Selection marker
        if is_chosen {
            let marker = format!("{:^w$}", "★ SELECTED", w = CARD_W + 2);
            surface.print(x as u16, y + 10, &marker, green);
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
    let status_style = if chosen.len() == 3 { green } else { grey };
    surface.print(sx as u16, status_y, &status, status_style);
}
