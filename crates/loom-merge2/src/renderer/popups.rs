use loom_engine::render::{Attrs, Color, Style, Surface};

fn fg(color: Color) -> Style {
    Style { fg: color, ..Default::default() }
}

fn bold(color: Color) -> Style {
    Style { fg: color, attrs: Attrs::BOLD, ..Default::default() }
}

// ── Help overlay ──────────────────────────────────────────────────────────

/// Portable counterpart to [`render_help`] — the entry point the
/// `GameEngine` trait adapter uses (no `Stdout`/frame lifecycle available
/// from a portable caller).
pub fn render_help_to_surface(
    surface: &mut dyn Surface,
    help_lines: &[(&str, &str)],
    engine: Option<&crate::engine::GameEngine>,
) {
    render_help_inner(surface, help_lines, engine);
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

/// Portable counterpart to [`render_celebration`] — see [`render_help_to_surface`].
pub fn render_celebration_to_surface(
    surface: &mut dyn Surface,
    engine: &crate::engine::GameEngine,
    geo: &super::LayoutGeometry,
    tick: u8,
) {
    render_celebration_inner(surface, engine, geo, tick);
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

