use crate::order::OrderDef;

pub const TRACK_COUNT: usize = 3;
pub const TRACK_NAMES: &[&str] = &["Easy", "Medium", "Hard"];

pub struct LevelDef {
    pub board_height: u16,
    pub board_width: u16,
    pub color_count: u16,
    pub generator_count: u16,
    pub generator_charges: u16, // 0 = infinite
    pub blocked_cells: u16,
    pub generator_interval: u32,
    pub orders: Vec<Vec<OrderDef>>,
    pub ad_limit: u16,
}

/// Get level definitions for a campaign track.
pub fn levels_for_track(track_idx: usize) -> Vec<LevelDef> {
    match track_idx {
        0 => easy_levels(),
        1 => medium_levels(),
        2 => hard_levels(),
        _ => easy_levels(),
    }
}

fn easy_levels() -> Vec<LevelDef> {
    vec![
        // Level 1: tiny board, 1 color, simple order
        LevelDef {
            board_height: 3, board_width: 3, color_count: 1,
            generator_count: 1, generator_charges: 0, blocked_cells: 0,
            generator_interval: 6,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 2, quantity: 1 }]],
            ad_limit: 5,
        },
        // Level 2
        LevelDef {
            board_height: 3, board_width: 3, color_count: 1,
            generator_count: 1, generator_charges: 0, blocked_cells: 0,
            generator_interval: 6,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 2, quantity: 2 }]],
            ad_limit: 5,
        },
        // Level 3: introduce tier 3
        LevelDef {
            board_height: 3, board_width: 4, color_count: 1,
            generator_count: 1, generator_charges: 0, blocked_cells: 0,
            generator_interval: 6,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 3, quantity: 1 }]],
            ad_limit: 5,
        },
        // Level 4: 2 colors
        LevelDef {
            board_height: 3, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 6,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 2, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 2, quantity: 1 }],
            ],
            ad_limit: 5,
        },
        // Level 5
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 7,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 3, quantity: 1 },
                OrderDef { color_idx: 1, tier: 2, quantity: 2 },
            ]],
            ad_limit: 5,
        },
        // Level 6
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 7,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 3, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 3, quantity: 1 }],
            ],
            ad_limit: 4,
        },
        // Level 7
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 15, blocked_cells: 0,
            generator_interval: 7,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 3, quantity: 2 },
            ]],
            ad_limit: 4,
        },
        // Level 8
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 8,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 2, quantity: 2 }],
                vec![OrderDef { color_idx: 1, tier: 3, quantity: 1 }],
            ],
            ad_limit: 4,
        },
        // Level 9
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 20, blocked_cells: 0,
            generator_interval: 8,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 3, quantity: 1 },
                OrderDef { color_idx: 1, tier: 3, quantity: 1 },
            ]],
            ad_limit: 3,
        },
        // Level 10
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 8,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 3, quantity: 2 }],
                vec![OrderDef { color_idx: 1, tier: 2, quantity: 3 }],
            ],
            ad_limit: 3,
        },
        // Level 11
        LevelDef {
            board_height: 4, board_width: 5, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 8,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 3, quantity: 2 },
                OrderDef { color_idx: 1, tier: 3, quantity: 1 },
            ]],
            ad_limit: 3,
        },
        // Level 12: finale
        LevelDef {
            board_height: 4, board_width: 5, color_count: 2,
            generator_count: 2, generator_charges: 0, blocked_cells: 0,
            generator_interval: 8,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 3, quantity: 2 }],
                vec![OrderDef { color_idx: 1, tier: 3, quantity: 2 }],
            ],
            ad_limit: 3,
        },
    ]
}

fn medium_levels() -> Vec<LevelDef> {
    vec![
        // Level 1
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 15, blocked_cells: 0,
            generator_interval: 7,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 3, quantity: 1 }]],
            ad_limit: 3,
        },
        // Level 2
        LevelDef {
            board_height: 4, board_width: 4, color_count: 2,
            generator_count: 2, generator_charges: 15, blocked_cells: 0,
            generator_interval: 7,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 3, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 2, quantity: 2 }],
            ],
            ad_limit: 3,
        },
        // Level 3: introduce 3 colors
        LevelDef {
            board_height: 4, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 0,
            generator_interval: 7,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 3, quantity: 1 },
                OrderDef { color_idx: 1, tier: 3, quantity: 1 },
            ]],
            ad_limit: 3,
        },
        // Level 4: introduce blocked cells
        LevelDef {
            board_height: 4, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 1,
            generator_interval: 7,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 3, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 3, quantity: 1 }],
            ],
            ad_limit: 3,
        },
        // Level 5: tier 4 introduction
        LevelDef {
            board_height: 5, board_width: 5, color_count: 2,
            generator_count: 2, generator_charges: 20, blocked_cells: 1,
            generator_interval: 8,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }]],
            ad_limit: 3,
        },
        // Level 6
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 15, blocked_cells: 1,
            generator_interval: 8,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 3, quantity: 2 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 1 }],
            ],
            ad_limit: 3,
        },
        // Level 7
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 2,
            generator_interval: 8,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 4, quantity: 1 },
                OrderDef { color_idx: 1, tier: 3, quantity: 2 },
            ]],
            ad_limit: 2,
        },
        // Level 8
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 15, blocked_cells: 2,
            generator_interval: 8,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 3, quantity: 2 }],
            ],
            ad_limit: 2,
        },
        // Level 9
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 2,
            generator_interval: 8,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 4, quantity: 1 },
                OrderDef { color_idx: 1, tier: 4, quantity: 1 },
            ]],
            ad_limit: 2,
        },
        // Level 10
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 10, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 3, quantity: 3 }],
                vec![OrderDef { color_idx: 2, tier: 3, quantity: 2 }],
            ],
            ad_limit: 2,
        },
        // Level 11
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 10, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }, OrderDef { color_idx: 1, tier: 4, quantity: 1 }],
            ],
            ad_limit: 2,
        },
        // Level 12
        LevelDef {
            board_height: 5, board_width: 6, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 3, quantity: 2 }],
            ],
            ad_limit: 2,
        },
        // Level 13
        LevelDef {
            board_height: 5, board_width: 6, color_count: 3,
            generator_count: 3, generator_charges: 10, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 4, quantity: 2 },
                OrderDef { color_idx: 1, tier: 3, quantity: 3 },
            ]],
            ad_limit: 1,
        },
        // Level 14
        LevelDef {
            board_height: 5, board_width: 6, color_count: 3,
            generator_count: 3, generator_charges: 10, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 4, quantity: 1 }],
            ],
            ad_limit: 1,
        },
        // Level 15: finale
        LevelDef {
            board_height: 5, board_width: 6, color_count: 3,
            generator_count: 3, generator_charges: 8, blocked_cells: 2,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 2 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 1 }, OrderDef { color_idx: 2, tier: 4, quantity: 1 }],
            ],
            ad_limit: 1,
        },
    ]
}

