use crossterm::style::Color;
use serde::{Deserialize, Serialize};

pub const MAX_TIER: u8 = 5;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(with = "loom_engine::color_serde")]
    pub color: Color,
    pub tier: u8,
}

impl Item {
    pub fn new(color: Color, tier: u8) -> Self {
        Self { color, tier: tier.min(MAX_TIER) }
    }

    /// Whether this item can merge with another (same color, same tier, below max).
    pub fn can_merge(&self, other: &Item) -> bool {
        self.color == other.color && self.tier == other.tier && self.tier < MAX_TIER
    }

    /// Produce the merged result (tier + 1).
    pub fn merged(&self) -> Item {
        Item { color: self.color, tier: (self.tier + 1).min(MAX_TIER) }
    }

    /// Score value for creating this item via merge.
    pub fn score_value(&self) -> u32 {
        match self.tier {
            1 => 10,
            2 => 30,
            3 => 100,
            4 => 300,
            5 => 1000,
            _ => 0,
        }
    }

    /// Single-character glyph for this tier.
    pub fn glyph(&self) -> &'static str {
        match self.tier {
            1 => "·",
            2 => "○",
            3 => "●",
            4 => "◆",
            5 => "★",
            _ => "?",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_merge_same_color_same_tier() {
        let a = Item::new(Color::Red, 1);
        let b = Item::new(Color::Red, 1);
        assert!(a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_different_color() {
        let a = Item::new(Color::Red, 1);
        let b = Item::new(Color::Blue, 1);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_different_tier() {
        let a = Item::new(Color::Red, 1);
        let b = Item::new(Color::Red, 2);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_at_max_tier() {
        let a = Item::new(Color::Red, MAX_TIER);
        let b = Item::new(Color::Red, MAX_TIER);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn merged_produces_next_tier() {
        let a = Item::new(Color::Red, 2);
        let m = a.merged();
        assert_eq!(m.tier, 3);
        assert_eq!(m.color, Color::Red);
    }

    #[test]
    fn merged_clamps_at_max() {
        let a = Item::new(Color::Red, MAX_TIER);
        assert_eq!(a.merged().tier, MAX_TIER);
    }

    #[test]
    fn score_values_increase_with_tier() {
        let scores: Vec<u32> = (1..=5).map(|t| Item::new(Color::Red, t).score_value()).collect();
        for i in 1..scores.len() {
            assert!(scores[i] > scores[i - 1]);
        }
    }

    #[test]
    fn glyphs_unique_per_tier() {
        let glyphs: Vec<&str> = (1..=5).map(|t| Item::new(Color::Red, t).glyph()).collect();
        for i in 0..glyphs.len() {
            for j in (i + 1)..glyphs.len() {
                assert_ne!(glyphs[i], glyphs[j]);
            }
        }
    }

    #[test]
    fn new_clamps_tier() {
        let item = Item::new(Color::Red, 10);
        assert_eq!(item.tier, MAX_TIER);
    }

    #[test]
    fn serde_roundtrip() {
        let item = Item::new(Color::Red, 3);
        let json = serde_json::to_string(&item).unwrap();
        let restored: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(item, restored);
    }
}
