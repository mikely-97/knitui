use serde::{Deserialize, Serialize};

use loom_engine::campaign::CampaignEntry;
pub use loom_engine::campaign::CampaignSaves;

use crate::board::Board;
use crate::campaign_levels::{mission_count, track_def};
use crate::energy::Energy;
use crate::engine::GameEngine;
use crate::inventory::Inventory;
use crate::item::Family;
use crate::order::Order;

// ── Persistent campaign state ─────────────────────────────────────────────

/// Full persistent state for one campaign track.
/// The board, inventory, energy and orders all live here between sessions.
#[derive(Serialize, Deserialize, Clone)]
pub struct CampaignState {
    pub track_idx: usize,
    pub current_mission: usize,
    pub completed: bool,
    pub blessings: Vec<String>,

    // Persistent game world
    pub board: Board,
    pub inventory: Inventory,
    pub energy: Energy,
    pub active_orders: Vec<Order>,

    // Statistics
    pub total_merges: u64,
    pub cells_thawed: usize,
    pub stars: u16,
    pub score: u32,

    // Progression flags
    pub story_orders_completed: usize,
}

impl CampaignEntry for CampaignState {
    fn track_idx(&self) -> usize {
        self.track_idx
    }
    fn current_level(&self) -> usize {
        self.current_mission
    }
    fn total_levels(&self) -> usize {
        mission_count(self.track_idx)
    }
    fn is_completed(&self) -> bool {
        self.completed
    }
}

impl CampaignState {
    /// Create a brand-new campaign state for a track, building the initial board.
    pub fn new(track_idx: usize) -> Self {
        let def = track_def(track_idx);
        let board = def.initial_layout.build();
        let energy = Energy::new(def.energy_max, def.energy_regen_secs);
        let inventory = Inventory::new(def.inventory_slots as usize);

        Self {
            track_idx,
            current_mission: 0,
            completed: false,
            blessings: Vec::new(),
            board,
            inventory,
            energy,
            active_orders: Vec::new(),
            total_merges: 0,
            cells_thawed: 0,
            stars: 0,
            score: 0,
            story_orders_completed: 0,
        }
    }

    pub fn total_missions(&self) -> usize {
        mission_count(self.track_idx)
    }

    /// Whether the current mission's story orders are all fulfilled.
    pub fn current_mission_complete(&self) -> bool {
        let story_count = track_def(self.track_idx).missions[self.current_mission]
            .story_orders
            .len();
        self.story_orders_completed
            >= self
                .story_orders_completed
                .max(story_count)
                .min(story_count)
    }

    /// Advance to the next mission. Returns true if campaign is now complete.
    pub fn advance_mission(&mut self) -> bool {
        self.story_orders_completed = 0;
        self.current_mission += 1;
        if self.current_mission >= self.total_missions() {
            self.completed = true;
        }
        self.completed
    }

    /// Build a GameEngine from this campaign state.
    pub fn build_engine(&self) -> GameEngine {
        let def = track_def(self.track_idx);
        let available_families = self.derive_available_families();

        GameEngine::from_state(
            self.board.clone(),
            self.inventory.clone(),
            self.energy.clone(),
            self.active_orders.clone(),
            self.score,
            self.stars,
            self.total_merges,
            self.cells_thawed,
            1, // scale — loaded from settings separately
            def.ad_limit,
            def.random_order_count,
            def.max_order_tier,
            def.generator_cost,
            def.generator_cooldown,
            def.soft_gen_chance,
            available_families,
            &self.blessings,
        )
    }

    /// Sync engine state back into this campaign state for persistence.
    pub fn sync_from_engine(&mut self, engine: &GameEngine) {
        self.board = engine.board.clone();
        self.inventory = engine.inventory.clone();
        self.energy = engine.energy.clone();
        self.active_orders = engine.active_orders.clone();
        self.total_merges = engine.total_merges;
        self.cells_thawed = engine.cells_thawed;
        self.stars = engine.stars;
        self.score = engine.score;
    }

    /// Derive which families are currently available for random order generation.
    ///
    /// A family is only available if the player has an **active** (unfrozen) generator
    /// for it. Frozen pieces and frozen generators do not count — the player can't
    /// produce more of those items yet.
    fn derive_available_families(&self) -> Vec<Family> {
        use crate::board::Cell;
        let mut families: std::collections::HashSet<Family> = std::collections::HashSet::new();
        for row in &self.board.cells {
            for cell in row {
                match cell {
                    Cell::HardGenerator { family, .. } | Cell::SoftGenerator { family, .. } => {
                        families.insert(*family);
                    }
                    // Frozen pieces/generators do NOT make a family available.
                    _ => {}
                }
            }
        }
        let mut result: Vec<Family> = families.into_iter().collect();
        result.sort_by_key(|f| f.index());
        result
    }

    /// Get the current mission's story orders as active orders (for a new mission).
    pub fn load_mission_orders(&mut self) {
        let def = track_def(self.track_idx);
        if self.current_mission < def.missions.len() {
            let mission = &def.missions[self.current_mission];
            // Apply mission bonuses
            self.energy.max += mission.energy_max_bonus;
            self.energy.current = self.energy.current.min(self.energy.max);
            self.inventory.expand(mission.inventory_slot_bonus as usize);

            // Add story orders
            let story_orders: Vec<Order> = mission
                .story_orders
                .iter()
                .map(|def| def.build())
                .collect();
            self.active_orders.retain(|o| !matches!(o.order_type, crate::order::OrderType::Story));
            self.active_orders.extend(story_orders);
            self.story_orders_completed = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_mission_zero() {
        let s = CampaignState::new(0);
        assert_eq!(s.current_mission, 0);
        assert!(!s.completed);
        assert!(s.total_merges == 0);
    }

    #[test]
    fn new_builds_board() {
        let s = CampaignState::new(0);
        // Grove track: 9×7 board
        assert_eq!(s.board.rows, 9);
        assert_eq!(s.board.cols, 7);
    }

    #[test]
    fn advance_mission_progresses() {
        let mut s = CampaignState::new(0);
        assert!(!s.advance_mission());
        assert_eq!(s.current_mission, 1);
    }

    #[test]
    fn advance_mission_completes_on_last() {
        let mut s = CampaignState::new(0);
        let total = s.total_missions();
        for _ in 0..(total - 1) {
            assert!(!s.advance_mission());
        }
        assert!(s.advance_mission());
        assert!(s.completed);
    }

    #[test]
    fn build_engine_uses_board() {
        let s = CampaignState::new(0);
        let engine = s.build_engine();
        assert_eq!(engine.board.rows, s.board.rows);
        assert_eq!(engine.board.cols, s.board.cols);
    }

    #[test]
    fn sync_back_updates_score() {
        let mut s = CampaignState::new(0);
        let mut engine = s.build_engine();
        engine.score = 9999;
        s.sync_from_engine(&engine);
        assert_eq!(s.score, 9999);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        let mut s = CampaignState::new(1);
        s.advance_mission();
        s.score = 1234;
        saves.upsert(s);
        let json = serde_json::to_string(&saves).unwrap();
        let loaded: CampaignSaves<CampaignState> = serde_json::from_str(&json).unwrap();
        let s = loaded.get(1).unwrap();
        assert_eq!(s.current_mission, 1);
        assert_eq!(s.score, 1234);
    }
}
