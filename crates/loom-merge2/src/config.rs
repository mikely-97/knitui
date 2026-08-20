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

    /// Set by `M2Game::endless_wave_config` to mark an endless run. Not a
    /// CLI flag — tells the `GameEngine` adapter to build via
    /// `endless::new_endless_engine` (fixed board + generators, difficulty
    /// scales continuously with total_merges) instead of the config-driven
    /// constructor used for Custom Game.
    #[arg(skip)]
    pub is_endless: bool,
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
            is_endless: false,
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
    fn custom_fields(&self) -> Vec<(&'static str, u16)> {
        vec![
            ("Board Rows", self.board_rows),
            ("Board Cols", self.board_cols),
            ("Scale", self.scale),
            ("Energy Max", self.energy_max),
            ("Energy Regen Secs", self.energy_regen_secs as u16),
            ("Generator Cost", self.generator_cost),
            ("Family Count", self.family_count),
            ("Random Orders", self.random_order_count),
            ("Max Order Tier", self.max_order_tier as u16),
            ("Inventory Slots", self.inventory_slots),
        ]
    }

    // Field indices and clamps match the original tui.rs's adjust_config_field
    // exactly (field 0 there was preset-cycling, handled generically by Shell
    // now, not here).
    fn adjust_custom_field(&mut self, field: usize, delta: i16) {
        let d = delta as i32;
        match field {
            1 => self.board_rows = (self.board_rows as i32 + d).clamp(4, 20) as u16,
            2 => self.board_cols = (self.board_cols as i32 + d).clamp(4, 16) as u16,
            3 => self.scale = (self.scale as i32 + d).clamp(1, 3) as u16,
            4 => self.energy_max = (self.energy_max as i32 + d * 10).clamp(10, 500) as u16,
            5 => self.energy_regen_secs = (self.energy_regen_secs as i32 + d * 5).clamp(5, 300) as u32,
            6 => self.generator_cost = (self.generator_cost as i32 + d).clamp(0, 20) as u16,
            7 => self.family_count = (self.family_count as i32 + d).clamp(1, 6) as u16,
            8 => self.random_order_count = (self.random_order_count as i32 + d).clamp(0, 5) as u16,
            9 => self.max_order_tier = (self.max_order_tier as i32 + d).clamp(1, 8) as u8,
            10 => self.inventory_slots = (self.inventory_slots as i32 + d).clamp(0, 16) as u16,
            _ => {}
        }
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
