/// Campaign blessings — passive modifiers chosen at campaign start.
///
/// 12 blessings across 4 tiers (D/C/B/A). Higher tiers unlock as the
/// player completes campaign tracks.

use crate::blessings::Tier::*;

// ── Tier ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier { D, C, B, A }

impl Tier {
    pub fn label(self) -> &'static str {
        match self { D => "D", C => "C", B => "B", A => "A" }
    }
}

/// The highest tier unlocked given the number of completed campaign tracks.
pub fn unlocked_tier(completed_tracks: usize) -> Tier {
    match completed_tracks {
        0 => D,
        1 => C,
        2 => B,
        _ => A,
    }
}

// ── Blessing definition ───────────────────────────────────────────────

pub struct Blessing {
    pub id: &'static str,
    pub name: &'static str,
    pub tier: Tier,
    pub description: &'static str,
    pub ascii_art: [&'static str; 5],
}

// ── The 12 blessings ──────────────────────────────────────────────────

pub const ALL_BLESSINGS: &[Blessing] = &[
    // ── D-tier: QoL / minor boosts ──
    Blessing {
        id: "extra_moves",
        name: "Extra Moves",
        tier: D,
        description: "+3 moves each level",
        ascii_art: [
            "  ┌─────┐",
            "  │ +3  │",
            "  │moves│",
            "  │  ►  │",
            "  └─────┘",
        ],
    },
    Blessing {
        id: "keen_eye",
        name: "Keen Eye",
        tier: D,
        description: "Highlight a valid swap",
        ascii_art: [
            "    ◉    ",
            "  ╱   ╲  ",
            " │  ↔  │ ",
            "  ╲   ╱  ",
            "    ◎    ",
        ],
    },
    Blessing {
        id: "lucky_start",
        name: "Lucky Start",
        tier: D,
        description: "Start with 1 special piece",
        ascii_art: [
            "    ★    ",
            "   ╱ ╲   ",
            "  │ ◆ │  ",
            "   ╲ ╱   ",
            "    ▽    ",
        ],
    },

    // ── C-tier: moderate gameplay ──
    Blessing {
        id: "ice_breaker",
        name: "Ice Breaker",
        tier: C,
        description: "Ice tiles start with -1 HP",
        ascii_art: [
            "  ╔═══╗  ",
            "  ║ ❄ ║  ",
            "  ║ ↓ ║  ",
            "  ║-1 ║  ",
            "  ╚═══╝  ",
        ],
    },
    Blessing {
        id: "bonus_stash",
        name: "Bonus Stash",
        tier: C,
        description: "+1 hammer each level",
        ascii_art: [
            "  ┌───┐  ",
            "  │ 🔨│  ",
            "  │+1 │  ",
            "  │   │  ",
            "  └───┘  ",
        ],
    },
    Blessing {
        id: "cascade_master",
        name: "Cascade Master",
        tier: C,
        description: "Cascades score 50% more",
        ascii_art: [
            "  ● ● ●  ",
            "   ↓↓↓   ",
            "  ●●●●●  ",
            "   ↓↓↓   ",
            "  ×1.5!  ",
        ],
    },

    // ── B-tier: significant gameplay ──
    Blessing {
        id: "crate_cracker",
        name: "Crate Cracker",
        tier: B,
        description: "Crates start with -1 HP",
        ascii_art: [
            "  ╔═══╗  ",
            "  ║ ▣ ║  ",
            "  ║ ↓ ║  ",
            "  ║-1 ║  ",
            "  ╚═══╝  ",
        ],
    },
    Blessing {
        id: "chain_reaction",
        name: "Chain Reaction",
        tier: B,
        description: "Specials trigger neighbors",
        ascii_art: [
            "  ◆→◆→◆  ",
            "  ↓     ",
            "  ◆→◆   ",
            "  ↓     ",
            "  ◆ BOOM",
        ],
    },
    Blessing {
        id: "color_surge",
        name: "Color Surge",
        tier: B,
        description: "5+ match clears 2 extras",
        ascii_art: [
            " ●●●●●  ",
            "   + +   ",
            "  ● → ×  ",
            "  ● → ×  ",
            "  surge! ",
        ],
    },

    // ── A-tier: powerful ──
    Blessing {
        id: "last_stand",
        name: "Last Stand",
        tier: A,
        description: "Out of moves? +3 once",
        ascii_art: [
            "  ┌─────┐",
            "  │ 0→3 │",
            "  │moves│",
            "  │LAST!│",
            "  └─────┘",
        ],
    },
    Blessing {
        id: "gem_magnet",
        name: "Gem Magnet",
        tier: A,
        description: "4-match → AreaBomb always",
        ascii_art: [
            "  ●●●●  ",
            "   ↓↓   ",
            "  ◆◆◆◆  ",
            "  BOOM  ",
            "  ●●●●  ",
        ],
    },
    Blessing {
        id: "double_score",
        name: "Double Score",
        tier: A,
        description: "All scores ×2",
        ascii_art: [
            "  ╔═══╗  ",
            "  ║ ×2║  ",
            "  ║pts║  ",
            "  ║ ! ║  ",
            "  ╚═══╝  ",
        ],
    },
];

// ── Helpers ───────────────────────────────────────────────────────────

/// All blessings the player may choose from, given completed track count.
pub fn available_blessings(completed_tracks: usize) -> Vec<&'static Blessing> {
    let max_tier = unlocked_tier(completed_tracks);
    ALL_BLESSINGS.iter().filter(|b| b.tier <= max_tier).collect()
}

