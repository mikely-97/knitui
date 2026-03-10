use crate::board_entity::Direction;

pub fn entity_glyph_thread(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["╲╱╲╱", "╱╲╱╲"],
        3 => vec!["╲╱╲╱╲╱", "╱╲╱╲╱╲", "╲╱╲╱╲╱"],
        _ => vec!["╲╱"],
    }
}

pub fn entity_glyph_key_thread(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["╲╱⚷╱", "╱╲╲╱"],
        3 => vec!["╲╱╲╱╲╱", "╱╲⚷╲╱╲", "╲╱╲╱╲╱"],
        _ => vec!["⚷╱"],
    }
}

pub fn entity_glyph_obstacle(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["░░░░", "░░░░"],
        3 => vec!["░░░░░░", "░░░░░░", "░░░░░░"],
        _ => vec!["░░"],
    }
}

pub fn entity_glyph_generator(dir: Direction, scale: u16) -> Vec<&'static str> {
    match (dir, scale) {
        (Direction::Right, 2) => vec!["⊞──▸", "⊞──▸"],
        (Direction::Left,  2) => vec!["◂──⊞", "◂──⊞"],
        (Direction::Down,  2) => vec!["·⊞⊞·", "·▾▾·"],
        (Direction::Up,    2) => vec!["·▴▴·", "·⊞⊞·"],
        (Direction::Right, 3) => vec!["⊞────▸", "⊞────▸", "⊞────▸"],
        (Direction::Left,  3) => vec!["◂────⊞", "◂────⊞", "◂────⊞"],
        (Direction::Down,  3) => vec!["··⊞⊞··", "··╏╏··", "··▾▾··"],
        (Direction::Up,    3) => vec!["··▴▴··", "··╏╏··", "··⊞⊞··"],
        (Direction::Right, _) => vec!["▸·"],
        (Direction::Left,  _) => vec!["◂·"],
        (Direction::Down,  _) => vec!["▾·"],
        (Direction::Up,    _) => vec!["▴·"],
    }
}

pub fn entity_glyph_depleted(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["⊞──·", "⊞──·"],
        3 => vec!["⊞───··", "⊞───··", "⊞───··"],
        _ => vec!["⊞·"],
    }
}

pub fn entity_glyph_void(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["    ", "    "],
        3 => vec!["      ", "      ", "      "],
        _ => vec!["  "],
    }
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
    fn scale2_thread_returns_cross_stitch() {
        let rows = entity_glyph_thread(2);
        assert_eq!(rows, vec!["╲╱╲╱", "╱╲╱╲"]);
    }

    #[test]
    fn scale3_thread_returns_tiled_cross_stitch() {
        let rows = entity_glyph_thread(3);
        assert_eq!(rows, vec!["╲╱╲╱╲╱", "╱╲╱╲╱╲", "╲╱╲╱╲╱"]);
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
        assert_eq!(rows, vec!["╲╱⚷╱", "╱╲╲╱"]);
    }

    #[test]
    fn scale2_void_returns_spaces() {
        let rows = entity_glyph_void(2);
        assert_eq!(rows, vec!["    ", "    "]);
    }

    #[test]
    fn all_scale2_patterns_have_correct_width() {
        let all: Vec<Vec<&'static str>> = vec![
            entity_glyph_thread(2),
            entity_glyph_key_thread(2),
            entity_glyph_obstacle(2),
            entity_glyph_generator(Direction::Right, 2),
            entity_glyph_generator(Direction::Left, 2),
            entity_glyph_generator(Direction::Down, 2),
            entity_glyph_generator(Direction::Up, 2),
            entity_glyph_depleted(2),
            entity_glyph_void(2),
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
        let all: Vec<Vec<&'static str>> = vec![
            entity_glyph_thread(3),
            entity_glyph_key_thread(3),
            entity_glyph_obstacle(3),
            entity_glyph_generator(Direction::Right, 3),
            entity_glyph_generator(Direction::Left, 3),
            entity_glyph_generator(Direction::Down, 3),
            entity_glyph_generator(Direction::Up, 3),
            entity_glyph_depleted(3),
            entity_glyph_void(3),
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
}
