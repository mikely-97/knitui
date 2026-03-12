/// Campaign blessings — passive modifiers chosen at campaign start.
///
/// 12 blessings across 4 tiers (D/C/B/A), designed for persistent-board
/// merge-2 gameplay with generators, frozen cells, energy, and inventory.

use crate::blessings::Tier::*;

// ── Tier ──────────────────────────────────────────────────────────────────

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

// ── Blessing definition ───────────────────────────────────────────────────

pub struct Blessing {
    pub id: &'static str,
    pub name: &'static str,
    pub tier: Tier,
    pub description: &'static str,
    pub ascii_art: [&'static str; 5],
}

// ── The 12 blessings ─────────────────────────────────────────────────────

pub const ALL_BLESSINGS: &[Blessing] = &[

    // ── D-Tier: immediate quality-of-life ──────────────────────────────
    Blessing {
        id: "energy_saver",
        name: "Energy Saver",
        tier: D,
        description: "25% chance generator tap\ncosts 0 energy",
        ascii_art: [
            "  ⚡ ─ ⚡  ",
            "  │  25% │ ",
            "  │  free│ ",
            "  │ tap! │ ",
            "  └──────┘ ",
        ],
    },
    Blessing {
        id: "quick_regen",
        name: "Quick Regen",
        tier: D,
        description: "Energy regenerates 50%\nfaster (20s instead of 30s)",
        ascii_art: [
            "  ⚡ → ⚡  ",
            "  30s→20s  ",
            "  ┌──────┐ ",
            "  │ fast │ ",
            "  └──────┘ ",
        ],
    },
    Blessing {
        id: "keen_eye",
        name: "Keen Eye",
        tier: D,
        description: "Highlights a valid merge\npair on the board",
        ascii_art: [
            "    ◉     ",
            "  ╱   ╲   ",
            " │  ↔  │  ",
            "  ╲   ╱   ",
            "    ◎     ",
        ],
    },

    // ── C-Tier: moderate expansion ─────────────────────────────────────
    Blessing {
        id: "bigger_pockets",
        name: "Bigger Pockets",
        tier: C,
        description: "+2 inventory slots\nat campaign start",
        ascii_art: [
            "  ┌─────┐  ",
            "  │[ ][ ]│ ",
            "  │  +2  │ ",
            "  │slots │ ",
            "  └─────┘  ",
        ],
    },
    Blessing {
        id: "thaw_aura",
        name: "Thaw Aura",
        tier: C,
        description: "Merging into a frozen cell\nalso thaws adjacent frozen",
        ascii_art: [
            "  ░ → □   ",
            " ░[✦]░→□  ",
            "  ░ → □   ",
            "  aura!   ",
            "          ",
        ],
    },
    Blessing {
        id: "lucky_orders",
        name: "Lucky Orders",
        tier: C,
        description: "Random orders require\n1 fewer item (min 1)",
        ascii_art: [
            "  ╔═════╗  ",
            "  ║ ✓ ✓ ║  ",
            "  ║ -1  ║  ",
            "  ║ qty ║  ",
            "  ╚═════╝  ",
        ],
    },

    // ── B-Tier: significant gameplay impact ────────────────────────────
    Blessing {
        id: "chain_merge",
        name: "Chain Merge",
        tier: B,
        description: "After a merge, auto-merge\nresult up to 3 more times",
        ascii_art: [
            "  ○→○→●   ",
            "      ↓   ",
            "  ●→●→◆   ",
            "      ↓   ",
            "  chain!  ",
        ],
    },
    Blessing {
        id: "tier_boost",
        name: "Tier Boost",
        tier: B,
        description: "15% chance a merge\nskips one tier",
        ascii_art: [
            "  ○ + ○   ",
            "    ↓     ",
            "  ● → ◆   ",
            "   15%    ",
            "  skip!   ",
        ],
    },
    Blessing {
        id: "generator_surge",
        name: "Generator Surge",
        tier: B,
        description: "Hard generators 20% chance\nto produce a T2 item",
        ascii_art: [
            "  ┌─★─┐   ",
            "  │ G∞│   ",
            "  │T2!│   ",
            "  │20%│   ",
            "  └───┘   ",
        ],
    },

    // ── A-Tier: powerful campaign-defining effects ─────────────────────
    Blessing {
        id: "double_deliver",
        name: "Double Deliver",
        tier: A,
        description: "Each delivery satisfies\n2 of the requirement",
        ascii_art: [
            "  ╔═════╗  ",
            "  ║ × 2 ║  ",
            "  ║ del ║  ",
            "  ║  !  ║  ",
            "  ╚═════╝  ",
        ],
    },
    Blessing {
        id: "soft_gen_master",
        name: "Soft Gen Master",
        tier: A,
        description: "Soft generators +3 charges;\n30% chance not to consume",
        ascii_art: [
            "  ┌─────┐  ",
            "  │ G+3 │  ",
            "  │  &  │  ",
            "  │ 30% │  ",
            "  └─────┘  ",
        ],
    },
    Blessing {
        id: "deep_thaw",
        name: "Deep Thaw",
        tier: A,
        description: "Thawed items automatically\nupgrade +1 tier",
        ascii_art: [
            "  ░[○]░   ",
            "    ↓     ",
            "  □[●]□   ",
            "  +1 tier ",
            "  on thaw ",
        ],
    },
];

// ── Helpers ───────────────────────────────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_blessings_count() {
        assert_eq!(ALL_BLESSINGS.len(), 12);
    }

    #[test]
    fn three_per_tier() {
        for tier in [D, C, B, A] {
            let count = ALL_BLESSINGS.iter().filter(|b| b.tier == tier).count();
            assert_eq!(count, 3, "tier {:?} should have 3 blessings", tier);
        }
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
        assert_eq!(avail.len(), 3);
        assert!(avail.iter().all(|b| b.tier == D));
    }

    #[test]
    fn available_at_one_track() {
        assert_eq!(available_blessings(1).len(), 6);
    }

    #[test]
    fn available_at_two_tracks() {
        assert_eq!(available_blessings(2).len(), 9);
    }

    #[test]
    fn available_at_three_tracks() {
        assert_eq!(available_blessings(3).len(), 12);
    }

    #[test]
    fn lookup_finds_by_id() {
        let b = lookup("deep_thaw").unwrap();
        assert_eq!(b.name, "Deep Thaw");
        assert_eq!(b.tier, A);
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup("nonexistent").is_none());
    }

    #[test]
    fn has_checks_presence() {
        let ids = vec!["energy_saver".to_string(), "keen_eye".to_string()];
        assert!(has(&ids, "energy_saver"));
        assert!(!has(&ids, "deep_thaw"));
    }

    #[test]
    fn all_ids_are_unique() {
        let mut ids: Vec<&str> = ALL_BLESSINGS.iter().map(|b| b.id).collect();
        let before = ids.len();
        ids.dedup();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), before, "duplicate IDs found");
    }

    #[test]
    fn ascii_art_rows_correct_count() {
        for b in ALL_BLESSINGS {
            assert_eq!(b.ascii_art.len(), 5, "{} art wrong", b.id);
        }
    }
}
