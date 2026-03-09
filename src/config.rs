use clap::Parser;
use crate::palette::ColorMode;

#[derive(Parser)]
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

    #[arg(long, default_value_t = 7, help = "Max active threads held at once")]
    pub active_threads_limit: usize,

    #[arg(long, default_value_t = 3, help = "Times each thread must be processed to complete")]
    pub knit_volume: u16,

    #[arg(long, default_value_t = 4, help = "Number of yarn columns")]
    pub yarn_lines: u16,

    #[arg(long, default_value_t = 5, help = "Percent chance each cell is an obstacle (0-100)")]
    pub obstacle_percentage: u16,

    #[arg(long, default_value_t = 6, help = "Visible yarn rows shown on screen")]
    pub visible_patches: u16,

    #[arg(long, default_value_t = 3, help = "Threads each generator produces before depleting")]
    pub generator_capacity: u16,
}

impl Config {
    pub fn parsed_color_mode(&self) -> ColorMode {
        match self.color_mode.to_lowercase().as_str() {
            "bright" | "light" => ColorMode::Bright,
            "colorblind" | "grey" | "gray" => ColorMode::Colorblind,
            _ => ColorMode::Dark,
        }
    }
}
