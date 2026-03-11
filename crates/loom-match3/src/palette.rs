use crossterm::style::Color;

/// Return the first `count` colors from the pool for the given mode.
pub fn select_palette(color_mode: &str, count: u8) -> Vec<Color> {
    let pool: &[Color] = match color_mode {
        "bright" | "bright-rgb" => BRIGHT_POOL,
        "colorblind" | "colorblind-rgb" => COLORBLIND_POOL,
        _ => DARK_POOL,
    };
    pool.iter().take(count as usize).cloned().collect()
}

static DARK_POOL: &[Color] = &[
    Color::Red,
    Color::Blue,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
    Color::White,
];

static BRIGHT_POOL: &[Color] = &[
    Color::Rgb { r: 255, g: 80,  b: 80  },
    Color::Rgb { r: 80,  g: 150, b: 255 },
    Color::Rgb { r: 80,  g: 220, b: 80  },
    Color::Rgb { r: 255, g: 220, b: 0   },
    Color::Rgb { r: 220, g: 80,  b: 220 },
    Color::Rgb { r: 0,   g: 220, b: 220 },
    Color::Rgb { r: 220, g: 220, b: 220 },
];

static COLORBLIND_POOL: &[Color] = &[
    Color::Rgb { r: 0,   g: 114, b: 178 },
    Color::Rgb { r: 230, g: 159, b: 0   },
    Color::Rgb { r: 0,   g: 158, b: 115 },
    Color::Rgb { r: 240, g: 228, b: 66  },
    Color::Rgb { r: 204, g: 121, b: 167 },
    Color::Rgb { r: 86,  g: 180, b: 233 },
    Color::Rgb { r: 213, g: 94,  b: 0   },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_correct_count() {
        assert_eq!(select_palette("dark", 5).len(), 5);
        assert_eq!(select_palette("bright", 3).len(), 3);
    }

    #[test]
    fn count_clamped_to_pool_size() {
        // Pool has 7 entries; requesting 10 → still 7
        assert_eq!(select_palette("dark", 10).len(), 7);
    }

    #[test]
    fn dark_and_bright_differ() {
        let d = select_palette("dark", 3);
        let b = select_palette("bright", 3);
        assert_ne!(d, b);
    }

    #[test]
    fn bright_rgb_returns_same_pool_as_bright() {
        // "bright" and "bright-rgb" use the same pool (rgb suffix only affects ANSI vs true-color output)
        let a = select_palette("bright", 4);
        let b = select_palette("bright-rgb", 4);
        assert_eq!(a, b);
    }

    #[test]
    fn unknown_mode_falls_back_to_dark() {
        let d = select_palette("dark", 3);
        let u = select_palette("unknown", 3);
        assert_eq!(d, u);
    }
}
