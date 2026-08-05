// Every function here is a Stdout-wrapping mid-frame TermSurface screen
// (none has a Surface-only entry point web.rs needs), so the whole module
// is native-only rather than gating each function individually.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{self, Stdout};

use loom_engine::render::{Attrs, Color, Style, Surface};
use loom_engine_term::TermSurface;

use crate::blessings::{self, ALL_BLESSINGS};

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

fn bold(color: Color) -> Style {
    Style { fg: color, attrs: Attrs::BOLD, ..Default::default() }
}

// ── Help overlay ──────────────────────────────────────────────────────────

pub fn render_help(
    stdout: &mut Stdout,
    help_lines: &[(&str, &str)],
    engine: Option<&crate::engine::GameEngine>,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_help_inner(&mut surface, help_lines, engine);
    surface.done()
}

fn render_help_inner(
    surface: &mut dyn Surface,
    help_lines: &[(&str, &str)],
    engine: Option<&crate::engine::GameEngine>,
) {
    let (term_w, term_h) = surface.size();
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

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "MERGE-2 HELP", w = inner), bold(Color::Reset));
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    for (i, (key, desc)) in help_lines.iter().enumerate() {
        let y = by + 3 + i as u16;
        let remaining = inner.saturating_sub(1 + col1_w + 1);
        let mut cx = bx;
        surface.print(cx, y, "║", fg(Color::DarkGrey));
        cx += 1;
        surface.print(cx, y, &format!(" {:<w$}", key, w = col1_w), fg(Color::Yellow));
        cx += 1 + col1_w as u16;
        surface.print(cx, y, &format!("{:<w$}", desc, w = remaining), fg(Color::White));
        cx += remaining as u16;
        surface.print(cx, y, "║", fg(Color::DarkGrey));
    }

    let sep_y = by + 3 + help_lines.len() as u16;
    surface.print(bx, sep_y, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    // Active blessings
    surface.print(bx, sep_y + 1, "║", fg(Color::DarkGrey));
    surface.print(bx + 1, sep_y + 1, &format!("{:^w$}", "Active Blessings", w = inner), fg(Color::Cyan));
    surface.print(bx + 1 + inner as u16, sep_y + 1, "║", fg(Color::DarkGrey));

    if bless_active.is_empty() {
        surface.print(bx, sep_y + 2, &format!("║{:^w$}║", "none", w = inner), fg(Color::DarkGrey));
    }
    for (i, (name, desc)) in bless_active.iter().enumerate() {
        let y = sep_y + 2 + i as u16;
        let remaining = inner.saturating_sub(1 + col1_w + 1);
        let mut cx = bx;
        surface.print(cx, y, "║", fg(Color::DarkGrey));
        cx += 1;
        surface.print(cx, y, &format!(" {:<w$}", name, w = col1_w), fg(Color::Green));
        cx += 1 + col1_w as u16;
        surface.print(cx, y, &format!("{:<w$}", desc, w = remaining), fg(Color::White));
        cx += remaining as u16;
        surface.print(cx, y, "║", fg(Color::DarkGrey));
    }

    let sep2_y = sep_y + 2 + bless_rows;
    surface.print(bx, sep2_y, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    // Bonus inventory
    surface.print(bx, sep2_y + 1, "║", fg(Color::DarkGrey));
    surface.print(bx + 1, sep2_y + 1, &format!("{:^w$}", "Inventory", w = inner), fg(Color::Cyan));
    surface.print(bx + 1 + inner as u16, sep2_y + 1, "║", fg(Color::DarkGrey));

    let slots = engine.map(|e| e.inventory.slot_count()).unwrap_or(0);
    if slots == 0 {
        surface.print(bx, sep2_y + 2, &format!("║{:^w$}║", "empty", w = inner), fg(Color::DarkGrey));
    }
    for i in 0..slots {
        let y = sep2_y + 2 + i as u16;
        let (content_style, label) = if let Some(piece) = engine.and_then(|e| e.inventory.peek(i)) {
            let piece_name = match piece {
                crate::item::Piece::Regular(item) => format!("{} T{}", item.family.name(), item.tier),
                crate::item::Piece::Blueprint(fam) => format!("Blueprint({})", fam.name()),
            };
            (fg(Color::White), format!("  Slot {}: {}", i + 1, piece_name))
        } else {
            (fg(Color::DarkGrey), format!("  Slot {}: (empty)", i + 1))
        };
        surface.print(bx, y, "║", fg(Color::DarkGrey));
        surface.print(bx + 1, y, &format!("{:<w$}", label, w = inner), content_style);
        surface.print(bx + 1 + inner as u16, y, "║", fg(Color::DarkGrey));
    }

    let end_y = sep2_y + 2 + slots.max(1) as u16;
    surface.print(bx, end_y, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));
    surface.print(bx, end_y + 1, &format!("{:^w$}", "Press any key to close", w = box_w as usize), fg(Color::DarkGrey));
}

