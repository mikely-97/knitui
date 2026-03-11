/// Return the level list for a campaign track (0=Beginner, 1=Adventurer, 2=Master).
pub fn levels_for_track(track_idx: usize) -> Vec<LevelDef> {
    match track_idx {
        0 => short_track(),
        1 => medium_track(),
        2 => long_track(),
        _ => vec![],
    }
}

// ── Beginner track (15 levels) ────────────────────────────────────────────

fn short_track() -> Vec<LevelDef> {
    (0..15usize).map(|i| {
        let difficulty = i as u32;
        LevelDef {
            board_height:     (5 + i / 3) as u16,
            board_width:      (5 + i / 3) as u16,
            color_number:     (4 + i / 5) as u8,
            move_limit:       30 + difficulty * 2,
            special_tile_pct: (i * 2) as u16,
            objective: LevelObjective {
                score_target:       Some(1000 + difficulty * 500),
                gem_quota:          vec![],
                clear_all_specials: i >= 10,
            },
            reward_hammer:  if i % 5 == 4 { 1 } else { 0 },
            reward_laser:   if i % 7 == 6 { 1 } else { 0 },
            reward_blaster: 0,
            reward_warp:    if i == 14   { 1 } else { 0 },
        }
    }).collect()
}

// ── Adventurer track (30 levels) ─────────────────────────────────────────

fn medium_track() -> Vec<LevelDef> {
    (0..30usize).map(|i| {
        let difficulty = i as u32;
        LevelDef {
            board_height:     (6 + i / 5) as u16,
            board_width:      (6 + i / 5) as u16,
            color_number:     (5 + i / 6) as u8,
            move_limit:       28 + difficulty,
            special_tile_pct: (5 + i * 2) as u16,
            objective: LevelObjective {
                score_target:       Some(2000 + difficulty * 800),
                gem_quota:          if i % 4 == 0 { vec![(0, 20 + difficulty * 2)] } else { vec![] },
                clear_all_specials: i >= 20,
            },
            reward_hammer:  if i % 3 == 2 { 1 } else { 0 },
            reward_laser:   if i % 4 == 3 { 1 } else { 0 },
            reward_blaster: if i % 5 == 4 { 1 } else { 0 },
            reward_warp:    if i % 7 == 6 { 1 } else { 0 },
        }
    }).collect()
}

// ── Master track (50 levels) ──────────────────────────────────────────────

fn long_track() -> Vec<LevelDef> {
    (0..50usize).map(|i| {
        let difficulty = i as u32;
        LevelDef {
            board_height:     (7 + i / 7) as u16,
            board_width:      (7 + i / 7) as u16,
            color_number:     (6 + i / 8) as u8,
            move_limit:       25 + difficulty,
            special_tile_pct: (10 + i * 2) as u16,
            objective: LevelObjective {
                score_target:       Some(5000 + difficulty * 1000),
                gem_quota:          if i % 3 == 0 { vec![(0, 30 + difficulty * 2)] } else { vec![] },
                clear_all_specials: i >= 30,
            },
            reward_hammer:  if i % 3 == 2 { 1 } else { 0 },
            reward_laser:   if i % 4 == 3 { 1 } else { 0 },
            reward_blaster: if i % 5 == 4 { 1 } else { 0 },
            reward_warp:    if i % 7 == 6 { 1 } else { 0 },
        }
    }).collect()
}

#[derive(Clone, Debug)]
pub struct LevelDef {
    pub board_height:     u16,
    pub board_width:      u16,
    pub color_number:     u8,
    pub move_limit:       u32,
    pub special_tile_pct: u16,
    pub objective:        LevelObjective,
    pub reward_hammer:    u16,
    pub reward_laser:     u16,
    pub reward_blaster:   u16,
    pub reward_warp:      u16,
}

#[derive(Clone, Debug)]
pub struct LevelObjective {
    pub score_target:       Option<u32>,
    pub gem_quota:          Vec<(u8, u32)>,
    pub clear_all_specials: bool,
}

pub const TRACK_COUNT: usize = 3;
pub const TRACK_NAMES: &[&str] = &["Beginner", "Adventurer", "Master"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_track_has_15_levels() {
        assert_eq!(levels_for_track(0).len(), 15);
    }

    #[test]
    fn medium_track_has_30_levels() {
        assert_eq!(levels_for_track(1).len(), 30);
    }

    #[test]
    fn long_track_has_50_levels() {
        assert_eq!(levels_for_track(2).len(), 50);
    }

    #[test]
    fn unknown_track_returns_empty() {
        assert_eq!(levels_for_track(99).len(), 0);
    }

    #[test]
    fn level_0_short_basic_objective() {
        let level = &levels_for_track(0)[0];
        assert!(level.move_limit > 0);
        assert!(level.board_height <= 6);
        assert!(level.board_width <= 6);
    }

    #[test]
    fn later_levels_harder_than_early() {
        let short = levels_for_track(0);
        let first = &short[0];
        let last  = &short[14];
        let first_score = first.objective.score_target.unwrap_or(0);
        let last_score  = last.objective.score_target.unwrap_or(0);
        assert!(last_score >= first_score || last.board_height > first.board_height);
    }

    #[test]
    fn track_names_count_matches_tracks() {
        assert_eq!(TRACK_NAMES.len(), TRACK_COUNT);
    }
}
