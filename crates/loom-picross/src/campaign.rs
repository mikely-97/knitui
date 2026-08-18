use serde::{Deserialize, Serialize};

use loom_engine::campaign::CampaignEntry;

use crate::puzzles::all_puzzles;
use crate::puzzle::Puzzle;

/// Campaign-progress entry for picross. Minimal by design: picross's own
/// `tui.rs` doesn't have a campaign flow yet (see the Phase 3 scaffolding
/// survey — picross bypasses `Game`/`Surface` entirely today), so this only
/// exists to satisfy `Game::CampaignEntry` for the shared `Shell<G>`.
#[derive(Serialize, Deserialize, Clone)]
pub struct PicrossCampaignEntry {
    pub track_idx: usize,
    pub current_level: usize,
    pub completed: bool,
}

impl CampaignEntry for PicrossCampaignEntry {
    fn track_idx(&self) -> usize { self.track_idx }
    fn current_level(&self) -> usize { self.current_level }
    fn total_levels(&self) -> usize { level_count(self.track_idx) }
    fn is_completed(&self) -> bool { self.completed }
}

impl PicrossCampaignEntry {
    pub fn new(track_idx: usize) -> Self {
        Self { track_idx, current_level: 0, completed: false }
    }

    /// Advance to the next puzzle. Returns true if the track is now complete.
    pub fn complete_level(&mut self) -> bool {
        self.current_level += 1;
        if self.current_level >= level_count(self.track_idx) {
            self.completed = true;
        }
        self.completed
    }
}

pub const TRACK_COUNT: usize = 3;
pub const TRACK_NAMES: &[&str] = &["Easy", "Medium", "Hard"];

/// Returns the puzzles for a given difficulty track (0=Easy, 1=Medium, 2=Hard).
pub fn levels_for_track(track: usize) -> Vec<Puzzle> {
    use crate::puzzle::Difficulty;
    let all = all_puzzles();
    let diff = match track {
        0 => Difficulty::Easy,
        1 => Difficulty::Medium,
        2 => Difficulty::Hard,
        _ => return vec![],
    };
    all.into_iter().filter(|p| p.difficulty == diff).collect()
}

pub fn level_count(track: usize) -> usize {
    levels_for_track(track).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_track_has_puzzles() {
        for t in 0..TRACK_COUNT {
            assert!(!levels_for_track(t).is_empty(), "track {} is empty", t);
        }
    }

    #[test]
    fn track_names_count_matches() {
        assert_eq!(TRACK_NAMES.len(), TRACK_COUNT);
    }
}