/// Render a celebration sweep overlay on the merge board.
pub fn render_celebration(
    stdout: &mut Stdout,
    engine: &crate::engine::GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_celebration_inner(&mut surface, engine, geo, tick);
    surface.done()
}

fn render_celebration_inner(
    surface: &mut dyn Surface,
    engine: &crate::engine::GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) {
    let (cw, ch) = crate::glyphs::cell_dims(geo.scale);
    let cols = engine.board.cols;
    let rows = engine.board.rows;
    let lit_col = (tick / 2) as usize % cols.max(1);
    let color = if (tick / 2) % 2 == 0 { Color::Yellow } else { Color::Green };
    let style = Style { fg: color, attrs: Attrs::BOLD, ..Default::default() };

    for row in 0..rows {
        for sy in 0..ch {
            let y = geo.board_y + (row as u16) * (ch as u16 + 1) + 1 + sy as u16;
            let x = geo.board_x + 1 + (lit_col as u16) * (cw as u16 + 1);
            surface.print(x, y, &"✦".repeat(cw), style);
        }
    }
}

/// Render the mission-complete score summary screen.
pub fn render_mission_summary(
    stdout: &mut Stdout,
    ctx: &crate::campaign::CampaignState,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_mission_summary_inner(&mut surface, ctx);
    surface.done()
}

fn render_mission_summary_inner(surface: &mut dyn Surface, ctx: &crate::campaign::CampaignState) {
    let (term_w, term_h) = surface.size();
    let box_w = 44u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = term_h / 4;
    let inner = box_w as usize - 2;

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Yellow));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "MISSION COMPLETE!", w = inner), Style::default());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    let track = crate::campaign_levels::TRACK_NAMES.get(ctx.track_idx).copied().unwrap_or("Unknown");
    surface.print(bx, by + 3, &format!("║ {:<w$}║",
        format!("{} — Mission {}/{}", track, ctx.current_mission + 1, ctx.total_missions()),
        w = inner - 1), fg(Color::White));

    surface.print(bx, by + 4, &format!("║ {:<w$}║",
        format!("Score: {}  Stars: ★{}", ctx.score, ctx.stars),
        w = inner - 1), Style::default());

    surface.print(bx, by + 5, &format!("║ {:<w$}║",
        format!("Total merges: {}  Thawed: {}", ctx.total_merges, ctx.cells_thawed),
        w = inner - 1), Style::default());

    surface.print(bx, by + 6, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));
    surface.print(bx, by + 7, &format!("║{:^w$}║", "Inventory Carried Over", w = inner), fg(Color::Cyan));

    let slots = ctx.inventory.slot_count();
    let mut row_off = 8u16;
    let mut any = false;
    for i in 0..slots {
        if let Some(piece) = ctx.inventory.peek(i) {
            let piece_name = match piece {
                crate::item::Piece::Regular(item) => format!("{} T{}", item.family.name(), item.tier),
                crate::item::Piece::Blueprint(fam) => format!("Blueprint({})", fam.name()),
            };
            let s = format!("  {}", piece_name);
            surface.print(bx, by + row_off, &format!("║{:<w$}║", s, w = inner), fg(Color::White));
            row_off += 1;
            any = true;
        }
    }
    if !any {
        surface.print(bx, by + row_off, &format!("║{:^w$}║", "none", w = inner), fg(Color::DarkGrey));
        row_off += 1;
    }

    surface.print(bx, by + row_off, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Yellow));
    surface.print(bx, by + row_off + 1, &format!("{:^w$}", "Press Enter to continue", w = box_w as usize), fg(Color::DarkGrey));
}

