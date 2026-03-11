use crate::board::Cell;
use crate::item::Item;

/// Single-character glyph for an item tier (scale 1).
pub fn tier_glyph(tier: u8) -> &'static str {
    match tier {
        1 => "·",
        2 => "○",
        3 => "●",
        4 => "◆",
        5 => "★",
        _ => "?",
    }
}

/// Multi-line glyph for an item at a given scale.
/// Returns `scale` rows, each `scale * 2` characters wide.
pub fn item_glyph(item: &Item, scale: u16) -> Vec<String> {
    let w = (scale * 2) as usize;
    let h = scale as usize;
    let symbol = tier_glyph(item.tier);

    if h == 1 {
        return vec![format!("{symbol} ")];
    }

    let mut rows = Vec::with_capacity(h);
    let mid = h / 2;
    for r in 0..h {
        if r == 0 {
            rows.push(format!("▄{}▄", "▄".repeat(w.saturating_sub(2))));
        } else if r == h - 1 {
            rows.push(format!("▀{}▀", "▀".repeat(w.saturating_sub(2))));
        } else if r == mid {
            let pad = (w.saturating_sub(2)) / 2;
            let sym = format!("{}{}{}", " ".repeat(pad), symbol, " ".repeat(w.saturating_sub(2).saturating_sub(pad + 1)));
            rows.push(format!("│{}│", sym));
        } else {
            rows.push(format!("│{}│", " ".repeat(w.saturating_sub(2))));
        }
    }
    rows
}

/// Glyph for a generator cell.
pub fn generator_glyph(scale: u16) -> Vec<String> {
    let w = (scale * 2) as usize;
    let h = scale as usize;
    if h == 1 {
        return vec!["G ".to_string()];
    }
    let mut rows = Vec::with_capacity(h);
    let mid = h / 2;
    for r in 0..h {
        if r == mid {
            let pad = (w.saturating_sub(2)) / 2;
            let sym = format!("{}{}{}", " ".repeat(pad), "G", " ".repeat(w.saturating_sub(2).saturating_sub(pad + 1)));
            rows.push(format!("╔{}╗", sym));
        } else {
            rows.push(format!("║{}║", " ".repeat(w.saturating_sub(2))));
        }
    }
    rows
}

/// Glyph for a blocked cell.
pub fn blocked_glyph(scale: u16) -> Vec<String> {
    let w = (scale * 2) as usize;
    let h = scale as usize;
    if h == 1 {
        return vec!["▓▓".to_string()];
    }
    (0..h).map(|_| "▓".repeat(w)).collect()
}

/// Glyph for an empty cell.
pub fn empty_glyph(scale: u16) -> Vec<String> {
    let w = (scale * 2) as usize;
    let h = scale as usize;
    if h == 1 {
        return vec!["  ".to_string()];
    }
    (0..h).map(|_| " ".repeat(w)).collect()
}

/// Get the appropriate glyph rows for a cell.
pub fn cell_glyph(cell: &Cell, scale: u16) -> Vec<String> {
    match cell {
        Cell::Item(item) => item_glyph(item, scale),
        Cell::Empty => empty_glyph(scale),
        Cell::Generator { .. } => generator_glyph(scale),
        Cell::Blocked => blocked_glyph(scale),
    }
}