/// Look up a blessing by ID.
pub fn lookup(id: &str) -> Option<&'static Blessing> {
    ALL_BLESSINGS.iter().find(|b| b.id == id)
}

/// Check whether a list of selected blessing IDs contains a given ID.
pub fn has(ids: &[String], target: &str) -> bool {
    ids.iter().any(|s| s == target)
}

/// Whether a blessing's tier is unlocked at the given track-completion count.
pub fn is_unlocked(blessing: &Blessing, completed_tracks: usize) -> bool {
    blessing.tier <= unlocked_tier(completed_tracks)
}

/// How many tracks must be completed to unlock the given tier.
pub fn tracks_required(tier: Tier) -> usize {
    match tier { D => 0, C => 1, B => 2, A => 3 }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_blessings_count() {
        assert_eq!(ALL_BLESSINGS.len(), 12);
    }

    #[test]
    fn tier_ordering() {
        assert!(D < C);
        assert!(C < B);
        assert!(B < A);
    }

    #[test]
    fn unlocked_tier_progression() {
        assert_eq!(unlocked_tier(0), D);
        assert_eq!(unlocked_tier(1), C);
        assert_eq!(unlocked_tier(2), B);
        assert_eq!(unlocked_tier(3), A);
        assert_eq!(unlocked_tier(99), A);
    }

    #[test]
    fn available_at_zero_tracks() {
        let avail = available_blessings(0);
        assert_eq!(avail.len(), 3); // 3 D-tier
        assert!(avail.iter().all(|b| b.tier == D));
    }

    #[test]
    fn available_at_one_track() {
        let avail = available_blessings(1);
        assert_eq!(avail.len(), 6); // 3 D + 3 C
    }

    #[test]
    fn available_at_two_tracks() {
        let avail = available_blessings(2);
        assert_eq!(avail.len(), 9); // 3 D + 3 C + 3 B
    }

    #[test]
    fn available_at_three_tracks() {
        let avail = available_blessings(3);
        assert_eq!(avail.len(), 12); // all
    }

    #[test]
    fn lookup_finds_by_id() {
        let b = lookup("double_score").unwrap();
        assert_eq!(b.name, "Double Score");
        assert_eq!(b.tier, A);
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup("nonexistent").is_none());
    }

    #[test]
    fn has_checks_presence() {
        let ids = vec!["extra_moves".to_string(), "keen_eye".to_string()];
        assert!(has(&ids, "extra_moves"));
        assert!(!has(&ids, "double_score"));
    }
}