// ── Main menu ─────────────────────────────────────────────────────────────

pub fn render_main_menu(
    stdout: &mut Stdout,
    items: &[&str],
    selected: usize,
    flash: Option<&str>,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_main_menu_inner(&mut surface, items, selected, flash);
    surface.done()
}

fn render_main_menu_inner(surface: &mut dyn Surface, items: &[&str], selected: usize, flash: Option<&str>) {
    let (term_w, term_h) = surface.size();
    let cx = term_w / 2;
    let cy = term_h / 2;
    let box_w = 28u16;
    let bx = cx.saturating_sub(box_w / 2);
    let by = cy.saturating_sub((items.len() as u16 + 4) / 2);

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Cyan));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "MERGE-2", w = box_w as usize - 2), Style::default());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    for (i, item) in items.iter().enumerate() {
        let y = by + 3 + i as u16;
        let style = if i == selected {
            Style { fg: Color::Black, bg: Color::Cyan, ..Default::default() }
        } else {
            Style { fg: Color::White, ..Default::default() }
        };
        let marker = if i == selected { "▶" } else { " " };
        surface.print(bx, y, &format!("║ {} {:<w$}║", marker, item, w = box_w as usize - 5), style);
    }

    let ey = by + 3 + items.len() as u16;
    surface.print(bx, ey, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));

    if let Some(msg) = flash {
        surface.print(bx, ey + 1, &format!(" {}", msg), fg(Color::Red));
    }
}

// ── Campaign track select ─────────────────────────────────────────────────

pub fn render_campaign_select(
    stdout: &mut Stdout,
    tracks: &[&str],
    progress: &[String],
    selected: usize,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_campaign_select_inner(&mut surface, tracks, progress, selected);
    surface.done()
}

fn render_campaign_select_inner(surface: &mut dyn Surface, tracks: &[&str], progress: &[String], selected: usize) {
    let (term_w, term_h) = surface.size();
    let box_w = 34u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub((tracks.len() as u16 + 4) / 2);

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Green));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "SELECT TRACK", w = box_w as usize - 2), Style::default());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    for (i, (name, prog)) in tracks.iter().zip(progress.iter()).enumerate() {
        let y = by + 3 + i as u16;
        let style = if i == selected {
            Style { fg: Color::Black, bg: Color::Green, ..Default::default() }
        } else {
            Style { fg: Color::White, ..Default::default() }
        };
        let marker = if i == selected { "▶" } else { " " };
        surface.print(bx, y, &format!("║ {} {:<18}{:>8}║", marker, name, prog), style);
    }

    let ey = by + 3 + tracks.len() as u16;
    surface.print(bx, ey, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Green));
}

// ── Campaign level intro ──────────────────────────────────────────────────

pub fn render_level_intro(
    stdout: &mut Stdout,
    lines: &[String],
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_level_intro_inner(&mut surface, lines);
    surface.done()
}

