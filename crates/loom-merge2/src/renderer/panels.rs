use std::io::{self, Stdout};

use loom_engine::render::{Attrs, Color, Style, Surface};
use loom_engine_term::TermSurface;

use crate::board::Cell;
use crate::engine::{GameEngine, GameStatus};
use crate::glyphs;
use crate::order::OrderType;
use super::LayoutGeometry;

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

fn bold(color: Color) -> Style {
    Style { fg: color, attrs: Attrs::BOLD, ..Default::default() }
}

// ── HUD (score / energy / stars) ─────────────────────────────────────────

/// Only ever called from inside tui.rs's centralized Clear/flush dispatcher,
/// so this draws into a mid-frame `TermSurface` (no clear/flush of its own).
pub fn render_hud(stdout: &mut Stdout, engine: &GameEngine, label: &str) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_hud_inner(&mut surface, engine, label);
    surface.done()
}

fn render_hud_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    label: &str,
) {
    // Row 0: game label
    surface.print(1, 0, label, bold(Color::White));

    // Row 1: Score  ⚡NN/NN [bar] +Xs  ★NN
    let mut x = 1u16;
    let score_str = format!("Score: {:>7}", engine.score);
    surface.print(x, 1, &score_str, bold(Color::Yellow));
    x += score_str.chars().count() as u16;

    // Energy bar
    let e = &engine.energy;
    let bar_total = 10usize;
    let filled = if e.max > 0 {
        ((e.current as usize * bar_total) / e.max as usize).min(bar_total)
    } else {
        bar_total
    };
    let bar: String = "█".repeat(filled) + &"░".repeat(bar_total - filled);
    let secs = e.secs_until_next();
    let regen_str = if e.is_full() {
        "  full ".to_string()
    } else {
        format!(" +{}s ", secs)
    };

    x += 2;
    surface.print(x, 1, "  ", Style::default());
    let energy_str = format!("⚡{}/{}", e.current, e.max);
    surface.print(x, 1, &energy_str, fg(Color::Cyan));
    x += energy_str.chars().count() as u16;
    let bar_str = format!(" [{}]", bar);
    surface.print(x, 1, &bar_str, fg(Color::DarkGrey));
    x += bar_str.chars().count() as u16;
    surface.print(x, 1, &regen_str, fg(Color::DarkGrey));
    x += regen_str.chars().count() as u16;

    // Stars
    let star_str = format!(" ★{}", engine.stars);
    surface.print(x, 1, &star_str, fg(Color::Yellow));
    x += star_str.chars().count() as u16;

    // Ad hint
    if engine.can_watch_ad() {
        let next_reward = crate::ad::reward_for_use(engine.ads_used, &engine.available_families);
        let ad_str = format!("  [A] {}", crate::ad::hud_label(&next_reward)
            .trim_start_matches("[AD] ")
            .trim_end_matches(" — press A"));
        surface.print(x, 1, &ad_str, fg(Color::Magenta));
    }
}

// ── Orders panel ─────────────────────────────────────────────────────────

pub fn render_orders(stdout: &mut Stdout, engine: &GameEngine, geo: &LayoutGeometry) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_orders_inner(&mut surface, engine, geo);
    surface.done()
}

