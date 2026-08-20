//! Generic game-shell state machine: the campaign/endless/config/blessings/
//! menu scaffolding that was duplicated near-identically across all 4
//! games' `tui.rs` files, now written once against `Game`/`GameEngine`.
//!
//! `Shell<G>` is deliberately I/O-free and non-blocking, matching the core
//! loop shape decided for this whole pivot: every frontend drives it in
//! single steps (`handle_key`, `tick`, `render`) and owns its own event
//! loop / terminal I/O / timing. A native binary wraps this in a blocking
//! `crossterm` poll loop (see loom-knit's `tui.rs` after its Phase 3
//! cutover); a browser build could drive the same `Shell` from
//! `requestAnimationFrame` without any changes here.
//!
//! Screens that are genuinely per-game (the Help screen's live
//! blessing/bonus-state display, anything behind `extra_screens`) are not
//! handled here — those stay behind `GameEngine` trait methods, following
//! the same escape-hatch pattern already used for `create_campaign_engine`.

pub mod chrome;

use std::time::Instant;

use crate::campaign::{CampaignEntry, CampaignSaves};
use crate::endless::EndlessHighScore;
use crate::game::{Action, Game, GameConfig, GameEngine, GameStatus, MenuItem, RenderArea};
use crate::input::{Key, KeyEvent};
use crate::render::Surface;
use crate::settings::UserSettings;

/// Screen/mode the shell is currently in. Generic over `G` because
/// `CustomGame` holds a live, editable `G::Config`.
pub enum TuiState<G: Game> {
    MainMenu { selected: usize, flash: Option<String> },
    CustomGame { preset_idx: usize, selected_field: usize, config: G::Config },
    CampaignSelect { selected: usize },
    BlessingSelection { cursor: usize, chosen: Vec<usize> },
    CampaignLevelIntro,
    Options { selected: usize },
    Playing,
    Celebration { ticks_remaining: u8, next_status: GameStatus },
    GameOver(GameStatus),
    Help,
    WatchingAd { started_at: Instant, quote: String },
}

/// Number of celebration animation ticks before settling into GameOver.
const CELEBRATION_TICKS: u8 = 16;
const AD_DURATION_SECS: u64 = 15;

pub struct Shell<G: Game> {
    game: G,
    cli_config: G::Config,
    user_settings: UserSettings,
    campaign_saves: CampaignSaves<G::CampaignEntry>,
    campaign_ctx: Option<G::CampaignEntry>,
    /// Current endless wave, or `None` if not in an endless run. Endless is
    /// a mode flag layered over Playing/Celebration/GameOver in every game
    /// that has it, not a distinct screen — matching how all 3 full games'
    /// tui.rs already modeled it.
    endless_wave: Option<u32>,
    endless_hs: EndlessHighScore,
    game_config: G::Config,
    engine: Option<Box<dyn GameEngine>>,
    state: TuiState<G>,
    ad_quotes: Vec<String>,
    /// What watching an ad grants, for the ad-overlay screen's header
    /// (e.g. "FREE SCISSORS"). Empty string if this game has no ad reward
    /// to advertise (the overlay just won't be reachable: `can_watch_ad()`
    /// gates it).
    ad_reward_label: String,
    want_quit: bool,
}

