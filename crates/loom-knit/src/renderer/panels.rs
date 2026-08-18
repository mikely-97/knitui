use loom_engine::render::{Attrs, Color, Style, Surface};
use crate::blessings::ALL_BLESSINGS;
use crate::engine::GameEngine;
use crate::board_entity::BoardEntity;

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

fn bold() -> Style {
    Style { attrs: Attrs::BOLD, ..Default::default() }
}

/// Surface-only entry point for the help screen (native-independent — see
/// `render_vertical_to_surface` for the rationale behind this naming).
pub fn render_help_to_surface(surface: &mut dyn Surface, engine: &GameEngine) {
    render_help_inner(surface, engine);
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
/// Surface-only entry point for the celebration sweep effect (see
/// `render_vertical_to_surface` for the naming rationale).
pub fn render_celebration_to_surface(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    board_x: u16,
    board_y: u16,
    scale: u16,
    tick: u8,
) {
    render_celebration_inner(surface, engine, board_x, board_y, scale, tick);
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

