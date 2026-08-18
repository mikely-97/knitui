//! Generic menu/UI-chrome screens shared by every game's `Shell<G>`.
//!
//! Ported from loom-knit's `renderer/panels.rs`, which had already
//! (Phase 1) migrated every one of these to take only primitive params and
//! a `&mut dyn Surface` — no knit-specific types. Two spots hardcoded
//! "KNITUI" in a title string; those now take a `game_name`/`reward_label`
//! parameter instead. Screens that genuinely need per-game runtime state
//! (the Help screen's active-blessings/bonus-inventory sections) are NOT
//! here — those stay behind `GameEngine::render_help`, a per-game trait
//! method, since a generic version would just be a worse copy of what each
//! game already has.

use std::time::Instant;

use crate::blessings::{self, Blessing};
use crate::render::{Attrs, Color, Style, Surface};

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

/// Render the main menu screen.
pub fn render_main_menu(surface: &mut dyn Surface, game_name: &str, selected: usize, flash: Option<&str>) {
    let items = ["Quick Game", "Custom Game", "Campaign", "Endless", "Options", "Quit"];
    let (term_w, term_h) = surface.size();
    let start_y = term_h / 2 - (items.len() as u16 + 4) / 2;

    let title = format!("═══ {} ═══", game_name.to_uppercase());
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, &title, Style::default());

    for (i, item) in items.iter().enumerate() {
        let y = start_y + 2 + i as u16;
        let prefix = if i == selected { "> " } else { "  " };
        let line = format!("{}{}", prefix, item);
        let x = (term_w.saturating_sub(line.chars().count() as u16 + 4)) / 2;
        let style = if i == selected { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
        surface.print(x, y, &line, style);
    }

    if let Some(msg) = flash {
        let flash_y = start_y + 2 + items.len() as u16 + 1;
        let flash_x = (term_w.saturating_sub(msg.chars().count() as u16)) / 2;
        surface.print(flash_x, flash_y, msg, fg(Color::DarkGrey));
    }
}

