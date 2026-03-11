use crossterm::style::Color;
use loom_engine::game::{Game, GameId, GameEngine};

use crate::config::Config;
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, levels_for_track};
use crate::campaign::CampaignState;
use crate::endless::EndlessState;
use crate::preset::PRESETS;

pub struct M3Game;

impl Game for M3Game {
    type Config = Config;

    fn id(&self) -> GameId { GameId::Match3 }
    fn name(&self) -> &'static str { "Match-3" }
    fn config_dir(&self) -> &'static str { "m3tui" }

    fn create_engine(&self, _config: &Config, _palette: &[Color]) -> Box<dyn GameEngine> {
        unimplemented!("M3Game::create_engine will be wired in a future phase")
    }

    fn default_config(&self) -> Config {
        use clap::Parser;
        Config::parse_from::<[&str; 0], &str>([])
    }

    fn track_names(&self) -> &'static [&'static str] { TRACK_NAMES }
    fn track_count(&self) -> usize { TRACK_COUNT }

    fn level_count(&self, track: usize) -> usize {
        levels_for_track(track).len()
    }

    fn level_config(&self, track: usize, level: usize, base: &Config) -> Config {
        let mut state = CampaignState::new(track);
        state.current_level = level;
        state.to_config(base)
    }

    fn level_intro_lines(&self, track: usize, level: usize) -> Vec<String> {
        let levels = levels_for_track(track);
        let l = &levels[level];
        vec![
            format!("{} — Level {}/{}", TRACK_NAMES[track], level + 1, levels.len()),
            format!("Board: {}×{}, {} colors", l.board_height, l.board_width, l.color_number),
            format!("Moves: {}, Special tiles: {}%", l.move_limit, l.special_tile_pct),
        ]
    }

    fn endless_wave_config(&self, wave: u32, base: &Config) -> Config {
        let mut state = EndlessState::new();
        for _ in 1..wave { state.advance(); }
        state.to_config(base)
    }

    fn help_lines(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Arrow keys", "Move cursor"),
            ("Enter/Space", "Select / Swap"),
            ("1", "Use Hammer"),
            ("2", "Use Laser"),
            ("3", "Use Blaster"),
            ("4", "Use Warp"),
            ("H", "Toggle help"),
            ("N/P", "Next/prev color mode"),
            ("+/-", "Scale up/down"),
            ("Esc", "Cancel / back"),
            ("Q", "Quit"),
        ]
    }

    fn presets(&self) -> Vec<(&'static str, Config)> {
        PRESETS.iter().map(|p| {
            (p.name, p.to_config(&self.default_config()))
        }).collect()
    }
}
