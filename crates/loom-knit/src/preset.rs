use crate::config::Config;

pub struct GamePreset {
    pub name: &'static str,
    pub board_height: u16,
    pub board_width: u16,
    pub color_number: u16,
    pub obstacle_percentage: u16,
    pub conveyor_percentage: u16,
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
}

pub const PRESETS: &[GamePreset] = &[
    GamePreset {
        name: "Small",
        board_height: 4, board_width: 4, color_number: 4,
        obstacle_percentage: 0, conveyor_percentage: 0,
        scissors: 0, tweezers: 0, balloons: 0,
    },
    GamePreset {
        name: "Medium",
        board_height: 6, board_width: 6, color_number: 6,
        obstacle_percentage: 5, conveyor_percentage: 5,
        scissors: 0, tweezers: 0, balloons: 0,
    },
    GamePreset {
        name: "Large",
        board_height: 8, board_width: 8, color_number: 8,
        obstacle_percentage: 10, conveyor_percentage: 10,
        scissors: 1, tweezers: 1, balloons: 1,
    },
    GamePreset {
        name: "Chaos",
        board_height: 10, board_width: 10, color_number: 8,
        obstacle_percentage: 20, conveyor_percentage: 15,
        scissors: 2, tweezers: 2, balloons: 2,
    },
];

impl GamePreset {
    /// Apply preset game values on top of a base Config.
    /// Scale and color_mode are display preferences — not overridden by presets.
    pub fn to_config(&self, base: &Config) -> Config {
        let mut cfg = base.clone();
        cfg.board_height = self.board_height;
        cfg.board_width = self.board_width;
        cfg.color_number = self.color_number;
        cfg.obstacle_percentage = self.obstacle_percentage;
        cfg.conveyor_percentage = self.conveyor_percentage;
        cfg.scissors = self.scissors;
        cfg.tweezers = self.tweezers;
        cfg.balloons = self.balloons;
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use clap::Parser;

    fn default_config() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn presets_have_valid_board_sizes() {
        for p in PRESETS {
            assert!(p.board_height >= 2, "{}: board_height too small", p.name);
            assert!(p.board_width >= 2, "{}: board_width too small", p.name);
            assert!(p.color_number >= 2, "{}: color_number too small", p.name);
        }
    }

    #[test]
    fn presets_count_is_four() {
        assert_eq!(PRESETS.len(), 4);
    }

    #[test]
    fn to_config_applies_preset_values() {
        let base = default_config();
        let preset = &PRESETS[2]; // Large
        let cfg = preset.to_config(&base);
        assert_eq!(cfg.board_height, 8);
        assert_eq!(cfg.board_width, 8);
        assert_eq!(cfg.color_number, 8);
        assert_eq!(cfg.scissors, 1);
    }

    #[test]
    fn to_config_preserves_scale_and_color() {
        let mut base = default_config();
        base.scale = 3;
        base.color_mode = "bright-rgb".to_string();
        let cfg = PRESETS[0].to_config(&base);
        assert_eq!(cfg.scale, 3, "preset should not override scale");
        assert_eq!(cfg.color_mode, "bright-rgb", "preset should not override color_mode");
    }

    #[test]
    fn to_config_inherits_non_preset_fields() {
        let mut base = default_config();
        base.color_mode = "bright-rgb".to_string();
        base.spool_capacity = 5;
        let cfg = PRESETS[0].to_config(&base);
        assert_eq!(cfg.color_mode, "bright-rgb");
        assert_eq!(cfg.spool_capacity, 5);
    }
}
