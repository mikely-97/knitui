use crate::board_entity::Direction;

/// Thread spool/bobbin: ▄██▄ flanges top/bottom, solid barrel in middle.
pub fn entity_glyph_thread(scale: u16) -> Vec<String> {
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

/// Key thread spool: same as thread but with ⚷ symbol visible.
pub fn entity_glyph_key_thread(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    if sh <= 1 {
        return vec!["⚷█".to_string()];
    }
    let mut rows = Vec::with_capacity(sh);
    if sh == 2 {
        // Key in top flange
        rows.push(format!("▄⚷{}▄", "█".repeat(sw - 3)));
        rows.push(format!("▀{}▀", "█".repeat(sw - 2)));
    } else {
        rows.push(format!("▄{}▄", "█".repeat(sw - 2)));
        // Key in first middle row
        let before = sw / 2 - 1;
        let after = sw - before - 2; // 2 for ·⚷
        rows.push(format!("{}·⚷{}", "█".repeat(before), "█".repeat(after)));
        for _ in 2..sh - 1 {
            rows.push("█".repeat(sw));
        }
        rows.push(format!("▀{}▀", "█".repeat(sw - 2)));
    }
    rows
}

/// Yarn patch knit stitch texture: ╲╱ cross-stitch for unlocked, ╳ for locked.
pub fn yarn_patch_glyph(locked: bool, scale: u16) -> Vec<String> {
    let sh = scale as usize;
    let unit = scale as usize;
    if sh <= 1 {
        return vec![String::new()]; // scale 1 uses direct Patch::Display
    }
    let mut rows = Vec::with_capacity(sh);
    for sy in 0..sh {
        if locked {
            rows.push("╳".repeat(unit * 2));
        } else if sy % 2 == 0 {
            rows.push("╲╱".repeat(unit));
        } else {
            rows.push("╱╲".repeat(unit));
        }
    }
    rows
}

/// Obstacle: light shade fill.
pub fn entity_glyph_obstacle(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    vec!["░".repeat(sw); sh]
}

/// Generator: directional arrow from source box.
pub fn entity_glyph_generator(dir: Direction, scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    if sh <= 1 {
        return match dir {
            Direction::Right => vec!["▸·".to_string()],
            Direction::Left  => vec!["◂·".to_string()],
            Direction::Down  => vec!["▾·".to_string()],
            Direction::Up    => vec!["▴·".to_string()],
        };
    }
    match dir {
        Direction::Right => {
            vec![format!("⊞{}▸", "─".repeat(sw - 2)); sh]
        }
        Direction::Left => {
            vec![format!("◂{}⊞", "─".repeat(sw - 2)); sh]
        }
        Direction::Down => {
            let pad = (sw - 2) / 2;
            let pad_r = sw - 2 - pad;
            let mut rows = Vec::with_capacity(sh);
            rows.push(format!("{}⊞⊞{}", "·".repeat(pad), "·".repeat(pad_r)));
            for _ in 1..sh - 1 {
                rows.push(format!("{}╏╏{}", "·".repeat(pad), "·".repeat(pad_r)));
            }
            rows.push(format!("{}▾▾{}", "·".repeat(pad), "·".repeat(pad_r)));
            rows
        }
        Direction::Up => {
            let pad = (sw - 2) / 2;
            let pad_r = sw - 2 - pad;
            let mut rows = Vec::with_capacity(sh);
            rows.push(format!("{}▴▴{}", "·".repeat(pad), "·".repeat(pad_r)));
            for _ in 1..sh - 1 {
                rows.push(format!("{}╏╏{}", "·".repeat(pad), "·".repeat(pad_r)));
            }
            rows.push(format!("{}⊞⊞{}", "·".repeat(pad), "·".repeat(pad_r)));
            rows
        }
    }
}

/// Depleted generator: source box with fading dashes.
pub fn entity_glyph_depleted(scale: u16) -> Vec<String> {
    let sh = scale as usize;
    if sh <= 1 {
        return vec!["⊞·".to_string()];
    }
    // ⊞ + ─×scale + ·×(scale-1) = 1 + scale + scale-1 = 2*scale = sw
    let row = format!("⊞{}{}", "─".repeat(scale as usize), "·".repeat(scale as usize - 1));
    vec![row; sh]
}

/// Void: empty space.
pub fn entity_glyph_void(scale: u16) -> Vec<String> {
    let sw = (scale * 2) as usize;
    let sh = scale as usize;
    vec![" ".repeat(sw); sh]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_entity::Direction;

    #[test]
    fn scale1_thread_returns_single_row() {
        let rows = entity_glyph_thread(1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].chars().count(), 2);
    }

    #[test]
    fn scale2_thread_returns_spool() {
        let rows = entity_glyph_thread(2);
        assert_eq!(rows, vec!["▄██▄", "▀██▀"]);
    }

    #[test]
    fn scale3_thread_returns_spool() {
        let rows = entity_glyph_thread(3);
        assert_eq!(rows, vec!["▄████▄", "██████", "▀████▀"]);
    }

    #[test]
    fn scale2_obstacle_returns_shade() {
        let rows = entity_glyph_obstacle(2);
        assert_eq!(rows, vec!["░░░░", "░░░░"]);
    }

    #[test]
    fn scale2_generator_right() {
        let rows = entity_glyph_generator(Direction::Right, 2);
        assert_eq!(rows, vec!["⊞──▸", "⊞──▸"]);
    }

    #[test]
    fn scale2_generator_left() {
        let rows = entity_glyph_generator(Direction::Left, 2);
        assert_eq!(rows, vec!["◂──⊞", "◂──⊞"]);
    }

    #[test]
    fn scale2_generator_down() {
        let rows = entity_glyph_generator(Direction::Down, 2);
        assert_eq!(rows, vec!["·⊞⊞·", "·▾▾·"]);
    }

    #[test]
    fn scale2_generator_up() {
        let rows = entity_glyph_generator(Direction::Up, 2);
        assert_eq!(rows, vec!["·▴▴·", "·⊞⊞·"]);
    }

    #[test]
    fn scale2_depleted_generator() {
        let rows = entity_glyph_depleted(2);
        assert_eq!(rows, vec!["⊞──·", "⊞──·"]);
    }

    #[test]
    fn scale2_key_thread() {
        let rows = entity_glyph_key_thread(2);
        assert_eq!(rows, vec!["▄⚷█▄", "▀██▀"]);
    }

    #[test]
    fn scale2_yarn_patch_unlocked() {
        let rows = yarn_patch_glyph(false, 2);
        assert_eq!(rows, vec!["╲╱╲╱", "╱╲╱╲"]);
    }

    #[test]
    fn scale2_yarn_patch_locked() {
        let rows = yarn_patch_glyph(true, 2);
        assert_eq!(rows, vec!["╳╳╳╳", "╳╳╳╳"]);
    }

    #[test]
    fn scale3_yarn_patch_unlocked() {
        let rows = yarn_patch_glyph(false, 3);
        assert_eq!(rows, vec!["╲╱╲╱╲╱", "╱╲╱╲╱╲", "╲╱╲╱╲╱"]);
    }

    #[test]
    fn scale2_void_returns_spaces() {
        let rows = entity_glyph_void(2);
        assert_eq!(rows, vec!["    ", "    "]);
    }

    #[test]
    fn all_scale2_patterns_have_correct_width() {
        let all: Vec<Vec<String>> = vec![
            entity_glyph_thread(2),
            entity_glyph_key_thread(2),
            entity_glyph_obstacle(2),
            entity_glyph_generator(Direction::Right, 2),
            entity_glyph_generator(Direction::Left, 2),
            entity_glyph_generator(Direction::Down, 2),
            entity_glyph_generator(Direction::Up, 2),
            entity_glyph_depleted(2),
            entity_glyph_void(2),
            yarn_patch_glyph(false, 2),
            yarn_patch_glyph(true, 2),
        ];
        for pattern in &all {
            for row in pattern {
                assert_eq!(
                    row.chars().count(),
                    4,
                    "Expected width 4 for row {:?}",
                    row
                );
            }
        }
    }

    #[test]
    fn all_scale3_patterns_have_correct_dimensions() {
        let all: Vec<Vec<String>> = vec![
            entity_glyph_thread(3),
            entity_glyph_key_thread(3),
            entity_glyph_obstacle(3),
            entity_glyph_generator(Direction::Right, 3),
            entity_glyph_generator(Direction::Left, 3),
            entity_glyph_generator(Direction::Down, 3),
            entity_glyph_generator(Direction::Up, 3),
            entity_glyph_depleted(3),
            entity_glyph_void(3),
            yarn_patch_glyph(false, 3),
            yarn_patch_glyph(true, 3),
        ];
        for pattern in &all {
            assert_eq!(pattern.len(), 3, "Expected 3 rows, got {:?}", pattern);
            for row in pattern {
                assert_eq!(
                    row.chars().count(),
                    6,
                    "Expected width 6 for row {:?}",
                    row
                );
            }
        }
    }

    #[test]
    fn all_scale4_patterns_have_correct_dimensions() {
        let all: Vec<Vec<String>> = vec![
            entity_glyph_thread(4),
            entity_glyph_key_thread(4),
            entity_glyph_obstacle(4),
            entity_glyph_generator(Direction::Right, 4),
            entity_glyph_generator(Direction::Left, 4),
            entity_glyph_generator(Direction::Down, 4),
            entity_glyph_generator(Direction::Up, 4),
            entity_glyph_depleted(4),
            entity_glyph_void(4),
            yarn_patch_glyph(false, 4),
            yarn_patch_glyph(true, 4),
        ];
        for pattern in &all {
            assert_eq!(pattern.len(), 4, "Expected 4 rows, got {:?}", pattern);
            for row in pattern {
                assert_eq!(
                    row.chars().count(),
                    8,
                    "Expected width 8 for row {:?}",
                    row
                );
            }
        }
    }
}
