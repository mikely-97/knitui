use crossterm::style::Color;
use loom_engine::game::{Game, GameId, GameEngine};

use crate::config::Config;
use crate::campaign_levels::{self, TRACK_NAMES, TRACK_COUNT, levels_for_track};
use crate::campaign::CampaignState;
use crate::endless::EndlessState;
use crate::preset::PRESETS;

pub struct KnitGame;

impl Game for KnitGame {
    type Config = Config;

    fn id(&self) -> GameId { GameId::Knit }
    fn name(&self) -> &'static str { "Knit" }
    fn config_dir(&self) -> &'static str { "knitui" }

    fn create_engine(&self, config: &Config, _palette: &[Color]) -> Box<dyn GameEngine> {
        // TODO: Phase 5 — wrap the existing knitui::engine::GameEngine
        // in a GameEngine trait adapter. For now, panic as placeholder.
        unimplemented!("KnitGame::create_engine will be wired in Phase 5")
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
            format!("Obstacles: {}%, Conveyors: {}%", l.obstacle_percentage, l.conveyor_percentage),
        ]
    }

    fn endless_wave_config(&self, wave: u32, base: &Config) -> Config {
        let mut state = EndlessState::new();
        // Fast-forward to the requested wave
        for _ in 1..wave { state.advance(); }
        state.to_config(base)
    }

    fn help_lines(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Arrow keys", "Move cursor"),
            ("Enter/Space", "Pick up spool"),
            ("Z", "Use scissors"),
            ("X", "Use tweezers"),
            ("C", "Use balloons"),
            ("A", "Watch ad for bonus"),
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
