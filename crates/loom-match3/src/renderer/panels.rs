use loom_engine::render::{Attrs, Color, Style, Surface};

use crate::blessings::ALL_BLESSINGS;
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

/// Portable counterpart to [`render_help`] — the entry point the
/// `GameEngine` trait adapter uses (no `Stdout`/frame lifecycle available
/// from a portable caller).
pub fn render_help_to_surface(surface: &mut dyn Surface, engine: Option<&GameEngine>) {
    render_help_inner(surface, engine);
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

/// Portable counterpart to [`render_celebration`] — see [`render_help_to_surface`].
pub fn render_celebration_to_surface(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) {
    render_celebration_inner(surface, engine, geo, tick);
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

// ── render_game_over ──────────────────────────────────────────────────────

/// Portable counterpart to [`render_game_over`], with an optional message
/// override (e.g. campaign level-progress text) that replaces the whole box
/// with a single status line — mirrors loom-knit's `draw_overlay_to_surface`.
pub fn render_game_over_to_surface(
    surface: &mut dyn Surface,
    status: &GameStatus,
    score: u32,
    overlay_msg: Option<&str>,
) {
    if let Some(msg) = overlay_msg {
        surface.print(0, 0, msg, Style::default());
        return;
    }
    render_game_over_inner(surface, status, score);
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

