use clap::Parser;
use loom_engine::game::GameConfig;
use serde::{Deserialize, Serialize};

pub const MAX_BOARD_DIM: u16 = 8;

#[derive(Parser, Clone, Debug, Serialize, Deserialize)]
#[command(about = "Merge-2 puzzle game")]
pub struct Config {
    #[arg(long, default_value_t = 5)]
    pub board_height: u16,
    #[arg(long, default_value_t = 5)]
    pub board_width: u16,
    #[arg(long, default_value_t = 3)]
    pub color_count: u16,
    #[arg(long, default_value_t = 1)]
    pub scale: u16,
    #[arg(long, default_value = "dark")]
    pub color_mode: String,
    #[arg(long, default_value_t = 2)]
    pub generator_count: u16,
    /// Generator charges (0 = infinite).
    #[arg(long, default_value_t = 10)]
    pub generator_charges: u16,
    #[arg(long, default_value_t = 0)]
    pub blocked_cells: u16,
    /// Ticks between generator spawns.
    #[arg(long, default_value_t = 8)]
    pub generator_interval: u32,
    #[arg(long, default_value_t = 2)]
    pub order_count: u16,
    /// Maximum tier that orders can require.
    #[arg(long, default_value_t = 3)]
    pub max_order_tier: u8,
    /// Ad watches allowed per level (0 = none).
    #[arg(long, default_value_t = 3)]
    pub ad_limit: u16,
}

impl GameConfig for Config {
    fn board_width(&self) -> usize { self.board_width as usize }
    fn board_height(&self) -> usize { self.board_height as usize }
    fn color_count(&self) -> usize { self.color_count as usize }
    fn scale(&self) -> u16 { self.scale }
    fn color_mode(&self) -> &str { &self.color_mode }
    fn set_scale(&mut self, s: u16) { self.scale = s; }
    fn set_color_mode(&mut self, m: String) { self.color_mode = m; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn default_config_parses() {
        let cfg = Config::parse_from::<[&str; 0], &str>([]);
        assert_eq!(cfg.board_height, 5);
        assert_eq!(cfg.board_width, 5);
        assert_eq!(cfg.color_count, 3);
    }

    #[test]
    fn game_config_trait() {
        let cfg = Config::parse_from::<[&str; 0], &str>([]);
        assert_eq!(cfg.board_width(), 5);
        assert_eq!(cfg.board_height(), 5);
        assert_eq!(cfg.color_count(), 3);
        assert_eq!(cfg.scale(), 1);
        assert_eq!(cfg.color_mode(), "dark");
    }
}
