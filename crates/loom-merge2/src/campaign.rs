use serde::{Deserialize, Serialize};

use loom_engine::campaign::CampaignEntry;
pub use loom_engine::campaign::CampaignSaves;

use crate::blessings;
use crate::campaign_levels::levels_for_track;
use crate::config::Config;

#[derive(Serialize, Deserialize, Clone)]
pub struct CampaignState {
    pub track_idx: usize,
    pub current_level: usize,
    pub completed: bool,
    #[serde(default)]
    pub blessings: Vec<String>,
}

impl CampaignEntry for CampaignState {
    fn track_idx(&self) -> usize { self.track_idx }
    fn current_level(&self) -> usize { self.current_level }
    fn total_levels(&self) -> usize { levels_for_track(self.track_idx).len() }
    fn is_completed(&self) -> bool { self.completed }
}

impl CampaignState {
    pub fn new(track_idx: usize) -> Self {
        Self { track_idx, current_level: 0, completed: false, blessings: Vec::new() }
    }

    /// Build a game Config for the current level.
    pub fn to_config(&self, base: &Config) -> Config {
        let levels = levels_for_track(self.track_idx);
        let level = &levels[self.current_level];
        let mut cfg = base.clone();
        cfg.board_height = level.board_height;
        cfg.board_width = level.board_width;
        cfg.color_count = level.color_count;
        cfg.generator_count = level.generator_count;
        cfg.generator_charges = level.generator_charges;
        cfg.blocked_cells = level.blocked_cells;
        cfg.generator_interval = level.generator_interval;
        cfg.order_count = level.orders.len() as u16;
        cfg.ad_limit = level.ad_limit;

        // Apply blessing config effects
        if blessings::has(&self.blessings, "extra_ad") {
            cfg.ad_limit += 1;
        }
        if blessings::has(&self.blessings, "fast_spawn") {
            cfg.generator_interval = cfg.generator_interval.saturating_sub(2).max(1);
        }
        if blessings::has(&self.blessings, "extra_charges") {
            cfg.generator_charges += 3;
        }
        if blessings::has(&self.blessings, "clear_path") {
            cfg.blocked_cells = cfg.blocked_cells.saturating_sub(1);
        }
        cfg
    }

    /// Advance to the next level. Returns true if campaign is now complete.
    pub fn complete_level(&mut self) -> bool {
        let levels = levels_for_track(self.track_idx);
        self.current_level += 1;
        if self.current_level >= levels.len() {
            self.completed = true;
        }
        self.completed
    }

    pub fn total_levels(&self) -> usize {
        levels_for_track(self.track_idx).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn default_config() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn new_starts_at_level_zero() {
        let s = CampaignState::new(0);
        assert_eq!(s.current_level, 0);
        assert!(!s.completed);
    }

    #[test]
    fn to_config_applies_level() {
        let s = CampaignState::new(0);
        let cfg = s.to_config(&default_config());
        assert_eq!(cfg.board_height, 3);
        assert_eq!(cfg.board_width, 3);
    }

    #[test]
    fn complete_level_advances() {
        let mut s = CampaignState::new(0);
        assert!(!s.complete_level());
        assert_eq!(s.current_level, 1);
    }

    #[test]
    fn complete_level_marks_done_on_last() {
        let mut s = CampaignState::new(0);
        let total = s.total_levels();
        for _ in 0..total - 1 {
            assert!(!s.complete_level());
        }
        assert!(s.complete_level());
        assert!(s.completed);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        let mut s = CampaignState::new(1);
        s.current_level = 3;
        saves.upsert(s);
        let json = serde_json::to_string(&saves).unwrap();
        let loaded: CampaignSaves<CampaignState> = serde_json::from_str(&json).unwrap();
        let s = loaded.get(1).unwrap();
        assert_eq!(s.current_level, 3);
    }
}
