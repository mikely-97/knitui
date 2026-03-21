use crate::puzzles::all_puzzles;
use crate::puzzle::Puzzle;

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