fn render_orders_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) {
    let x = geo.order_x;
    let mut y = geo.order_y;

    let story: Vec<_>  = engine.active_orders.iter()
        .filter(|o| matches!(o.order_type, OrderType::Story)).collect();
    let random: Vec<_> = engine.active_orders.iter()
        .filter(|o| matches!(o.order_type, OrderType::Random)).collect();
    let timed: Vec<_>  = engine.active_orders.iter()
        .filter(|o| matches!(o.order_type, OrderType::TimeLimited { .. })).collect();

    let panel_w = 22usize;
    let inner_w = panel_w - 2;

    let render_section = |surface: &mut dyn Surface, label: &str, orders: &[&&crate::order::Order], y: &mut u16| {
        if orders.is_empty() { return; }

        surface.print(x, *y, label, bold(Color::White));
        *y += 1;

        surface.print(x, *y, &format!("┌{}┐", "─".repeat(inner_w)), fg(Color::DarkGrey));
        *y += 1;

        for order in orders.iter() {
            // Timed countdown
            if let OrderType::TimeLimited { ticks_remaining } = &order.order_type {
                let secs = ticks_remaining / 5; // 200ms ticks
                let content = format!("⏱ {:>3}s{}", secs, " ".repeat(inner_w.saturating_sub(8)));
                surface.print(x, *y, "│", fg(Color::DarkGrey));
                surface.print(x + 1, *y, &content, fg(Color::Yellow));
                surface.print(x + 1 + content.chars().count() as u16, *y, "│", fg(Color::DarkGrey));
                *y += 1;
            }

            for req in &order.requirements {
                let done = req.delivered;
                let need = req.required;
                let name = req.family.tier_name(req.tier);
                let fam_color = glyphs::family_color(req.family);

                let progress = format!("[{}/{}]", done, need);
                let glyph = req.family.glyph(req.tier);
                let desc = format!("{}×{} {}", need - done, glyph, name);
                let line = format!(" {:<13}{:>5} ", desc, progress);
                let line = if line.chars().count() > inner_w {
                    line.chars().take(inner_w).collect::<String>()
                } else {
                    format!("{:<w$}", line, w = inner_w)
                };

                let content_color = if done >= need { Color::Green } else { fam_color };
                surface.print(x, *y, "│", fg(Color::DarkGrey));
                surface.print(x + 1, *y, &line, fg(content_color));
                surface.print(x + 1 + inner_w as u16, *y, "│", fg(Color::DarkGrey));
                *y += 1;
            }

            // Rewards preview
            let reward_str: String = order.rewards.iter().map(|r| {
                use crate::order::Reward;
                match r {
                    Reward::Score(n)       => format!("+{}pts ", n),
                    Reward::Energy(n)      => format!("+{}⚡ ", n),
                    Reward::Stars(n)       => format!("+{}★ ", n),
                    Reward::InventorySlot  => "+inv ".to_string(),
                    Reward::SpawnPiece(_)  => "+item ".to_string(),
                }
            }).collect();
            if !reward_str.is_empty() {
                let rs_raw = format!(" {}", reward_str.trim());
                let rs = if rs_raw.chars().count() >= inner_w {
                    rs_raw.chars().take(inner_w).collect::<String>()
                } else {
                    format!("{:<w$}", rs_raw, w = inner_w)
                };
                surface.print(x, *y, "│", fg(Color::DarkGrey));
                surface.print(x + 1, *y, &format!("{:<w$}", rs, w = inner_w), fg(Color::Yellow));
                surface.print(x + 1 + inner_w as u16, *y, "│", fg(Color::DarkGrey));
                *y += 1;
            }

            // Follow-up indicator
            if order.follow_up.is_some() {
                let fu_line = format!("{:<w$}", " → more", w = inner_w);
                surface.print(x, *y, "│", fg(Color::DarkGrey));
                surface.print(x + 1, *y, &fu_line, fg(Color::Cyan));
                surface.print(x + 1 + inner_w as u16, *y, "│", fg(Color::DarkGrey));
                *y += 1;
            }
        }

        surface.print(x, *y, &format!("└{}┘", "─".repeat(inner_w)), fg(Color::DarkGrey));
        *y += 1;
    };

    if !story.is_empty() {
        render_section(surface, "STORY ORDERS", &story.iter().collect::<Vec<_>>().as_slice(), &mut y);
    }
    if !random.is_empty() {
        render_section(surface, "ORDERS      ", &random.iter().collect::<Vec<_>>().as_slice(), &mut y);
    }
    if !timed.is_empty() {
        render_section(surface, "TIMED       ", &timed.iter().collect::<Vec<_>>().as_slice(), &mut y);
    }
}

// ── Inventory strip ───────────────────────────────────────────────────────

pub fn render_inventory(stdout: &mut Stdout, engine: &GameEngine, geo: &LayoutGeometry, selected_slot: Option<usize>) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_inventory_inner(&mut surface, engine, geo, selected_slot);
    surface.done()
}

