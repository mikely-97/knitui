use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_TIER: u8 = 8;

/// The six item families in the merge-2 game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Family {
    Wood,
    Stone,
    Metal,
    Cloth,
    Crystal,
    Ember,
}

pub const ALL_FAMILIES: &[Family] = &[
    Family::Wood,
    Family::Stone,
    Family::Metal,
    Family::Cloth,
    Family::Crystal,
    Family::Ember,
];

impl Family {
    /// Index into ALL_FAMILIES (0..6).
    pub fn index(self) -> usize {
        match self {
            Family::Wood => 0,
            Family::Stone => 1,
            Family::Metal => 2,
            Family::Cloth => 3,
            Family::Crystal => 4,
            Family::Ember => 5,
        }
    }

    pub fn from_index(i: usize) -> Option<Family> {
        ALL_FAMILIES.get(i).copied()
    }

    /// Human-readable family name.
    pub fn name(self) -> &'static str {
        match self {
            Family::Wood => "Wood",
            Family::Stone => "Stone",
            Family::Metal => "Metal",
            Family::Cloth => "Cloth",
            Family::Crystal => "Crystal",
            Family::Ember => "Ember",
        }
    }

    /// Single-char glyph for a given tier (1-based).
    pub fn glyph(self, tier: u8) -> &'static str {
        match self {
            Family::Wood => match tier {
                1 => "·",
                2 => "○",
                3 => "●",
                4 => "◆",
                5 => "★",
                6 => "✦",
                7 => "❖",
                8 => "✿",
                _ => "?",
            },
            Family::Stone => match tier {
                1 => "▪",
                2 => "□",
                3 => "■",
                4 => "◇",
                5 => "◈",
                6 => "◉",
                7 => "⬡",
                8 => "⬢",
                _ => "?",
            },
            Family::Metal => match tier {
                1 => "∘",
                2 => "△",
                3 => "▲",
                4 => "⊡",
                5 => "⊞",
                6 => "⊠",
                7 => "⚙",
                8 => "⛓",
                _ => "?",
            },
            Family::Cloth => match tier {
                1 => "~",
                2 => "≈",
                3 => "§",
                4 => "¶",
                5 => "⊕",
                6 => "⊗",
                7 => "⊛",
                8 => "✤",
                _ => "?",
            },
            Family::Crystal => match tier {
                1 => "⋄",
                2 => "◇",
                3 => "◈",
                4 => "◉",
                5 => "⬡",
                6 => "⬢",
                7 => "✧",
                8 => "✵",
                _ => "?",
            },
            Family::Ember => match tier {
                1 => "'",
                2 => "‹",
                3 => "«",
                4 => "♦",
                5 => "♣",
                6 => "♠",
                7 => "♛",
                8 => "♔",
                _ => "?",
            },
        }
    }

    /// Tier name for display in orders and inventory.
    pub fn tier_name(self, tier: u8) -> &'static str {
        match self {
            Family::Wood => match tier {
                1 => "Twig",
                2 => "Stick",
                3 => "Plank",
                4 => "Log",
                5 => "Lumber",
                6 => "Beam",
                7 => "Frame",
                8 => "Manor",
                _ => "???",
            },
            Family::Stone => match tier {
                1 => "Pebble",
                2 => "Rock",
                3 => "Boulder",
                4 => "Pillar",
                5 => "Obelisk",
                6 => "Monolith",
                7 => "Citadel",
                8 => "Bastion",
                _ => "???",
            },
            Family::Metal => match tier {
                1 => "Scrap",
                2 => "Nail",
                3 => "Gear",
                4 => "Anvil",
                5 => "Ingot",
                6 => "Crucible",
                7 => "Forge",
                8 => "Foundry",
                _ => "???",
            },
            Family::Cloth => match tier {
                1 => "Thread",
                2 => "Yarn",
                3 => "Ribbon",
                4 => "Bolt",
                5 => "Tapestry",
                6 => "Vestment",
                7 => "Regalia",
                8 => "Resplend",
                _ => "???",
            },
            Family::Crystal => match tier {
                1 => "Shard",
                2 => "Chip",
                3 => "Gem",
                4 => "Jewel",
                5 => "Prism",
                6 => "Radiant",
                7 => "Astral",
                8 => "Eternal",
                _ => "???",
            },
            Family::Ember => match tier {
                1 => "Spark",
                2 => "Flame",
                3 => "Blaze",
                4 => "Pyre",
                5 => "Inferno",
                6 => "Furnace",
                7 => "Phoenix",
                8 => "Solaris",
                _ => "???",
            },
        }
    }

    /// 3-letter abbreviation for compact display.
    pub fn abbrev(self, tier: u8) -> &'static str {
        match self {
            Family::Wood => match tier {
                1 => "Twg", 2 => "Stk", 3 => "Plk", 4 => "Log",
                5 => "Lmb", 6 => "Bea", 7 => "Frm", 8 => "Mnr",
                _ => "???",
            },
            Family::Stone => match tier {
                1 => "Pbl", 2 => "Rck", 3 => "Bld", 4 => "Plr",
                5 => "Obl", 6 => "Mon", 7 => "Ctd", 8 => "Bst",
                _ => "???",
            },
            Family::Metal => match tier {
                1 => "Scr", 2 => "Nal", 3 => "Ger", 4 => "Anv",
                5 => "Igt", 6 => "Crc", 7 => "Frg", 8 => "Fnd",
                _ => "???",
            },
            Family::Cloth => match tier {
                1 => "Thr", 2 => "Yrn", 3 => "Rbn", 4 => "Blt",
                5 => "Tps", 6 => "Vst", 7 => "Rgl", 8 => "Rsp",
                _ => "???",
            },
            Family::Crystal => match tier {
                1 => "Shd", 2 => "Chp", 3 => "Gem", 4 => "Jwl",
                5 => "Prm", 6 => "Rad", 7 => "Ast", 8 => "Etr",
                _ => "???",
            },
            Family::Ember => match tier {
                1 => "Spk", 2 => "Flm", 3 => "Blz", 4 => "Pyr",
                5 => "Inf", 6 => "Fnc", 7 => "Phx", 8 => "Sol",
                _ => "???",
            },
        }
    }
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A regular item with a family and tier.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Item {
    pub family: Family,
    pub tier: u8,
}

