use loom_engine::blessings::Blessing;
use loom_engine::render::Color;
use loom_engine::game::{Game, GameId, GameEngine as GameEngineTrait, MenuItem};

use crate::campaign::CampaignState;
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, mission_count, track_def};
use crate::config::Config;
use crate::endless::EndlessParams;
use crate::preset::PRESETS;

pub struct M2Game;

impl Game for M2Game {
    type Config = Config;
    type CampaignEntry = CampaignState;

    fn id(&self) -> GameId { GameId::Merge2 }
    fn name(&self) -> &'static str { "Merge-2" }
    fn config_dir(&self) -> &'static str { "m2tui" }

    fn create_engine(&self, config: &Config, _palette: &[Color]) -> Box<dyn GameEngineTrait> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let engine = if config.is_endless {
                crate::endless::new_endless_engine(&[])
            } else {
                crate::engine::GameEngine::new_endless(config, &[])
            };
            let label = if config.is_endless { "Endless".to_string() } else { "Custom Game".to_string() };
            Box::new(m2_adapter::M2EngineAdapter::new(engine, None, label))
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = config;
            unimplemented!("merge2's Shell integration is native-only; web.rs drives the engine directly")
        }
    }

    fn main_menu_items(&self) -> &'static [MenuItem] {
        // No standalone "Quick Game" -- the original tui.rs's menu is
        // Custom/Campaign/Endless/Options/Quit.
        &[MenuItem::CustomGame, MenuItem::Campaign, MenuItem::Endless, MenuItem::Options, MenuItem::Quit]
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
            is_endless:          false,
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

    // Merge-2's CampaignState embeds live game-world state (board, inventory,
    // energy, active orders), not just level params. `campaign_config`'s
    // return value is only used for display (chrome's level-intro board
    // dims) -- the real engine is always built from `entry.build_engine()`
    // in `create_campaign_engine`, not reconstructed from this Config.
    fn new_campaign_entry(&self, track: usize) -> CampaignState {
        CampaignState::new(track)
    }

    fn campaign_config(&self, entry: &CampaignState, base: &Config) -> Config {
        Config {
            board_rows: entry.board.rows as u16,
            board_cols: entry.board.cols as u16,
            scale: base.scale,
            color_mode: base.color_mode.clone(),
            ..base.clone()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_campaign_engine(
        &self, entry: &CampaignState, _config: &Config, _palette: &[Color],
    ) -> Box<dyn GameEngineTrait> {
        let engine = entry.build_engine();
        let story_count = track_def(entry.track_idx).missions[entry.current_mission].story_orders.len();
        let label = format!(
            "{} M{}/{}",
            TRACK_NAMES.get(entry.track_idx).copied().unwrap_or(""),
            entry.current_mission + 1, entry.total_missions(),
        );
        Box::new(m2_adapter::M2EngineAdapter::new(engine, Some(story_count), label))
    }
    #[cfg(target_arch = "wasm32")]
    fn create_campaign_engine(
        &self, entry: &CampaignState, config: &Config, palette: &[Color],
    ) -> Box<dyn GameEngineTrait> {
        let _ = (entry, config, palette);
        unimplemented!("merge2's Shell integration is native-only; web.rs drives the engine directly")
    }

    /// Advance to the next mission and (unless the track just completed)
    /// queue up its story orders -- matches the original's exact coupling:
    /// `tui.rs`'s MissionSummary handler calls `ctx.load_mission_orders()`
    /// immediately after `ctx.advance_mission()`, not lazily later.
    fn complete_campaign_level(&self, entry: &mut CampaignState) -> bool {
        let done = entry.advance_mission();
        if !done {
            entry.load_mission_orders();
        }
        done
    }

    /// Also matches a real original coupling: confirming blessing selection
    /// is the point `tui.rs` calls `ctx.load_mission_orders()` for a
    /// track's *first* mission (there's no earlier hook that does it). Note
    /// this reloads story orders -- and therefore resets in-progress
    /// delivery counters -- every time a track is re-entered through
    /// blessing selection, even mid-mission. That's the original's real
    /// behavior (confirmed via git history), not something introduced here.
    fn confirm_blessings(&self, entry: &mut CampaignState, ids: &[String]) {
        entry.blessings = ids.to_vec();
        entry.load_mission_orders();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn sync_campaign_entry(&self, engine: &dyn GameEngineTrait, entry: &mut CampaignState) {
        if let Some(adapter) = engine.as_any().downcast_ref::<m2_adapter::M2EngineAdapter>() {
            entry.sync_from_engine(&adapter.engine);
        }
    }

    fn all_blessings(&self) -> &'static [Blessing] {
        crate::blessings::ALL_BLESSINGS
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
            is_endless:         true,
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

/// The `GameEngine` trait adapter. Native-only: merge2's Shell integration
/// (menu/campaign/inventory/help screens) is never used from `web.rs`,
/// which drives `crate::engine::GameEngine` directly (see its own doc
/// comment) -- so this whole module, and the render calls it makes into
/// the native-gated parts of `renderer::popups`, only need to exist on
/// native targets.
///
/// **Disclosed scope cuts from the original `tui.rs`** (890 lines,
/// substantially larger than knit/match3's): no in-play +/-/n/p scale or
/// color-mode hotkeys (use the Options screen, like the other 3 games); no
/// vim-style hjkl movement or the digit-prefix move-count repeat; no
/// MissionSummary screen between campaign missions (same call as match3's
/// dropped LevelSummary -- Shell has no generic slot for it); the
/// inventory-expansion bonus is auto-granted instead of prompting Y/N.
#[cfg(not(target_arch = "wasm32"))]
mod m2_adapter {
    use loom_engine::render::Surface;
    use loom_engine::game::{Action, GameEngine as GameEngineTrait, GameStatus as EngineGameStatus, RenderArea};
    use loom_engine::input::{Key, KeyEvent};

    use crate::board::Cell;
    use crate::engine::GameEngine as M2Engine;
    use crate::item::Piece;
    use crate::renderer::{self, LayoutGeometry};

    /// Sub-state entered from Playing via 'I' (browse/pick up inventory
    /// items) and from there via Enter (holding a picked-up piece, moving
    /// it to a board cell). Kept adapter-internal rather than as new
    /// `Shell`/`Action` variants -- Shell's generic state machine has no
    /// concept of either, and inventing shared ones for a single game
    /// would be a much larger, unjustified change than keeping this local.
    enum SubState {
        Playing,
        Inventory { selected_slot: usize },
        Placing { piece: Piece },
    }

    pub struct M2EngineAdapter {
        pub engine: M2Engine,
        /// `Some(n)` in campaign mode: mission is won once
        /// `engine.story_orders_completed >= n`. `None` for Custom Game and
        /// Endless, which never "win" (matches the original: `check_status`
        /// only checks `ctx.current_mission_complete()` when a campaign_ctx
        /// is active).
        mission_story_count: Option<usize>,
        label: String,
        sub_state: SubState,
    }

    impl M2EngineAdapter {
        pub fn new(engine: M2Engine, mission_story_count: Option<usize>, label: String) -> Self {
            Self { engine, mission_story_count, label, sub_state: SubState::Playing }
        }

        fn selected_slot(&self) -> Option<usize> {
            match self.sub_state {
                SubState::Inventory { selected_slot } => Some(selected_slot),
                _ => None,
            }
        }

        fn geo(&self, area: RenderArea) -> LayoutGeometry {
            LayoutGeometry::for_width(&self.engine, area.width)
        }

        fn handle_playing_key(&mut self, key: KeyEvent) -> Action {
            match key.key {
                Key::Up => { self.engine.move_cursor(-1, 0); Action::Redraw }
                Key::Down => { self.engine.move_cursor(1, 0); Action::Redraw }
                Key::Left => { self.engine.move_cursor(0, -1); Action::Redraw }
                Key::Right => { self.engine.move_cursor(0, 1); Action::Redraw }
                Key::Enter | Key::Char(' ') => { self.engine.activate(); Action::Redraw }
                Key::Char('d') | Key::Char('D') => { self.engine.deliver_from_board(); Action::Redraw }
                Key::Char('s') | Key::Char('S') => { self.engine.store_selected_to_inventory(); Action::Redraw }
                Key::Char('i') | Key::Char('I') => {
                    self.sub_state = SubState::Inventory { selected_slot: 0 };
                    Action::Redraw
                }
                Key::Char('e') | Key::Char('E') => { self.engine.activate_enhanced(); Action::Redraw }
                Key::Char('u') | Key::Char('U') => { self.engine.upgrade_generator_at_cursor(); Action::Redraw }
                Key::Char('h') | Key::Char('H') => Action::ShowHelp,
                Key::Char('q') | Key::Char('Q') | Key::Esc => Action::QuitToMenu,
                _ => Action::None,
            }
        }

        fn handle_inventory_key(&mut self, key: KeyEvent) -> Action {
            let SubState::Inventory { selected_slot } = &mut self.sub_state else { unreachable!() };
            match key.key {
                Key::Left => {
                    if *selected_slot > 0 { *selected_slot -= 1; }
                    Action::Redraw
                }
                Key::Right => {
                    let max = self.engine.inventory.slot_count().saturating_sub(1);
                    *selected_slot = (*selected_slot + 1).min(max);
                    Action::Redraw
                }
                Key::Enter => {
                    let slot = *selected_slot;
                    if let Some(piece) = self.engine.inventory.take(slot) {
                        self.sub_state = SubState::Placing { piece };
                    }
                    Action::Redraw
                }
                Key::Char('d') | Key::Char('D') => {
                    let slot = *selected_slot;
                    self.engine.deliver_from_inventory(slot);
                    Action::Redraw
                }
                Key::Esc => {
                    self.sub_state = SubState::Playing;
                    Action::Redraw
                }
                _ => Action::None,
            }
        }

        fn handle_placing_key(&mut self, key: KeyEvent) -> Action {
            let SubState::Placing { piece } = &self.sub_state else { unreachable!() };
            let piece = piece.clone();
            match key.key {
                Key::Up => { self.engine.move_cursor(-1, 0); Action::Redraw }
                Key::Down => { self.engine.move_cursor(1, 0); Action::Redraw }
                Key::Left => { self.engine.move_cursor(0, -1); Action::Redraw }
                Key::Right => { self.engine.move_cursor(0, 1); Action::Redraw }
                Key::Enter => {
                    let (r, c) = (self.engine.cursor_row, self.engine.cursor_col);
                    if self.engine.board.cells[r][c].is_empty() {
                        self.engine.board.cells[r][c] = Cell::Piece(piece);
                        self.sub_state = SubState::Playing;
                    } else if let Some(idx) = find_piece_in_inv(&self.engine, &piece) {
                        self.engine.merge_from_inventory(idx);
                        self.sub_state = SubState::Playing;
                    }
                    Action::Redraw
                }
                Key::Esc => {
                    self.engine.inventory.store(piece);
                    self.sub_state = SubState::Playing;
                    Action::Redraw
                }
                _ => Action::None,
            }
        }
    }

    fn find_piece_in_inv(engine: &M2Engine, piece: &Piece) -> Option<usize> {
        engine.inventory.slots.iter().position(|s| s.as_ref() == Some(piece))
    }

    impl GameEngineTrait for M2EngineAdapter {
        fn handle_key(&mut self, key: KeyEvent) -> Action {
            match self.sub_state {
                SubState::Playing => self.handle_playing_key(key),
                SubState::Inventory { .. } => self.handle_inventory_key(key),
                SubState::Placing { .. } => self.handle_placing_key(key),
            }
        }

        fn tick(&mut self) -> bool {
            let changed = self.engine.tick();
            // Auto-grant the inventory-expansion bonus rather than blocking
            // on a Y/N popup (see module doc comment's disclosed cuts).
            if self.engine.inv_expansion_pending {
                self.engine.accept_inv_expansion();
            }
            changed
        }

        fn status(&self) -> EngineGameStatus {
            if let Some(story_count) = self.mission_story_count {
                if self.engine.story_orders_completed >= story_count {
                    return EngineGameStatus::Won { score: Some(self.engine.score) };
                }
            }
            if self.engine.is_stuck() {
                EngineGameStatus::Stuck
            } else {
                EngineGameStatus::Playing
            }
        }

        fn render(&self, surface: &mut dyn Surface, area: RenderArea) {
            let geo = self.geo(area);
            renderer::render_hud_inner(surface, &self.engine, &self.label);
            renderer::render_board_inner(surface, &self.engine, &geo);
            renderer::render_orders_inner(surface, &self.engine, &geo);
            renderer::render_inventory_inner(surface, &self.engine, &geo, self.selected_slot());
            renderer::render_key_bar_inner(surface, &self.engine, &geo);
        }

        fn render_keybar(&self, _surface: &mut dyn Surface, _y: u16) {
            // No-op: render() above already draws the key bar as part of
            // the same pass, matching current terminal/wasm behavior.
        }

        fn render_game_over_overlay(&self, surface: &mut dyn Surface, status: &EngineGameStatus, overlay_msg: Option<&str>) {
            let m2_status = match status {
                EngineGameStatus::Won { .. } => crate::engine::GameStatus::Won,
                EngineGameStatus::Stuck => crate::engine::GameStatus::Stuck,
                EngineGameStatus::Lost { .. } => crate::engine::GameStatus::Lost,
                EngineGameStatus::Playing => crate::engine::GameStatus::Playing,
            };
            renderer::render_game_over_to_surface(surface, &m2_status, self.engine.score, overlay_msg);
        }

        fn render_help(&self, surface: &mut dyn Surface) {
            let help_lines = [
                ("Arrows", "Move cursor"),
                ("Enter", "Select / Merge / Activate gen"),
                ("D", "Deliver selected to order"),
                ("S", "Store selected in inventory"),
                ("I", "Open inventory"),
                ("E", "Enhanced generator activation"),
                ("U", "Upgrade generator (costs 3\u{2605})"),
                ("A", "Watch ad (if available)"),
                ("Q / Esc", "Back / Quit"),
            ];
            renderer::render_help_to_surface(surface, &help_lines, Some(&self.engine));
        }

        fn render_celebration(&self, surface: &mut dyn Surface, area: RenderArea, tick: u8) {
            let geo = self.geo(area);
            renderer::render_celebration_to_surface(surface, &self.engine, &geo, tick);
        }

        fn score(&self) -> Option<u32> { Some(self.engine.score) }

        fn can_watch_ad(&self) -> bool { self.engine.can_watch_ad() }
        fn watch_ad(&mut self) {
            let reward = crate::ad::reward_for_use(self.engine.ads_used, &self.engine.available_families);
            self.engine.watch_ad_reward(reward);
        }

        fn scale(&self) -> u16 { self.engine.scale }
        fn set_scale(&mut self, scale: u16) {
            self.engine.scale = scale;
        }

        fn board_dims(&self) -> (u16, u16) { (self.engine.board.rows as u16, self.engine.board.cols as u16) }

        fn as_any(&self) -> &dyn std::any::Any { self }
    }
}

