use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(name = "m3tui", about = "TUI match-3 game")]
pub struct Config {
    /// Board height in cells
    #[arg(long, default_value_t = 8)]
    pub board_height: u16,

    /// Board width in cells
    #[arg(long, default_value_t = 8)]
    pub board_width: u16,

    /// Number of gem colors (2–7)
    #[arg(long, default_value_t = 6)]
    pub color_number: u8,

    /// Move limit per game
    #[arg(long, default_value_t = 30)]
    pub move_limit: u32,

    /// Percentage of cells that start with a special tile modifier (0–100)
    #[arg(long, default_value_t = 5)]
    pub special_tile_pct: u16,

    /// Render scale (1–5)
    #[arg(long, default_value_t = 1)]
    pub scale: u16,

    /// Color mode: dark | bright | colorblind | dark-rgb | bright-rgb | colorblind-rgb
    #[arg(long, default_value = "dark")]
    pub color_mode: String,

    /// Starting Hammer bonus count
    #[arg(long, default_value_t = 2)]
    pub hammer: u16,

    /// Starting Laser bonus count
    #[arg(long, default_value_t = 1)]
    pub laser: u16,

    /// Starting Blaster bonus count
    #[arg(long, default_value_t = 1)]
    pub blaster: u16,

    /// Starting Warp bonus count
    #[arg(long, default_value_t = 1)]
    pub warp: u16,

    /// Starting Color Bomb bonus count
    #[arg(long, default_value_t = 0)]
    pub color_bomb: u16,

    /// Percentage of cells that start with Ice tile modifier (0–100, applied on top of special_tile_pct)
    #[arg(long, default_value_t = 0)]
    pub ice_tile_pct: u16,
}

impl loom_engine::game::GameConfig for Config {
    fn board_width(&self) -> usize { self.board_width as usize }
    fn board_height(&self) -> usize { self.board_height as usize }
    fn color_count(&self) -> usize { self.color_number as usize }
    fn scale(&self) -> u16 { self.scale }
    fn color_mode(&self) -> &str { &self.color_mode }
    fn set_scale(&mut self, s: u16) { self.scale = s; }
    fn set_color_mode(&mut self, m: String) { self.color_mode = m; }

    fn custom_fields(&self) -> Vec<(&'static str, u16)> {
        vec![
            ("Board Height", self.board_height),
            ("Board Width", self.board_width),
            ("Color Count", self.color_number as u16),
            ("Move Limit", self.move_limit as u16),
            ("Special Tile %", self.special_tile_pct),
        ]
    }

    fn adjust_custom_field(&mut self, field: usize, delta: i16) {
        let d = delta as i32;
        match field {
            1 => self.board_height = (self.board_height as i32 + d).clamp(4, 16) as u16,
            2 => self.board_width = (self.board_width as i32 + d).clamp(4, 16) as u16,
            3 => self.color_number = (self.color_number as i32 + d).clamp(3, 7) as u8,
            4 => self.move_limit = (self.move_limit as i32 + d).clamp(10, 99) as u32,
            5 => self.special_tile_pct = (self.special_tile_pct as i32 + d).clamp(0, 50) as u16,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn config_defaults() {
        let cfg = Config::parse_from::<[&str; 0], &str>([]);
        assert_eq!(cfg.board_height, 8);
        assert_eq!(cfg.board_width, 8);
        assert_eq!(cfg.color_number, 6);
        assert_eq!(cfg.move_limit, 30);
        assert_eq!(cfg.special_tile_pct, 5);
        assert_eq!(cfg.scale, 1);
        assert_eq!(cfg.color_mode, "dark");
        assert_eq!(cfg.hammer, 2);
        assert_eq!(cfg.laser, 1);
        assert_eq!(cfg.blaster, 1);
        assert_eq!(cfg.warp, 1);
    }

    #[test]
    fn config_clone() {
        let cfg = Config::parse_from::<[&str; 0], &str>([]);
        let cfg2 = cfg.clone();
        assert_eq!(cfg.board_height, cfg2.board_height);
    }
}