fn render_level_intro_inner(surface: &mut dyn Surface, lines: &[String]) {
    let (term_w, term_h) = surface.size();
    let box_w = 44u16;
    let box_h = lines.len() as u16 + 4;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(box_h / 2);

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Yellow));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "MISSION START", w = box_w as usize - 2), Style::default());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    for (i, line) in lines.iter().enumerate() {
        surface.print(bx, by + 3 + i as u16, &format!("║ {:<w$}║", line, w = box_w as usize - 3), fg(Color::White));
    }

    let ey = by + 3 + lines.len() as u16;
    surface.print(bx, ey, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Yellow));
    surface.print(bx, ey + 1, "  Press Enter to begin", fg(Color::DarkGrey));
}

// ── Ad watching overlay ───────────────────────────────────────────────────

pub fn render_ad_overlay(
    stdout: &mut Stdout,
    quote: &str,
    elapsed_secs: u64,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_ad_overlay_inner(&mut surface, quote, elapsed_secs);
    surface.done()
}

fn render_ad_overlay_inner(surface: &mut dyn Surface, quote: &str, elapsed_secs: u64) {
    let (term_w, term_h) = surface.size();
    let box_w = 44u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = term_h / 4;
    let ad_dur = 10u64;
    let remaining = ad_dur.saturating_sub(elapsed_secs);

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Magenta));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "★  SPONSORED MESSAGE  ★", w = box_w as usize - 2), Style::default());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

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
        surface.print(bx, by + 3 + i as u16, &format!("║  {:<w$}  ║", row, w = inner), fg(Color::White));
    }

    let ey = by + 3 + rows.len() as u16;
    surface.print(bx, ey, &format!("║{:^w$}║", format!("{}s remaining…", remaining), w = box_w as usize - 2), fg(Color::DarkGrey));
    surface.print(bx, ey + 1, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Magenta));
}

// ── Options screen ────────────────────────────────────────────────────────

pub fn render_options(
    stdout: &mut Stdout,
    settings: &loom_engine::settings::UserSettings,
    selected: usize,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_options_inner(&mut surface, settings, selected);
    surface.done()
}

fn render_options_inner(surface: &mut dyn Surface, settings: &loom_engine::settings::UserSettings, selected: usize) {
    let (term_w, _) = surface.size();
    let cx = term_w / 2;
    let box_w = 32u16;
    let bx = cx.saturating_sub(box_w / 2);

    surface.print(bx, 3, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Cyan));
    surface.print(bx, 4, &format!("║{:^w$}║", "OPTIONS", w = box_w as usize - 2), Style::default());
    surface.print(bx, 5, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    let rows: &[(&str, String)] = &[
        ("Color mode", settings.color_mode.clone()),
        ("Scale",      settings.scale.to_string()),
    ];
    for (i, (name, val)) in rows.iter().enumerate() {
        let y = 6 + i as u16;
        let style = if i == selected {
            Style { fg: Color::Black, bg: Color::Cyan, ..Default::default() }
        } else {
            Style { fg: Color::White, ..Default::default() }
        };
        let marker = if i == selected { "▶" } else { " " };
        surface.print(bx, y, &format!("║ {} {:<14} {:>9}║", marker, name, val), style);
    }

    surface.print(bx, 6 + rows.len() as u16, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));
    surface.print(bx, 7 + rows.len() as u16, "  ←→ Adjust  Esc: Back", fg(Color::DarkGrey));
}

// ── Custom game ───────────────────────────────────────────────────────────

pub fn render_custom_game(
    stdout: &mut Stdout,
    config: &crate::config::Config,
    preset_name: &str,
    selected: usize,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_custom_game_inner(&mut surface, config, preset_name, selected);
    surface.done()
}