fn render_inventory_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &LayoutGeometry,
    selected_slot: Option<usize>,
) {
    let y = geo.inventory_y(engine);
    let x = geo.board_x;

    let used = engine.inventory.used_count();
    let total = engine.inventory.slot_count();
    let header = format!("Inv [{}/{}]: ", used, total);
    surface.print(x, y, &header, fg(Color::DarkGrey));
    let mut cx = x + header.chars().count() as u16;

    for (i, slot) in engine.inventory.slots.iter().enumerate() {
        let is_sel = selected_slot == Some(i);
        let bg = if is_sel { Color::Rgb { r: 0, g: 60, b: 0 } } else { Color::Reset };
        match slot {
            None => {
                surface.print(cx, y, "[   ]", Style { fg: Color::DarkGrey, bg, ..Default::default() });
                cx += 5;
            }
            Some(piece) => {
                let (label, color, _) = glyphs::cell_label(&Cell::Piece(piece.clone()));
                let text = format!("[{:<3}]", label);
                surface.print(cx, y, &text, Style { fg: color, bg, ..Default::default() });
                cx += text.chars().count() as u16;
            }
        }
        surface.print(cx, y, " ", fg(Color::DarkGrey));
        cx += 1;
    }
}

// ── Key bar ───────────────────────────────────────────────────────────────

pub fn render_key_bar(stdout: &mut Stdout, engine: &GameEngine, geo: &LayoutGeometry) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_key_bar_inner(&mut surface, engine, geo);
    surface.done()
}

fn render_key_bar_inner(
    surface: &mut dyn Surface,
    engine: &GameEngine,
    geo: &LayoutGeometry,
) {
    let y = geo.key_bar_y(engine);

    let hints = [
        ("↑↓←→", "Move"),
        ("Enter", "Select/Merge"),
        ("D", "Deliver"),
        ("S", "Store"),
        ("I", "Inventory"),
        ("U", "Upgrade gen"),
        ("A", "Ad"),
        ("H", "Help"),
        ("Q", "Quit"),
    ];
    let line: String = hints.iter()
        .map(|(k, v)| format!("{} {} ", k, v))
        .collect();
    surface.print(1, y, &line, fg(Color::DarkGrey));
}

// ── Game over overlay ─────────────────────────────────────────────────────

pub fn render_game_over(stdout: &mut Stdout, status: &GameStatus, score: u32) -> io::Result<()> {
    let mut surface = TermSurface::new(stdout);
    render_game_over_inner(&mut surface, status, score);
    surface.done()
}

fn render_game_over_inner(
    surface: &mut dyn Surface,
    status: &GameStatus,
    score: u32,
) {
    let (term_w, term_h) = surface.size();
    let cx = term_w / 2;
    let cy = term_h / 2;
    let box_w = 26u16;
    let bx = cx.saturating_sub(box_w / 2);
    let by = cy.saturating_sub(3);

    let (title, color) = match status {
        GameStatus::Won    => ("MISSION COMPLETE!", Color::Green),
        GameStatus::Lost   => ("BOARD FULL!",       Color::Red),
        GameStatus::Stuck  => ("STUCK!",             Color::Yellow),
        GameStatus::Playing => return,
    };

    surface.print(bx, by, &format!("╔{}╗", "═".repeat(box_w as usize - 2)), fg(color));

    let msg = format!("{:^w$}", title, w = box_w as usize - 2);
    surface.print(bx, by + 1, &format!("║{}║", msg), Style { attrs: Attrs::BOLD, ..Default::default() });

    let sc = format!("Score: {:>8}", score);
    surface.print(bx, by + 2, &format!("║{:^w$}║", sc, w = box_w as usize - 2), fg(color));

    surface.print(bx, by + 3, &format!("║{:^w$}║", "Enter: menu  Q: quit", w = box_w as usize - 2), fg(Color::DarkGrey));

    surface.print(bx, by + 4, &format!("╚{}╝", "═".repeat(box_w as usize - 2)), fg(color));
}
