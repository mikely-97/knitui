use loom_engine::render::{Attrs, Color, Style, Surface};
use loom_engine_term::TermSurface;
use std::io::{self, Stdout};

use crate::blessings::{self, ALL_BLESSINGS};
use crate::bonuses::BonusState;
use crate::engine::{GameEngine, GameStatus};

use super::LayoutGeometry;

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

fn bold() -> Style {
    Style { attrs: Attrs::BOLD, ..Default::default() }
}

// ── render_hud ────────────────────────────────────────────────────────────

/// Render the HUD panel: score, moves, bonus inventory.
///
/// Only ever called from inside `do_render`'s already-open surface, so this
/// takes `&mut dyn Surface` directly rather than owning its own frame.
pub fn render_hud(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    objective_label: &str,
) {
    let x = geo.hud_x;
    let mut y = geo.hud_y;

    let moves_left = engine.move_limit.saturating_sub(engine.moves_used);

    surface.print(x, y, &format!("Score: {:>8}", engine.score), Style::default());
    y += 1;

    // Combo multiplier display
    if engine.combo_display_ticks > 0 {
        surface.print(x, y, &format!("  x{} COMBO!", engine.cascade_depth + 1), fg(Color::Yellow));
        y += 1;
    }

    // Feature 1: warn when moves are running low
    if engine.move_limit > 0 && moves_left <= 5 {
        surface.print(x, y, &format!("Moves:  {:>7}", moves_left), fg(Color::Red));
    } else {
        surface.print(x, y, &format!("Moves:  {:>7}", moves_left), Style::default());
    }
    y += 1;

    if !objective_label.is_empty() {
        surface.print(x, y, &format!("Goal: {}", objective_label), Style::default());
        y += 1;
    }

    y += 1;

    let hammer_str     = format!("[Z] Hammer x{}", engine.bonuses.hammer);
    let laser_str      = format!("[X] Laser  x{}", engine.bonuses.laser);
    let blaster_str    = format!("[C] Blast  x{}", engine.bonuses.blaster);
    let warp_str       = format!("[V] Warp   x{}", engine.bonuses.warp);
    let color_bomb_str = format!("[B] CBomb  x{}", engine.bonuses.color_bomb);

    let dim_style = |count: u16| -> Style {
        if count == 0 { Style { attrs: Attrs::DIM, ..Default::default() } } else { Style::default() }
    };

    let mut cx = x;
    surface.print(cx, y, &hammer_str, dim_style(engine.bonuses.hammer));
    cx += hammer_str.chars().count() as u16 + 2;
    surface.print(cx, y, &laser_str, dim_style(engine.bonuses.laser));
    y += 1;

    let mut cx = x;
    surface.print(cx, y, &blaster_str, dim_style(engine.bonuses.blaster));
    cx += blaster_str.chars().count() as u16 + 2;
    surface.print(cx, y, &warp_str, dim_style(engine.bonuses.warp));
    y += 1;

    surface.print(x, y, &color_bomb_str, dim_style(engine.bonuses.color_bomb));
}

// ── render_key_bar ────────────────────────────────────────────────────────

/// Render the persistent key bar at the bottom of the terminal.
///
/// Only ever called from inside `do_render`'s already-open surface.
pub fn render_key_bar(surface: &mut dyn Surface, bonus_state: &BonusState) {
    let (term_w, term_h) = surface.size();

    let bar = match bonus_state {
        BonusState::HammerActive { .. } =>
            "Arrows Move  Enter Destroy  Esc Cancel".to_string(),
        BonusState::ColorBombActive { .. } =>
            "Arrows Move  Enter Clear Color  Esc Cancel".to_string(),
        BonusState::None =>
            "Arrows Move  Enter Select  H Help  Z Hammer  X Laser  C Blast  V Warp  B CBomb  Esc Menu  Q Quit".to_string(),
    };

    let padded = format!("{:<width$}", bar, width = term_w as usize);
    surface.print(0, term_h - 1, &padded, Style { attrs: Attrs::REVERSE, ..Default::default() });
}

