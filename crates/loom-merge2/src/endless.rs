pub use loom_engine::endless::EndlessHighScore;

use crate::engine::GameEngine;
use crate::item::{Family, ALL_FAMILIES};

const CONFIG_DIR: &str = "m2tui";

pub fn load_high_score() -> EndlessHighScore {
    EndlessHighScore::load(CONFIG_DIR)
}

pub fn save_high_score(hs: &EndlessHighScore) {
    hs.save(CONFIG_DIR);
}

// ── Difficulty scaling ────────────────────────────────────────────────────

/// Parameters that scale with total merges in endless mode.
pub struct EndlessParams {
    pub energy_max: u16,
    pub energy_regen_secs: u32,
    pub random_order_count: usize,
    pub max_order_tier: u8,
    pub generator_cost: u16,
    pub generator_cooldown: u32,
    pub soft_gen_chance: u8,
    pub families: Vec<Family>,
}

impl EndlessParams {
    pub fn from_merges(total_merges: u64) -> Self {
        let stages = total_merges / 50;
        let energy_max = (100u16).saturating_sub((stages as u16) * 5).max(30);
        let max_order_tier = (3u8 + (total_merges / 30) as u8).min(7);
        let family_count = (2 + total_merges / 40).min(6) as usize;
        let families: Vec<Family> = ALL_FAMILIES.iter().take(family_count).copied().collect();
        let generator_cooldown = (total_merges / 80) as u32;
        let random_order_count = (2 + stages / 5).min(4) as usize;

        EndlessParams {
            energy_max,
            energy_regen_secs: 30,
            random_order_count,
            max_order_tier,
            generator_cost: 1,
            generator_cooldown,
            soft_gen_chance: 20,
            families,
        }
    }
}

/// Create a fresh endless engine. Board is 10×8, fully unfrozen, 2 hard generators.
pub fn new_endless_engine(blessings: &[String]) -> GameEngine {
    use crate::board::Cell;
    use crate::config::Config;

    let config = Config::default();
    let mut engine = GameEngine::new_endless(&config, blessings);

    engine.available_families = vec![Family::Wood, Family::Stone];
    // Regenerate initial random orders using only the families above;
    // new_endless() generated them before available_families was narrowed.
    engine.regenerate_orders();

    engine.board.cells[0][0] = Cell::HardGenerator {
        family: Family::Wood,
        tier: 1,
        cooldown_remaining: 0,
    };
    let last_row = engine.board.rows - 1;
    let last_col = engine.board.cols - 1;
    engine.board.cells[last_row][last_col] = Cell::HardGenerator {
        family: Family::Stone,
        tier: 1,
        cooldown_remaining: 0,
    };

    engine
}

/// Update endless engine difficulty based on total merges completed.
pub fn apply_scaling(engine: &mut GameEngine) {
    let params = EndlessParams::from_merges(engine.total_merges);
    engine.energy.max = params.energy_max;
    engine.energy.current = engine.energy.current.min(engine.energy.max);
    engine.max_order_tier = params.max_order_tier;
    engine.random_order_count = params.random_order_count;
    engine.generator_cooldown = params.generator_cooldown;
    engine.available_families = params.families;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_start_easy() {
        let p = EndlessParams::from_merges(0);
        assert_eq!(p.energy_max, 100);
        assert_eq!(p.max_order_tier, 3);
        assert_eq!(p.families.len(), 2);
    }

    #[test]
    fn params_scale_with_merges() {
        let p0 = EndlessParams::from_merges(0);
        let p100 = EndlessParams::from_merges(100);
        assert!(p100.energy_max <= p0.energy_max);
        assert!(p100.max_order_tier >= p0.max_order_tier);
    }

    #[test]
    fn params_energy_floor() {
        assert!(EndlessParams::from_merges(10_000).energy_max >= 30);
    }

    #[test]
    fn params_max_tier_cap() {
        assert!(EndlessParams::from_merges(10_000).max_order_tier <= 7);
    }

    #[test]
    fn params_family_cap() {
        assert!(EndlessParams::from_merges(10_000).families.len() <= 6);
    }

    #[test]
    fn new_endless_engine_valid() {
        let engine = new_endless_engine(&[]);
        assert_eq!(engine.board.rows, 10);
        assert_eq!(engine.board.cols, 8);
        assert_eq!(engine.available_families.len(), 2);
    }
}
