use loom_engine::blessings::Blessing;
use loom_engine::render::{Color, Surface};
use loom_engine::game::{
    Action, Game, GameId, GameEngine as GameEngineTrait, GameStatus as EngineGameStatus,
    RenderArea,
};
use loom_engine::input::{Key, KeyEvent};

use crate::config::Config;
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, levels_for_track, LevelObjective};
use crate::campaign::{CampaignState, objective_met};
use crate::endless::EndlessState;
use crate::engine::{GameEngine as M3Engine, GamePhase, GameStatus as M3Status};
use crate::bonuses::BonusState;
use crate::preset::PRESETS;
use crate::renderer::{self, LayoutGeometry};

pub struct M3Game;

impl Game for M3Game {
    type Config = Config;
    type CampaignEntry = CampaignState;

    fn id(&self) -> GameId { GameId::Match3 }
    fn name(&self) -> &'static str { "Match-3" }
    fn config_dir(&self) -> &'static str { "m3tui" }

    fn create_engine(&self, config: &Config, _palette: &[Color]) -> Box<dyn GameEngineTrait> {
        Box::new(M3EngineAdapter::new(config, None))
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

    fn new_campaign_entry(&self, track: usize) -> CampaignState {
        CampaignState::new(track)
    }

    fn campaign_config(&self, entry: &CampaignState, base: &Config) -> Config {
        entry.to_config(base)
    }

    fn create_campaign_engine(
        &self, entry: &CampaignState, config: &Config, palette: &[Color],
    ) -> Box<dyn GameEngineTrait> {
        let mut adapter = M3EngineAdapter::new(config, Some(entry.current_level_def().objective));
        adapter.engine.set_blessings(&entry.blessings);
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

/// Adapts loom-match3's concrete `GameEngine` (game logic + rendering,
/// already proven in both the terminal and wasm builds) to the portable
/// `loom_engine::game::GameEngine` trait.
///
/// Win/lose status is cached rather than computed on demand: the trait's
/// `status(&self)` is immutable, but match3's endless-mode Stuck recovery
/// (auto-consuming a Warp charge) is a mutation, so status is (re)computed
/// inside `tick()`/`handle_key()` — the only two `&mut self` entry points —
/// and `status()` just reads the cached result.
///
/// **Preserves a real quirk of the original `tui.rs`**: in campaign mode,
/// only the level objective is ever checked; a campaign run has no
/// Stuck/OutOfMoves game-over path at all (only Quick/Custom/Endless do).
/// This looks surprising but is the actual pre-existing behavior — not
/// something this migration changes.
struct M3EngineAdapter {
    engine: M3Engine,
    config: Config,
    objective: Option<LevelObjective>,
    cached_status: EngineGameStatus,
}

impl M3EngineAdapter {
    fn new(config: &Config, objective: Option<LevelObjective>) -> Self {
        Self {
            engine: M3Engine::new(config),
            config: config.clone(),
            objective,
            cached_status: EngineGameStatus::Playing,
        }
    }

    fn recompute_status(&mut self) {
        if !matches!(self.engine.phase, GamePhase::PlayerInput) {
            self.cached_status = EngineGameStatus::Playing;
            return;
        }

        if let Some(obj) = &self.objective {
            let special_remaining = self.engine.board.count_modifier(|_| true);
            self.cached_status = if objective_met(obj, self.engine.score, &[], special_remaining) {
                EngineGameStatus::Won { score: Some(self.engine.score) }
            } else {
                EngineGameStatus::Playing
            };
            return;
        }

        if self.config.is_endless && self.engine.moves_used >= self.engine.move_limit && self.engine.score > 0 {
            // Wave complete: Shell's endless handling advances the wave on
            // any `Won` status while an endless run is active.
            self.cached_status = EngineGameStatus::Won { score: Some(self.engine.score) };
            return;
        }

        self.cached_status = match self.engine.game_status() {
            M3Status::Playing => EngineGameStatus::Playing,
            M3Status::OutOfMoves => EngineGameStatus::Lost { reason: "Out of moves".to_string() },
            M3Status::Stuck => {
                if self.config.is_endless && self.engine.bonuses.warp > 0 {
                    self.engine.activate_warp();
                    EngineGameStatus::Playing
                } else {
                    EngineGameStatus::Stuck
                }
            }
            // The concrete engine's own `game_status()` never returns Won —
            // win depends on campaign/endless context it has no knowledge
            // of, which is exactly why this adapter exists.
            M3Status::Won => EngineGameStatus::Playing,
        };
    }
}

impl GameEngineTrait for M3EngineAdapter {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        let bonus_active = !matches!(self.engine.bonus_state, BonusState::None);
        let action = match key.key {
            Key::Up    => { self.engine.move_cursor(-1, 0); Action::Redraw }
            Key::Down  => { self.engine.move_cursor(1, 0); Action::Redraw }
            Key::Left  => { self.engine.move_cursor(0, -1); Action::Redraw }
            Key::Right => { self.engine.move_cursor(0, 1); Action::Redraw }
            Key::Enter | Key::Char(' ') => {
                if matches!(self.engine.bonus_state, BonusState::HammerActive { .. }) {
                    self.engine.confirm_hammer();
                } else if matches!(self.engine.bonus_state, BonusState::ColorBombActive { .. }) {
                    self.engine.confirm_color_bomb();
                } else if matches!(self.engine.phase, GamePhase::PlayerInput) {
                    self.engine.confirm_selection();
                }
                Action::Redraw
            }
            Key::Esc => {
                if bonus_active {
                    self.engine.cancel_bonus();
                    Action::Redraw
                } else if self.engine.selected.is_some() {
                    self.engine.selected = None;
                    Action::Redraw
                } else {
                    Action::QuitToMenu
                }
            }
            Key::Char('h') | Key::Char('H') => {
                if !bonus_active { Action::ShowHelp } else { Action::None }
            }
            Key::Char('z') | Key::Char('Z') => {
                if !bonus_active && self.engine.bonuses.hammer > 0 { self.engine.activate_hammer(); }
                Action::Redraw
            }
            Key::Char('x') | Key::Char('X') => {
                if !bonus_active && self.engine.bonuses.laser > 0 { self.engine.activate_laser(); }
                Action::Redraw
            }
            Key::Char('c') | Key::Char('C') => {
                if !bonus_active && self.engine.bonuses.blaster > 0 { self.engine.activate_blaster(); }
                Action::Redraw
            }
            Key::Char('v') | Key::Char('V') => {
                if !bonus_active && self.engine.bonuses.warp > 0 { self.engine.activate_warp(); }
                Action::Redraw
            }
            Key::Char('b') | Key::Char('B') => {
                if !bonus_active && self.engine.bonuses.color_bomb > 0 { self.engine.activate_color_bomb(); }
                Action::Redraw
            }
            Key::Char('q') | Key::Char('Q') => Action::QuitToMenu,
            _ => Action::None,
        };
        self.recompute_status();
        action
    }

    fn tick(&mut self) -> bool {
        let changed = self.engine.tick();
        self.recompute_status();
        changed
    }

    fn status(&self) -> EngineGameStatus {
        self.cached_status.clone()
    }

    fn render(&self, surface: &mut dyn Surface, area: RenderArea) {
        let geo = LayoutGeometry::for_height(
            self.engine.board.height as usize, self.engine.board.width as usize,
            self.config.scale, area.height,
        );
        let label = objective_label(&self.engine, self.objective.as_ref());
        renderer::do_render_to_surface(surface, &self.engine, &geo, &label);
    }

    fn render_keybar(&self, _surface: &mut dyn Surface, _y: u16) {
        // No-op: do_render_to_surface already draws the key bar as part of
        // the same pass (matches current terminal/wasm behavior).
    }

    fn render_game_over_overlay(&self, surface: &mut dyn Surface, status: &EngineGameStatus, overlay_msg: Option<&str>) {
        let m3_status = match status {
            EngineGameStatus::Won { .. } => M3Status::Won,
            EngineGameStatus::Lost { .. } => M3Status::OutOfMoves,
            EngineGameStatus::Stuck => M3Status::Stuck,
            EngineGameStatus::Playing => M3Status::Playing,
        };
        renderer::render_game_over_to_surface(surface, &m3_status, self.engine.score, overlay_msg);
    }

    fn render_help(&self, surface: &mut dyn Surface) {
        renderer::render_help_to_surface(surface, Some(&self.engine));
    }

    fn render_celebration(&self, surface: &mut dyn Surface, area: RenderArea, tick: u8) {
        let geo = LayoutGeometry::for_height(
            self.engine.board.height as usize, self.engine.board.width as usize,
            self.config.scale, area.height,
        );
        renderer::render_celebration_to_surface(surface, &self.engine, &geo, tick);
    }

    fn score(&self) -> Option<u32> { Some(self.engine.score) }

    fn scale(&self) -> u16 { self.config.scale }
    fn set_scale(&mut self, scale: u16) { self.config.scale = scale; }

    fn board_dims(&self) -> (u16, u16) { (self.engine.board.height as u16, self.engine.board.width as u16) }
}

fn objective_label(engine: &M3Engine, objective: Option<&LevelObjective>) -> String {
    let Some(obj) = objective else { return String::new() };
    let mut parts = Vec::new();
    if let Some(target) = obj.score_target {
        parts.push(format!("Score: {}/{}", engine.score, target));
    }
    if !obj.gem_quota.is_empty() {
        parts.push("Quota: see HUD".to_string());
    }
    if obj.clear_all_specials {
        let remaining = engine.board.count_modifier(|_| true);
        parts.push(format!("Tiles left: {}", remaining));
    }
    parts.join("  ")
}