// ── render_help ───────────────────────────────────────────────────────────

/// Full-screen help overlay. Any keypress will dismiss it.
pub fn render_help(stdout: &mut Stdout, engine: Option<&GameEngine>) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_help_inner(&mut surface, engine);
    surface.finish()
}

fn render_help_inner(surface: &mut dyn Surface, engine: Option<&GameEngine>) {
    let (tw, _th) = surface.size();
    let box_w = 54u16;
    let bx = (tw / 2).saturating_sub(box_w / 2);
    let by = 1u16;

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));
    surface.print(bx, by + 1, &format!("║{:^w$}║", "MATCH-3 HELP", w = box_w as usize - 2), bold());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    let col1_w = 16usize;
    let inner = box_w as usize - 2;
    let keys: &[(&str, &str)] = &[
        ("Arrow keys",     "Move cursor"),
        ("Enter / Space",  "Select gem / confirm swap"),
        ("Esc",            "Cancel selection / bonus / menu"),
        ("H",              "Show this help screen"),
        ("Q",              "Quit to menu"),
        ("Z  Hammer",      "Destroy one cell (requires targeting)"),
        ("X  Laser",       "Destroy entire row instantly"),
        ("C  Blaster",     "Destroy entire column instantly"),
        ("V  Warp",        "Shuffle the whole board"),
        ("B  Color Bomb",  "Clear all gems of one color"),
    ];

    for (i, (key, desc)) in keys.iter().enumerate() {
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

    let sep_y = by + 3 + keys.len() as u16;
    surface.print(bx, sep_y, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    // Active blessings
    surface.print(bx, sep_y + 1, "║", fg(Color::DarkGrey));
    surface.print(bx + 1, sep_y + 1, &format!("{:^w$}", "Active Blessings", w = inner), fg(Color::Cyan));
    surface.print(bx + 1 + inner as u16, sep_y + 1, "║", fg(Color::DarkGrey));

    let active: Vec<(&str, &str)> = ALL_BLESSINGS.iter()
        .filter(|b| engine.map_or(false, |e| e.blessing_flags.has_id(b.id)))
        .map(|b| (b.name, b.description))
        .collect();

    if active.is_empty() {
        surface.print(bx, sep_y + 2, &format!("║{:^w$}║", "none", w = inner), fg(Color::DarkGrey));
    }
    for (i, (name, desc)) in active.iter().enumerate() {
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

    let bless_rows = active.len().max(1) as u16;
    let sep2_y = sep_y + 2 + bless_rows;
    surface.print(bx, sep2_y, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    // Bonus inventory
    surface.print(bx, sep2_y + 1, "║", fg(Color::DarkGrey));
    surface.print(bx + 1, sep2_y + 1, &format!("{:^w$}", "Bonus Inventory", w = inner), fg(Color::Cyan));
    surface.print(bx + 1 + inner as u16, sep2_y + 1, "║", fg(Color::DarkGrey));

    let (h, l, bl, w, cb) = engine.map_or((0,0,0,0,0), |e| (
        e.bonuses.hammer as u32,
        e.bonuses.laser as u32,
        e.bonuses.blaster as u32,
        e.bonuses.warp as u32,
        e.bonuses.color_bomb as u32,
    ));
    let bonuses = [
        ("Hammer",     h),
        ("Laser",      l),
        ("Blaster",    bl),
        ("Warp",       w),
        ("Color Bomb", cb),
    ];
    for (i, (name, count)) in bonuses.iter().enumerate() {
        let y = sep2_y + 2 + i as u16;
        let line = format!("  {:<w$} x{}", name, count, w = col1_w);
        let pad = inner.saturating_sub(line.len());
        let color = if *count > 0 { Color::White } else { Color::DarkGrey };
        let mut cx = bx;
        surface.print(cx, y, "║", fg(Color::DarkGrey));
        cx += 1;
        surface.print(cx, y, &format!("{}{:>w$}", line, "", w = pad), fg(color));
        cx += (line.len() + pad) as u16;
        surface.print(cx, y, "║", fg(Color::DarkGrey));
    }

    let end_y = sep2_y + 2 + bonuses.len() as u16;
    surface.print(bx, end_y, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Cyan));
    surface.print(bx, end_y + 1, &format!("{:^w$}", "Press any key to close", w = box_w as usize), fg(Color::DarkGrey));
}

/// Render a celebration sweep overlay on the board.
///
/// Drawn on top of an already-rendered `do_render` frame, so this only
/// draws into a mid-frame `TermSurface` and does not clear/flush itself.
pub fn render_celebration(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_celebration_inner(&mut surface, engine, geo, tick);
    surface.done()
}

fn render_celebration_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) {
    use super::CELL_GAP;
    let sw = geo.scale * 2;
    let sh = geo.scale;
    let cols = engine.board.width;
    let rows = engine.board.height;
    let lit_col = (tick / 2) as usize % cols.max(1);
    let color = if (tick / 2) % 2 == 0 { Color::Yellow } else { Color::Green };
    let style = Style { fg: color, attrs: Attrs::BOLD, ..Default::default() };

    let cell_w = sw + CELL_GAP;
    for row in 0..rows {
        for sy in 0..sh {
            let y = geo.board_y + (row as u16) * (sh + CELL_GAP) + sy;
            let x = geo.board_x + (lit_col as u16) * cell_w;
            surface.print(x, y, &"✦".repeat(sw as usize), style);
        }
    }
}

/// Render the level-complete score summary screen (between campaign levels).
pub fn render_level_summary(
    stdout: &mut Stdout,
    score: u32,
    target: Option<u32>,
    level_num: usize,
    total_levels: usize,
    bonuses: &crate::bonuses::BonusInventory,
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_level_summary_inner(&mut surface, score, target, level_num, total_levels, bonuses);
    surface.finish()
}

fn render_level_summary_inner(
    surface: &mut dyn Surface,
    score: u32,
    target: Option<u32>,
    level_num: usize,
    total_levels: usize,
    bonuses: &crate::bonuses::BonusInventory,
) {
    let (tw, th) = surface.size();
    let box_w = 34u16;
    let bx = (tw / 2).saturating_sub(box_w / 2);
    let by = th / 4;
    let inner = box_w as usize - 2;

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), Style { fg: Color::Yellow, attrs: Attrs::BOLD, ..Default::default() });
    surface.print(bx, by + 1, &format!("║{:^w$}║", "LEVEL COMPLETE!", w = inner), bold());
    surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));

    let level_str = format!("Level {}/{}", level_num, total_levels);
    surface.print(bx, by + 3, &format!("║ {:<w$}║", level_str, w = inner - 1), fg(Color::White));

    let score_str = if let Some(t) = target {
        format!("Score: {} / {}", score, t)
    } else {
        format!("Score: {}", score)
    };
    surface.print(bx, by + 4, &format!("║ {:<w$}║", score_str, w = inner - 1), Style::default());

    let stars: u8 = if let Some(t) = target {
        if score >= t * 3 / 2 { 3 } else if score >= t { 2 } else { 1 }
    } else { 3 };
    let star_str = format!("Stars: {}{}", "★".repeat(stars as usize), "☆".repeat(3 - stars as usize));
    surface.print(bx, by + 5, &format!("║ {:<w$}║", star_str, w = inner - 1), fg(Color::Yellow));

    surface.print(bx, by + 6, &format!("╠{}╣", "═".repeat(box_w as usize - 2)), fg(Color::DarkGrey));
    surface.print(bx, by + 7, &format!("║{:^w$}║", "Bonuses Carried Over", w = inner), fg(Color::Cyan));

    let bonus_list = [
        ("Hammer",     bonuses.hammer as u32),
        ("Laser",      bonuses.laser as u32),
        ("Blaster",    bonuses.blaster as u32),
        ("Warp",       bonuses.warp as u32),
        ("Color Bomb", bonuses.color_bomb as u32),
    ];
    let mut row_off = 8u16;
    for (name, count) in &bonus_list {
        if *count > 0 {
            let s = format!("  {} x{}", name, count);
            surface.print(bx, by + row_off, &format!("║{:<w$}║", s, w = inner), fg(Color::White));
            row_off += 1;
        }
    }
    if row_off == 8 {
        surface.print(bx, by + row_off, &format!("║{:^w$}║", "none", w = inner), fg(Color::DarkGrey));
        row_off += 1;
    }

    surface.print(bx, by + row_off, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(Color::Yellow));
    surface.print(bx, by + row_off + 1, &format!("{:^w$}", "Press Enter to continue", w = box_w as usize), fg(Color::DarkGrey));
}

