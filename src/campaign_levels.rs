/// A single campaign level definition.
pub struct CampaignLevel {
    pub board_height: u16,
    pub board_width: u16,
    pub color_number: u16,
    pub obstacle_percentage: u16,
    pub generator_percentage: u16,
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
    pub ad_limit: u16,
    pub reward_scissors: u16,
    pub reward_tweezers: u16,
    pub reward_balloons: u16,
}

/// Short campaign: 15 levels, aggressive difficulty ramp.
pub const SHORT_CAMPAIGN: &[CampaignLevel] = &[
    // 1: Tutorial — tiny board, few colors
    CampaignLevel { board_height: 3, board_width: 3, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 2
    CampaignLevel { board_height: 3, board_width: 4, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 3: First obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 5, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 4
    CampaignLevel { board_height: 4, board_width: 5, color_number: 4, obstacle_percentage: 10, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 5: First generators
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 5, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 6
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 7
    CampaignLevel { board_height: 5, board_width: 6, color_number: 5, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 8: Mid-campaign spike
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 1 },
    // 9
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 10
    CampaignLevel { board_height: 6, board_width: 7, color_number: 7, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 11
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 12
    CampaignLevel { board_height: 7, board_width: 8, color_number: 7, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 13: Endgame
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 1 },
    // 14
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 15: Final
    CampaignLevel { board_height: 9, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 1, tweezers: 1, balloons: 1, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Medium campaign: 25 levels, moderate ramp with plateau zones.
pub const MEDIUM_CAMPAIGN: &[CampaignLevel] = &[
    // 1-3: Warm-up
    CampaignLevel { board_height: 3, board_width: 3, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 3, board_width: 4, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 4-6: Introduce obstacles
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 5, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 4, board_width: 5, color_number: 4, obstacle_percentage: 10, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 5, color_number: 4, obstacle_percentage: 10, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 7-10: Introduce generators
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 5, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 6, color_number: 5, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 6, color_number: 5, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 11-14: Growing complexity
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 7, color_number: 6, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 15-18: Higher colors
    CampaignLevel { board_height: 6, board_width: 7, color_number: 7, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 19-22: Endgame ramp
    CampaignLevel { board_height: 7, board_width: 8, color_number: 8, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 1 },
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 8, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 23-25: Final stretch
    CampaignLevel { board_height: 9, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 9, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 1 },
    CampaignLevel { board_height: 10, board_width: 10, color_number: 8, obstacle_percentage: 20, generator_percentage: 20, scissors: 1, tweezers: 1, balloons: 1, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Long campaign: 40 levels, gentle ramp with breathing room.
pub const LONG_CAMPAIGN: &[CampaignLevel] = &[
    // 1-5: Extended warm-up
    CampaignLevel { board_height: 3, board_width: 3, color_number: 2, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 3, board_width: 3, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 3, board_width: 4, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 4, board_width: 4, color_number: 3, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 0, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 6-10: Introduce obstacles slowly
    CampaignLevel { board_height: 4, board_width: 4, color_number: 4, obstacle_percentage: 5, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 4, board_width: 5, color_number: 4, obstacle_percentage: 5, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 4, board_width: 5, color_number: 4, obstacle_percentage: 10, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 5, color_number: 4, obstacle_percentage: 10, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 3, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, generator_percentage: 0, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    // 11-16: Introduce generators
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 5, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 5, color_number: 5, obstacle_percentage: 10, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 6, color_number: 5, obstacle_percentage: 10, generator_percentage: 5, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 6, color_number: 5, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 5, board_width: 6, color_number: 5, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 5, board_width: 6, color_number: 6, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    // 17-22: Growing complexity
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 6, board_width: 6, color_number: 6, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 2, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 7, color_number: 6, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 7, color_number: 6, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 6, board_width: 7, color_number: 7, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    // 23-28: Higher colors plateau
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 10, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 7, board_width: 7, color_number: 7, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 7, board_width: 8, color_number: 7, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 7, board_width: 8, color_number: 8, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 29-34: Endgame ramp
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 15, generator_percentage: 10, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 15, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 8, board_width: 8, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 1, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    CampaignLevel { board_height: 8, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 8, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 9, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 15, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 0 },
    // 35-38: Final gauntlet
    CampaignLevel { board_height: 9, board_width: 9, color_number: 8, obstacle_percentage: 20, generator_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 1 },
    CampaignLevel { board_height: 9, board_width: 10, color_number: 8, obstacle_percentage: 20, generator_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 0, reward_tweezers: 1, reward_balloons: 0 },
    CampaignLevel { board_height: 10, board_width: 10, color_number: 8, obstacle_percentage: 20, generator_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 1 },
    CampaignLevel { board_height: 10, board_width: 10, color_number: 8, obstacle_percentage: 25, generator_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 0, reward_balloons: 0 },
    // 39-40: Victory lap
    CampaignLevel { board_height: 10, board_width: 10, color_number: 8, obstacle_percentage: 25, generator_percentage: 20, scissors: 0, tweezers: 0, balloons: 0, ad_limit: 0, reward_scissors: 1, reward_tweezers: 1, reward_balloons: 1 },
    CampaignLevel { board_height: 10, board_width: 10, color_number: 8, obstacle_percentage: 25, generator_percentage: 25, scissors: 1, tweezers: 1, balloons: 1, ad_limit: 0, reward_scissors: 0, reward_tweezers: 0, reward_balloons: 0 },
];

/// Get the levels slice for a given track index (0=Short, 1=Medium, 2=Long).
pub fn levels_for_track(track_idx: usize) -> &'static [CampaignLevel] {
    match track_idx {
        0 => SHORT_CAMPAIGN,
        1 => MEDIUM_CAMPAIGN,
        _ => LONG_CAMPAIGN,
    }
}

pub const TRACK_NAMES: &[&str] = &["Short", "Medium", "Long"];
pub const TRACK_COUNT: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_campaign_has_15_levels() {
        assert_eq!(SHORT_CAMPAIGN.len(), 15);
    }

    #[test]
    fn medium_campaign_has_25_levels() {
        assert_eq!(MEDIUM_CAMPAIGN.len(), 25);
    }

    #[test]
    fn long_campaign_has_40_levels() {
        assert_eq!(LONG_CAMPAIGN.len(), 40);
    }

    #[test]
    fn all_levels_have_valid_board_sizes() {
        for levels in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN] {
            for (i, level) in levels.iter().enumerate() {
                assert!(level.board_height >= 2, "level {}: board_height too small", i);
                assert!(level.board_width >= 2, "level {}: board_width too small", i);
                assert!(level.color_number >= 2, "level {}: color_number too small", i);
            }
        }
    }

    #[test]
    fn levels_for_track_returns_correct_slice() {
        assert_eq!(levels_for_track(0).len(), 15);
        assert_eq!(levels_for_track(1).len(), 25);
        assert_eq!(levels_for_track(2).len(), 40);
    }
}
