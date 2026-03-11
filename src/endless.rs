use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::{Config, MAX_BOARD_DIM};

const ENDLESS_FILE: &str = "endless.json";

fn endless_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("knitui").join(ENDLESS_FILE))
}

/// Per-session state for an Endless run.
pub struct EndlessState {
    pub wave: usize,
    /// Scissors/tweezers/balloons earned during the run (not spent).
    pub banked_scissors: u16,
    pub banked_tweezers: u16,
    pub banked_balloons: u16,
}

impl EndlessState {
    pub fn new() -> Self {
        Self { wave: 1, banked_scissors: 0, banked_tweezers: 0, banked_balloons: 0 }
    }

    /// Advance to the next wave. Awards one random-ish bonus.
    pub fn advance(&mut self) {
        // Award a bonus based on wave parity (simple rotation: scissors → tweezers → balloons)
        match self.wave % 3 {
            0 => self.banked_scissors += 1,
            1 => self.banked_tweezers += 1,
            _ => self.banked_balloons += 1,
        }
        self.wave += 1;
    }

    /// Build a Config for the current wave. Difficulty scales smoothly with wave number.
    pub fn to_config(&self, base: &Config) -> Config {
        let w = self.wave as u16;
        let mut cfg = base.clone();
        // Board grows from 4×4 up to MAX_BOARD_DIM×MAX_BOARD_DIM
        cfg.board_height = (4 + w / 3).min(MAX_BOARD_DIM);
        cfg.board_width  = (4 + w / 3).min(MAX_BOARD_DIM);
        // Colors grow from 3 up to 8
        cfg.color_number = (2 + w / 4).min(8).max(2);
        // Obstacles and conveyors grow from 0 up to 20%
        cfg.obstacle_percentage  = (w * 2).min(20);
        cfg.conveyor_percentage = (w * 2).min(20);
        // Bonuses: base level's earned carry-over
        cfg.scissors = self.banked_scissors;
        cfg.tweezers = self.banked_tweezers;
        cfg.balloons = self.banked_balloons;
        cfg
    }
}

/// Persistent high score for Endless mode.
#[derive(Serialize, Deserialize, Default)]
pub struct EndlessHighScore {
    pub best_wave: usize,
}

impl EndlessHighScore {
    pub fn load() -> Self {
        let Some(path) = endless_path() else { return Self::default() };
        match fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let Some(path) = endless_path() else { return };
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
    fn new_state_starts_at_wave_one() {
        let s = EndlessState::new();
        assert_eq!(s.wave, 1);
        assert_eq!(s.banked_scissors, 0);
    }

    #[test]
    fn advance_increments_wave_and_awards_bonus() {
        let mut s = EndlessState::new();
        s.advance(); // wave 1 → 2
        assert_eq!(s.wave, 2);
        // wave 1 % 3 == 1 → tweezers
        assert_eq!(s.banked_tweezers, 1);
    }

    #[test]
    fn to_config_scales_with_wave() {
        let base = Config {
            board_height: 6, board_width: 6, color_number: 6,
            color_mode: "dark".into(), spool_limit: 7,
            spool_capacity: 3, yarn_lines: 4, obstacle_percentage: 5,
            visible_stitches: 6, conveyor_capacity: 3, conveyor_percentage: 5,
            layout: "auto".into(), scale: 1,
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2, ad_file: None,
            max_solutions: None,
        };

        let s1 = EndlessState::new(); // wave 1
        let cfg1 = s1.to_config(&base);
        assert_eq!(cfg1.board_height, 4); // 4 + 1/3 = 4
        assert_eq!(cfg1.color_number, 2); // 2 + 1/4 = 2

        let mut s10 = EndlessState::new();
        for _ in 0..9 { s10.advance(); } // wave 10
        assert_eq!(s10.wave, 10);
        let cfg10 = s10.to_config(&base);
        assert!(cfg10.board_height > cfg1.board_height);
        assert!(cfg10.color_number > cfg1.color_number);
    }

    #[test]
    fn high_score_update_initial_record() {
        let mut hs = EndlessHighScore::default();
        assert!(hs.update(3)); // 3 > 0 → new record
        assert_eq!(hs.best_wave, 3);
    }

    #[test]
    fn high_score_update_returns_true_for_new_record() {
        let mut hs = EndlessHighScore::default();
        assert!(hs.update(5));
        assert_eq!(hs.best_wave, 5);
        assert!(!hs.update(3)); // not better
        assert!(hs.update(7)); // new record
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