// ── render_game_over ──────────────────────────────────────────────────────

/// Game-over / won overlay.
///
/// Drawn on top of an already-rendered board frame (no clear of its own).
pub fn render_game_over(
    stdout: &mut Stdout,
    status: &GameStatus,
    score: u32,
) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_game_over_inner(&mut surface, status, score);
    surface.done()
}

fn render_game_over_inner(surface: &mut dyn Surface, status: &GameStatus, score: u32) {
    let (tw, th) = surface.size();
    let cx = tw / 2 - 10;
    let cy = th / 2 - 3;

    let title = match status {
        GameStatus::Won        => "  ★  YOU WIN!  ★  ",
        GameStatus::OutOfMoves => "  OUT OF MOVES  ",
        GameStatus::Stuck      => "  BOARD STUCK  ",
        GameStatus::Playing    => return,
    };

    surface.print(cx, cy, "╔══════════════════╗", Style::default());
    surface.print(cx, cy + 1, &format!("║{:^20}║", title), Style::default());
    surface.print(cx, cy + 2, &format!("║  Score: {:>10} ║", score), Style::default());
    surface.print(cx, cy + 3, "║                    ║", Style::default());
    surface.print(cx, cy + 4, "║ R Retry  Q Quit    ║", Style::default());
    surface.print(cx, cy + 5, "╚══════════════════╝", Style::default());
}

