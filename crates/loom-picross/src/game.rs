use loom_engine::render::Color;
use loom_engine::game::{Game, GameId, GameEngine};

use crate::campaign::{TRACK_NAMES, TRACK_COUNT, levels_for_track, level_count};
use crate::config::Config;

pub struct PicrossGame;

impl Game for PicrossGame {
    type Config = Config;

    fn id(&self) -> GameId { GameId::Picross }
    fn name(&self) -> &'static str { "Picross" }
    fn config_dir(&self) -> &'static str { "picross" }

    fn create_engine(&self, _config: &Config, _palette: &[Color]) -> Box<dyn GameEngine> {
        unimplemented!("Use tui::run_from_menu() for Picross")
    }

    fn default_config(&self) -> Config {
        Config::default()
    }

    fn track_names(&self) -> &'static [&'static str] { TRACK_NAMES }
    fn track_count(&self) -> usize { TRACK_COUNT }

    fn level_count(&self, track: usize) -> usize {
        level_count(track)
    }

    fn level_config(&self, _track: usize, _level: usize, base: &Config) -> Config {
        base.clone()
    }

    fn level_intro_lines(&self, track: usize, level: usize) -> Vec<String> {
        let levels = levels_for_track(track);
        if let Some(p) = levels.into_iter().nth(level) {
            vec![
                format!("{} — Puzzle {}/{}", TRACK_NAMES[track], level + 1, level_count(track)),
                format!("\"{}\" ({}×{})", p.name, p.rows, p.cols),
            ]
        } else {
            vec!["Unknown puzzle".to_string()]
        }
    }

    fn endless_wave_config(&self, _wave: u32, base: &Config) -> Config {
        base.clone()
    }

    fn help_lines(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Arrow keys", "Move cursor"),
            ("Space / Enter", "Fill cell"),
            ("X", "Cross out cell"),
            ("Q / Esc", "Quit to menu"),
        ]
    }

    fn presets(&self) -> Vec<(&'static str, Config)> {
        vec![]
    }
}
