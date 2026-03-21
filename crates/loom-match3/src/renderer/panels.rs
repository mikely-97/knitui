use std::io::{self, Write, Stdout};

use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Color, SetForegroundColor, ResetColor, SetAttribute, Attribute},
    terminal::{self, Clear, ClearType},
    cursor::{MoveTo, Hide},
};

use crate::blessings::{self, ALL_BLESSINGS};
use crate::bonuses::BonusState;
use crate::engine::{GameEngine, GameStatus};

use super::LayoutGeometry;

// ── render_hud ────────────────────────────────────────────────────────────

/// Render the HUD panel: score, moves, bonus inventory.
pub fn render_hud(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    objective_label: &str,
) -> io::Result<()> {
    let x = geo.hud_x;
    let mut y = geo.hud_y;

    let moves_left = engine.move_limit.saturating_sub(engine.moves_used);

    stdout.queue(MoveTo(x, y))?;
    stdout.queue(Print(format!("Score: {:>8}", engine.score)))?;
    y += 1;

    // Combo multiplier display
    if engine.combo_display_ticks > 0 {
        stdout.queue(MoveTo(x, y))?;
        stdout.queue(SetForegroundColor(Color::Yellow))?;
        stdout.queue(Print(format!("  x{} COMBO!", engine.cascade_depth + 1)))?;
        stdout.queue(ResetColor)?;
        y += 1;
    }

    stdout.queue(MoveTo(x, y))?;
    // Feature 1: warn when moves are running low
    if engine.move_limit > 0 && moves_left <= 5 {
        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(Print(format!("Moves:  {:>7}", moves_left)))?;
        stdout.queue(ResetColor)?;
    } else {
        stdout.queue(Print(format!("Moves:  {:>7}", moves_left)))?;
    }
    y += 1;

    if !objective_label.is_empty() {
        stdout.queue(MoveTo(x, y))?;
        stdout.queue(Print(format!("Goal: {}", objective_label)))?;
        y += 1;
    }

    y += 1;

    stdout.queue(MoveTo(x, y))?;
    let hammer_str     = format!("[Z] Hammer x{}", engine.bonuses.hammer);
    let laser_str      = format!("[X] Laser  x{}", engine.bonuses.laser);
    let blaster_str    = format!("[C] Blast  x{}", engine.bonuses.blaster);
    let warp_str       = format!("[V] Warp   x{}", engine.bonuses.warp);
    let color_bomb_str = format!("[B] CBomb  x{}", engine.bonuses.color_bomb);

    let dim_if_zero = |s: String, count: u16| {
        if count == 0 { format!("\x1b[2m{}\x1b[0m", s) } else { s }
    };

    stdout.queue(Print(dim_if_zero(hammer_str,  engine.bonuses.hammer)))?;
    stdout.queue(Print("  "))?;
    stdout.queue(Print(dim_if_zero(laser_str,   engine.bonuses.laser)))?;
    y += 1;
    stdout.queue(MoveTo(x, y))?;
    stdout.queue(Print(dim_if_zero(blaster_str, engine.bonuses.blaster)))?;
    stdout.queue(Print("  "))?;
    stdout.queue(Print(dim_if_zero(warp_str,    engine.bonuses.warp)))?;
    y += 1;
    stdout.queue(MoveTo(x, y))?;
    stdout.queue(Print(dim_if_zero(color_bomb_str, engine.bonuses.color_bomb)))?;

    Ok(())
}

// ── render_key_bar ────────────────────────────────────────────────────────

/// Render the persistent key bar at the bottom of the terminal.
pub fn render_key_bar(stdout: &mut Stdout, bonus_state: &BonusState) -> io::Result<()> {
    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    stdout.queue(MoveTo(0, term_h - 1))?;

    let bar = match bonus_state {
        BonusState::HammerActive { .. } =>
            "Arrows Move  Enter Destroy  Esc Cancel".to_string(),
        BonusState::ColorBombActive { .. } =>
            "Arrows Move  Enter Clear Color  Esc Cancel".to_string(),
        BonusState::None =>
            "Arrows Move  Enter Select  H Help  Z Hammer  X Laser  C Blast  V Warp  B CBomb  Esc Menu  Q Quit".to_string(),
    };

    let padded = format!("{:<width$}", bar, width = term_w as usize);
    stdout.queue(Print(padded.negative()))?;

    Ok(())
}

// ── render_help ───────────────────────────────────────────────────────────

