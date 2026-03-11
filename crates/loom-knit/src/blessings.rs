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
    // ── D-tier: QoL / informational ──
    Blessing {
        id: "scouts_eye",
        name: "Scout's Eye",
        tier: D,
        description: "Highlight locked stitches",
        ascii_art: [
            "    ◉    ",
            "  ╱   ╲  ",
            " │  ●  │ ",
            "  ╲   ╱  ",
            "    ◎    ",
        ],
    },
    Blessing {
        id: "wrap_around",
        name: "Wrap Around",
        tier: D,
        description: "Cursor wraps at edges",
        ascii_art: [
            "  ←─────→",
            "  │     │",
            "  │  ◆  │",
            "  │     │",
            "  ←─────→",
        ],
    },
    Blessing {
        id: "tidy_workspace",
        name: "Tidy Workspace",
        tier: D,
        description: "Auto-sort held spools",
        ascii_art: [
            " ╔═╗╔═╗  ",
            " ║1║║2║  ",
            " ╚═╝╚═╝  ",
            " ╔═╗╔═╗  ",
            " ║3║║4║  ",
        ],
    },
    Blessing {
        id: "conveyor_peek",
        name: "Conveyor Peek",
        tier: D,
        description: "See next conveyor spool",
        ascii_art: [
            "  ┌───┐  ",
            "  │ ▶ │  ",
            "  │ T │  ",
            "  │ ? │  ",
            "  └───┘  ",
        ],
    },
    Blessing {
        id: "color_count",
        name: "Color Count",
        tier: D,
        description: "Show spool counts by color",
        ascii_art: [
            "  R:3 G:2 ",
            "  B:4 Y:1 ",
            "  ───── ",
            "  total  ",
            "   10    ",
        ],
    },
    Blessing {
        id: "match_hint",
        name: "Match Hint",
        tier: D,
        description: "Highlight matching yarn",
        ascii_art: [
            "  ▦ ▦ ▦  ",
            "  · ★ ·  ",
            "  ▦ ▦ ▦  ",
            "  · · ★  ",
            "  ▦ ▦ ▦  ",
        ],
    },

    // ── C-tier: minor gameplay ──
    Blessing {
        id: "lucky_find",
        name: "Lucky Find",
        tier: C,
        description: "3% obstacles become voids",
        ascii_art: [
            "    ★    ",
            "   ╱ ╲   ",
            "  │ ♣ │  ",
            "   ╲ ╱   ",
            "    ▽    ",
        ],
    },
    Blessing {
        id: "apprentices_kit",
        name: "Apprentice's Kit",
        tier: C,
        description: "+1 scissors at start",
        ascii_art: [
            "    ✂    ",
            "   ╱ ╲   ",
            "  ╱   ╲  ",
            " ╱     ╲ ",
            " ─     ─ ",
        ],
    },
    Blessing {
        id: "light_pockets",
        name: "Light Pockets",
        tier: C,
        description: "+1 balloon at start",
        ascii_art: [
            "    ⊛    ",
            "   ╱ ╲   ",
            "  │   │  ",
            "   ╲ ╱   ",
            "    │    ",
        ],
    },

    // ── B-tier: significant gameplay ──
    Blessing {
        id: "extra_slot",
        name: "Extra Slot",
        tier: B,
        description: "+1 spool limit",
        ascii_art: [
            " ┌─┬─┬─┐ ",
            " │T│T│T│ ",
            " ├─┼─┼─┤ ",
            " │T│T│+│ ",
            " └─┴─┴─┘ ",
        ],
    },
    Blessing {
        id: "sharp_start",
        name: "Sharp Start",
        tier: B,
        description: "+1 scissors each level",
        ascii_art: [
            "   ✂ ✂   ",
            "  ╱╲╱╲  ",
            " ╱    ╲ ",
            " ─    ─ ",
            "  ✂  ✂  ",
        ],
    },

    // ── A-tier: powerful ──
    Blessing {
        id: "double_cut",
        name: "Double Cut",
        tier: A,
        description: "Scissors wind 2 spools",
        ascii_art: [
            "  ✂ ═══ ✂",
            "  ║ T T ║",
            "  ║ ↑ ↑ ║",
            "  ║ ★ ★ ║",
            "  ✂ ═══ ✂",
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
        assert_eq!(avail.len(), 6); // 6 D-tier
        assert!(avail.iter().all(|b| b.tier == D));
    }

    #[test]
    fn available_at_one_track() {
        let avail = available_blessings(1);
        assert_eq!(avail.len(), 9); // 6 D + 3 C
    }

    #[test]
    fn available_at_two_tracks() {
        let avail = available_blessings(2);
        assert_eq!(avail.len(), 11); // 6 D + 3 C + 2 B
    }

    #[test]
    fn available_at_three_tracks() {
        let avail = available_blessings(3);
        assert_eq!(avail.len(), 12); // all
    }

    #[test]
    fn lookup_finds_by_id() {
        let b = lookup("double_cut").unwrap();
        assert_eq!(b.name, "Double Cut");
        assert_eq!(b.tier, A);
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup("nonexistent").is_none());
    }

    #[test]
    fn has_checks_presence() {
        let ids = vec!["scouts_eye".to_string(), "extra_slot".to_string()];
        assert!(has(&ids, "scouts_eye"));
        assert!(!has(&ids, "double_cut"));
    }
}
