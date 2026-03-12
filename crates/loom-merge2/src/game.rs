use crossterm::style::Color;
use loom_engine::game::{Game, GameId, GameEngine};

use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, mission_count, track_def};
use crate::config::Config;
use crate::endless::EndlessParams;
use crate::preset::PRESETS;

pub struct M2Game;

impl Game for M2Game {
    type Config = Config;

    fn id(&self) -> GameId { GameId::Merge2 }
    fn name(&self) -> &'static str { "Merge-2" }
    fn config_dir(&self) -> &'static str { "m2tui" }

    fn create_engine(&self, _config: &Config, _palette: &[Color]) -> Box<dyn GameEngine> {
        // Merge-2 drives gameplay through its own tui::run_from_menu() loop.
        unimplemented!("Use tui::run_from_menu() for Merge-2")
    }

    fn default_config(&self) -> Config {
        Config::default()
    }

    fn track_names(&self) -> &'static [&'static str] { TRACK_NAMES }
    fn track_count(&self) -> usize { TRACK_COUNT }
    fn level_count(&self, track: usize) -> usize { mission_count(track) }

    fn level_config(&self, track: usize, _level: usize, base: &Config) -> Config {
        let td = track_def(track);
        Config {
            board_rows:          td.initial_layout.rows as u16,
            board_cols:          td.initial_layout.cols as u16,
            energy_max:          td.energy_max,
            energy_regen_secs:   td.energy_regen_secs,
            generator_cost:      td.generator_cost,
            generator_cooldown:  td.generator_cooldown,
            random_order_count:  td.random_order_count as u16,
            max_order_tier:      td.max_order_tier,
            soft_gen_chance:     td.soft_gen_chance,
            inventory_slots:     td.inventory_slots,
            ad_limit:            td.ad_limit,
            scale:               base.scale,
            color_mode:          base.color_mode.clone(),
            family_count:        6,
        }
    }

    fn level_intro_lines(&self, track: usize, level: usize) -> Vec<String> {
        let td = track_def(track);
        let total = td.missions.len();
        let desc = td.missions.get(level)
            .map(|m| m.description)
            .unwrap_or("Unknown mission");
        vec![
            format!("{} — Mission {}/{}", TRACK_NAMES[track], level + 1, total),
            desc.to_string(),
        ]
    }

    fn endless_wave_config(&self, wave: u32, base: &Config) -> Config {
        let merges = wave as u64 * 50;
        let params = EndlessParams::from_merges(merges);
        Config {
            energy_max:         params.energy_max,
            energy_regen_secs:  params.energy_regen_secs,
            generator_cooldown: params.generator_cooldown,
            random_order_count: params.random_order_count as u16,
            max_order_tier:     params.max_order_tier,
            generator_cost:     params.generator_cost,
            family_count:       params.families.len() as u16,
            scale:              base.scale,
            color_mode:         base.color_mode.clone(),
            ..base.clone()
        }
    }

    fn help_lines(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("↑↓←→",    "Move cursor"),
            ("Enter",   "Select / Merge / Activate gen"),
            ("D",       "Deliver selected to order"),
            ("S",       "Store in inventory"),
            ("I",       "Open inventory"),
            ("A",       "Watch ad (if available)"),
            ("H",       "Toggle help"),
            ("N/P",     "Cycle color mode"),
            ("+/-",     "Scale up/down"),
            ("Q / Esc", "Back / Quit"),
        ]
    }

    fn presets(&self) -> Vec<(&'static str, Config)> {
        PRESETS.iter().map(|p| (p.name, p.to_config(&self.default_config()))).collect()
    }
}
