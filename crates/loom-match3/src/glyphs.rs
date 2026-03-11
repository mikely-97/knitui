use crate::board::{Orientation, SpecialPiece, TileModifier};

// ── Gem (normal colored block) ────────────────────────────────────────────

/// Normal gem: spool shape (▄███▄ / ████ / ▀███▀).
pub fn gem_glyph(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    if sh <= 1 {
        return vec!["██".to_string()];
    }
    let mut rows = Vec::with_capacity(sh);
    rows.push(format!("▄{}▄", "█".repeat(sw - 2)));
    for _ in 1..sh - 1 {
        rows.push("█".repeat(sw));
    }
    rows.push(format!("▀{}▀", "█".repeat(sw - 2)));
    rows
}

// ── Special piece glyphs ──────────────────────────────────────────────────

/// Gem with a special-piece marker in the center.
pub fn special_glyph(sp: &SpecialPiece, scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    let marker: &str = match sp {
        SpecialPiece::LineBomb(Orientation::Horizontal) => "──",
        SpecialPiece::LineBomb(Orientation::Vertical)   => "│ ",
        SpecialPiece::ColorBomb                         => "✦ ",
        SpecialPiece::AreaBomb { radius: 2 }            => "⊛⊛",
        SpecialPiece::AreaBomb { .. }                   => "⊛ ",
    };

    if sh <= 1 {
        let m: String = marker.chars().take(2).collect();
        return vec![format!("{:<2}", m)];
    }

    let mut rows = Vec::with_capacity(sh);
    rows.push(format!("▄{}▄", "█".repeat(sw - 2)));

    let mid = sh / 2;
    for i in 1..sh - 1 {
        if i == mid {
            let side = (sw - 2) / 2;
            rows.push(format!(
                "{}{}{}",
                "█".repeat(side),
                marker.chars().take(2).collect::<String>(),
                "█".repeat(sw - 2 - side),
            ));
        } else {
            rows.push("█".repeat(sw));
        }
    }
    rows.push(format!("▀{}▀", "█".repeat(sw - 2)));
    rows
}

// ── Tile modifier overlays ────────────────────────────────────────────────

/// Full-cell overlay rendered ON TOP of the gem glyph (or replacing it for Stone).
pub fn modifier_overlay(modifier: &TileModifier, scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;

    match modifier {
        TileModifier::Ice { hp } => {
            let marker = if *hp > 1 { "❄" } else { "·" };
            if sh <= 1 {
                return vec![format!("{:<2}", marker)];
            }
            let mut rows = vec!["░".repeat(sw); sh];
            rows[0] = format!("{}{}", marker, "░".repeat(sw - 1));
            rows
        }
        TileModifier::Stone => {
            vec!["▓".repeat(sw); sh]
        }
        TileModifier::Crate { hp } => {
            let digit = hp.to_string();
            if sh <= 1 {
                return vec![format!("▢{}", digit.chars().next().unwrap_or(' '))];
            }
            let inner_w = sw - 2;
            let mut rows = Vec::with_capacity(sh);
            rows.push(format!("┌{}┐", "─".repeat(inner_w)));
            for i in 1..sh - 1 {
                if i == sh / 2 {
                    let side = inner_w / 2;
                    rows.push(format!(
                        "│{}{}{}│",
                        " ".repeat(side),
                        digit.chars().next().unwrap_or(' '),
                        " ".repeat(inner_w - side - 1),
                    ));
                } else {
                    rows.push(format!("│{}│", " ".repeat(inner_w)));
                }
            }
            rows.push(format!("└{}┘", "─".repeat(inner_w)));
            rows
        }
        TileModifier::Locked => {
            if sh <= 1 {
                return vec!["≡≡".to_string()];
            }
            let mut rows = vec!["█".repeat(sw); sh];
            let mid = sh / 2;
            let marker = "⛓ ";
            let side = (sw - 2) / 2;
            rows[mid] = format!(
                "{}{}{}",
                "█".repeat(side),
                marker.chars().take(2).collect::<String>(),
                "█".repeat(sw - 2 - side),
            );
            rows
        }
    }
}

// ── Empty & cursor ────────────────────────────────────────────────────────

/// Empty cell: spaces.
pub fn empty_glyph(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    vec![" ".repeat(sw); scale as usize]
}

