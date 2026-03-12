use crate::board::{BoardLayout, CellInit};
use crate::item::Family;
use crate::order::{Reward, StoryOrderDef};

pub const TRACK_COUNT: usize = 3;
pub const TRACK_NAMES: &[&str] = &["The Grove", "The Foundry", "The Sanctum"];

// ── Mission definition ────────────────────────────────────────────────────

/// A single mission within a campaign track. The board is never reset;
/// each mission adds new story orders to the existing persistent board.
pub struct MissionDef {
    pub description: &'static str,
    pub story_orders: Vec<StoryOrderDef>,
    /// New families that become visible/usable in this mission.
    pub unlock_families: Vec<Family>,
    /// Optional: "thaw N cells" bonus objective.
    pub thaw_target: Option<usize>,
    /// Energy max bonus granted on mission start.
    pub energy_max_bonus: u16,
    /// Inventory slot bonus granted on mission start.
    pub inventory_slot_bonus: u16,
}

/// The initial board layout for the first mission of a track.
/// Subsequent missions reuse the persistent board.
pub struct TrackDef {
    pub initial_layout: BoardLayout,
    pub missions: Vec<MissionDef>,
    pub energy_max: u16,
    pub energy_regen_secs: u32,
    pub generator_cost: u16,
    pub generator_cooldown: u32,
    pub random_order_count: usize,
    pub max_order_tier: u8,
    pub soft_gen_chance: u8,
    pub inventory_slots: u16,
    pub ad_limit: u16,
}

pub fn track_def(track_idx: usize) -> TrackDef {
    match track_idx {
        0 => grove_track(),
        1 => foundry_track(),
        2 => sanctum_track(),
        _ => grove_track(),
    }
}

pub fn mission_count(track_idx: usize) -> usize {
    match track_idx {
        0 => 14,
        1 => 14,
        2 => 14,
        _ => 14,
    }
}

// ── Track 1: The Grove ────────────────────────────────────────────────────
//
// Easy. Board 9×7. One Wood generator active from start.
// Stone gen locked in ice (needs Blueprint(Stone) to unlock).
// Metal gen and Cloth gen deeper under ice.
//
// Family unlock order: Wood → Stone (M6 blueprint reward) → Metal (M10) → Cloth (M13)
// Never ask for items from a family whose generator is still frozen.