fn render_custom_game_inner(surface: &mut dyn Surface, config: &crate::config::Config, preset_name: &str, selected: usize) {
    let (term_w, _) = surface.size();
    let box_w = 36u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);

    surface.print(bx, 2, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Cyan));
    surface.print(bx, 3, &format!("║{:^w$}║", "CUSTOM GAME", w = box_w as usize - 2), Style::default());
    surface.print(bx, 4, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

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
        let style = if i == selected {
            Style { fg: Color::Black, bg: Color::Cyan, ..Default::default() }
        } else {
            Style { fg: Color::White, ..Default::default() }
        };
        let marker = if i == selected { "▶" } else { " " };
        surface.print(bx, y, &format!("║ {} {:<18} {:>11}║", marker, name, val), style);
    }

    let ey = 5 + fields.len() as u16;
    surface.print(bx, ey, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));
    surface.print(bx, ey + 1, "  ↑↓ Select  ←→ Adjust  Enter Start  Esc Back", fg(Color::DarkGrey));
}

// ── Blessing selection ────────────────────────────────────────────────────

pub fn render_blessing_selection(
    stdout: &mut Stdout,
    cursor: usize,
    chosen: &[usize],
    completed_tracks: usize,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_blessing_selection_inner(&mut surface, cursor, chosen, completed_tracks);
    surface.done()
}

fn render_blessing_selection_inner(surface: &mut dyn Surface, cursor: usize, chosen: &[usize], completed_tracks: usize) {
    let (term_w, _) = surface.size();
    let box_w = 50u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);

    surface.print(bx, 1, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Yellow));
    surface.print(bx, 2, &format!("║{:^w$}║", "SELECT BLESSINGS", w = box_w as usize - 2), Style::default());
    surface.print(bx, 3, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    for (i, blessing) in ALL_BLESSINGS.iter().enumerate() {
        let y = 4 + i as u16;
        let is_cursor  = i == cursor;
        let is_chosen  = chosen.contains(&i);
        let is_locked  = !blessings::is_unlocked(blessing, completed_tracks);
        let req_tracks = blessings::tracks_required(blessing.tier);

        let (text_fg, bg) = if is_cursor && !is_locked {
            (Color::Black, Color::Yellow)
        } else if is_chosen {
            (Color::Green, Color::Reset)
        } else if is_locked {
            (Color::DarkGrey, Color::Reset)
        } else {
            (Color::White, Color::Reset)
        };
        let tier_s = format!("[{}]", blessing.tier.label());
        let lock_s = if is_locked { "*" } else if is_chosen { "v" } else { " " };
        let line = format!("║{} {:<4} {:<26} {:>14}║",
            lock_s, tier_s, blessing.name,
            format!("{} tracks", req_tracks));
        surface.print(bx, y, &line, Style { fg: text_fg, bg, ..Default::default() });
    }

    let ey = 4 + ALL_BLESSINGS.len() as u16;
    surface.print(bx, ey, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Yellow));
    surface.print(bx, ey + 1, "  ↑↓ Move  Enter Toggle  Space Start  Esc Back", fg(Color::DarkGrey));
}

// ── Inventory expansion popup ─────────────────────────────────────────────

pub fn render_inv_expansion_popup(stdout: &mut Stdout) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_inv_expansion_popup_inner(&mut surface);
    surface.done()
}

fn render_inv_expansion_popup_inner(surface: &mut dyn Surface) {
    let (term_w, term_h) = surface.size();
    let box_w = 32u16;
    let bx = (term_w / 2).saturating_sub(box_w / 2);
    let by = (term_h / 2).saturating_sub(3);

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), bold(Color::Yellow));

    let title = format!("{:^w$}", "INVENTORY EXPANDED!", w = box_w as usize - 2);
    surface.print(bx, by + 1, &format!("║{}║", title), Style::default());

    let msg = format!("{:^w$}", "+1 inventory slot unlocked", w = box_w as usize - 2);
    surface.print(bx, by + 2, &format!("║{}║", msg), fg(Color::White));

    let hint = format!("{:^w$}", "Y Accept  N Decline", w = box_w as usize - 2);
    surface.print(bx, by + 3, &format!("║{}║", hint), fg(Color::DarkGrey));

    surface.print(bx, by + 4, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Yellow));
}
