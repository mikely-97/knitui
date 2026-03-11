use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const ENDLESS_FILE: &str = "endless.json";

fn endless_path(config_dir: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(config_dir).join(ENDLESS_FILE))
}

/// Persistent high score for Endless mode.
#[derive(Serialize, Deserialize, Default)]
pub struct EndlessHighScore {
    pub best_wave: usize,
}

impl EndlessHighScore {
    pub fn load(config_dir: &str) -> Self {
        let Some(path) = endless_path(config_dir) else { return Self::default() };
        match fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, config_dir: &str) {
        let Some(path) = endless_path(config_dir) else { return };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    /// Update if current wave beats the record. Returns true if new record.
    pub fn update(&mut self, wave: usize) -> bool {
        if wave > self.best_wave {
            self.best_wave = wave;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_score_update_initial_record() {
        let mut hs = EndlessHighScore::default();
        assert!(hs.update(3));
        assert_eq!(hs.best_wave, 3);
    }

    #[test]
    fn high_score_update_returns_true_for_new_record() {
        let mut hs = EndlessHighScore::default();
        assert!(hs.update(5));
        assert_eq!(hs.best_wave, 5);
        assert!(!hs.update(3));
        assert!(hs.update(7));
        assert_eq!(hs.best_wave, 7);
    }

    #[test]
    fn high_score_serialization_roundtrip() {
        let mut hs = EndlessHighScore::default();
        hs.update(12);
        let json = serde_json::to_string(&hs).unwrap();
        let loaded: EndlessHighScore = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.best_wave, 12);
    }
}