fn grove_track() -> TrackDef {
    use CellInit::*;
    use Family::*;

    // 9 rows × 7 cols
    // Top-left 3×3 unfrozen (rows 0-2, cols 0-2); rest frozen.
    // Stone gen hidden at (2,6) as FrozenHardGen — unlock with Blueprint(Stone).
    // Metal gen at (5,2), Cloth gen at (7,3).
    let layout = BoardLayout {
        rows: 9,
        cols: 7,
        cells: vec![
            // row 0
            vec![HardGenerator(Wood,1), Item(Wood,1), Item(Wood,1), Frozen, FrozenItem(Stone,1), Frozen, Frozen],
            // row 1
            vec![Item(Wood,1), Item(Wood,1), Empty, Frozen, Frozen, FrozenItem(Stone,2), Frozen],
            // row 2
            vec![Item(Wood,1), Empty, Empty, Frozen, Frozen, Frozen, FrozenHardGen(Stone)],
            // row 3
            vec![Frozen, Frozen, Frozen, Frozen, Frozen, Frozen, Frozen],
            // row 4
            vec![Frozen, FrozenItem(Wood,1), Frozen, Frozen, FrozenItem(Metal,1), Frozen, Frozen],
            // row 5
            vec![Frozen, Frozen, FrozenHardGen(Metal), Frozen, Frozen, Frozen, Frozen],
            // row 6
            vec![Frozen, Frozen, Frozen, Frozen, Frozen, FrozenItem(Cloth,1), Frozen],
            // row 7
            vec![Frozen, Frozen, Frozen, FrozenHardGen(Cloth), Frozen, Frozen, Frozen],
            // row 8
            vec![Frozen, FrozenItem(Wood,2), Frozen, Frozen, Frozen, FrozenItem(Stone,2), Frozen],
        ],
    };

    TrackDef {
        initial_layout: layout,
        energy_max: 100,
        energy_regen_secs: 30,
        generator_cost: 1,
        generator_cooldown: 0,
        random_order_count: 2,
        max_order_tier: 3,
        soft_gen_chance: 25,
        inventory_slots: 4,
        ad_limit: 5,
        missions: vec![
            // M1 — warmup: just tap the generator and deliver Wood T1
            MissionDef {
                description: "Welcome to the Grove! Tap the Wood generator (Enter on it) and deliver 2 Wood twigs.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 1, 2)],
                    rewards: vec![Reward::Score(100), Reward::Energy(10)],
                }],
                unlock_families: vec![Wood],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M2 — first merge: combine two T1s into a T2
            MissionDef {
                description: "Two twigs twist into a branch. Merge your Wood T1s and deliver a T2.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 2, 1)],
                    rewards: vec![Reward::Score(200), Reward::Energy(5)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M3 — more T2s
            MissionDef {
                description: "The grove needs more timber. Deliver two Wood branches.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 2, 2)],
                    rewards: vec![Reward::Score(300), Reward::Stars(2)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M4 — first T3
            MissionDef {
                description: "A plank is needed. Merge up to Wood T3 and deliver it.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 1)],
                    rewards: vec![Reward::Score(400), Reward::Stars(3)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },
            // M5 — thawing introduction
            MissionDef {
                description: "Something glints in the ice. Thaw some cells and bring more Wood.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 2, 2), (Wood, 3, 1)],
                    rewards: vec![Reward::Score(600), Reward::Energy(15)],
                }],
                unlock_families: vec![],
                thaw_target: Some(4),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M6 — blueprint reward unlocks Stone gen. No Stone orders yet.
            MissionDef {
                description: "A Stone blueprint was found in the ice! Deliver high-quality Wood to claim it.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 2)],
                    rewards: vec![
                        Reward::Score(800),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Stone)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M7 — Stone generator now unlockable. Early Stone orders (T1 only).
            MissionDef {
                description: "Merge the Blueprint into the frozen Stone generator to awaken it. Deliver first Stone.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Stone, 1, 2)],
                    rewards: vec![Reward::Score(400), Reward::Energy(10)],
                }],
                unlock_families: vec![Stone],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M8 — Stone T2, Wood T3 together
            MissionDef {
                description: "The grove and stone work in harmony. Deliver a Wood plank and Stone pebbles.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 1), (Stone, 2, 2)],
                    rewards: vec![Reward::Score(900), Reward::Stars(4)],
                }],
                unlock_families: vec![],
                thaw_target: Some(6),
                energy_max_bonus: 5,
                inventory_slot_bonus: 1,
            },
            // M9 — higher Wood + Stone
            MissionDef {
                description: "The forest grows bolder. Bring a fine log and a stone boulder.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 4, 1), (Stone, 3, 1)],
                    rewards: vec![Reward::Score(1200), Reward::Stars(5)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M10 — deeper thaw + Metal blueprint reward
            MissionDef {
                description: "Metal veins run deep. Thaw further and claim the Metal blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 2), (Stone, 3, 1)],
                    rewards: vec![
                        Reward::Score(1000),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Metal)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: Some(10),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M11 — Metal now unlockable. First Metal orders.
            MissionDef {
                description: "The Metal generator stirs. Awaken it and forge the first ingots.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 1, 2)],
                    rewards: vec![Reward::Score(500), Reward::Energy(15)],
                }],
                unlock_families: vec![Metal],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },
            // M12 — mixed Wood+Stone+Metal
            MissionDef {
                description: "Three families in concert. The grove is coming alive.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 4, 1), (Stone, 3, 1), (Metal, 2, 1)],
                    rewards: vec![Reward::Score(1500), Reward::Stars(6)],
                }],
                unlock_families: vec![],
                thaw_target: Some(14),
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M13 — Cloth blueprint reward. No Cloth orders yet.
            MissionDef {
                description: "Silken threads buried in the frost. Bring strong offerings to claim them.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 3, 1), (Stone, 4, 1)],
                    rewards: vec![
                        Reward::Score(2000),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Cloth)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M14 — Grande finale: all four families
            MissionDef {
                description: "The Grove blooms. Complete the final offering from all four families.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Wood, 4, 1), (Stone, 4, 1)],
                        rewards: vec![Reward::Score(2500), Reward::Stars(8)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Metal, 3, 1), (Cloth, 2, 1)],
                        rewards: vec![Reward::Score(3000), Reward::Stars(12), Reward::InventorySlot],
                    },
                ],
                unlock_families: vec![Cloth],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
        ],
    }
}