impl Item {
    pub fn new(family: Family, tier: u8) -> Self {
        Self { family, tier: tier.min(MAX_TIER).max(1) }
    }

    pub fn can_merge(&self, other: &Item) -> bool {
        self.family == other.family && self.tier == other.tier && self.tier < MAX_TIER
    }

    pub fn merged(&self) -> Item {
        Item { family: self.family, tier: (self.tier + 1).min(MAX_TIER) }
    }

    pub fn glyph(&self) -> &'static str {
        self.family.glyph(self.tier)
    }

    pub fn name(&self) -> &'static str {
        self.family.tier_name(self.tier)
    }

    pub fn abbrev(&self) -> &'static str {
        self.family.abbrev(self.tier)
    }

    pub fn score_value(&self) -> u32 {
        match self.tier {
            1 => 10,
            2 => 30,
            3 => 100,
            4 => 300,
            5 => 1_000,
            6 => 3_000,
            7 => 10_000,
            8 => 30_000,
            _ => 0,
        }
    }
}

/// A piece on the board — either a regular item or a blueprint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Piece {
    Regular(Item),
    Blueprint(Family),
}

impl Piece {
    /// Whether two pieces can merge.
    pub fn can_merge(&self, other: &Piece) -> bool {
        match (self, other) {
            (Piece::Regular(a), Piece::Regular(b)) => a.can_merge(b),
            (Piece::Blueprint(a), Piece::Blueprint(b)) => a == b,
            _ => false,
        }
    }

    /// The family this piece belongs to.
    pub fn family(&self) -> Family {
        match self {
            Piece::Regular(item) => item.family,
            Piece::Blueprint(fam) => *fam,
        }
    }

    pub fn glyph(&self) -> &'static str {
        match self {
            Piece::Regular(item) => item.glyph(),
            Piece::Blueprint(_) => "B",
        }
    }

    pub fn abbrev(&self) -> String {
        match self {
            Piece::Regular(item) => item.abbrev().to_string(),
            Piece::Blueprint(fam) => format!("B{}", &fam.name()[..1]),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Piece::Regular(item) => format!("{} {}", item.family.name(), item.name()),
            Piece::Blueprint(fam) => format!("{} Blueprint", fam.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_merge_same_family_same_tier() {
        let a = Item::new(Family::Wood, 1);
        let b = Item::new(Family::Wood, 1);
        assert!(a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_different_family() {
        let a = Item::new(Family::Wood, 1);
        let b = Item::new(Family::Stone, 1);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_different_tier() {
        let a = Item::new(Family::Wood, 1);
        let b = Item::new(Family::Wood, 2);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn cannot_merge_at_max_tier() {
        let a = Item::new(Family::Wood, MAX_TIER);
        let b = Item::new(Family::Wood, MAX_TIER);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn merged_produces_next_tier() {
        let a = Item::new(Family::Metal, 3);
        let m = a.merged();
        assert_eq!(m.tier, 4);
        assert_eq!(m.family, Family::Metal);
    }

    #[test]
    fn merged_clamps_at_max() {
        let a = Item::new(Family::Ember, MAX_TIER);
        assert_eq!(a.merged().tier, MAX_TIER);
    }

    #[test]
    fn piece_merge_regular() {
        let a = Piece::Regular(Item::new(Family::Wood, 2));
        let b = Piece::Regular(Item::new(Family::Wood, 2));
        assert!(a.can_merge(&b));
    }

    #[test]
    fn piece_merge_blueprints() {
        let a = Piece::Blueprint(Family::Metal);
        let b = Piece::Blueprint(Family::Metal);
        assert!(a.can_merge(&b));
    }

    #[test]
    fn piece_no_merge_blueprint_with_regular() {
        let a = Piece::Blueprint(Family::Wood);
        let b = Piece::Regular(Item::new(Family::Wood, 1));
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn piece_no_merge_different_blueprint_families() {
        let a = Piece::Blueprint(Family::Wood);
        let b = Piece::Blueprint(Family::Stone);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn score_values_increase_with_tier() {
        let scores: Vec<u32> = (1..=8).map(|t| Item::new(Family::Wood, t).score_value()).collect();
        for i in 1..scores.len() {
            assert!(scores[i] > scores[i - 1]);
        }
    }

    #[test]
    fn new_clamps_tier() {
        let item = Item::new(Family::Ember, 20);
        assert_eq!(item.tier, MAX_TIER);
        let item = Item::new(Family::Ember, 0);
        assert_eq!(item.tier, 1);
    }

    #[test]
    fn family_index_roundtrip() {
        for fam in ALL_FAMILIES {
            assert_eq!(Family::from_index(fam.index()), Some(*fam));
        }
    }

    #[test]
    fn serde_roundtrip() {
        let item = Item::new(Family::Crystal, 5);
        let json = serde_json::to_string(&item).unwrap();
        let restored: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(item, restored);
    }

    #[test]
    fn piece_serde_roundtrip() {
        let p = Piece::Blueprint(Family::Metal);
        let json = serde_json::to_string(&p).unwrap();
        let restored: Piece = serde_json::from_str(&json).unwrap();
        assert_eq!(p, restored);
    }
}