/// Full-screen help overlay. Any keypress will dismiss it.
pub fn render_help(stdout: &mut Stdout, engine: Option<&GameEngine>) -> io::Result<()> {
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let box_w = 54u16;
    let bx = (tw / 2).saturating_sub(box_w / 2);
    let by = 1u16;

    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("║{:^w$}║", "MATCH-3 HELP", w = box_w as usize - 2)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

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

    let sep_y = by + 3 + keys.len() as u16;
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

    use crate::blessings::ALL_BLESSINGS;
    let active: Vec<(&str, &str)> = ALL_BLESSINGS.iter()
        .filter(|b| engine.map_or(false, |e| e.blessing_flags.has_id(b.id)))
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
        let remaining = inner.saturating_sub(1 + col1_w + 1);
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
        stdout.queue(MoveTo(bx, y))?;
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print("║"))?;
        if *count > 0 {
            stdout.queue(SetForegroundColor(Color::White))?;
        } else {
            stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        }
        let line = format!("  {:<w$} x{}", name, count, w = col1_w);
        let pad = inner.saturating_sub(line.len());
        stdout.queue(Print(format!("{}{:>w$}", line, "", w = pad)))?;
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

    Ok(())
}

/// Render a celebration sweep overlay on the board.
pub fn render_celebration(
    stdout: &mut Stdout,
    engine: &GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) -> io::Result<()> {
    use super::CELL_GAP;
    let sw = geo.scale * 2;
    let sh = geo.scale;
    let cols = engine.board.width;
    let rows = engine.board.height;
    let lit_col = (tick / 2) as usize % cols.max(1);
    let color = if (tick / 2) % 2 == 0 { Color::Yellow } else { Color::Green };

    stdout.queue(SetForegroundColor(color))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    let cell_w = sw + CELL_GAP;
    for row in 0..rows {
        for sy in 0..sh {
            let y = geo.board_y + (row as u16) * (sh + CELL_GAP) + sy;
            let x = geo.board_x + (lit_col as u16) * cell_w;
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

/// Render the level-complete score summary screen (between campaign levels).
pub fn render_level_summary(
    stdout: &mut Stdout,
    score: u32,
    target: Option<u32>,
    level_num: usize,
    total_levels: usize,
    bonuses: &crate::bonuses::BonusInventory,
) -> io::Result<()> {
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let box_w = 34u16;
    let bx = (tw / 2).saturating_sub(box_w / 2);
    let by = th / 4;
    let inner = box_w as usize - 2;

    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    stdout.queue(MoveTo(bx, by))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(SetAttribute(Attribute::Bold))?;
    stdout.queue(Print(format!("╔{}╗", "═".repeat(box_w as usize - 2))))?;
    stdout.queue(MoveTo(bx, by + 1))?;
    stdout.queue(Print(format!("║{:^w$}║", "LEVEL COMPLETE!", w = inner)))?;
    stdout.queue(SetAttribute(Attribute::Reset))?;
    stdout.queue(MoveTo(bx, by + 2))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 3))?;
    stdout.queue(SetForegroundColor(Color::White))?;
    let level_str = format!("Level {}/{}", level_num, total_levels);
    stdout.queue(Print(format!("║ {:<w$}║", level_str, w = inner - 1)))?;

    stdout.queue(MoveTo(bx, by + 4))?;
    let score_str = if let Some(t) = target {
        format!("Score: {} / {}", score, t)
    } else {
        format!("Score: {}", score)
    };
    stdout.queue(Print(format!("║ {:<w$}║", score_str, w = inner - 1)))?;

    let stars: u8 = if let Some(t) = target {
        if score >= t * 3 / 2 { 3 } else if score >= t { 2 } else { 1 }
    } else { 3 };
    stdout.queue(MoveTo(bx, by + 5))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    let star_str = format!("Stars: {}{}", "★".repeat(stars as usize), "☆".repeat(3 - stars as usize));
    stdout.queue(Print(format!("║ {:<w$}║", star_str, w = inner - 1)))?;

    stdout.queue(MoveTo(bx, by + 6))?;
    stdout.queue(SetForegroundColor(Color::DarkGrey))?;
    stdout.queue(Print(format!("╠{}╣", "═".repeat(box_w as usize - 2))))?;

    stdout.queue(MoveTo(bx, by + 7))?;
    stdout.queue(SetForegroundColor(Color::Cyan))?;
    stdout.queue(Print(format!("║{:^w$}║", "Bonuses Carried Over", w = inner)))?;

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
            stdout.queue(MoveTo(bx, by + row_off))?;
            stdout.queue(SetForegroundColor(Color::White))?;
            let s = format!("  {} x{}", name, count);
            stdout.queue(Print(format!("║{:<w$}║", s, w = inner)))?;
            row_off += 1;
        }
    }
    if row_off == 8 {
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

// ── render_game_over ──────────────────────────────────────────────────────

/// Game-over / won overlay.
pub fn render_game_over(
    stdout: &mut Stdout,
    status: &GameStatus,
    score: u32,
) -> io::Result<()> {
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let cx = tw / 2 - 10;
    let cy = th / 2 - 3;

    let title = match status {
        GameStatus::Won        => "  ★  YOU WIN!  ★  ",
        GameStatus::OutOfMoves => "  OUT OF MOVES  ",
        GameStatus::Stuck      => "  BOARD STUCK  ",
        GameStatus::Playing    => return Ok(()),
    };

    stdout.queue(MoveTo(cx, cy))?;
    stdout.queue(Print(format!("╔══════════════════╗")))?;
    stdout.queue(MoveTo(cx, cy + 1))?;
    stdout.queue(Print(format!("║{:^20}║", title)))?;
    stdout.queue(MoveTo(cx, cy + 2))?;
    stdout.queue(Print(format!("║  Score: {:>10} ║", score)))?;
    stdout.queue(MoveTo(cx, cy + 3))?;
    stdout.queue(Print(format!("║                    ║")))?;
    stdout.queue(MoveTo(cx, cy + 4))?;
    stdout.queue(Print(format!("║ R Retry  Q Quit    ║")))?;
    stdout.queue(MoveTo(cx, cy + 5))?;
    stdout.queue(Print(format!("╚══════════════════╝")))?;

    Ok(())
}

// ── render_main_menu ──────────────────────────────────────────────────────

/// Main menu screen.
pub fn render_main_menu(
    stdout: &mut Stdout,
    selected: usize,
    flash: Option<&str>,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let cx = tw / 2 - 12;
    let cy = th / 4;

    stdout.queue(MoveTo(cx, cy))?;
    stdout.queue(Print("  ╔══════════════════════╗"))?;
    stdout.queue(MoveTo(cx, cy + 1))?;
    stdout.queue(Print("  ║    m3tui  Match-3    ║"))?;
    stdout.queue(MoveTo(cx, cy + 2))?;
    stdout.queue(Print("  ╚══════════════════════╝"))?;

    const ITEMS: &[&str] = &[
        "Quick Game",
        "Custom Game",
        "Campaign",
        "Endless",
        "Options",
        "Quit",
    ];

    for (i, item) in ITEMS.iter().enumerate() {
        stdout.queue(MoveTo(cx + 2, cy + 4 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {:}", item).negative()))?;
        } else {
            stdout.queue(Print(format!("  {:}", item)))?;
        }
    }

    if let Some(msg) = flash {
        stdout.queue(MoveTo(cx, cy + 4 + ITEMS.len() as u16 + 1))?;
        stdout.queue(Print(format!("  {}", msg)))?;
    }

    Ok(())
}

// ── render_options ────────────────────────────────────────────────────────

/// Options screen (scale + color mode).
pub fn render_options(
    stdout: &mut Stdout,
    selected: usize,
    scale: u16,
    color_mode: &str,
) -> io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;
    let (tw, th) = terminal::size().unwrap_or((80, 24));
    let cx = tw / 2 - 12;
    let cy = th / 4;

    stdout.queue(MoveTo(cx, cy))?;
    stdout.queue(Print("  OPTIONS"))?;

    let fields = [
        format!("Scale:      {}", scale),
        format!("Color Mode: {}", color_mode),
        "Back".to_string(),
    ];

    for (i, field) in fields.iter().enumerate() {
        stdout.queue(MoveTo(cx, cy + 2 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {}", field).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", field)))?;
        }
    }

    stdout.queue(MoveTo(cx, cy + 2 + fields.len() as u16 + 1))?;
    stdout.queue(Print("  ← → change value   Esc back"))?;

    Ok(())
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
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, _term_h) = terminal::size().unwrap_or((80, 24));
    let total_blessings = ALL_BLESSINGS.len();
    let rows = (total_blessings + CARD_COLS - 1) / CARD_COLS;

    // Title
    let title = "═══ CHOOSE 3 BLESSINGS ═══";
    let title_x = term_w.saturating_sub(title.len() as u16) / 2;
    stdout.queue(MoveTo(title_x, 0))?;
    stdout.queue(Print(title))?;

    // Grid origin
    let grid_w = (CARD_W + 3) * CARD_COLS + 1;
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

        let (tl, tr, bl, br, hz, vt) = if is_chosen {
            ('╔', '╗', '╚', '╝', '═', '║')
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

    stdout.flush()
}
