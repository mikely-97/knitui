use crossterm::style::Color;

use crate::board::Cell;
use crate::item::{Family, Piece};

/// Terminal foreground color for a given item family.
pub fn family_color(family: Family) -> Color {
    match family {
        Family::Wood    => Color::Green,
        Family::Stone   => Color::White,
        Family::Metal   => Color::Cyan,
        Family::Cloth   => Color::Magenta,
        Family::Crystal => Color::Yellow,
        Family::Ember   => Color::Red,
    }
}

/// Cell inner dimensions (width in columns, height in rows) at a given scale.
pub fn cell_dims(scale: u16) -> (usize, usize) {
    let w = scale as usize * 2 + 2; // 4 at s=1, 6 at s=2, 8 at s=3
    let h = scale as usize;          // 1 at s=1, 2 at s=2, 3 at s=3
    (w, h)
}

/// Returns `(label, fg_color, bold)` for a cell at scale 1.
/// `label` fits in two visible characters (may be multi-byte unicode).
pub fn cell_label(cell: &Cell) -> (String, Color, bool) {
    match cell {
        Cell::Empty => ("  ".to_string(), Color::Reset, false),

        Cell::Piece(piece) => match piece {
            Piece::Regular(item) => (
                item.glyph().to_string(),
                family_color(item.family),
                false,
            ),
            Piece::Blueprint(fam) => (
                format!("B{}", &fam.name()[..1]),
                family_color(*fam),
                true,
            ),
        },

        // Frozen: same glyph but dimmed grey
        Cell::Frozen(piece) => {
            let glyph = match piece {
                Piece::Regular(item) => item.glyph().to_string(),
                Piece::Blueprint(fam) => format!("B{}", &fam.name()[..1]),
            };
            (glyph, Color::DarkGrey, false)
        }

        // Hard generator: "G∞" for T1, "G2"/"G3"... for higher tiers. Bold.
        Cell::HardGenerator { family, tier, cooldown_remaining } => {
            let color = if *cooldown_remaining > 0 { Color::DarkGrey } else { family_color(*family) };
            let label = if *tier == 1 { "G∞".to_string() } else { format!("G{}", tier) };
            (label, color, true)
        }

        // Soft generator: "Gn" where n = remaining charges
        Cell::SoftGenerator { family, charges, cooldown_remaining, .. } => {
            let color = if *charges == 0 || *cooldown_remaining > 0 {
                Color::DarkGrey
            } else {
                family_color(*family)
            };
            (format!("G{}", charges), color, false)
        }
    }
}
