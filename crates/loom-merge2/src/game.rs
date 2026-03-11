use crossterm::style::Color;
use loom_engine::game::{Game, GameId, GameEngine, GameConfig};

/// Minimal Config stub for merge-2.
#[derive(Clone, Debug)]
pub struct Config {
    pub board_width: u16,
    pub board_height: u16,
    pub color_count: u8,
    pub scale: u16,
    pub color_mode: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            board_width: 6,
            board_height: 6,
            color_count: 5,
            scale: 1,
            color_mode: "dark".to_string(),
        }
    }
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

pub struct M2Game;

impl Game for M2Game {
    type Config = Config;

    fn id(&self) -> GameId { GameId::Merge2 }
    fn name(&self) -> &'static str { "Merge-2" }
    fn config_dir(&self) -> &'static str { "m2tui" }

    fn create_engine(&self, _config: &Config, _palette: &[Color]) -> Box<dyn GameEngine> {
        unimplemented!("Merge-2 is not yet implemented")
    }

    fn default_config(&self) -> Config {
        Config::default()
    }

    fn track_names(&self) -> &'static [&'static str] { &["Starter"] }
    fn track_count(&self) -> usize { 1 }
    fn level_count(&self, _track: usize) -> usize { 0 }

    fn level_config(&self, _track: usize, _level: usize, base: &Config) -> Config {
        base.clone()
    }

    fn level_intro_lines(&self, _track: usize, _level: usize) -> Vec<String> {
        vec!["Merge-2 coming soon!".to_string()]
    }

    fn endless_wave_config(&self, _wave: u32, base: &Config) -> Config {
        base.clone()
    }

    fn help_lines(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Arrow keys", "Move cursor"),
            ("Enter", "Select / Merge"),
            ("Esc", "Back"),
            ("Q", "Quit"),
        ]
    }

    fn presets(&self) -> Vec<(&'static str, Config)> {
        vec![("Default", Config::default())]
    }
}
