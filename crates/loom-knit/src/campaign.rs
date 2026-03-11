use serde::{Deserialize, Serialize};

use loom_engine::campaign::CampaignEntry;
pub use loom_engine::campaign::CampaignSaves;

use crate::campaign_levels::levels_for_track;
use crate::config::Config;

#[derive(Serialize, Deserialize, Clone)]
pub struct CampaignState {
    pub track_idx: usize,
    pub current_level: usize,
    pub banked_scissors: u16,
    pub banked_tweezers: u16,
    pub banked_balloons: u16,
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
            banked_scissors: 0,
            banked_tweezers: 0,
            banked_balloons: 0,
            completed: false,
        }
    }

    /// Build a game Config for the current level, merging base display settings,
    /// level params, and banked bonuses.
    pub fn to_config(&self, base: &Config) -> Config {
        let levels = levels_for_track(self.track_idx);
        let level = &levels[self.current_level];
        let mut cfg = base.clone();
        cfg.board_height = level.board_height;
        cfg.board_width = level.board_width;
        cfg.color_number = level.color_number;
        cfg.obstacle_percentage = level.obstacle_percentage;
        cfg.conveyor_percentage = level.conveyor_percentage;
        cfg.scissors = level.scissors + self.banked_scissors;
        cfg.tweezers = level.tweezers + self.banked_tweezers;
        cfg.balloons = level.balloons + self.banked_balloons;
        cfg
    }

    /// Get the ad_limit for the current level.
    pub fn ad_limit(&self) -> u16 {
        let levels = levels_for_track(self.track_idx);
        levels[self.current_level].ad_limit
    }

    /// Award level completion rewards. Returns true if campaign is now complete.
    pub fn complete_level(&mut self) -> bool {
        let levels = levels_for_track(self.track_idx);
        let level = &levels[self.current_level];
        self.banked_scissors += level.reward_scissors;
        self.banked_tweezers += level.reward_tweezers;
        self.banked_balloons += level.reward_balloons;
        self.current_level += 1;
        if self.current_level >= levels.len() {
            self.completed = true;
        }
        self.completed
    }

    /// Total levels in this campaign track.
    pub fn total_levels(&self) -> usize {
        levels_for_track(self.track_idx).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use clap::Parser;

    fn default_config() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn new_campaign_state_starts_at_level_zero() {
        let s = CampaignState::new(0);
        assert_eq!(s.current_level, 0);
        assert_eq!(s.banked_scissors, 0);
        assert!(!s.completed);
    }

    #[test]
    fn to_config_applies_level_and_banked_bonuses() {
        let mut s = CampaignState::new(0);
        s.banked_scissors = 2;
        s.banked_tweezers = 1;
        let cfg = s.to_config(&default_config());
        assert_eq!(cfg.board_height, 3);
        assert_eq!(cfg.board_width, 3);
        assert_eq!(cfg.color_number, 3);
        assert_eq!(cfg.scissors, 2);
        assert_eq!(cfg.tweezers, 1);
    }

    #[test]
    fn complete_level_advances_and_banks_rewards() {
        let mut s = CampaignState::new(0);
        let done = s.complete_level();
        assert!(!done);
        assert_eq!(s.current_level, 1);
        assert_eq!(s.banked_scissors, 1);
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
    fn progress_label_shows_level() {
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
    fn progress_label_empty_for_no_save() {
        let saves = CampaignSaves::<CampaignState>::default();
        assert_eq!(saves.progress_label(0), "");
    }

    #[test]
    fn serialization_roundtrip() {
        let mut saves = CampaignSaves::<CampaignState>::default();
        let mut s = CampaignState::new(1);
        s.current_level = 3;
        s.banked_scissors = 5;
        saves.upsert(s);
        let json = serde_json::to_string(&saves).unwrap();
        let loaded: CampaignSaves<CampaignState> = serde_json::from_str(&json).unwrap();
        let s = loaded.get(1).unwrap();
        assert_eq!(s.current_level, 3);
        assert_eq!(s.banked_scissors, 5);
    }
}
