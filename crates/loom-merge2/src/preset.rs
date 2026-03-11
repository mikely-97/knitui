use crate::config::Config;

pub struct Preset {
    pub name: &'static str,
    pub board_height: u16,
    pub board_width: u16,
    pub color_count: u16,
    pub generator_count: u16,
    pub generator_charges: u16,
    pub blocked_cells: u16,
    pub generator_interval: u32,
    pub order_count: u16,
    pub max_order_tier: u8,
    pub ad_limit: u16,
}

impl Preset {
    pub fn to_config(&self, base: &Config) -> Config {
        let mut cfg = base.clone();
        cfg.board_height = self.board_height;
        cfg.board_width = self.board_width;
        cfg.color_count = self.color_count;
        cfg.generator_count = self.generator_count;
        cfg.generator_charges = self.generator_charges;
        cfg.blocked_cells = self.blocked_cells;
        cfg.generator_interval = self.generator_interval;
        cfg.order_count = self.order_count;
        cfg.max_order_tier = self.max_order_tier;
        cfg.ad_limit = self.ad_limit;
        cfg
    }
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Easy",
        board_height: 4, board_width: 4, color_count: 2,
        generator_count: 2, generator_charges: 0,
        blocked_cells: 0, generator_interval: 6,
        order_count: 1, max_order_tier: 3, ad_limit: 5,
    },
    Preset {
        name: "Medium",
        board_height: 5, board_width: 5, color_count: 3,
        generator_count: 3, generator_charges: 12,
        blocked_cells: 2, generator_interval: 8,
        order_count: 2, max_order_tier: 4, ad_limit: 3,
    },
    Preset {
        name: "Hard",
        board_height: 6, board_width: 6, color_count: 4,
        generator_count: 4, generator_charges: 8,
        blocked_cells: 4, generator_interval: 10,
        order_count: 3, max_order_tier: 5, ad_limit: 1,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn default_config() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn preset_to_config_applies_fields() {
        let cfg = PRESETS[0].to_config(&default_config());
        assert_eq!(cfg.board_height, 4);
        assert_eq!(cfg.board_width, 4);
        assert_eq!(cfg.color_count, 2);
    }

    #[test]
    fn all_presets_produce_valid_configs() {
        for p in PRESETS {
            let cfg = p.to_config(&default_config());
            assert!(cfg.board_height >= 3);
            assert!(cfg.board_width >= 3);
            assert!(cfg.color_count >= 1);
        }
    }
}
