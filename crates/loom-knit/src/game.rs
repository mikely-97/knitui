use loom_engine::blessings::Blessing;
use loom_engine::render::{Color, Surface};
use loom_engine::game::{
    Action, Game, GameId, GameEngine as GameEngineTrait, GameStatus as EngineGameStatus,
    RenderArea,
};
use loom_engine::input::{Key, KeyEvent};

use crate::board_entity::Direction;
use crate::config::Config;
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, levels_for_track, is_hard_track};
use crate::campaign::CampaignState;
use crate::endless::EndlessState;
use crate::engine::{BonusState, GameEngine as KnitEngine, GameStatus as KnitStatus};
use crate::preset::PRESETS;
use crate::renderer;

pub struct KnitGame;

impl Game for KnitGame {
    type Config = Config;
    type CampaignEntry = CampaignState;

    fn id(&self) -> GameId { GameId::Knit }
    fn name(&self) -> &'static str { "Knit" }
    fn config_dir(&self) -> &'static str { "knitui" }

    fn create_engine(&self, config: &Config, _palette: &[Color]) -> Box<dyn GameEngineTrait> {
        Box::new(KnitEngineAdapter {
            engine: KnitEngine::new(config),
            config: config.clone(),
        })
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

    fn new_campaign_entry(&self, track: usize) -> CampaignState {
        CampaignState::new(track)
    }

    fn campaign_config(&self, entry: &CampaignState, base: &Config) -> Config {
        let mut cfg = entry.to_config(base);
        cfg.hard_mode = is_hard_track(entry.track_idx);
        cfg
    }

    fn create_campaign_engine(
        &self, entry: &CampaignState, config: &Config, palette: &[Color],
    ) -> Box<dyn GameEngineTrait> {
        let mut adapter = KnitEngineAdapter {
            engine: KnitEngine::new(config),
            config: config.clone(),
        };
        adapter.engine.set_ad_limit(entry.ad_limit());
        if !config.hard_mode {
            adapter.engine.set_blessings(&entry.blessings);
        }
        let _ = palette;
        Box::new(adapter)
    }

    fn complete_campaign_level(&self, entry: &mut CampaignState) -> bool {
        entry.complete_level()
    }

    fn all_blessings(&self) -> &'static [Blessing] {
        crate::blessings::ALL_BLESSINGS
    }

    fn confirm_blessings(&self, entry: &mut CampaignState, ids: &[String]) {
        entry.blessings = ids.to_vec();
        if crate::blessings::has(&entry.blessings, "apprentices_kit") {
            entry.banked_scissors += 1;
        }
        if crate::blessings::has(&entry.blessings, "light_pockets") {
            entry.banked_balloons += 1;
        }
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

/// Adapts loom-knit's concrete `GameEngine` (game logic + rendering, already
/// proven in both the terminal and wasm builds) to the portable
/// `loom_engine::game::GameEngine` trait, so the generic `Shell<G>` can drive
/// it without knowing knit-specific types. `config` is kept alongside the
/// engine because rendering geometry (scale, layout, board dimensions) is a
/// presentation concern the concrete engine itself doesn't store.
struct KnitEngineAdapter {
    engine: KnitEngine,
    config: Config,
}

impl GameEngineTrait for KnitEngineAdapter {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.key {
            Key::Left  => { let _ = self.engine.move_cursor(Direction::Left);  Action::Redraw }
            Key::Right => { let _ = self.engine.move_cursor(Direction::Right); Action::Redraw }
            Key::Up    => { let _ = self.engine.move_cursor(Direction::Up);    Action::Redraw }
            Key::Down  => { let _ = self.engine.move_cursor(Direction::Down);  Action::Redraw }
            Key::Enter => {
                let _ = self.engine.pick_up();
                Action::Redraw
            }
            Key::Esc => {
                if self.engine.bonus_state != BonusState::None {
                    self.engine.cancel_tweezers();
                    Action::Redraw
                } else {
                    Action::QuitToMenu
                }
            }
            Key::Char('z') | Key::Char('Z') => { let _ = self.engine.use_scissors(); Action::Redraw }
            Key::Char('x') | Key::Char('X') => { let _ = self.engine.use_tweezers(); Action::Redraw }
            Key::Char('c') | Key::Char('C') => { let _ = self.engine.use_balloons(); Action::Redraw }
            Key::Char('h') | Key::Char('H') => Action::ShowHelp,
            Key::Char('?') => {
                if self.engine.blessing_flags.match_hint {
                    if let Some(cell) = self.engine.compute_hint() {
                        self.engine.hint_cell = Some(cell);
                        self.engine.hint_ticks = 60;
                    }
                }
                Action::Redraw
            }
            _ => Action::None,
        }
    }

    fn tick(&mut self) -> bool {
        self.engine.tick_hint();
        if !self.engine.held_spools.is_empty() {
            self.engine.process_all_active();
            true
        } else {
            false
        }
    }

    fn status(&self) -> EngineGameStatus {
        match self.engine.status() {
            KnitStatus::Playing => EngineGameStatus::Playing,
            KnitStatus::Won => EngineGameStatus::Won { score: None },
            KnitStatus::Stuck => EngineGameStatus::Stuck,
        }
    }

    fn render(&self, surface: &mut dyn Surface, area: RenderArea) {
        let (layout, yarn_x, board_x, board_y) =
            renderer::compute_geometry(&self.config, area.height);
        match layout {
            renderer::Layout::Vertical => {
                renderer::render_vertical_to_surface(surface, &self.engine, board_y, self.config.scale);
            }
            renderer::Layout::Horizontal => {
                renderer::render_horizontal_to_surface(surface, &self.engine, yarn_x, board_x, self.config.scale);
            }
        }
        let status = self.engine.status();
        if !matches!(status, KnitStatus::Playing) {
            renderer::draw_overlay_to_surface(surface, &self.engine, &status, None);
        }
    }

    fn render_keybar(&self, _surface: &mut dyn Surface, _y: u16) {
        // No-op: render_{vertical,horizontal}_to_surface already draw the
        // keybar as part of the same pass (matching current terminal/wasm
        // behavior, which always redraws the whole frame atomically).
    }

    fn score(&self) -> Option<u32> { None }

    fn can_watch_ad(&self) -> bool { self.engine.can_watch_ad() }
    fn watch_ad(&mut self) { self.engine.watch_ad(); }

    fn scale(&self) -> u16 { self.config.scale }
    fn set_scale(&mut self, scale: u16) { self.config.scale = scale; }

    fn board_dims(&self) -> (u16, u16) { (self.engine.board.height, self.engine.board.width) }
}
