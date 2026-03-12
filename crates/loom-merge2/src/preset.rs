use crate::config::Config;

pub struct Preset {
    pub name: &'static str,
    pub board_rows: u16,
    pub board_cols: u16,
    pub family_count: u16,
    pub energy_max: u16,
    pub energy_regen_secs: u32,
    pub generator_cost: u16,
    pub generator_cooldown: u32,
    pub inventory_slots: u16,
    pub random_order_count: u16,
    pub max_order_tier: u8,
    pub ad_limit: u16,
    pub soft_gen_chance: u8,
}

impl Preset {
    pub fn to_config(&self, base: &Config) -> Config {
        let mut cfg = base.clone();
        cfg.board_rows = self.board_rows;
        cfg.board_cols = self.board_cols;
        cfg.family_count = self.family_count;
        cfg.energy_max = self.energy_max;
        cfg.energy_regen_secs = self.energy_regen_secs;
        cfg.generator_cost = self.generator_cost;
        cfg.generator_cooldown = self.generator_cooldown;
        cfg.inventory_slots = self.inventory_slots;
        cfg.random_order_count = self.random_order_count;
        cfg.max_order_tier = self.max_order_tier;
        cfg.ad_limit = self.ad_limit;
        cfg.soft_gen_chance = self.soft_gen_chance;
        cfg
    }
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "Relaxed",
        board_rows: 8,
        board_cols: 6,
        family_count: 3,
        energy_max: 150,
        energy_regen_secs: 20,
        generator_cost: 1,
        generator_cooldown: 0,
        inventory_slots: 6,
        random_order_count: 1,
        max_order_tier: 3,
        ad_limit: 5,
        soft_gen_chance: 30,
    },
    Preset {
        name: "Standard",
        board_rows: 10,
        board_cols: 8,
        family_count: 4,
        energy_max: 100,
        energy_regen_secs: 30,
        generator_cost: 1,
        generator_cooldown: 2,
        inventory_slots: 4,
        random_order_count: 2,
        max_order_tier: 5,
        ad_limit: 3,
        soft_gen_chance: 20,
    },
    Preset {
        name: "Challenge",
        board_rows: 10,
        board_cols: 8,
        family_count: 6,
        energy_max: 60,
        energy_regen_secs: 45,
        generator_cost: 2,
        generator_cooldown: 5,
        inventory_slots: 2,
        random_order_count: 3,
        max_order_tier: 7,
        ad_limit: 1,
        soft_gen_chance: 10,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_to_config_applies_fields() {
        let cfg = PRESETS[0].to_config(&Config::default());
        assert_eq!(cfg.board_rows, 8);
        assert_eq!(cfg.board_cols, 6);
        assert_eq!(cfg.family_count, 3);
    }

    #[test]
    fn all_presets_produce_valid_configs() {
        for p in PRESETS {
            let cfg = p.to_config(&Config::default());
            assert!(cfg.board_rows >= 4);
            assert!(cfg.board_cols >= 4);
            assert!(cfg.family_count >= 1);
            assert!(cfg.energy_max >= 30);
        }
    }
}