/// Cursor highlight: corner brackets around the cell area.
pub fn cursor_glyph(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    if sh <= 1 {
        return vec!["[]".to_string()];
    }
    let mut rows = Vec::with_capacity(sh);
    rows.push(format!("╔{}╗", "═".repeat(sw - 2)));
    for _ in 1..sh - 1 {
        rows.push(format!("║{}║", " ".repeat(sw - 2)));
    }
    rows.push(format!("╚{}╝", "═".repeat(sw - 2)));
    rows
}

/// Selection highlight: same as cursor but different character.
pub fn selected_glyph(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    if sh <= 1 {
        return vec!["<>".to_string()];
    }
    let mut rows = Vec::with_capacity(sh);
    rows.push(format!("┌{}┐", "─".repeat(sw - 2)));
    for _ in 1..sh - 1 {
        rows.push(format!("│{}│", " ".repeat(sw - 2)));
    }
    rows.push(format!("└{}┘", "─".repeat(sw - 2)));
    rows
}

/// Bouncing cells: displayed dimmed to indicate the invalid swap.
pub fn bounce_glyph(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    vec!["░".repeat(sw); scale as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Orientation, SpecialPiece, TileModifier};

    fn assert_glyph_dimensions(rows: &[String], scale: u16) {
        let sh = scale as usize;
        let sw = (scale * 2) as usize;
        assert_eq!(rows.len(), sh, "Wrong row count at scale {scale}: got {}", rows.len());
        for (i, row) in rows.iter().enumerate() {
            let w = row.chars().count();
            assert_eq!(w, sw, "Row {i} has width {w}, expected {sw} at scale {scale}");
        }
    }

    // ── Gem glyphs ──────────────────────────────────────────────────────────

    #[test]
    fn gem_glyph_scale1_dimensions() {
        assert_glyph_dimensions(&gem_glyph(1), 1);
    }

    #[test]
    fn gem_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&gem_glyph(2), 2);
    }

    #[test]
    fn gem_glyph_scale3_dimensions() {
        assert_glyph_dimensions(&gem_glyph(3), 3);
    }

    #[test]
    fn gem_glyph_scale4_dimensions() {
        assert_glyph_dimensions(&gem_glyph(4), 4);
    }

    #[test]
    fn gem_glyph_scale5_dimensions() {
        assert_glyph_dimensions(&gem_glyph(5), 5);
    }

    // ── Special piece glyphs ────────────────────────────────────────────────

    #[test]
    fn line_bomb_h_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&special_glyph(&SpecialPiece::LineBomb(Orientation::Horizontal), 2), 2);
    }

    #[test]
    fn line_bomb_v_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&special_glyph(&SpecialPiece::LineBomb(Orientation::Vertical), 2), 2);
    }

    #[test]
    fn color_bomb_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&special_glyph(&SpecialPiece::ColorBomb, 2), 2);
    }

    #[test]
    fn area_bomb_r1_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&special_glyph(&SpecialPiece::AreaBomb { radius: 1 }, 2), 2);
    }

    #[test]
    fn area_bomb_r2_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&special_glyph(&SpecialPiece::AreaBomb { radius: 2 }, 2), 2);
    }

    // ── Modifier overlay ────────────────────────────────────────────────────

    #[test]
    fn modifier_overlay_ice_scale1() {
        let overlay = modifier_overlay(&TileModifier::Ice { hp: 2 }, 1);
        assert_eq!(overlay.len(), 1);
        assert_eq!(overlay[0].chars().count(), 2);
    }

    #[test]
    fn modifier_overlay_stone_scale2() {
        let overlay = modifier_overlay(&TileModifier::Stone, 2);
        assert_glyph_dimensions(&overlay, 2);
    }

    #[test]
    fn modifier_overlay_crate_scale2() {
        let overlay = modifier_overlay(&TileModifier::Crate { hp: 3 }, 2);
        assert_glyph_dimensions(&overlay, 2);
    }

    #[test]
    fn modifier_overlay_locked_scale2() {
        let overlay = modifier_overlay(&TileModifier::Locked, 2);
        assert_glyph_dimensions(&overlay, 2);
    }

    // ── Empty / cursor ──────────────────────────────────────────────────────

    #[test]
    fn empty_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&empty_glyph(2), 2);
    }

    #[test]
    fn cursor_glyph_scale2_dimensions() {
        assert_glyph_dimensions(&cursor_glyph(2), 2);
    }
}
