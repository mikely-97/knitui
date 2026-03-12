use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Energy system: generators cost energy to activate; energy regens over wall-clock time.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: u16,
    pub max: u16,
    /// Seconds between each +1 energy regen.
    pub regen_rate_secs: u32,
    /// Unix epoch seconds of last regen tick.
    pub last_regen_epoch: u64,
}

impl Energy {
    pub fn new(max: u16, regen_rate_secs: u32) -> Self {
        Self {
            current: max,
            max,
            regen_rate_secs,
            last_regen_epoch: now_epoch(),
        }
    }

    /// Try to spend energy. Returns true if successful.
    pub fn spend(&mut self, amount: u16) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    /// Add energy (e.g. from ads, order rewards). Capped at max.
    pub fn add(&mut self, amount: u16) {
        self.current = (self.current + amount).min(self.max);
    }

    /// Fill to max.
    pub fn fill(&mut self) {
        self.current = self.max;
    }

    /// Process wall-clock regen. Call this on every tick.
    /// Returns the amount of energy regenerated.
    pub fn regen_tick(&mut self) -> u16 {
        if self.current >= self.max || self.regen_rate_secs == 0 {
            self.last_regen_epoch = now_epoch();
            return 0;
        }

        let now = now_epoch();
        let elapsed = now.saturating_sub(self.last_regen_epoch);
        let intervals = elapsed / self.regen_rate_secs as u64;

        if intervals == 0 {
            return 0;
        }

        let regen = intervals as u16;
        let old = self.current;
        self.current = (self.current + regen).min(self.max);
        self.last_regen_epoch += intervals * self.regen_rate_secs as u64;

        self.current - old
    }

    /// Seconds until next +1 energy.
    pub fn secs_until_next(&self) -> u32 {
        if self.current >= self.max || self.regen_rate_secs == 0 {
            return 0;
        }
        let now = now_epoch();
        let elapsed = now.saturating_sub(self.last_regen_epoch);
        let remaining = self.regen_rate_secs as u64 - elapsed.min(self.regen_rate_secs as u64);
        remaining as u32
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }

    pub fn fraction(&self) -> f32 {
        if self.max == 0 {
            return 1.0;
        }
        self.current as f32 / self.max as f32
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_full() {
        let e = Energy::new(100, 30);
        assert_eq!(e.current, 100);
        assert!(e.is_full());
    }

    #[test]
    fn spend_deducts() {
        let mut e = Energy::new(100, 30);
        assert!(e.spend(10));
        assert_eq!(e.current, 90);
    }

    #[test]
    fn spend_fails_insufficient() {
        let mut e = Energy::new(5, 30);
        assert!(!e.spend(10));
        assert_eq!(e.current, 5);
    }

    #[test]
    fn add_caps_at_max() {
        let mut e = Energy::new(100, 30);
        e.current = 95;
        e.add(10);
        assert_eq!(e.current, 100);
    }

    #[test]
    fn fill_restores() {
        let mut e = Energy::new(100, 30);
        e.current = 0;
        e.fill();
        assert_eq!(e.current, 100);
    }

    #[test]
    fn fraction_correct() {
        let mut e = Energy::new(100, 30);
        e.current = 50;
        assert!((e.fraction() - 0.5).abs() < 0.01);
    }

    #[test]
    fn serde_roundtrip() {
        let e = Energy::new(100, 30);
        let json = serde_json::to_string(&e).unwrap();
        let restored: Energy = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.max, 100);
        assert_eq!(restored.regen_rate_secs, 30);
    }
}