impl<G: Game> Shell<G> {
    /// `skip_menu`: start directly in Playing with `cli_config` (mirrors
    /// each game's `--skip-menu`-equivalent CLI flag). Loading
    /// campaign/settings/high-score state and reading ad quotes off disk
    /// are the native binary's job (I/O) — pass the results in here.
    pub fn new(
        game: G,
        cli_config: G::Config,
        user_settings: UserSettings,
        campaign_saves: CampaignSaves<G::CampaignEntry>,
        endless_hs: EndlessHighScore,
        ad_quotes: Vec<String>,
        ad_reward_label: String,
        skip_menu: bool,
    ) -> Self {
        let game_config = cli_config.clone();
        let (engine, state) = if skip_menu {
            let e = game.create_engine(&game_config, &[]);
            (Some(e), TuiState::Playing)
        } else {
            (None, TuiState::MainMenu { selected: 0, flash: None })
        };
        Self {
            game,
            cli_config,
            user_settings,
            campaign_saves,
            campaign_ctx: None,
            endless_wave: None,
            endless_hs,
            game_config,
            engine,
            state,
            ad_quotes,
            ad_reward_label,
            want_quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.want_quit
    }

    /// Persist whatever needs saving on the way out (in-progress campaign
    /// run). Call once, right before exiting the native event loop.
    pub fn save_on_exit(&mut self) {
        self.sync_campaign_state();
        if let Some(ctx) = self.campaign_ctx.take() {
            self.campaign_saves.upsert(ctx);
            self.campaign_saves.save(self.game.config_dir());
        }
    }

    /// Sync live engine state into the in-progress campaign entry, for games
    /// whose `CampaignEntry` embeds mutating world state (see
    /// `Game::sync_campaign_entry`'s doc comment). No-op default for the
    /// other games. Called before every point that might save `campaign_ctx`.
    fn sync_campaign_state(&mut self) {
        if let (Some(engine), Some(entry)) = (self.engine.as_ref(), self.campaign_ctx.as_mut()) {
            self.game.sync_campaign_entry(engine.as_ref(), entry);
        }
    }

    // ── Rendering ────────────────────────────────────────────────────────

    pub fn render(&self, surface: &mut dyn Surface) {
        let (w, h) = surface.size();
        match &self.state {
            TuiState::MainMenu { selected, flash } => {
                chrome::render_main_menu(surface, self.game.name(), self.game.main_menu_items(), *selected, flash.as_deref());
            }
            TuiState::CustomGame { preset_idx, selected_field, config } => {
                let fields = config.custom_fields();
                let presets = self.game.presets();
                chrome::render_custom_game(surface, presets[*preset_idx].0, &fields, *selected_field);
            }
            TuiState::CampaignSelect { selected } => {
                self.render_campaign_select(surface, *selected);
            }
            TuiState::BlessingSelection { cursor, chosen } => {
                let completed = self.campaign_saves.completed_count();
                chrome::render_blessing_selection(surface, self.game.all_blessings(), *cursor, chosen, completed);
            }
            TuiState::CampaignLevelIntro => {
                if let Some(entry) = &self.campaign_ctx {
                    let cfg = self.game.campaign_config(entry, &self.cli_config);
                    let names = self.game.track_names();
                    chrome::render_level_intro(
                        surface, names[entry.track_idx()], entry.current_level() + 1, entry.total_levels(),
                        cfg.board_height() as u16, cfg.board_width() as u16, cfg.color_count() as u16,
                    );
                }
            }
            TuiState::Options { selected } => {
                chrome::render_options(surface, *selected, self.user_settings.scale, &self.user_settings.color_mode);
            }
            TuiState::Playing => {
                if let Some(engine) = &self.engine {
                    engine.render(surface, RenderArea { x: 0, y: 0, width: w, height: h });
                }
            }
            TuiState::Celebration { ticks_remaining, .. } => {
                if let Some(engine) = &self.engine {
                    let area = RenderArea { x: 0, y: 0, width: w, height: h };
                    engine.render(surface, area);
                    let tick = CELEBRATION_TICKS.saturating_sub(*ticks_remaining);
                    engine.render_celebration(surface, area, tick);
                }
            }
            TuiState::GameOver(status) => {
                if let Some(engine) = &self.engine {
                    engine.render(surface, RenderArea { x: 0, y: 0, width: w, height: h });
                    if let Some(wave) = self.endless_wave {
                        if matches!(status, GameStatus::Stuck) {
                            chrome::render_endless_gameover(surface, wave as usize, self.endless_hs.best_wave);
                            return;
                        }
                    }
                    engine.render_game_over_overlay(surface, status, self.campaign_overlay_msg(status).as_deref());
                }
            }
            TuiState::Help => {
                if let Some(engine) = &self.engine {
                    engine.render_help(surface);
                }
            }
            TuiState::WatchingAd { started_at, quote } => {
                chrome::render_ad_overlay(surface, &self.ad_reward_label, quote, started_at, AD_DURATION_SECS);
            }
        }
    }

    fn render_campaign_select(&self, surface: &mut dyn Surface, selected: usize) {
        let names = self.game.track_names();
        let sizes: Vec<usize> = (0..self.game.track_count()).map(|i| self.game.level_count(i)).collect();
        let labels: Vec<String> = (0..self.game.track_count()).map(|i| self.campaign_saves.progress_label(i)).collect();
        chrome::render_campaign_select(surface, selected, names, &sizes, &labels);
    }

    fn campaign_overlay_msg(&self, status: &GameStatus) -> Option<String> {
        let ctx = self.campaign_ctx.as_ref()?;
        let label = format!("[{}/{}]", ctx.current_level() + 1, ctx.total_levels());
        Some(match status {
            GameStatus::Won { .. } => format!("{} You won! N:Next Level  M:Menu  Q:Quit", label),
            GameStatus::Stuck => format!("{} You're lost! R:Retry  A:Ad  M:Menu  Q:Quit", label),
            GameStatus::Lost { .. } => format!("{} You lost! R:Retry  M:Menu  Q:Quit", label),
            GameStatus::Playing => return None,
        })
    }

    // ── Ticking (animation / background processing) ────────────────────────

    pub fn tick(&mut self) {
        match &mut self.state {
            TuiState::Playing => {
                if let Some(engine) = self.engine.as_mut() {
                    engine.tick();
                }
                self.after_status_change();
            }
            TuiState::Celebration { ticks_remaining, .. } => {
                if *ticks_remaining > 0 {
                    *ticks_remaining -= 1;
                } else if let TuiState::Celebration { next_status, .. } = &self.state {
                    let status = next_status.clone();
                    self.state = TuiState::GameOver(status);
                }
            }
            _ => {}
        }
    }

    // ── Status-driven transitions (Won/Stuck after a play action) ──────────

    fn after_status_change(&mut self) {
        self.sync_campaign_state();
        let Some(engine) = self.engine.as_ref() else { return };
        let status = engine.status();
        match status {
            GameStatus::Playing => {}
            GameStatus::Won { .. } if self.endless_wave.is_some() => {
                self.advance_endless_wave();
            }
            GameStatus::Won { .. } => {
                self.state = TuiState::Celebration { ticks_remaining: CELEBRATION_TICKS, next_status: status };
            }
            _ => {
                self.state = TuiState::GameOver(status);
            }
        }
    }

    fn advance_endless_wave(&mut self) {
        let wave = self.endless_wave.unwrap_or(1) + 1;
        self.endless_wave = Some(wave);
        self.game_config = self.game.endless_wave_config(wave, &self.cli_config);
        self.engine = Some(self.game.create_engine(&self.game_config, &[]));
    }

    // ── Input ────────────────────────────────────────────────────────────

    pub fn handle_key(&mut self, key: KeyEvent) {
        match &mut self.state {
            TuiState::MainMenu { .. } => self.handle_main_menu(key),
            TuiState::CustomGame { .. } => self.handle_custom_game(key),
            TuiState::CampaignSelect { .. } => self.handle_campaign_select(key),
            TuiState::BlessingSelection { .. } => self.handle_blessing_selection(key),
            TuiState::CampaignLevelIntro => self.handle_campaign_level_intro(key),
            TuiState::Options { .. } => self.handle_options(key),
            TuiState::Playing => self.handle_playing(key),
            TuiState::GameOver(_) => self.handle_game_over(key),
            TuiState::Help => {
                self.state = TuiState::Playing;
            }
            TuiState::WatchingAd { started_at, .. } => {
                if key.key == Key::Esc && started_at.elapsed().as_secs() >= AD_DURATION_SECS {
                    if let Some(engine) = self.engine.as_mut() {
                        engine.watch_ad();
                    }
                    self.after_status_change_or_playing();
                }
            }
            TuiState::Celebration { .. } => {} // driven by tick(), ignores keys
        }
    }

    fn after_status_change_or_playing(&mut self) {
        let Some(engine) = self.engine.as_ref() else { return };
        self.state = match engine.status() {
            GameStatus::Playing => TuiState::Playing,
            status => TuiState::GameOver(status),
        };
    }

    fn handle_main_menu(&mut self, key: KeyEvent) {
        let items = self.game.main_menu_items();
        let TuiState::MainMenu { selected, flash } = &mut self.state else { unreachable!() };
        *flash = None;
        match key.key {
            Key::Up => { if *selected > 0 { *selected -= 1; } }
            Key::Down => { if *selected < items.len().saturating_sub(1) { *selected += 1; } }
            Key::Esc | Key::Char('q') | Key::Char('Q') => { self.want_quit = true; }
            Key::Enter => {
                match items.get(*selected).copied() {
                    Some(MenuItem::QuickGame) => {
                        self.game_config = self.cli_config.clone();
                        self.engine = Some(self.game.create_engine(&self.game_config, &[]));
                        self.state = TuiState::Playing;
                    }
                    Some(MenuItem::CustomGame) => {
                        let presets = self.game.presets();
                        let cfg = presets.get(1).or(presets.get(0)).map(|(_, c)| c.clone())
                            .unwrap_or_else(|| self.cli_config.clone());
                        self.state = TuiState::CustomGame { preset_idx: 1.min(presets.len().saturating_sub(1)), selected_field: 0, config: cfg };
                    }
                    Some(MenuItem::Campaign) => { self.state = TuiState::CampaignSelect { selected: 0 }; }
                    Some(MenuItem::Endless) => {
                        self.endless_wave = Some(1);
                        self.game_config = self.game.endless_wave_config(1, &self.cli_config);
                        self.engine = Some(self.game.create_engine(&self.game_config, &[]));
                        self.state = TuiState::Playing;
                    }
                    Some(MenuItem::Options) => { self.state = TuiState::Options { selected: 0 }; }
                    Some(MenuItem::Quit) | None => { self.want_quit = true; }
                }
            }
            _ => {}
        }
    }

    fn handle_custom_game(&mut self, key: KeyEvent) {
        let presets = self.game.presets();
        let TuiState::CustomGame { preset_idx, selected_field, config } = &mut self.state else { unreachable!() };
        let field_count = config.custom_fields().len();
        match key.key {
            Key::Up => { if *selected_field > 0 { *selected_field -= 1; } }
            Key::Down => { if *selected_field < field_count { *selected_field += 1; } }
            Key::Left => {
                if *selected_field == 0 {
                    *preset_idx = if *preset_idx > 0 { *preset_idx - 1 } else { presets.len().saturating_sub(1) };
                    *config = presets[*preset_idx].1.clone();
                } else {
                    config.adjust_custom_field(*selected_field, -1);
                }
            }
            Key::Right => {
                if *selected_field == 0 {
                    *preset_idx = (*preset_idx + 1) % presets.len().max(1);
                    *config = presets[*preset_idx].1.clone();
                } else {
                    config.adjust_custom_field(*selected_field, 1);
                }
            }
            Key::Enter => {
                self.game_config = config.clone();
                self.engine = Some(self.game.create_engine(&self.game_config, &[]));
                self.state = TuiState::Playing;
            }
            Key::Esc => {
                self.state = TuiState::MainMenu { selected: 1, flash: None };
            }
            _ => {}
        }
    }

    fn handle_campaign_select(&mut self, key: KeyEvent) {
        let TuiState::CampaignSelect { selected } = &mut self.state else { unreachable!() };
        let track_count = self.game.track_count();
        match key.key {
            Key::Up => { if *selected > 0 { *selected -= 1; } }
            Key::Down => { if *selected < track_count.saturating_sub(1) { *selected += 1; } }
            Key::Esc => { self.state = TuiState::MainMenu { selected: 2, flash: None }; }
            Key::Enter => {
                let track_idx = *selected;
                let existing = self.campaign_saves.get(track_idx).cloned();
                let entry = match existing {
                    Some(e) if e.is_completed() => {
                        self.campaign_saves.reset(track_idx);
                        self.game.new_campaign_entry(track_idx)
                    }
                    Some(e) => e,
                    None => self.game.new_campaign_entry(track_idx),
                };
                let needs_blessings = entry.needs_blessing_selection();
                self.campaign_ctx = Some(entry);
                if needs_blessings {
                    self.state = TuiState::BlessingSelection { cursor: 0, chosen: vec![] };
                } else {
                    self.state = TuiState::CampaignLevelIntro;
                }
            }
            _ => {}
        }
    }

    fn handle_blessing_selection(&mut self, key: KeyEvent) {
        let completed = self.campaign_saves.completed_count();
        let total = self.game.all_blessings().len();
        let cols = 3usize;
        let TuiState::BlessingSelection { cursor, chosen } = &mut self.state else { unreachable!() };
        match key.key {
            Key::Up => { if *cursor >= cols { *cursor -= cols; } }
            Key::Down => { if *cursor + cols < total { *cursor += cols; } }
            Key::Left => { if *cursor % cols > 0 { *cursor -= 1; } }
            Key::Right => { if *cursor % cols < cols - 1 && *cursor + 1 < total { *cursor += 1; } }
            Key::Enter | Key::Char(' ') => {
                let b = &self.game.all_blessings()[*cursor];
                if crate::blessings::is_unlocked(b, completed) {
                    if let Some(pos) = chosen.iter().position(|&i| i == *cursor) {
                        chosen.remove(pos);
                    } else if chosen.len() < 3 {
                        chosen.push(*cursor);
                    }
                }
            }
            Key::Char('c') | Key::Char('C') if chosen.len() == 3 => {
                let ids: Vec<String> = chosen.iter().map(|&i| self.game.all_blessings()[i].id.to_string()).collect();
                if let Some(entry) = self.campaign_ctx.as_mut() {
                    self.game.confirm_blessings(entry, &ids);
                    self.campaign_saves.upsert(entry.clone());
                    self.campaign_saves.save(self.game.config_dir());
                }
                self.state = TuiState::CampaignLevelIntro;
            }
            Key::Esc => {
                self.campaign_ctx = None;
                self.state = TuiState::CampaignSelect { selected: 0 };
            }
            _ => {}
        }
    }

    fn handle_campaign_level_intro(&mut self, key: KeyEvent) {
        match key.key {
            Key::Enter => {
                if let Some(entry) = &self.campaign_ctx {
                    let cfg = self.game.campaign_config(entry, &self.cli_config);
                    self.engine = Some(self.game.create_campaign_engine(entry, &cfg, &[]));
                    self.game_config = cfg;
                    self.state = TuiState::Playing;
                }
            }
            Key::Esc => {
                self.campaign_ctx = None;
                self.state = TuiState::CampaignSelect { selected: 0 };
            }
            _ => {}
        }
    }

    fn handle_options(&mut self, key: KeyEvent) {
        let TuiState::Options { selected } = &mut self.state else { unreachable!() };
        match key.key {
            Key::Up => { if *selected > 0 { *selected -= 1; } }
            Key::Down => { if *selected < 1 { *selected += 1; } }
            Key::Left => match *selected {
                0 => { if self.user_settings.scale > 1 { self.user_settings.scale -= 1; } }
                1 => { self.user_settings.color_mode = crate::settings::prev_color_mode(&self.user_settings.color_mode).to_string(); }
                _ => {}
            },
            Key::Right => match *selected {
                0 => { if self.user_settings.scale < 5 { self.user_settings.scale += 1; } }
                1 => { self.user_settings.color_mode = crate::settings::next_color_mode(&self.user_settings.color_mode).to_string(); }
                _ => {}
            },
            Key::Esc => {
                self.user_settings.save(self.game.config_dir());
                self.cli_config.set_scale(self.user_settings.scale);
                self.cli_config.set_color_mode(self.user_settings.color_mode.clone());
                self.state = TuiState::MainMenu { selected: 4, flash: None };
            }
            _ => {}
        }
    }

    fn handle_playing(&mut self, key: KeyEvent) {
        if matches!(key.key, Key::Char('a') | Key::Char('A')) {
            if let Some(engine) = self.engine.as_ref() {
                if engine.can_watch_ad() {
                    let quote = pick_quote(&self.ad_quotes);
                    self.state = TuiState::WatchingAd { started_at: Instant::now(), quote };
                }
            }
            return;
        }
        let Some(engine) = self.engine.as_mut() else { return };
        match engine.handle_key(key) {
            Action::ShowHelp => { self.state = TuiState::Help; }
            Action::QuitToMenu => { self.quit_to_menu(); }
            Action::Quit => { self.want_quit = true; }
            Action::Redraw | Action::None => { self.after_status_change(); }
        }
    }

    fn quit_to_menu(&mut self) {
        self.sync_campaign_state();
        if let Some(ctx) = self.campaign_ctx.take() {
            self.campaign_saves.upsert(ctx);
            self.campaign_saves.save(self.game.config_dir());
        }
        self.endless_wave = None;
        self.state = TuiState::MainMenu { selected: 0, flash: None };
    }

    fn handle_game_over(&mut self, key: KeyEvent) {
        let TuiState::GameOver(status) = &self.state else { unreachable!() };
        let status = status.clone();
        match key.key {
            Key::Char('a') | Key::Char('A') => {
                if let Some(engine) = self.engine.as_ref() {
                    if engine.can_watch_ad() {
                        let quote = pick_quote(&self.ad_quotes);
                        self.state = TuiState::WatchingAd { started_at: Instant::now(), quote };
                    }
                }
            }
            Key::Char('r') | Key::Char('R') | Key::Char('n') | Key::Char('N') => {
                if self.endless_wave.is_some() {
                    self.endless_wave = Some(1);
                    self.game_config = self.game.endless_wave_config(1, &self.cli_config);
                    self.engine = Some(self.game.create_engine(&self.game_config, &[]));
                    self.state = TuiState::Playing;
                } else if let Some(mut entry) = self.campaign_ctx.take() {
                    if matches!(status, GameStatus::Won { .. }) {
                        let done = self.game.complete_campaign_level(&mut entry);
                        self.campaign_saves.upsert(entry.clone());
                        self.campaign_saves.save(self.game.config_dir());
                        if done {
                            self.campaign_ctx = None;
                            self.state = TuiState::MainMenu { selected: 2, flash: Some("Campaign complete!".to_string()) };
                        } else {
                            self.campaign_ctx = Some(entry);
                            self.state = TuiState::CampaignLevelIntro;
                        }
                    } else {
                        let cfg = self.game.campaign_config(&entry, &self.cli_config);
                        self.engine = Some(self.game.create_campaign_engine(&entry, &cfg, &[]));
                        self.game_config = cfg;
                        self.campaign_ctx = Some(entry);
                        self.state = TuiState::Playing;
                    }
                } else {
                    self.engine = Some(self.game.create_engine(&self.game_config, &[]));
                    self.state = TuiState::Playing;
                }
            }
            Key::Char('m') | Key::Char('M') | Key::Esc => {
                self.quit_to_menu();
            }
            Key::Char('q') | Key::Char('Q') => {
                if let Some(ctx) = self.campaign_ctx.take() {
                    self.campaign_saves.upsert(ctx);
                    self.campaign_saves.save(self.game.config_dir());
                }
                self.want_quit = true;
            }
            _ => {
                // Bonus-recovery keys (scissors/tweezers/balloons-equivalent)
                // are forwarded to the engine, matching how tui.rs lets 'Z'/
                // 'X'/'C' work from the Stuck game-over screen, not just
                // while Playing.
                if let Some(engine) = self.engine.as_mut() {
                    if matches!(status, GameStatus::Stuck) {
                        engine.handle_key(key);
                        self.after_status_change();
                    }
                }
            }
        }
    }
}

fn pick_quote(quotes: &[String]) -> String {
    if quotes.is_empty() {
        return String::new();
    }
    // Simple rotation rather than pulling in a rand dependency here -- ad
    // quote variety is cosmetic, not gameplay-affecting.
    let idx = (Instant::now().elapsed().subsec_nanos() as usize) % quotes.len();
    quotes[idx].clone()
}
