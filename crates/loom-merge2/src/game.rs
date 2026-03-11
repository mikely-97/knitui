use crossterm::style::Color;
use loom_engine::game::{Game, GameId, GameEngine};

use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, levels_for_track};
use crate::config::Config;
use crate::endless::EndlessState;
use crate::preset::PRESETS;

pub struct M2Game;

impl Game for M2Game {
    type Config = Config;

    fn id(&self) -> GameId { GameId::Merge2 }
    fn name(&self) -> &'static str { "Merge-2" }
    fn config_dir(&self) -> &'static str { "m2tui" }

    fn create_engine(&self, _config: &Config, _palette: &[Color]) -> Box<dyn GameEngine> {
        // Merge-2 uses its own concrete engine via tui.rs directly.
        unimplemented!("Use M2Engine via tui::run_from_menu() instead")
    }

    fn default_config(&self) -> Config {
        use clap::Parser;
        Config::parse_from::<[&str; 0], &str>([])
    }

    fn track_names(&self) -> &'static [&'static str] { TRACK_NAMES }
    fn track_count(&self) -> usize { TRACK_COUNT }
    fn level_count(&self, track: usize) -> usize { levels_for_track(track).len() }

    fn level_config(&self, track: usize, level: usize, base: &Config) -> Config {
        use crate::campaign::CampaignState;
        let mut state = CampaignState::new(track);
        state.current_level = level;
        state.to_config(base)
    }

    fn level_intro_lines(&self, track: usize, level: usize) -> Vec<String> {
        let levels = levels_for_track(track);
        let l = &levels[level];
        vec![
            format!("{} — Level {}/{}", TRACK_NAMES[track], level + 1, levels.len()),
            format!("Board: {}×{}, {} colors", l.board_height, l.board_width, l.color_count),
            format!("Generators: {}, Orders: {}", l.generator_count, l.orders.len()),
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
            ("Enter/Space", "Select / Merge"),
            ("D", "Deliver to order"),
            ("A", "Watch ad"),
            ("Esc", "Deselect / Back"),
            ("H", "Toggle help"),
            ("N/P", "Next/prev color mode"),
            ("+/-", "Scale up/down"),
            ("Q", "Quit"),
        ]
    }

    fn presets(&self) -> Vec<(&'static str, Config)> {
        PRESETS.iter().map(|p| (p.name, p.to_config(&self.default_config()))).collect()
    }
}
