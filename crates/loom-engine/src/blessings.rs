/// Shared blessing infrastructure for all loom games.

// ── Tier ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier { D, C, B, A }

impl Tier {
    pub fn label(self) -> &'static str {
        match self { Tier::D => "D", Tier::C => "C", Tier::B => "B", Tier::A => "A" }
    }
}

/// The highest tier unlocked given the number of completed campaign tracks.
pub fn unlocked_tier(completed_tracks: usize) -> Tier {
    match completed_tracks {
        0 => Tier::D,
        1 => Tier::C,
        2 => Tier::B,
        _ => Tier::A,
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

// ── Shared helpers ────────────────────────────────────────────────────

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
    match tier { Tier::D => 0, Tier::C => 1, Tier::B => 2, Tier::A => 3 }
}