// ── Track 2: The Foundry ──────────────────────────────────────────────────
//
// Medium. Board 10×8. Metal and Ember generators active from start.
// Stone gen frozen at (3,1) — Blueprint(Stone) given at M5.
// Crystal gen frozen at (5,4) — Blueprint(Crystal) given at M10.
// Generator cooldown: 3 ticks. Tighter energy (80 max).
//
// Family unlock order: Metal+Ember → Stone (M6) → Crystal (M11) → all

fn foundry_track() -> TrackDef {
    use CellInit::*;
    use Family::*;

    let layout = BoardLayout {
        rows: 10,
        cols: 8,
        cells: vec![
            // row 0: two active generators
            vec![HardGenerator(Metal,1), Item(Metal,1), Item(Ember,1), Item(Metal,1), Frozen, Frozen, Frozen, Frozen],
            // row 1
            vec![Item(Ember,1), HardGenerator(Ember,1), Item(Metal,1), Item(Ember,1), FrozenItem(Metal,2), Frozen, Frozen, Frozen],
            // row 2
            vec![Frozen, Frozen, FrozenItem(Ember,2), Frozen, Frozen, Frozen, FrozenItem(Metal,1), Frozen],
            // row 3: Stone gen frozen
            vec![Frozen, FrozenHardGen(Stone), Frozen, Frozen, Frozen, FrozenItem(Ember,2), Frozen, Frozen],
            // row 4
            vec![Frozen, Frozen, Frozen, FrozenItem(Metal,3), Frozen, Frozen, Frozen, Frozen],
            // row 5: Crystal gen frozen
            vec![Frozen, Frozen, FrozenItem(Ember,1), Frozen, FrozenHardGen(Crystal), Frozen, Frozen, Frozen],
            // row 6
            vec![Frozen, FrozenItem(Metal,2), Frozen, Frozen, Frozen, Frozen, FrozenItem(Stone,2), Frozen],
            // row 7: extra Ember gen frozen
            vec![Frozen, Frozen, Frozen, FrozenHardGen(Ember), Frozen, Frozen, Frozen, FrozenItem(Crystal,1)],
            // row 8
            vec![Frozen, Frozen, FrozenItem(Stone,1), Frozen, Frozen, Frozen, Frozen, Frozen],
            // row 9
            vec![Frozen, FrozenItem(Metal,3), Frozen, Frozen, FrozenItem(Ember,3), Frozen, Frozen, Frozen],
        ],
    };

    TrackDef {
        initial_layout: layout,
        energy_max: 80,
        energy_regen_secs: 30,
        generator_cost: 1,
        generator_cooldown: 3,
        random_order_count: 2,
        max_order_tier: 4,
        soft_gen_chance: 20,
        inventory_slots: 3,
        ad_limit: 3,
        missions: vec![
            // M1 — warmup, Metal only
            MissionDef {
                description: "The Foundry roars to life. Mind the cooldowns. Deliver first Metal.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 1, 3)],
                    rewards: vec![Reward::Score(200), Reward::Energy(15)],
                }],
                unlock_families: vec![Metal, Ember],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M2 — Ember intro
            MissionDef {
                description: "The forges need fuel. Ember feeds the flame.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 1, 2), (Metal, 1, 2)],
                    rewards: vec![Reward::Score(400), Reward::Stars(2)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M3 — first T2s
            MissionDef {
                description: "Smelt the raw ore. Merge up to T2 and deliver.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 2, 2)],
                    rewards: vec![Reward::Score(600), Reward::Energy(10)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M4 — mixed T2 + thaw
            MissionDef {
                description: "Ember and Metal together. Thaw some cells and deliver a mixed batch.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 2, 2), (Metal, 2, 1)],
                    rewards: vec![Reward::Score(800), Reward::Stars(3)],
                }],
                unlock_families: vec![],
                thaw_target: Some(5),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M5 — T3 milestone + Blueprint(Stone) reward
            MissionDef {
                description: "Deepen the forge. A Stone blueprint lies buried — find it with quality Metal.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 3, 1)],
                    rewards: vec![
                        Reward::Score(1000),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Stone)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },
            // M6 — Stone gen unlockable now; first Stone T1 order
            MissionDef {
                description: "Merge the Blueprint into the Stone generator to unlock a new material.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Stone, 1, 2)],
                    rewards: vec![Reward::Score(500), Reward::Energy(15)],
                }],
                unlock_families: vec![Stone],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M7 — Metal+Ember T3 + Stone T2
            MissionDef {
                description: "The Stone feeds the forge. A stronger alloy is demanded.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 3, 1), (Stone, 2, 1)],
                    rewards: vec![Reward::Score(1200), Reward::Stars(4)],
                }],
                unlock_families: vec![],
                thaw_target: Some(8),
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M8 — mixed three families
            MissionDef {
                description: "Three forges burning. Balance the output.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 3, 1), (Stone, 2, 2)],
                    rewards: vec![Reward::Score(1400), Reward::Stars(5)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M9 — higher tier push
            MissionDef {
                description: "Refine the finest materials. T4 ore is needed.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 4, 1)],
                    rewards: vec![Reward::Score(2000), Reward::Stars(6)],
                }],
                unlock_families: vec![],
                thaw_target: Some(12),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M10 — Crystal blueprint reward. No Crystal orders yet.
            MissionDef {
                description: "Crystal deposits shimmer deep below. Forge enough to claim the blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 4, 1), (Stone, 3, 1)],
                    rewards: vec![
                        Reward::Score(1800),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Crystal)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M11 — Crystal gen now unlockable; first Crystal orders
            MissionDef {
                description: "Awaken the Crystal generator. Its light reveals paths forward.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 1, 2)],
                    rewards: vec![Reward::Score(600), Reward::Energy(20)],
                }],
                unlock_families: vec![Crystal],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M12 — Crystal T2 + Metal T4
            MissionDef {
                description: "The crucible demands crystalline clarity alongside refined metal.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 2, 2), (Metal, 4, 1)],
                    rewards: vec![Reward::Score(2500), Reward::Stars(8)],
                }],
                unlock_families: vec![],
                thaw_target: Some(18),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M13 — high-tier multi-family
            MissionDef {
                description: "The grand alloy requires every family's finest output.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 4, 1), (Stone, 4, 1), (Crystal, 3, 1)],
                    rewards: vec![Reward::Score(4000), Reward::Stars(12), Reward::InventorySlot],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M14 — Foundry finale
            MissionDef {
                description: "The Foundry's masterwork. Forge a legacy in Metal, Crystal and Ember.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Metal, 5, 1)],
                        rewards: vec![Reward::Score(5000), Reward::Stars(15)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Crystal, 4, 1), (Ember, 5, 1)],
                        rewards: vec![Reward::Score(6000), Reward::Stars(20), Reward::InventorySlot],
                    },
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
        ],
    }
}

