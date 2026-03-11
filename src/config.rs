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
