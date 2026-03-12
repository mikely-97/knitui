use clap::Parser;
use loom_engine::game::GameConfig;
use serde::{Deserialize, Serialize};

pub const MAX_BOARD_DIM: u16 = 12;
pub const DEFAULT_BOARD_ROWS: u16 = 10;
pub const DEFAULT_BOARD_COLS: u16 = 8;

#[derive(Parser, Clone, Debug, Serialize, Deserialize)]
#[command(about = "Merge-2 puzzle game")]
pub struct Config {
    #[arg(long, default_value_t = DEFAULT_BOARD_ROWS)]
    pub board_rows: u16,
    #[arg(long, default_value_t = DEFAULT_BOARD_COLS)]
    pub board_cols: u16,
    #[arg(long, default_value_t = 1)]
    pub scale: u16,
    #[arg(long, default_value = "dark")]
    pub color_mode: String,
    /// Max energy points.
    #[arg(long, default_value_t = 100)]
    pub energy_max: u16,
    /// Seconds per +1 energy regen.
    #[arg(long, default_value_t = 30)]
    pub energy_regen_secs: u32,
    /// Energy cost per generator activation.
    #[arg(long, default_value_t = 1)]
    pub generator_cost: u16,
    /// Generator cooldown in ticks after activation (0 = instant reuse).
    #[arg(long, default_value_t = 0)]
    pub generator_cooldown: u32,
    /// Starting inventory slots.
    #[arg(long, default_value_t = 4)]
    pub inventory_slots: u16,
    /// How many item families are available (1-6).
    #[arg(long, default_value_t = 6)]
    pub family_count: u16,
    /// Ad watches allowed per session.
    #[arg(long, default_value_t = 5)]
    pub ad_limit: u16,
    /// Number of simultaneous random orders.
    #[arg(long, default_value_t = 2)]
    pub random_order_count: u16,
    /// Max tier that random orders can require.
    #[arg(long, default_value_t = 4)]
    pub max_order_tier: u8,
    /// Chance (0-100) that merging two T7+ items creates a soft generator.
    #[arg(long, default_value_t = 25)]
    pub soft_gen_chance: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            board_rows: DEFAULT_BOARD_ROWS,
            board_cols: DEFAULT_BOARD_COLS,
            scale: 1,
            color_mode: "dark".to_string(),
            energy_max: 100,
            energy_regen_secs: 30,
            generator_cost: 1,
            generator_cooldown: 0,
            inventory_slots: 4,
            family_count: 6,
            ad_limit: 5,
            random_order_count: 2,
            max_order_tier: 4,
            soft_gen_chance: 25,
        }
    }
}

impl GameConfig for Config {
    fn board_width(&self) -> usize {
        self.board_cols as usize
    }
    fn board_height(&self) -> usize {
        self.board_rows as usize
    }
    fn color_count(&self) -> usize {
        self.family_count as usize
    }
    fn scale(&self) -> u16 {
        self.scale
    }
    fn color_mode(&self) -> &str {
        &self.color_mode
    }
    fn set_scale(&mut self, s: u16) {
        self.scale = s;
    }
    fn set_color_mode(&mut self, m: String) {
        self.color_mode = m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_valid() {
        let cfg = Config::default();
        assert_eq!(cfg.board_rows, DEFAULT_BOARD_ROWS);
        assert_eq!(cfg.board_cols, DEFAULT_BOARD_COLS);
        assert_eq!(cfg.energy_max, 100);
        assert_eq!(cfg.energy_regen_secs, 30);
    }

    #[test]
    fn game_config_trait() {
        let cfg = Config::default();
        assert_eq!(cfg.board_width(), DEFAULT_BOARD_COLS as usize);
        assert_eq!(cfg.board_height(), DEFAULT_BOARD_ROWS as usize);
        assert_eq!(cfg.scale(), 1);
        assert_eq!(cfg.color_mode(), "dark");
    }
}
