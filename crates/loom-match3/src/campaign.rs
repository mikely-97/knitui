use serde::{Deserialize, Serialize};

use loom_engine::campaign::CampaignEntry;
pub use loom_engine::campaign::CampaignSaves;

use crate::campaign_levels::{levels_for_track, LevelDef, LevelObjective};
use crate::config::Config;

// ── Objective helper ──────────────────────────────────────────────────────

/// Return true if all objective conditions are satisfied.
pub fn objective_met(
    obj: &LevelObjective,
    score: u32,
    gem_counts: &[u32],
    special_tiles_remaining: usize,
) -> bool {
    if let Some(target) = obj.score_target {
        if score < target { return false; }
    }
    for &(color_idx, needed) in &obj.gem_quota {
        let collected = gem_counts.get(color_idx as usize).copied().unwrap_or(0);
        if collected < needed { return false; }
    }
    if obj.clear_all_specials && special_tiles_remaining > 0 {
        return false;
    }
    true
}

// ── CampaignState ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct CampaignState {
    pub track_idx: usize,
    pub current_level: usize,
    pub banked_hammer: u16,
    pub banked_laser: u16,
    pub banked_blaster: u16,
    pub banked_warp: u16,
    pub completed: bool,
}

impl CampaignEntry for CampaignState {
    fn track_idx(&self) -> usize { self.track_idx }
    fn current_level(&self) -> usize { self.current_level }
    fn total_levels(&self) -> usize { levels_for_track(self.track_idx).len() }
    fn is_completed(&self) -> bool { self.completed }
}

impl CampaignState {
    pub fn new(track_idx: usize) -> Self {
        Self {
            track_idx,
            current_level: 0,
            banked_hammer: 0,
            banked_laser: 0,
            banked_blaster: 0,
            banked_warp: 0,
            completed: false,
        }
    }

    pub fn to_config(&self, base: &Config) -> Config {
        let levels = levels_for_track(self.track_idx);
        let idx = self.current_level.min(levels.len().saturating_sub(1));
        let level = &levels[idx];
        let mut cfg = base.clone();
        cfg.board_height     = level.board_height;
        cfg.board_width      = level.board_width;
        cfg.color_number     = level.color_number;
        cfg.move_limit       = level.move_limit;
        cfg.special_tile_pct = level.special_tile_pct;
        cfg.hammer  = self.banked_hammer;
        cfg.laser   = self.banked_laser;
        cfg.blaster = self.banked_blaster;
        cfg.warp    = self.banked_warp;
        cfg
    }

    pub fn complete_level(&mut self) -> bool {
        if self.completed {
            return true;
        }
        let levels = levels_for_track(self.track_idx);
        let level = &levels[self.current_level];
        self.banked_hammer  += level.reward_hammer;
        self.banked_laser   += level.reward_laser;
        self.banked_blaster += level.reward_blaster;
        self.banked_warp    += level.reward_warp;
        self.current_level += 1;
        if self.current_level >= levels.len() {
            self.completed = true;
        }
        self.completed
    }

    pub fn total_levels(&self) -> usize {
        levels_for_track(self.track_idx).len()
    }

    pub fn current_level_def(&self) -> LevelDef {
        let levels = levels_for_track(self.track_idx);
        let idx = self.current_level.min(levels.len().saturating_sub(1));
        levels[idx].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use clap::Parser;

    fn base() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn new_state_starts_at_level_zero() {
        let s = CampaignState::new(0);
        assert_eq!(s.current_level, 0);
        assert!(!s.completed);
    }

    #[test]
    fn to_config_applies_level_params() {
        let s = CampaignState::new(0);
        let cfg = s.to_config(&base());
        let levels = crate::campaign_levels::levels_for_track(0);
        assert_eq!(cfg.board_height, levels[0].board_height);
        assert_eq!(cfg.board_width,  levels[0].board_width);
        assert_eq!(cfg.color_number, levels[0].color_number);
        assert_eq!(cfg.move_limit,   levels[0].move_limit);
    }

    #[test]
    fn to_config_includes_banked_bonuses() {
        let mut s = CampaignState::new(0);
        s.banked_hammer = 2;
        s.banked_warp   = 1;
        let cfg = s.to_config(&base());
        assert_eq!(cfg.hammer, 2);
        assert_eq!(cfg.warp,   1);
    }

    #[test]
    fn complete_level_advances_and_banks_reward() {
        let mut s = CampaignState::new(0);
        let levels = crate::campaign_levels::levels_for_track(0);
        let reward_h = levels[0].reward_hammer;
        let reward_l = levels[0].reward_laser;
        let done = s.complete_level();
        assert!(!done);
        assert_eq!(s.current_level, 1);
        assert_eq!(s.banked_hammer, reward_h);
        assert_eq!(s.banked_laser,  reward_l);
    }

    #[test]
    fn complete_level_marks_campaign_done_on_last() {
        let mut s = CampaignState::new(0);
        for _ in 0..14 {
            assert!(!s.complete_level());
        }
        assert!(s.complete_level());
        assert!(s.completed);
    }

    #[test]
    fn complete_level_is_idempotent_when_done() {
        let mut s = CampaignState::new(0);
        for _ in 0..15 {
            s.complete_level();
        }
        assert!(s.completed);
        assert!(s.complete_level());
    }

    #[test]
    fn total_levels_matches_track_definition() {
        let s = CampaignState::new(0);
        assert_eq!(s.total_levels(), 15);
        let s = CampaignState::new(1);
        assert_eq!(s.total_levels(), 30);
    }

    #[test]
    fn objective_met_score_target() {
        let obj = crate::campaign_levels::LevelObjective {
            score_target: Some(1000),
            gem_quota: vec![],
            clear_all_specials: false,
        };
        assert!(objective_met(&obj, 1000, &[], 0));
        assert!(!objective_met(&obj, 999, &[], 0));
    }

    #[test]
    fn objective_met_no_score_target() {
        let obj = crate::campaign_levels::LevelObjective {
            score_target: None,
            gem_quota: vec![],
            clear_all_specials: false,
        };
        assert!(objective_met(&obj, 0, &[], 0));
    }

    #[test]
    fn campaign_saves_upsert_and_get() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        let state = CampaignState::new(1);
        saves.upsert(state);
        assert!(saves.get(1).is_some());
        assert!(saves.get(0).is_none());
    }

    #[test]
    fn campaign_saves_reset() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        saves.upsert(CampaignState::new(0));
        saves.upsert(CampaignState::new(1));
        saves.reset(0);
        assert!(saves.get(0).is_none());
        assert!(saves.get(1).is_some());
    }

    #[test]
    fn progress_label_formats_correctly() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        let mut s = CampaignState::new(0);
        s.current_level = 4;
        saves.upsert(s);
        assert_eq!(saves.progress_label(0), "Level 5/15");
    }

    #[test]
    fn progress_label_shows_complete() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        let mut s = CampaignState::new(0);
        s.completed = true;
        saves.upsert(s);
        assert_eq!(saves.progress_label(0), "Complete");
    }

    #[test]
    fn saves_serialization_roundtrip() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        saves.upsert(CampaignState::new(0));
        let json = serde_json::to_string(&saves).unwrap();
        let loaded: CampaignSaves<CampaignState> = serde_json::from_str(&json).unwrap();
        assert!(loaded.get(0).is_some());
    }
}