// ── render_main_menu ──────────────────────────────────────────────────────

/// Main menu screen.
pub fn render_main_menu(
    stdout: &mut Stdout,
    selected: usize,
    flash: Option<&str>,
) -> io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    render_main_menu_inner(&mut surface, selected, flash);
    surface.finish()
}

fn render_main_menu_inner(surface: &mut dyn Surface, selected: usize, flash: Option<&str>) {
    let (tw, th) = surface.size();
    let cx = tw / 2 - 12;
    let cy = th / 4;

    surface.print(cx, cy, "  ╔══════════════════════╗", Style::default());
    surface.print(cx, cy + 1, "  ║    m3tui  Match-3    ║", Style::default());
    surface.print(cx, cy + 2, "  ╚══════════════════════╝", Style::default());

    const ITEMS: &[&str] = &[
        "Quick Game",
        "Custom Game",
        "Campaign",
        "Endless",
        "Options",
        "Quit",
    ];

    for (i, item) in ITEMS.iter().enumerate() {
        let y = cy + 4 + i as u16;
        if i == selected {
            surface.print(cx + 2, y, &format!("► {:}", item), Style { attrs: Attrs::REVERSE, ..Default::default() });
        } else {
            surface.print(cx + 2, y, &format!("  {:}", item), Style::default());
        }
    }

    if let Some(msg) = flash {
        surface.print(cx, cy + 4 + ITEMS.len() as u16 + 1, &format!("  {}", msg), Style::default());
    }
}

// ── render_options ────────────────────────────────────────────────────────

/// Options screen (scale + color mode).
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
    let (tw, th) = surface.size();
    let cx = tw / 2 - 12;
    let cy = th / 4;

    surface.print(cx, cy, "  OPTIONS", Style::default());

    let fields = [
        format!("Scale:      {}", scale),
        format!("Color Mode: {}", color_mode),
        "Back".to_string(),
    ];

    for (i, field) in fields.iter().enumerate() {
        let y = cy + 2 + i as u16;
        if i == selected {
            surface.print(cx, y, &format!("► {}", field), Style { attrs: Attrs::REVERSE, ..Default::default() });
        } else {
            surface.print(cx, y, &format!("  {}", field), Style::default());
        }
    }

    surface.print(cx, cy + 2 + fields.len() as u16 + 1, "  ← → change value   Esc back", Style::default());
}