fn hard_levels() -> Vec<LevelDef> {
    vec![
        // Level 1
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 2,
            generator_interval: 8,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }]],
            ad_limit: 1,
        },
        // Level 2
        LevelDef {
            board_height: 5, board_width: 5, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 2,
            generator_interval: 8,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 4, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 1 }],
            ],
            ad_limit: 1,
        },
        // Level 3: 4 colors
        LevelDef {
            board_height: 5, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 10, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 4, quantity: 1 },
                OrderDef { color_idx: 1, tier: 3, quantity: 2 },
            ]],
            ad_limit: 1,
        },
        // Level 4: tier 5 introduction
        LevelDef {
            board_height: 5, board_width: 6, color_count: 3,
            generator_count: 3, generator_charges: 15, blocked_cells: 2,
            generator_interval: 9,
            orders: vec![vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }]],
            ad_limit: 1,
        },
        // Level 5
        LevelDef {
            board_height: 6, board_width: 6, color_count: 3,
            generator_count: 3, generator_charges: 12, blocked_cells: 3,
            generator_interval: 9,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 2 }],
            ],
            ad_limit: 1,
        },
        // Level 6
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 10, blocked_cells: 3,
            generator_interval: 9,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 4, quantity: 2 },
                OrderDef { color_idx: 1, tier: 4, quantity: 1 },
                OrderDef { color_idx: 2, tier: 3, quantity: 3 },
            ]],
            ad_limit: 1,
        },
        // Level 7
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 10, blocked_cells: 3,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 4, quantity: 2 }],
            ],
            ad_limit: 1,
        },
        // Level 8
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 8, blocked_cells: 3,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 5, quantity: 1 }],
            ],
            ad_limit: 1,
        },
        // Level 9
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 8, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 5, quantity: 1 },
                OrderDef { color_idx: 1, tier: 4, quantity: 2 },
            ]],
            ad_limit: 1,
        },
        // Level 10
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 8, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 4, quantity: 2 }],
            ],
            ad_limit: 1,
        },
        // Level 11
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 6, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 5, quantity: 1 },
                OrderDef { color_idx: 1, tier: 5, quantity: 1 },
            ]],
            ad_limit: 0,
        },
        // Level 12
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 6, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 4, quantity: 3 }],
            ],
            ad_limit: 0,
        },
        // Level 13
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 6, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![vec![
                OrderDef { color_idx: 0, tier: 5, quantity: 2 },
            ]],
            ad_limit: 0,
        },
        // Level 14
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 5, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }, OrderDef { color_idx: 1, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 5, quantity: 1 }],
            ],
            ad_limit: 0,
        },
        // Level 15: grand finale
        LevelDef {
            board_height: 6, board_width: 6, color_count: 4,
            generator_count: 4, generator_charges: 5, blocked_cells: 4,
            generator_interval: 10,
            orders: vec![
                vec![OrderDef { color_idx: 0, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 1, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 2, tier: 5, quantity: 1 }],
                vec![OrderDef { color_idx: 3, tier: 4, quantity: 2 }],
            ],
            ad_limit: 0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_count_matches_names() {
        assert_eq!(TRACK_COUNT, TRACK_NAMES.len());
    }

    #[test]
    fn all_tracks_have_levels() {
        for track in 0..TRACK_COUNT {
            let levels = levels_for_track(track);
            assert!(!levels.is_empty(), "track {} has no levels", track);
        }
    }

    #[test]
    fn easy_has_12_levels() {
        assert_eq!(levels_for_track(0).len(), 12);
    }

    #[test]
    fn medium_has_15_levels() {
        assert_eq!(levels_for_track(1).len(), 15);
    }

    #[test]
    fn hard_has_15_levels() {
        assert_eq!(levels_for_track(2).len(), 15);
    }

    #[test]
    fn all_levels_have_orders() {
        for track in 0..TRACK_COUNT {
            for (i, level) in levels_for_track(track).iter().enumerate() {
                assert!(!level.orders.is_empty(),
                    "track {} level {} has no orders", track, i);
            }
        }
    }

    #[test]
    fn total_levels_across_tracks() {
        let total: usize = (0..TRACK_COUNT).map(|t| levels_for_track(t).len()).sum();
        assert_eq!(total, 42);
    }
}
