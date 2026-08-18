use clap::Parser;
use std::path::PathBuf;
use crate::palette::ColorMode;

/// Hard cap on board dimensions (height and width).
pub const MAX_BOARD_DIM: u16 = 6;

#[derive(Parser, Clone)]
#[command(name = "knitui", about = "Terminal knitting puzzle game")]
pub struct Config {
    #[arg(long, default_value_t = 6, help = "Board height in rows")]
    pub board_height: u16,

    #[arg(long, default_value_t = 6, help = "Board width in columns")]
    pub board_width: u16,

    #[arg(long, default_value_t = 6, help = "Number of distinct colors")]
    pub color_number: u16,

    #[arg(long, default_value = "dark", help = "Color palette: dark | bright | colorblind")]
    pub color_mode: String,

    #[arg(long, default_value_t = 7, help = "Max held spools at once")]
    pub spool_limit: usize,

    #[arg(long, default_value_t = 3, help = "Times each spool must be wound to complete")]
    pub spool_capacity: u16,

    #[arg(long, default_value_t = 4, help = "Number of yarn columns")]
    pub yarn_lines: u16,

    #[arg(long, default_value_t = 5, help = "Percent chance each cell is an obstacle (0-100)")]
    pub obstacle_percentage: u16,

    #[arg(long, default_value_t = 6, help = "Visible yarn rows shown on screen")]
    pub visible_stitches: u16,

    #[arg(long, default_value_t = 3, help = "Spools each conveyor produces before depleting")]
    pub conveyor_capacity: u16,

    #[arg(long, default_value_t = 5, help = "Percent chance each cell becomes a conveyor (0-100)")]
    pub conveyor_percentage: u16,

    #[arg(long, default_value = "auto", help = "Layout: auto | horizontal | vertical")]
    pub layout: String,

    #[arg(long, default_value_t = 1, help = "Cell scale factor (1-3): render each entity as NxN characters")]
    pub scale: u16,

    #[arg(long, default_value_t = 0, help = "Starting scissors bonus count")]
    pub scissors: u16,

    #[arg(long, default_value_t = 0, help = "Starting tweezers bonus count")]
    pub tweezers: u16,

    #[arg(long, default_value_t = 0, help = "Starting balloons bonus count")]
    pub balloons: u16,

    #[arg(long, default_value_t = 1, help = "Spools cut per scissors use")]
    pub scissors_spools: u16,

    #[arg(long, default_value_t = 2, help = "Stitches lifted per yarn column per balloons use")]
    pub balloon_count: u16,

    #[arg(long, help = "Path to ad quotes file (one per line, default: ~/.config/knitui/ads.txt)")]
    pub ad_file: Option<PathBuf>,

    #[arg(long, help = "Max distinct winning pick sequences (1 = single forced-sequence puzzle). Slower generation for small values.")]
    pub max_solutions: Option<u64>,

    #[arg(long, default_value_t = false, help = "Hard mode: no blessings, no bonuses, no solvability guarantee")]
    pub hard_mode: bool,
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
            ("Color Count", self.color_number),
            ("Obstacle %", self.obstacle_percentage),
            ("Conveyor %", self.conveyor_percentage),
            ("Scissors", self.scissors),
            ("Tweezers", self.tweezers),
            ("Balloons", self.balloons),
            ("Hard Mode", if self.hard_mode { 1 } else { 0 }),
        ]
    }

    fn adjust_custom_field(&mut self, field: usize, delta: i16) {
        let apply = |val: &mut u16, min: u16, max: u16| {
            *val = (*val as i16 + delta).clamp(min as i16, max as i16) as u16;
        };
        match field {
            1 => apply(&mut self.board_height, 2, MAX_BOARD_DIM),
            2 => apply(&mut self.board_width, 2, MAX_BOARD_DIM),
            3 => apply(&mut self.color_number, 2, 8),
            4 => apply(&mut self.obstacle_percentage, 0, 50),
            5 => apply(&mut self.conveyor_percentage, 0, 50),
            6 => apply(&mut self.scissors, 0, 99),
            7 => apply(&mut self.tweezers, 0, 99),
            8 => apply(&mut self.balloons, 0, 99),
            9 => { self.hard_mode = !self.hard_mode; }
            _ => {}
        }
    }
}

impl Config {
    pub fn parsed_color_mode(&self) -> ColorMode {
        match self.color_mode.to_lowercase().as_str() {
            "bright" | "light" => ColorMode::Bright,
            "colorblind" | "grey" | "gray" => ColorMode::Colorblind,
            "dark-rgb" => ColorMode::DarkRgb,
            "bright-rgb" | "light-rgb" => ColorMode::BrightRgb,
            "colorblind-rgb" | "grey-rgb" | "gray-rgb" => ColorMode::ColorblindRgb,
            _ => ColorMode::Dark,
        }
    }
}
