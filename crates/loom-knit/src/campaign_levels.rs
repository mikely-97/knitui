/// A single campaign level definition.
pub struct CampaignLevel {
    pub board_height: u16,
    pub board_width: u16,
    pub color_number: u16,
    pub obstacle_percentage: u16,
    pub conveyor_percentage: u16,
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
    pub ad_limit: u16,
    pub reward_scissors: u16,
    pub reward_tweezers: u16,
    pub reward_balloons: u16,
}

/// Short campaign: 10 levels.
/// Theme: "First Thread" — learn the ropes on forgiving boards.
/// Early (1-4): 4x4, 2-3 colors, no conveyors.
/// Mid   (5-7): 5x5, 4-5 colors, conveyors introduced.
/// Late  (8-10): 6x6, 5-6 colors, conveyors + obstacles.
pub const SHORT_CAMPAIGN: &[CampaignLevel] = &[
    // 1: "Cast On" — 4x4, 2 colors, clean board
    CampaignLevel { board_height: 4, board_width: 4, color_number: 2, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 2: "Plain Stitch" — 4x4, 3 colors
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 3: "Dropped Stitch" — 4x4, 3 colors, first obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 5, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 4: "Tangled Yarn" — 4x4, 4 colors, more obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 10, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 5: "Conveyor Belt" — 5x5, 4 colors, first conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 4, obstacle_percentage: 5, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 6: "Warp and Weft" — 5x5, 5 colors, conveyors + obstacles
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 7: "The Loom Hums" — 5x5, 5 colors, busier conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 8: "Full Loom" — 6x6, 5 colors, conveyors enter the big board
    CampaignLevel { board_height: 6, board_width: 6, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 9: "Rainbow Rush" — 6x6, 6 colors, higher obstacle density
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 10: "The Finished Piece" — 6x6, 6 colors, final challenge with starting kit
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 15, scissors: 1, tweezers: 1, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Medium campaign: 15 levels.
/// Theme: "The Workshop" — deliberate pacing, broader color palette.
/// Early (1-4):  4x4, 2-4 colors, no conveyors.
/// Mid   (5-10): 5x5, 4-6 colors, conveyors scale up.
/// Late  (11-15): 6x6, 6-8 colors, conveyors + tighter obstacles.
pub const MEDIUM_CAMPAIGN: &[CampaignLevel] = &[
    // 1: "Open Studio" — 4x4, 2 colors
    CampaignLevel { board_height: 4, board_width: 4, color_number: 2, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 2: "Three-Ply" — 4x4, 3 colors
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 3: "First Knot" — 4x4, 3 colors, light obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 5, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 4: "Four Colors" — 4x4, 4 colors, more obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 10, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 5: "Shuttle Run" — 5x5, 4 colors, first conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 4, obstacle_percentage: 5, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 6: "Five Colors" — 5x5, 5 colors, conveyors + obstacles
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 7: "Loom Song" — 5x5, 5 colors, conveyors ramping
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 8: "Extended Warp" — 5x5, 6 colors, denser conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 9: "Crossweave" — 5x5, 6 colors, busy board
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 10: "Grand Loom" — 5x5, 6 colors, pre-final plateau
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 11: "Big Canvas" — 6x6, 6 colors, first big board
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 12: "Seven Threads" — 6x6, 7 colors
    CampaignLevel { board_height: 6, board_width: 6, color_number: 7, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 13: "Full Spectrum" — 6x6, 8 colors, conveyors tighten
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 15, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 14: "Endgame Weave" — 6x6, 8 colors, high obstacle + conveyor density
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 15, conveyor_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 15: "Masterwork" — 6x6, 8 colors, final challenge with bonus kit
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 20, conveyor_percentage: 15, scissors: 1, tweezers: 1, balloons: 1, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Long campaign: 20 levels.
/// Theme: "The Grand Tapestry" — generous pacing, full feature tour.
/// Early  (1-5):  4x4, 2-4 colors, no conveyors — breathing room.
/// Mid    (6-12): 5x5, 4-6 colors, conveyors grow steadily.
/// Late   (13-20): 6x6, 6-8 colors, conveyors + obstacles + key spools.
pub const LONG_CAMPAIGN: &[CampaignLevel] = &[
    // 1: "First Thread" — 4x4, 2 colors
    CampaignLevel { board_height: 4, board_width: 4, color_number: 2, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 2: "Two-Tone" — 4x4, 3 colors
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 3: "Light Knot" — 4x4, 3 colors, first obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 5, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 4: "Four Hues" — 4x4, 4 colors, more obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 10, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 5: "Wider Warp" — 4x4, 4 colors, consolidation
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 10, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 6: "Belt Starts" — 5x5, 4 colors, first conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 4, obstacle_percentage: 5, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 7: "Five Colors" — 5x5, 5 colors, conveyors + obstacles
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 8: "Shuttle Work" — 5x5, 5 colors, conveyors grow
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 9: "Six-Strand" — 5x5, 6 colors, denser conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 10: "Midpoint" — 5x5, 6 colors, plateau moment
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 11: "Dense Weave" — 5x5, 6 colors, tight obstacles
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 12: "Gateway" — 5x5, 6 colors, busy pre-escalation
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 13: "Grand Opening" — 6x6, 6 colors, first big board
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 14: "Seven Colors" — 6x6, 7 colors, conveyors persist
    CampaignLevel { board_height: 6, board_width: 6, color_number: 7, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 15: "Full Palette" — 6x6, 8 colors, enter max complexity
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 16: "Knot Garden" — 6x6, 8 colors, obstacles tighten
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 15, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 17: "Conveyor Storm" — 6x6, 8 colors, dense conveyors
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 15, conveyor_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 18: "High Tension" — 6x6, 8 colors, heavy obstacles + conveyors
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 20, conveyor_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 19: "Final Thread" — 6x6, 8 colors, near-max difficulty
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 20, conveyor_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 20: "The Grand Tapestry" — 6x6, 8 colors, bonus kit to finish
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 20, conveyor_percentage: 20, scissors: 1, tweezers: 1, balloons: 1, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Hard campaign: 10 levels.
/// Theme: "Unraveled" — same board sizes as Short, hard_mode=true (no bonuses,
/// no blessings, no solvability guarantee).  Set by is_hard_track(3).
pub const HARD_CAMPAIGN: &[CampaignLevel] = &[
    // 1: "Fraying Edge" — 4x4, 2 colors
    CampaignLevel { board_height: 4, board_width: 4, color_number: 2, obstacle_percentage: 0, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 2: "Snapped Thread" — 4x4, 3 colors, first obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 5, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 3: "Tangles" — 4x4, 4 colors, obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 10, conveyor_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 4: "First Conveyor" — 5x5, 4 colors, first conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 4, obstacle_percentage: 5, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 5: "No Mercy" — 5x5, 5 colors, obstacles + conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 6: "Static" — 5x5, 5 colors, denser conveyors
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 7: "Six Without Help" — 5x5, 6 colors, no lifelines
    CampaignLevel { board_height: 5, board_width: 5, color_number: 6, obstacle_percentage: 15, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 8: "Into the Deep" — 6x6, 6 colors, big board, no hints
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, conveyor_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 9: "Chaos Loom" — 6x6, 8 colors, heavy pressure
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 15, conveyor_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    // 10: "Unraveled" — 6x6, 8 colors, max difficulty, no safety net
    CampaignLevel { board_height: 6, board_width: 6, color_number: 8, obstacle_percentage: 20, conveyor_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Get the levels slice for a given track index (0=Short, 1=Medium, 2=Long, 3=Hard).
pub fn levels_for_track(track_idx: usize) -> &'static [CampaignLevel] {
    match track_idx {
        0 => SHORT_CAMPAIGN,
        1 => MEDIUM_CAMPAIGN,
        2 => LONG_CAMPAIGN,
        _ => HARD_CAMPAIGN,
    }
}

/// Returns true if the track uses hard mode (no blessings, no bonuses, no solvability guarantee).
pub fn is_hard_track(track_idx: usize) -> bool {
    track_idx == 3
}

pub const TRACK_NAMES: &[&str] = &["Short", "Medium", "Long", "Hard"];
pub const TRACK_COUNT: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_campaign_has_10_levels() {
        assert_eq!(SHORT_CAMPAIGN.len(), 10);
    }

    #[test]
    fn medium_campaign_has_15_levels() {
        assert_eq!(MEDIUM_CAMPAIGN.len(), 15);
    }

    #[test]
    fn long_campaign_has_20_levels() {
        assert_eq!(LONG_CAMPAIGN.len(), 20);
    }

    #[test]
    fn hard_campaign_has_10_levels() {
        assert_eq!(HARD_CAMPAIGN.len(), 10);
    }

    #[test]
    fn all_tracks_have_at_least_10_levels() {
        for (i, levels) in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN, HARD_CAMPAIGN].iter().enumerate() {
            assert!(levels.len() >= 10, "track {}: only {} levels", i, levels.len());
        }
    }

    #[test]
    fn all_levels_have_valid_board_sizes() {
        for levels in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN, HARD_CAMPAIGN] {
            for (i, level) in levels.iter().enumerate() {
                assert!(level.board_height >= 2, "level {}: board_height too small", i);
                assert!(level.board_width >= 2, "level {}: board_width too small", i);
                assert!(level.color_number >= 2, "level {}: color_number too small", i);
            }
        }
    }

    #[test]
    fn all_levels_respect_max_board_dim() {
        use crate::config::MAX_BOARD_DIM;
        for levels in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN, HARD_CAMPAIGN] {
            for (i, level) in levels.iter().enumerate() {
                assert!(level.board_height <= MAX_BOARD_DIM,
                    "level {}: board_height {} exceeds max {}", i, level.board_height, MAX_BOARD_DIM);
                assert!(level.board_width <= MAX_BOARD_DIM,
                    "level {}: board_width {} exceeds max {}", i, level.board_width, MAX_BOARD_DIM);
            }
        }
    }

    #[test]
    fn early_levels_small_boards() {
        // First 3 levels of each non-hard track must be 4x4 or smaller
        for levels in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN] {
            for (i, level) in levels.iter().take(3).enumerate() {
                assert!(level.board_height <= 4, "track early level {}: board_height {} > 4", i, level.board_height);
                assert!(level.board_width <= 4, "track early level {}: board_width {} > 4", i, level.board_width);
            }
        }
    }

    #[test]
    fn early_levels_no_conveyors() {
        // First 4 levels of non-hard tracks must have no conveyors.
        // Hard track may introduce conveyors at level 4 (5x5 board).
        for levels in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN] {
            for (i, level) in levels.iter().take(4).enumerate() {
                assert_eq!(level.conveyor_percentage, 0,
                    "level {}: has conveyors in early section", i);
            }
        }
        // Hard track: first 3 levels (4x4 boards) must have no conveyors
        for (i, level) in HARD_CAMPAIGN.iter().take(3).enumerate() {
            assert_eq!(level.conveyor_percentage, 0,
                "hard level {}: has conveyors before 5x5 phase", i);
        }
    }

    #[test]
    fn hard_campaign_no_rewards() {
        for (i, level) in HARD_CAMPAIGN.iter().enumerate() {
            assert_eq!(level.reward_scissors, 0, "hard level {}: has reward_scissors", i);
            assert_eq!(level.reward_tweezers, 0, "hard level {}: has reward_tweezers", i);
            assert_eq!(level.reward_balloons, 0, "hard level {}: has reward_balloons", i);
            assert_eq!(level.ad_limit, 0, "hard level {}: has ad_limit", i);
        }
    }

    #[test]
    fn levels_for_track_returns_correct_slice() {
        assert_eq!(levels_for_track(0).len(), 10);
        assert_eq!(levels_for_track(1).len(), 15);
        assert_eq!(levels_for_track(2).len(), 20);
        assert_eq!(levels_for_track(3).len(), 10);
    }

    #[test]
    fn hard_track_flag() {
        assert!(!is_hard_track(0));
        assert!(!is_hard_track(1));
        assert!(!is_hard_track(2));
        assert!(is_hard_track(3));
    }
}