// ── render_blessing_selection ─────────────────────────────────────────────

const CARD_W: usize = 17;
const CARD_H: usize = 11;
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
    let grid_w = (CARD_W + 3) * CARD_COLS + 1;
    let grid_x = (term_w as usize).saturating_sub(grid_w) / 2;
    let grid_y = 2u16;

    for idx in 0..total_blessings {
        let b = &ALL_BLESSINGS[idx];
        let row = idx / CARD_COLS;
        let col = idx % CARD_COLS;
        let x = (grid_x + col * (CARD_W + 3)) as u16;
        let y = grid_y + (row as u16) * (CARD_H as u16 + 1);

        let is_cursor = idx == cursor;
        let is_chosen = chosen.contains(&idx);
        let unlocked = blessings::is_unlocked(b, completed_tracks);

        let (tl, tr, bl, br, hz, vt) = if is_chosen {
            ('╔', '╗', '╚', '╝', '═', '║')
        } else {
            ('┌', '┐', '└', '┘', '─', '│')
        };

        let highlight_style = if is_chosen {
            fg(Color::Green)
        } else if is_cursor {
            fg(Color::Yellow)
        } else {
            Style::default()
        };

        // Top border
        let top = format!("{}{}{}", tl, hz.to_string().repeat(CARD_W), tr);
        surface.print(x, y, &top, highlight_style);

        // Art lines (5 lines)
        for (ai, art_line) in b.ascii_art.iter().enumerate() {
            let padded = format!("{:^w$}", art_line, w = CARD_W);
            let line = format!("{}{}{}", vt, padded, vt);
            let style = if !unlocked { fg(Color::DarkGrey) } else { highlight_style };
            surface.print(x, y + 1 + ai as u16, &line, style);
        }

        // Name line
        let name_str = format!("{:^w$}", b.name, w = CARD_W);
        let name_line = format!("{}{}{}", vt, name_str, vt);
        let name_style = if !unlocked {
            fg(Color::DarkGrey)
        } else {
            Style { fg: highlight_style.fg, attrs: highlight_style.attrs | Attrs::BOLD, ..Default::default() }
        };
        surface.print(x, y + 6, &name_line, name_style);

        // Tier line
        let tier_label = if unlocked {
            format!("{:^w$}", format!("─ {} Tier ─", b.tier.label()), w = CARD_W)
        } else {
            let needed = blessings::tracks_required(b.tier);
            format!("{:^w$}", format!("Locked ({}+ tracks)", needed), w = CARD_W)
        };
        let tier_line = format!("{}{}{}", vt, tier_label, vt);
        let tier_style = if !unlocked { fg(Color::DarkGrey) } else { Style::default() };
        surface.print(x, y + 7, &tier_line, tier_style);

        // Description line
        let desc = format!("{:^w$}", b.description, w = CARD_W);
        let desc_line = format!("{}{}{}", vt, desc, vt);
        let desc_style = if !unlocked { fg(Color::DarkGrey) } else { Style::default() };
        surface.print(x, y + 8, &desc_line, desc_style);

        // Bottom border
        let bot = format!("{}{}{}", bl, hz.to_string().repeat(CARD_W), br);
        surface.print(x, y + 9, &bot, highlight_style);

        // Selection marker
        if is_chosen {
            let marker = format!("{:^w$}", "★ SELECTED", w = CARD_W + 2);
            surface.print(x, y + 10, &marker, fg(Color::Green));
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
    let status_style = if chosen.len() == 3 { fg(Color::Green) } else { fg(Color::DarkGrey) };
    surface.print(sx as u16, status_y, &status, status_style);
}
