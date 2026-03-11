pub use loom_engine::endless::EndlessHighScore;

use crate::config::Config;

/// Endless mode state: tracks wave progression and banked bonuses.
#[derive(Clone, Debug)]
pub struct EndlessState {
    pub wave: usize,
    pub banked_hammer: u16,
    pub banked_laser: u16,
    pub banked_blaster: u16,
    pub banked_warp: u16,
}

impl EndlessState {
    pub fn new() -> Self {
        Self {
            wave: 1,
            banked_hammer: 0,
            banked_laser: 0,
            banked_blaster: 0,
            banked_warp: 0,
        }
    }

    pub fn advance(&mut self) {
        self.wave += 1;
        if self.wave % 3 == 0 {
            self.banked_hammer += 1;
        }
        if self.wave % 5 == 0 {
            self.banked_laser += 1;
        }
        if self.wave % 7 == 0 {
            self.banked_blaster += 1;
        }
        if self.wave % 11 == 0 {
            self.banked_warp += 1;
        }
    }

    pub fn to_config(&self, base: &Config) -> Config {
        let wave = self.wave as u16;
        let mut cfg = base.clone();
        cfg.board_height = (base.board_height + wave / 5).min(16);
        cfg.board_width = (base.board_width + wave / 5).min(16);
        cfg.color_number = (base.color_number + (wave / 10) as u8).min(8);
        cfg.move_limit = base.move_limit.saturating_sub((wave / 3) as u32).max(10);
        cfg.special_tile_pct = (base.special_tile_pct + wave / 4).min(30);
        cfg.hammer = self.banked_hammer;
        cfg.laser = self.banked_laser;
        cfg.blaster = self.banked_blaster;
        cfg.warp = self.banked_warp;
        cfg
    }
}

impl Default for EndlessState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn base_config() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn starts_at_wave_1() {
        assert_eq!(EndlessState::new().wave, 1);
    }

    #[test]
    fn advance_increments_wave() {
        let mut s = EndlessState::new();
        s.advance();
        assert_eq!(s.wave, 2);
    }

    #[test]
    fn advance_wave_3_banks_hammer() {
        let mut s = EndlessState::new();
        s.advance(); // wave 2
        s.advance(); // wave 3 → hammer
        assert_eq!(s.banked_hammer, 1);
    }

    #[test]
    fn advance_wave_5_banks_laser() {
        let mut s = EndlessState::new();
        for _ in 0..4 {
            s.advance();
        }
        assert_eq!(s.banked_laser, 1);
    }

    #[test]
    fn to_config_applies_scale_factor() {
        let base = base_config();
        let s1 = EndlessState::new();
        let s5 = {
            let mut s = EndlessState::new();
            for _ in 0..4 {
                s.advance();
            }
            s
        };
        let cfg1 = s1.to_config(&base);
        let cfg5 = s5.to_config(&base);
        assert!(cfg5.board_height >= cfg1.board_height);
        assert!(cfg5.color_number >= cfg1.color_number);
        assert!(cfg5.move_limit <= cfg1.move_limit);
    }

    #[test]
    fn to_config_sets_banked_bonuses() {
        let base = base_config();
        let mut s = EndlessState::new();
        s.banked_hammer = 3;
        s.banked_laser = 1;
        let cfg = s.to_config(&base);
        assert_eq!(cfg.hammer, 3);
        assert_eq!(cfg.laser, 1);
    }

    #[test]
    fn high_score_starts_at_zero() {
        assert_eq!(EndlessHighScore::default().best_wave, 0);
    }

    #[test]
    fn update_returns_true_for_new_record() {
        let mut hs = EndlessHighScore::default();
        assert!(hs.update(5));
        assert_eq!(hs.best_wave, 5);
    }

    #[test]
    fn update_returns_false_for_non_record() {
        let mut hs = EndlessHighScore::default();
        hs.update(5);
        assert!(!hs.update(3));
        assert_eq!(hs.best_wave, 5);
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
