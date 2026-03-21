/// Return the level list for a campaign track (0=Forest, 1=Ocean, 2=Fire).
pub fn levels_for_track(track_idx: usize) -> Vec<LevelDef> {
    match track_idx {
        0 => forest_track(),
        1 => ocean_track(),
        2 => fire_track(),
        _ => vec![],
    }
}

// ── Forest track — green palette, gentle introduction (15 levels) ─────────
// Early: small board, few colors, generous moves, no ice.
// Mid:   introduce ice tiles, tighter moves.
// Late:  max colors, orders to fill, clear-all-specials challenge.

fn forest_track() -> Vec<LevelDef> {
    vec![
        // 0 – Mossy Clearing
        LevelDef { name: "Mossy Clearing",      board_height: 5, board_width: 5, color_number: 3, move_limit: 40, special_tile_pct:  0, ice_tile_pct:  0, objective: obj_score(800),             reward_hammer: 0, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 1 – Fern Path
        LevelDef { name: "Fern Path",           board_height: 5, board_width: 5, color_number: 3, move_limit: 38, special_tile_pct:  2, ice_tile_pct:  0, objective: obj_score(1200),            reward_hammer: 0, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 2 – Sunlit Grove
        LevelDef { name: "Sunlit Grove",        board_height: 5, board_width: 6, color_number: 4, move_limit: 36, special_tile_pct:  4, ice_tile_pct:  0, objective: obj_score(1600),            reward_hammer: 1, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 3 – Twisted Roots
        LevelDef { name: "Twisted Roots",       board_height: 6, board_width: 6, color_number: 4, move_limit: 34, special_tile_pct:  6, ice_tile_pct:  0, objective: obj_score(2200),            reward_hammer: 0, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 4 – Frozen Hollow (first ice)
        LevelDef { name: "Frozen Hollow",       board_height: 6, board_width: 6, color_number: 4, move_limit: 32, special_tile_pct:  6, ice_tile_pct: 10, objective: obj_score(2800),            reward_hammer: 1, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 5 – Ancient Canopy
        LevelDef { name: "Ancient Canopy",      board_height: 6, board_width: 7, color_number: 5, move_limit: 30, special_tile_pct:  8, ice_tile_pct: 10, objective: obj_score(3400),            reward_hammer: 0, reward_laser: 1, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 6 – Mushroom Ring
        LevelDef { name: "Mushroom Ring",       board_height: 6, board_width: 7, color_number: 5, move_limit: 28, special_tile_pct: 10, ice_tile_pct: 12, objective: obj_score(4000),            reward_hammer: 0, reward_laser: 0, reward_blaster: 1, reward_warp: 0, reward_color_bomb: 0 },
        // 7 – Dew Drop Dell
        LevelDef { name: "Dew Drop Dell",       board_height: 7, board_width: 7, color_number: 5, move_limit: 26, special_tile_pct: 10, ice_tile_pct: 13, objective: obj_score(5000),            reward_hammer: 1, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 8 – Briar Maze
        LevelDef { name: "Briar Maze",          board_height: 7, board_width: 7, color_number: 5, move_limit: 25, special_tile_pct: 12, ice_tile_pct: 13, objective: obj_score(6000),            reward_hammer: 0, reward_laser: 0, reward_blaster: 0, reward_warp: 1, reward_color_bomb: 0 },
        // 9 – Willow Weep
        LevelDef { name: "Willow Weep",         board_height: 7, board_width: 8, color_number: 5, move_limit: 24, special_tile_pct: 12, ice_tile_pct: 14, objective: obj_score(7000),            reward_hammer: 0, reward_laser: 1, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 10 – Thorned Thicket (orders begin)
        LevelDef { name: "Thorned Thicket",     board_height: 7, board_width: 8, color_number: 6, move_limit: 23, special_tile_pct: 14, ice_tile_pct: 14, objective: obj_specials(8000),         reward_hammer: 0, reward_laser: 0, reward_blaster: 1, reward_warp: 0, reward_color_bomb: 0 },
        // 11 – Midnight Moss
        LevelDef { name: "Midnight Moss",       board_height: 7, board_width: 8, color_number: 6, move_limit: 22, special_tile_pct: 14, ice_tile_pct: 15, objective: obj_specials(9500),         reward_hammer: 1, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 12 – Gnarled Oak
        LevelDef { name: "Gnarled Oak",         board_height: 8, board_width: 8, color_number: 6, move_limit: 21, special_tile_pct: 16, ice_tile_pct: 15, objective: obj_specials(11000),        reward_hammer: 0, reward_laser: 0, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 1 },
        // 13 – Elder Grove
        LevelDef { name: "Elder Grove",         board_height: 8, board_width: 8, color_number: 6, move_limit: 20, special_tile_pct: 16, ice_tile_pct: 15, objective: obj_specials(13000),        reward_hammer: 1, reward_laser: 1, reward_blaster: 0, reward_warp: 0, reward_color_bomb: 0 },
        // 14 – Heart of the Forest
        LevelDef { name: "Heart of the Forest", board_height: 8, board_width: 8, color_number: 6, move_limit: 18, special_tile_pct: 18, ice_tile_pct: 15, objective: obj_specials(16000),        reward_hammer: 0, reward_laser: 0, reward_blaster: 0, reward_warp: 1, reward_color_bomb: 1 },
    ]
}

// ── Ocean track — blue palette, flowing difficulty (30 levels) ────────────
// Early: calm shallows — 5×5→6×6, 3→4 colors, no ice.
// Mid:   open water — 6×6→7×8, 5 colors, ice 10–15%.
// Late:  deep abyss — 8×8→9×9, 6–7 colors, ice 15–20%, specials.

fn ocean_track() -> Vec<LevelDef> {
    let names: &[&str] = &[
        "Sandy Shallows",     "Tide Pool",          "Kelp Forest",        "Coral Reef",
        "Drifting Currents",  "Sunken Galleon",      "Bioluminescent Bay", "Crab Grotto",
        "Whale Song",         "Twilight Zone",        "Abyss Edge",        "Angler's Lair",
        "Thermal Vent",       "Pressure Ridge",       "The Deep Blue",     "Leviathan Pass",
        "Crystal Cavern",     "Phosphor Sea",         "Serpent Trench",    "Abyssal Plain",
        "Ice Shelf",          "Frozen Floes",         "Glacier Run",       "Arctic Drift",
        "Polar Night",        "Midnight Ocean",       "Crushing Depths",   "Hadal Zone",
        "The Marianas",       "Ocean Heart",
    ];
    (0..30usize).map(|i| {
        let d = i as u32;
        // Board grows from 5×5 → 9×9 across 30 levels
        let bh = (5 + i * 4 / 29).min(9) as u16;
        let bw = bh;
        // Colors: 3 early → 7 late
        let colors = (3 + i * 4 / 29).min(7) as u8;
        // Move limit: generous early (40) → tight late (18)
        let ml = (40u32).saturating_sub(d * 22 / 29);
        // Special pct: 0 early → 18 late
        let spc = ((i * 18) / 29) as u16;
        // Ice: none first 4 levels → 10–20% mid/late
        let ice: u16 = if i < 4 { 0 } else if i < 15 { 10 + ((i - 4) as u16) / 2 } else { 15 + ((i - 15) as u16 / 5).min(5) };
        let obj = if i < 10 {
            obj_score(1500 + d * 600)
        } else if i < 20 {
            obj_score(8000 + d * 800)
        } else {
            obj_specials(18000 + d * 1000)
        };
        LevelDef {
            name: names[i],
            board_height: bh, board_width: bw, color_number: colors,
            move_limit: ml.max(18), special_tile_pct: spc, ice_tile_pct: ice,
            objective: obj,
            reward_hammer:     if i % 4 == 3 { 1 } else { 0 },
            reward_laser:      if i % 5 == 4 { 1 } else { 0 },
            reward_blaster:    if i % 6 == 5 { 1 } else { 0 },
            reward_warp:       if i % 8 == 7 { 1 } else { 0 },
            reward_color_bomb: if i % 7 == 6 { 1 } else { 0 },
        }
    }).collect()
}

// ── Fire track — red/orange palette, punishing endgame (50 levels) ────────
// Early: ember sparks — 5×5, 3 colors, high move limit.
// Mid:   wildfire — 7×7→8×8, 5–6 colors, ice 12–18%.
// Late:  inferno — 9×9, 7 colors, ice 18–20%, orders, brutal move limits.

fn fire_track() -> Vec<LevelDef> {
    let names: &[&str] = &[
        "Ember Spark",        "Smoldering Log",      "Kindling Trail",     "First Flame",
        "Brushfire",          "Crackling Hearth",     "Rising Heat",        "Char Pit",
        "Scorched Earth",     "Lava Flow",            "Magma Shelf",        "Fire Geyser",
        "Cinder Storm",       "Phoenix Nest",         "Burning Canopy",     "Dragon's Breath",
        "Molten River",       "Slag Heap",            "Infernal Gate",      "Ember Crown",
        "Soot Plains",        "Charcoal Maze",        "Blaze Corridor",     "Pyroclast",
        "Volcanic Arc",       "Hot Springs",          "Thermal Column",     "Ignition Point",
        "Fire Wall",          "Magma Core",           "Lava Tubes",         "Caldera Edge",
        "Eruption Eve",       "Brimstone Path",       "Fumarole Fields",    "Ashfall",
        "Cinder Cage",        "Smoke and Mirrors",    "Hellgate Ridge",     "Obsidian Wastes",
        "Crimson Flood",      "Iron Furnace",         "Forge of Ages",      "Ember Rain",
        "The Crucible",       "Eye of the Volcano",   "Final Eruption",     "Pyre Summit",
        "Eternal Flame",      "Heart of Fire",
    ];
    (0..50usize).map(|i| {
        let d = i as u32;
        // Board grows 5×5 → 9×9 over 50 levels
        let bh = (5 + i * 4 / 49).min(9) as u16;
        let bw = bh;
        // Colors: 3 → 7
        let colors = (3 + i * 4 / 49).min(7) as u8;
        // Move limit: 40 → 15 (fire is punishing)
        let ml = (40u32).saturating_sub(d * 25 / 49);
        // Special pct: 0 → 20
        let spc = ((i * 20) / 49) as u16;
        // Ice: none first 4 → 12–20% from mid onwards
        let ice: u16 = if i < 4 { 0 } else if i < 20 { 12 + ((i - 4) as u16) / 4 } else { 18 + ((i - 20) as u16 / 15).min(2) };
        let obj = if i < 12 {
            obj_score(2000 + d * 700)
        } else if i < 28 {
            obj_score(10000 + d * 900)
        } else {
            obj_specials(30000 + d * 1200)
        };
        LevelDef {
            name: names[i],
            board_height: bh, board_width: bw, color_number: colors,
            move_limit: ml.max(15), special_tile_pct: spc, ice_tile_pct: ice,
            objective: obj,
            reward_hammer:     if i % 3 == 2 { 1 } else { 0 },
            reward_laser:      if i % 4 == 3 { 1 } else { 0 },
            reward_blaster:    if i % 5 == 4 { 1 } else { 0 },
            reward_warp:       if i % 7 == 6 { 1 } else { 0 },
            reward_color_bomb: if i % 6 == 5 { 1 } else { 0 },
        }
    }).collect()
}

// ── Objective helpers ─────────────────────────────────────────────────────

fn obj_score(target: u32) -> LevelObjective {
    LevelObjective { score_target: Some(target), gem_quota: vec![], clear_all_specials: false }
}

fn obj_specials(score_floor: u32) -> LevelObjective {
    LevelObjective { score_target: Some(score_floor), gem_quota: vec![], clear_all_specials: true }
}

#[derive(Clone, Debug)]
pub struct LevelDef {
    /// Human-readable level name shown in the UI.
    pub name:              &'static str,
    pub board_height:      u16,
    pub board_width:       u16,
    pub color_number:      u8,
    pub move_limit:        u32,
    pub special_tile_pct:  u16,
    pub objective:         LevelObjective,
    pub reward_hammer:     u16,
    pub reward_laser:      u16,
    pub reward_blaster:    u16,
    pub reward_warp:       u16,
    pub reward_color_bomb: u16,
    /// Percentage of Ice tile modifiers to place on board (0 = none).
    pub ice_tile_pct:      u16,
}

#[derive(Clone, Debug)]
pub struct LevelObjective {
    pub score_target:       Option<u32>,
    pub gem_quota:          Vec<(u8, u32)>,
    pub clear_all_specials: bool,
}

pub const TRACK_COUNT: usize = 3;
pub const TRACK_NAMES: &[&str] = &["Forest", "Ocean", "Fire"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forest_track_has_15_levels() {
        assert_eq!(levels_for_track(0).len(), 15);
    }

    #[test]
    fn ocean_track_has_30_levels() {
        assert_eq!(levels_for_track(1).len(), 30);
    }

    #[test]
    fn fire_track_has_50_levels() {
        assert_eq!(levels_for_track(2).len(), 50);
    }

    #[test]
    fn unknown_track_returns_empty() {
        assert_eq!(levels_for_track(99).len(), 0);
    }

    #[test]
    fn level_0_forest_basic_objective() {
        let level = &levels_for_track(0)[0];
        assert!(level.move_limit > 0);
        assert!(level.board_height <= 6);
        assert!(level.board_width <= 6);
        assert_eq!(level.ice_tile_pct, 0);
        assert_eq!(level.name, "Mossy Clearing");
    }

    #[test]
    fn early_forest_levels_have_no_ice() {
        let levels = levels_for_track(0);
        for level in &levels[..4] {
            assert_eq!(level.ice_tile_pct, 0, "Level '{}' should have no ice", level.name);
        }
    }

    #[test]
    fn mid_forest_levels_have_ice() {
        let levels = levels_for_track(0);
        // Level 4 (index 4) is "Frozen Hollow" with ice
        assert!(levels[4].ice_tile_pct >= 10, "Mid level should have ice >= 10%");
    }

    #[test]
    fn late_forest_levels_have_orders() {
        let levels = levels_for_track(0);
        // Levels 10+ use clear_all_specials
        for level in &levels[10..] {
            assert!(level.objective.clear_all_specials,
                "Late level '{}' should require clearing specials", level.name);
        }
    }

    #[test]
    fn later_levels_harder_than_early() {
        let levels = levels_for_track(0);
        let first = &levels[0];
        let last  = &levels[14];
        let first_score = first.objective.score_target.unwrap_or(0);
        let last_score  = last.objective.score_target.unwrap_or(0);
        assert!(last_score >= first_score || last.board_height > first.board_height);
    }

    #[test]
    fn all_tracks_have_at_least_8_levels() {
        for track in 0..TRACK_COUNT {
            assert!(levels_for_track(track).len() >= 8,
                "Track {} should have at least 8 levels", TRACK_NAMES[track]);
        }
    }

    #[test]
    fn track_names_count_matches_tracks() {
        assert_eq!(TRACK_NAMES.len(), TRACK_COUNT);
    }

    #[test]
    fn all_levels_have_names() {
        for track in 0..TRACK_COUNT {
            for level in levels_for_track(track) {
                assert!(!level.name.is_empty(),
                    "Track {} has a level with an empty name", TRACK_NAMES[track]);
            }
        }
    }

    #[test]
    fn track_names_are_themed() {
        assert_eq!(TRACK_NAMES[0], "Forest");
        assert_eq!(TRACK_NAMES[1], "Ocean");
        assert_eq!(TRACK_NAMES[2], "Fire");
    }
}
