pub use loom_engine::endless::EndlessHighScore;

use crate::config::Config;

/// Endless mode state: tracks wave progression.
#[derive(Clone, Debug)]
pub struct EndlessState {
    pub wave: usize,
}

impl EndlessState {
    pub fn new() -> Self {
        Self { wave: 1 }
    }

    pub fn advance(&mut self) {
        self.wave += 1;
    }

    /// Scale config for current wave.
    pub fn to_config(&self, base: &Config) -> Config {
        let wave = self.wave as u16;
        let mut cfg = base.clone();
        // Board grows slowly
        cfg.board_height = (base.board_height + wave / 5).min(8);
        cfg.board_width = (base.board_width + wave / 5).min(8);
        // More colors over time
        cfg.color_count = (base.color_count + wave / 6).min(6);
        // Generator charges shrink
        if cfg.generator_charges > 0 {
            cfg.generator_charges = cfg.generator_charges.saturating_sub(wave / 3).max(4);
        }
        // Generator interval grows (slower spawning)
        cfg.generator_interval = base.generator_interval + (wave as u32 / 4);
        // Orders get harder
        cfg.max_order_tier = (2 + (wave / 4) as u8).min(5);
        cfg.order_count = (1 + wave / 5).min(4);
        // Fewer ads
        cfg.ad_limit = base.ad_limit.saturating_sub(wave / 4);
        // More blocked cells
        cfg.blocked_cells = (wave / 5).min(4);
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn base() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    #[test]
    fn starts_at_wave_1() {
        let s = EndlessState::new();
        assert_eq!(s.wave, 1);
    }

    #[test]
    fn advance_increments_wave() {
        let mut s = EndlessState::new();
        s.advance();
        assert_eq!(s.wave, 2);
    }

    #[test]
    fn to_config_scales_difficulty() {
        let mut s = EndlessState::new();
        let cfg1 = s.to_config(&base());
        for _ in 0..10 { s.advance(); }
        let cfg11 = s.to_config(&base());
        // Board should be larger or same
        assert!(cfg11.board_height >= cfg1.board_height);
        // Max order tier should increase
        assert!(cfg11.max_order_tier >= cfg1.max_order_tier);
    }
}