// ── Track 3: The Sanctum ──────────────────────────────────────────────────
//
// Hard. Board 10×8. Only Crystal generator active at start.
// Wood gen at (1,0) frozen — Blueprint(Wood) reward at M4.
// Ember gen at (2,5), Stone gen at (5,4), Metal gen at (6,7), Cloth gen at (8,1).
// Energy max 80, cooldown 5, only 1 random order.
//
// Family unlock order: Crystal → Wood (M5) → Ember (M8) → Stone (M10) → Metal+Cloth (M13)

fn sanctum_track() -> TrackDef {
    use CellInit::*;
    use Family::*;

    let layout = BoardLayout {
        rows: 10,
        cols: 8,
        cells: vec![
            // row 0: all frozen
            vec![Frozen, FrozenItem(Crystal,1), Frozen, Frozen, Frozen, FrozenItem(Wood,1), Frozen, Frozen],
            // row 1: Wood gen frozen at col 0
            vec![FrozenHardGen(Wood), Frozen, Frozen, FrozenItem(Crystal,2), Frozen, Frozen, Frozen, FrozenItem(Ember,1)],
            // row 2: Ember gen frozen at col 5
            vec![Frozen, Frozen, Frozen, Frozen, Frozen, FrozenHardGen(Ember), Frozen, Frozen],
            // row 3: 2×2 unfrozen center at cols 3-4
            vec![Frozen, Frozen, Frozen, HardGenerator(Crystal,1), Item(Crystal,1), Frozen, Frozen, Frozen],
            // row 4
            vec![Frozen, Frozen, Frozen, Item(Crystal,1), Empty, Frozen, Frozen, Frozen],
            // row 5: Stone gen frozen at col 4
            vec![Frozen, FrozenItem(Stone,1), Frozen, Frozen, FrozenHardGen(Stone), Frozen, FrozenItem(Crystal,2), Frozen],
            // row 6: Metal gen frozen at col 7
            vec![Frozen, Frozen, FrozenItem(Metal,1), Frozen, Frozen, Frozen, Frozen, FrozenHardGen(Metal)],
            // row 7
            vec![FrozenItem(Crystal,3), Frozen, Frozen, FrozenItem(Ember,2), Frozen, Frozen, Frozen, Frozen],
            // row 8: Cloth gen frozen at col 1
            vec![Frozen, FrozenHardGen(Cloth), Frozen, Frozen, Frozen, FrozenItem(Stone,2), Frozen, FrozenItem(Cloth,1)],
            // row 9
            vec![Frozen, Frozen, FrozenItem(Crystal,3), Frozen, FrozenItem(Metal,2), Frozen, Frozen, Frozen],
        ],
    };

    TrackDef {
        initial_layout: layout,
        energy_max: 80,
        energy_regen_secs: 30,
        generator_cost: 1,
        generator_cooldown: 5,
        random_order_count: 1,
        max_order_tier: 5,
        soft_gen_chance: 15,
        inventory_slots: 3,
        ad_limit: 2,
        missions: vec![
            // M1 — Crystal only, T1
            MissionDef {
                description: "The Sanctum reveals only what you earn. Crystal is all you have. Deliver two shards.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 1, 2)],
                    rewards: vec![Reward::Score(300), Reward::Energy(15)],
                }],
                unlock_families: vec![Crystal],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M2 — Crystal T2
            MissionDef {
                description: "Merge your crystals. A fragment is needed.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 2, 1)],
                    rewards: vec![Reward::Score(500), Reward::Stars(2)],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M3 — Crystal T2 × 2 + thaw
            MissionDef {
                description: "The ice begins to break. Thaw the perimeter while delivering crystals.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 2, 2)],
                    rewards: vec![Reward::Score(700), Reward::Energy(10)],
                }],
                unlock_families: vec![],
                thaw_target: Some(6),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M4 — Crystal T3 + Blueprint(Wood) reward
            MissionDef {
                description: "A Wood blueprint pulses beneath the ice. Deliver a Crystal gem to claim it.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 3, 1)],
                    rewards: vec![
                        Reward::Score(1000),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Wood)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },
            // M5 — Wood gen now unlockable; first Wood T1 orders
            MissionDef {
                description: "Wood emerges from the permafrost. Merge the Blueprint to awaken its generator.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 1, 2)],
                    rewards: vec![Reward::Score(400), Reward::Energy(10)],
                }],
                unlock_families: vec![Wood],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M6 — Crystal T3 + Wood T2
            MissionDef {
                description: "Two families unite. Crystal and Wood together reveal the Sanctum's depths.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 3, 1), (Wood, 2, 1)],
                    rewards: vec![Reward::Score(1200), Reward::Stars(4)],
                }],
                unlock_families: vec![],
                thaw_target: Some(10),
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M7 — deeper thaw + Crystal T4
            MissionDef {
                description: "Push deeper into the ice. A T4 gem is required.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 4, 1)],
                    rewards: vec![Reward::Score(2000), Reward::Stars(5)],
                }],
                unlock_families: vec![],
                thaw_target: Some(14),
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M8 — Blueprint(Ember) reward, no Ember orders yet
            MissionDef {
                description: "Ember stirs deep below. Offer Crystal and Wood to claim its blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 3, 1), (Wood, 3, 1)],
                    rewards: vec![
                        Reward::Score(1500),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Ember)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M9 — Ember gen now unlockable; first Ember T1 orders
            MissionDef {
                description: "Ember flares to life. Merge its Blueprint and kindle the first sparks.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 1, 2)],
                    rewards: vec![Reward::Score(600), Reward::Energy(15)],
                }],
                unlock_families: vec![Ember],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M10 — Blueprint(Stone), mixed Crystal+Wood+Ember
            MissionDef {
                description: "Stone waits beneath the permafrost. Three families must offer tribute.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 4, 1), (Ember, 2, 1)],
                    rewards: vec![
                        Reward::Score(2000),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Stone)),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: Some(18),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M11 — Stone gen now unlockable; first Stone T1 orders
            MissionDef {
                description: "Stone speaks. Four families are now in your hands — use them wisely.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Stone, 1, 2), (Ember, 2, 1)],
                    rewards: vec![Reward::Score(800), Reward::Stars(6)],
                }],
                unlock_families: vec![Stone],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 1,
            },
            // M12 — four families, T3-T4 range
            MissionDef {
                description: "The Sanctum's heart pulses. Four offerings of considerable power.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 5, 1), (Wood, 4, 1), (Stone, 3, 1)],
                    rewards: vec![Reward::Score(4000), Reward::Stars(10)],
                }],
                unlock_families: vec![],
                thaw_target: Some(24),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M13 — Metal+Cloth blueprints rewarded (both at once). No Metal/Cloth orders.
            MissionDef {
                description: "Metal and Cloth lie dormant in the deep ice. A worthy sacrifice unlocks both.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 5, 1), (Ember, 4, 1)],
                    rewards: vec![
                        Reward::Score(3000),
                        Reward::SpawnPiece(crate::item::Piece::Blueprint(Metal)),
                        Reward::Stars(8),
                    ],
                }],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M14 — finale: all six families
            MissionDef {
                description: "The Sanctum's heart is yours. Complete the ultimate offering from all families.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Crystal, 6, 1), (Wood, 5, 1)],
                        rewards: vec![Reward::Score(8000), Reward::Stars(20)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Ember, 5, 1), (Stone, 4, 1), (Metal, 3, 1)],
                        rewards: vec![Reward::Score(10000), Reward::Stars(30), Reward::InventorySlot],
                    },
                ],
                unlock_families: vec![Metal, Cloth],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_count_matches_names() {
        assert_eq!(TRACK_COUNT, TRACK_NAMES.len());
    }

    #[test]
    fn all_tracks_have_missions() {
        for track in 0..TRACK_COUNT {
            let def = track_def(track);
            assert!(!def.missions.is_empty(), "track {} has no missions", track);
        }
    }

    #[test]
    fn mission_counts_match_def() {
        for track in 0..TRACK_COUNT {
            let def = track_def(track);
            assert_eq!(
                def.missions.len(),
                mission_count(track),
                "track {} mission count mismatch",
                track
            );
        }
    }

    #[test]
    fn all_missions_have_orders() {
        for track in 0..TRACK_COUNT {
            let def = track_def(track);
            for (i, mission) in def.missions.iter().enumerate() {
                assert!(
                    !mission.story_orders.is_empty(),
                    "track {} mission {} has no story orders",
                    track,
                    i
                );
            }
        }
    }

    #[test]
    fn layouts_have_correct_dimensions() {
        let grove = track_def(0);
        assert_eq!(grove.initial_layout.rows, 9);
        assert_eq!(grove.initial_layout.cols, 7);

        let foundry = track_def(1);
        assert_eq!(foundry.initial_layout.rows, 10);
        assert_eq!(foundry.initial_layout.cols, 8);

        let sanctum = track_def(2);
        assert_eq!(sanctum.initial_layout.rows, 10);
        assert_eq!(sanctum.initial_layout.cols, 8);
    }
}