/// Render the custom game configuration screen.
pub fn render_custom_game(
    surface: &mut dyn Surface,
    preset_name: &str,
    fields: &[(&str, u16)],
    selected_field: usize,
) {
    let (term_w, term_h) = surface.size();
    let total_lines = 4 + fields.len() as u16 + 2;
    let start_y = term_h / 2 - total_lines / 2;

    let title = "═══ CUSTOM GAME ═══";
    let title_x = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    surface.print(title_x, start_y, title, Style::default());

    let preset_line = format!("Preset: ← [{}] →", preset_name);
    let preset_x = (term_w.saturating_sub(preset_line.chars().count() as u16)) / 2;
    let preset_style = if selected_field == 0 { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
    surface.print(preset_x, start_y + 2, &preset_line, preset_style);

    let col_x = (term_w.saturating_sub(30)) / 2;
    for (i, (name, value)) in fields.iter().enumerate() {
        let y = start_y + 4 + i as u16;
        let prefix = if i + 1 == selected_field { "> " } else { "  " };
        let line = format!("{}{:<20}{:>4}", prefix, name, value);
        let style = if i + 1 == selected_field { Style { attrs: Attrs::REVERSE, ..Default::default() } } else { Style::default() };
        surface.print(col_x, y, &line, style);
    }

    let hint = "↑↓ Navigate  ←→ Adjust  Enter: Start  Esc: Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 4 + fields.len() as u16 + 1;
    surface.print(hint_x, hint_y, hint, fg(Color::DarkGrey));
}

/// Render the options screen (scale, color mode).
pub fn render_options(surface: &mut dyn Surface, selected: usize, scale: u16, color_mode: &str) {
    let (term_w, term_h) = surface.size();
    let start_y = term_h / 2 - 5;

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

    let hint = "←→ Adjust  Esc: Save & Back";
    let hint_x = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    let hint_y = start_y + 2 + fields.len() as u16 + 1;
    surface.print(hint_x, hint_y, hint, fg(Color::DarkGrey));
}

/// Render the campaign track selection screen.
pub fn render_campaign_select(
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
pub fn render_endless_gameover(surface: &mut dyn Surface, wave: usize, best_wave: usize) {
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

/// Render the pseudo-ad full-screen overlay. `reward_label` names whatever
/// bonus watching the ad grants (e.g. "FREE SCISSORS" for knit).
pub fn render_ad_overlay(
    surface: &mut dyn Surface,
    reward_label: &str,
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
    let box_w = 50u16.min(term_w.saturating_sub(4));
    let box_inner = (box_w - 2) as usize;

    let wrapped = word_wrap(quote, box_inner);
    let box_h = 8 + wrapped.len() as u16;
    let x0 = (term_w.saturating_sub(box_w)) / 2;
    let y0 = (term_h.saturating_sub(box_h)) / 2;

    let mut y = y0;

    let mut top = String::from("╔");
    for _ in 0..box_inner { top.push('═'); }
    top.push('╗');
    surface.print(x0, y, &top, Style::default());
    y += 1;

    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

    print_boxed_line(surface, x0, y, box_inner, &center_text(&format!("✂ {} ✂", reward_label), box_inner));
    y += 1;

    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

    for line in &wrapped {
        print_boxed_line(surface, x0, y, box_inner, &center_text(line, box_inner));
        y += 1;
    }

    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

    let bar_width = box_inner.saturating_sub(8);
    let filled = (bar_width as u16 * progress / 100) as usize;
    let empty = bar_width - filled;
    let bar = format!("{}{}  {:>3}%", "█".repeat(filled), "░".repeat(empty), progress);
    print_boxed_line(surface, x0, y, box_inner, &center_text(&bar, box_inner));
    y += 1;

    if done {
        let msg = "[ Press ESC to collect your reward ]";
        print_boxed_line(surface, x0, y, box_inner, &center_text(msg, box_inner));
    } else {
        let msg = format!("[{}s remaining]", remaining);
        print_boxed_line(surface, x0, y, box_inner, &center_text(&msg, box_inner));
    }
    y += 1;

    print_boxed_line(surface, x0, y, box_inner, "");
    y += 1;

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

const CARD_W: usize = 17;
const CARD_H: usize = 11;
const CARD_COLS: usize = 3;

pub fn render_blessing_selection(
    surface: &mut dyn Surface,
    all_blessings: &[Blessing],
    cursor: usize,
    chosen: &[usize],
    completed_tracks: usize,
) {
    let (term_w, _term_h) = surface.size();
    let total_blessings = all_blessings.len();
    let rows = (total_blessings + CARD_COLS - 1) / CARD_COLS;

    let title = "═══ CHOOSE 3 BLESSINGS ═══";
    let title_x = term_w.saturating_sub(title.len() as u16) / 2;
    surface.print(title_x, 0, title, Style::default());

    let grid_w = (CARD_W + 3) * CARD_COLS + 1;
    let grid_x = (term_w as usize).saturating_sub(grid_w) / 2;
    let grid_y = 2u16;

    let green = fg(Color::Green);
    let yellow = fg(Color::Yellow);
    let grey = fg(Color::DarkGrey);
    let green_bold = Style { fg: Color::Green, attrs: Attrs::BOLD, ..Default::default() };
    let yellow_bold = Style { fg: Color::Yellow, attrs: Attrs::BOLD, ..Default::default() };
    let bold = Style { attrs: Attrs::BOLD, ..Default::default() };

    for idx in 0..total_blessings {
        let b = &all_blessings[idx];
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

        let top = format!("{}{}{}", tl, hz.to_string().repeat(CARD_W), tr);
        let border_style = if is_chosen { green } else if is_cursor { yellow } else { Style::default() };
        surface.print(x as u16, y, &top, border_style);

        for (ai, art_line) in b.ascii_art.iter().enumerate() {
            let padded = format!("{:^w$}", art_line, w = CARD_W);
            let line = format!("{}{}{}", vt, padded, vt);
            let style = if !unlocked { grey } else if is_chosen { green } else if is_cursor { yellow } else { Style::default() };
            surface.print(x as u16, y + 1 + ai as u16, &line, style);
        }

        let name_str = format!("{:^w$}", b.name, w = CARD_W);
        let name_line = format!("{}{}{}", vt, name_str, vt);
        let name_style = if !unlocked { grey } else if is_chosen { green_bold } else if is_cursor { yellow_bold } else { bold };
        surface.print(x as u16, y + 6, &name_line, name_style);

        let tier_label = if unlocked {
            format!("{:^w$}", format!("─ {} Tier ─", b.tier.label()), w = CARD_W)
        } else {
            let needed = blessings::tracks_required(b.tier);
            format!("{:^w$}", format!("Locked ({}+ tracks)", needed), w = CARD_W)
        };
        let tier_line = format!("{}{}{}", vt, tier_label, vt);
        let tier_style = if !unlocked { grey } else { Style::default() };
        surface.print(x as u16, y + 7, &tier_line, tier_style);

        let desc = format!("{:^w$}", b.description, w = CARD_W);
        let desc_line = format!("{}{}{}", vt, desc, vt);
        let desc_style = if !unlocked { grey } else { Style::default() };
        surface.print(x as u16, y + 8, &desc_line, desc_style);

        let bot = format!("{}{}{}", bl, hz.to_string().repeat(CARD_W), br);
        surface.print(x as u16, y + 9, &bot, border_style);

        if is_chosen {
            let marker = format!("{:^w$}", "★ SELECTED", w = CARD_W + 2);
            surface.print(x as u16, y + 10, &marker, green);
        }
    }

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
