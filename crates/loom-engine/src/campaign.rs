use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fs;
use std::path::PathBuf;

const CAMPAIGN_FILE: &str = "campaign.json";

fn campaign_path(config_dir: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(config_dir).join(CAMPAIGN_FILE))
}

/// Trait that each game's CampaignState must implement so the shared
/// CampaignSaves container can work generically.
pub trait CampaignEntry: Serialize + DeserializeOwned + Clone {
    fn track_idx(&self) -> usize;
    fn current_level(&self) -> usize;
    fn total_levels(&self) -> usize;
    fn is_completed(&self) -> bool;
}

/// Persistent storage for all campaign saves (one per track).
#[derive(Serialize, Deserialize)]
#[serde(bound = "E: Serialize + DeserializeOwned")]
pub struct CampaignSaves<E: CampaignEntry> {
    pub saves: Vec<E>,
}

impl<E: CampaignEntry> Default for CampaignSaves<E> {
    fn default() -> Self {
        Self { saves: Vec::new() }
    }
}

impl<E: CampaignEntry> CampaignSaves<E> {
    pub fn load(config_dir: &str) -> Self {
        let Some(path) = campaign_path(config_dir) else {
            return Self::default();
        };
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, config_dir: &str) {
        let Some(path) = campaign_path(config_dir) else { return };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    /// Get saved state for a track, if any.
    pub fn get(&self, track_idx: usize) -> Option<&E> {
        self.saves.iter().find(|s| s.track_idx() == track_idx)
    }

    /// Update or insert a campaign state for a track.
    pub fn upsert(&mut self, state: E) {
        let idx = state.track_idx();
        if let Some(existing) = self.saves.iter_mut().find(|s| s.track_idx() == idx) {
            *existing = state;
        } else {
            self.saves.push(state);
        }
    }

    /// Reset a track to fresh state.
    pub fn reset(&mut self, track_idx: usize) {
        self.saves.retain(|s| s.track_idx() != track_idx);
    }

    /// Summary string for a track: "Level 5/15" or "Complete" or empty.
    pub fn progress_label(&self, track_idx: usize) -> String {
        match self.get(track_idx) {
            Some(s) if s.is_completed() => "Complete".to_string(),
            Some(s) => format!("Level {}/{}", s.current_level() + 1, s.total_levels()),
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Clone)]
    struct TestEntry {
        track_idx: usize,
        level: usize,
        total: usize,
        done: bool,
    }

    impl CampaignEntry for TestEntry {
        fn track_idx(&self) -> usize { self.track_idx }
        fn current_level(&self) -> usize { self.level }
        fn total_levels(&self) -> usize { self.total }
        fn is_completed(&self) -> bool { self.done }
    }

    fn entry(track: usize, level: usize, total: usize, done: bool) -> TestEntry {
        TestEntry { track_idx: track, level, total, done }
    }

    #[test]
    fn upsert_and_get() {
        let mut saves = CampaignSaves::<TestEntry>::default();
        saves.upsert(entry(1, 3, 10, false));
        assert!(saves.get(1).is_some());
        assert!(saves.get(0).is_none());
    }

    #[test]
    fn reset_removes_track() {
        let mut saves = CampaignSaves::<TestEntry>::default();
        saves.upsert(entry(0, 0, 10, false));
        saves.upsert(entry(1, 0, 10, false));
        saves.reset(0);
        assert!(saves.get(0).is_none());
        assert!(saves.get(1).is_some());
    }

    #[test]
    fn progress_label_shows_level() {
        let mut saves = CampaignSaves::<TestEntry>::default();
        saves.upsert(entry(0, 4, 15, false));
        assert_eq!(saves.progress_label(0), "Level 5/15");
    }

    #[test]
    fn progress_label_shows_complete() {
        let mut saves = CampaignSaves::<TestEntry>::default();
        saves.upsert(entry(0, 14, 15, true));
        assert_eq!(saves.progress_label(0), "Complete");
    }

    #[test]
    fn progress_label_empty_for_no_save() {
        let saves = CampaignSaves::<TestEntry>::default();
        assert_eq!(saves.progress_label(0), "");
    }

    #[test]
    fn serialization_roundtrip() {
        let mut saves = CampaignSaves::<TestEntry>::default();
        saves.upsert(entry(1, 3, 10, false));
        let json = serde_json::to_string(&saves).unwrap();
        let loaded: CampaignSaves<TestEntry> = serde_json::from_str(&json).unwrap();
        let s = loaded.get(1).unwrap();
        assert_eq!(s.level, 3);
    }
}
