use crate::board::{BoardLayout, CellInit};
use crate::item::{Family, Piece};
use crate::order::{Order, OrderRequirement, OrderType, Reward, StoryOrderDef};

pub const TRACK_COUNT: usize = 3;
pub const TRACK_NAMES: &[&str] = &["The Grove", "The Foundry", "The Sanctum"];

// ── Mission definition ────────────────────────────────────────────────────

/// A single mission within a campaign track. The board is never reset;
/// each mission adds new story orders to the existing persistent board.
pub struct MissionDef {
    pub description: &'static str,
    pub story_orders: Vec<StoryOrderDef>,
    /// Pre-built orders (e.g. chain orders with follow_up). Appended after story_orders.
    pub extra_orders: Vec<Order>,
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

// ── Chain order helper ────────────────────────────────────────────────────

/// Build a two-stage chain order: complete the first to unlock the second.
fn chain_order(
    fam1: Family, tier1: u8, qty1: u16, rewards1: Vec<Reward>,
    fam2: Family, tier2: u8, qty2: u16, rewards2: Vec<Reward>,
) -> Order {
    let follow_up = Order {
        order_type: OrderType::Story,
        requirements: vec![OrderRequirement::new(fam2, tier2, qty2)],
        rewards: rewards2,
        follow_up: None,
        is_mission_story: false,
    };
    Order {
        order_type: OrderType::Story,
        requirements: vec![OrderRequirement::new(fam1, tier1, qty1)],
        rewards: rewards1,
        follow_up: Some(Box::new(follow_up)),
        is_mission_story: false,
    }
}

// ── Track 1: The Grove ────────────────────────────────────────────────────
//
// Easy. Board 9×7. One Wood generator active from start.
// Stone gen locked in ice — Blueprint(Stone) rewarded at M6.
// Metal gen deeper under ice — Blueprint(Metal) rewarded at M10.
// Cloth gen furthest frozen — Blueprint(Cloth) rewarded at M13.
//
// Family unlock order: Wood → Stone (M7) → Metal (M11) → Cloth (M14)
// Generator variety: HardGen (Wood) at start; SoftGen created via high-tier merges.
// Frozen cells introduced mid-game (M5); bubble cells appear as merge artefacts.

fn grove_track() -> TrackDef {
    use CellInit::*;
    use Family::*;

    // 9 rows × 7 cols
    // Top-left 3×3 unfrozen at start (rows 0-2, cols 0-2).
    // Stone gen at (2,6) as FrozenHardGen — unlock with Blueprint(Stone).
    // Metal gen at (5,2), Cloth gen at (7,3).
    let layout = BoardLayout {
        rows: 9,
        cols: 7,
        cells: vec![
            // row 0: active Wood gen + two twigs ready to merge
            vec![HardGenerator(Wood,1), Item(Wood,1), Item(Wood,1), Frozen, FrozenItem(Stone,1), Frozen, Frozen],
            // row 1: more starter material
            vec![Item(Wood,1), Item(Wood,1), Empty, Frozen, Frozen, FrozenItem(Stone,2), Frozen],
            // row 2: two empties + path toward frozen Stone gen
            vec![Item(Wood,1), Empty, Empty, Frozen, Frozen, Frozen, FrozenHardGen(Stone)],
            // row 3: solid frost wall
            vec![Frozen, Frozen, Frozen, Frozen, Frozen, Frozen, Frozen],
            // row 4: frozen goodies to thaw toward mid-game
            vec![Frozen, FrozenItem(Wood,1), Frozen, Frozen, FrozenItem(Metal,1), Frozen, Frozen],
            // row 5: Metal gen locked deep
            vec![Frozen, Frozen, FrozenHardGen(Metal), Frozen, Frozen, Frozen, Frozen],
            // row 6
            vec![Frozen, Frozen, Frozen, Frozen, Frozen, FrozenItem(Cloth,1), Frozen],
            // row 7: Cloth gen frozen
            vec![Frozen, Frozen, Frozen, FrozenHardGen(Cloth), Frozen, Frozen, Frozen],
            // row 8: late-game rewards frozen
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
            // ── Early missions: small board, one family, simple orders ─────

            // M1 — warmup: tap generator, collect tier-1 Wood
            MissionDef {
                description: "A sunlit clearing. The old Wood generator hums. Tap it twice and deliver two fresh twigs to the waiting cart.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 1, 2)],
                    rewards: vec![Reward::Score(100), Reward::Energy(10)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Wood],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M2 — first merge: two T1 → T2
            MissionDef {
                description: "Two twigs twist together into a sturdy branch. Merge your Wood T1s and hand over the result.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 2, 1)],
                    rewards: vec![Reward::Score(200), Reward::Energy(5)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M3 — more T2 practice
            MissionDef {
                description: "The sawmill needs timber. Two branches will do — keep the generator busy.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 2, 2)],
                    rewards: vec![Reward::Score(300), Reward::Stars(2)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M4 — first T3, intro to deeper merging
            MissionDef {
                description: "A proper plank is called for. Merge up through T3 and deliver it — the village bridge won't build itself.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 1)],
                    rewards: vec![Reward::Score(400), Reward::Stars(3)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },

            // ── Mid missions: frozen cells appear, second family unlocking ─

            // M5 — thawing introduction + two-requirement order
            MissionDef {
                description: "Ice creeps at the grove's edge. Merge deeply and chip away at the frost — something glints beneath.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 2, 2), (Wood, 3, 1)],
                    rewards: vec![Reward::Score(600), Reward::Energy(15)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: Some(4),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M6 — Blueprint(Stone) reward via chain order: deliver Wood T3 x2, then Wood T3 again
            MissionDef {
                description: "A Stone blueprint pulses behind the ice wall. First clear the timber order, then prove your skill once more to claim it.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 2)],
                    rewards: vec![Reward::Score(800)],
                }],
                extra_orders: vec![
                    chain_order(
                        Wood, 3, 1, vec![Reward::Score(400)],
                        Wood, 2, 1,
                        vec![
                            Reward::Score(600),
                            Reward::SpawnPiece(Piece::Blueprint(Stone)),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M7 — Stone gen awakens; first Stone T1 orders
            MissionDef {
                description: "The frozen Stone generator shudders to life. Merge the Blueprint into it, then deliver the first rough pebbles.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Stone, 1, 2)],
                    rewards: vec![Reward::Score(400), Reward::Energy(10)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Stone],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M8 — Stone T2 + Wood T3 together; thaw push
            MissionDef {
                description: "Grove and stone in harmony. Deliver a plank and two polished pebbles — the thaw is spreading, help it along.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 1), (Stone, 2, 2)],
                    rewards: vec![Reward::Score(900), Reward::Stars(4)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: Some(6),
                energy_max_bonus: 5,
                inventory_slot_bonus: 1,
            },

            // ── Late missions: larger active board, chain orders, third family ─

            // M9 — higher Wood + Stone, chain order unlocks energy reward
            MissionDef {
                description: "The forest grows bolder. A fine log and a stone boulder are needed — and if you can push further, extra energy awaits.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 4, 1), (Stone, 3, 1)],
                    rewards: vec![Reward::Score(1200), Reward::Stars(5)],
                }],
                extra_orders: vec![
                    chain_order(
                        Stone, 2, 1, vec![Reward::Score(400)],
                        Wood, 3, 1,
                        vec![Reward::Score(600), Reward::Energy(20)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M10 — deep thaw + Blueprint(Metal) reward
            MissionDef {
                description: "Metal veins run under the deepest ice. Shatter the frost and bring proof of your worth to claim the Metal blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 3, 2), (Stone, 3, 1)],
                    rewards: vec![
                        Reward::Score(1000),
                        Reward::SpawnPiece(Piece::Blueprint(Metal)),
                    ],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: Some(10),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M11 — Metal gen awakens; first Metal orders + chain into Stone T2
            MissionDef {
                description: "The Metal generator stirs in the deep grove. Forge the first ingots — then the stone-and-iron chain order will bring a new inventory slot.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 1, 2)],
                    rewards: vec![Reward::Score(500), Reward::Energy(15)],
                }],
                extra_orders: vec![
                    chain_order(
                        Metal, 2, 1, vec![Reward::Score(800), Reward::Stars(4)],
                        Stone, 2, 1,
                        vec![Reward::Score(600), Reward::InventorySlot],
                    ),
                ],
                unlock_families: vec![Metal],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },
            // M12 — three families in concert; multiple active orders
            MissionDef {
                description: "Three families in concert — the grove sings. Juggle Wood, Stone and Metal orders all at once.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Wood, 4, 1), (Stone, 3, 1)],
                        rewards: vec![Reward::Score(1500), Reward::Stars(6)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Metal, 2, 2)],
                        rewards: vec![Reward::Score(800), Reward::Energy(20)],
                    },
                ],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: Some(14),
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M13 — Cloth blueprint reward; chain order spans two families
            MissionDef {
                description: "Silken threads lie frozen at the grove's heart. Bring strong offerings — and follow the chain to claim the Cloth blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 3, 1), (Stone, 4, 1)],
                    rewards: vec![Reward::Score(2000)],
                }],
                extra_orders: vec![
                    chain_order(
                        Metal, 2, 1, vec![Reward::Score(600)],
                        Stone, 3, 1,
                        vec![
                            Reward::Score(1200),
                            Reward::SpawnPiece(Piece::Blueprint(Family::Cloth)),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M14 — Grand finale: all four families, multiple simultaneous orders
            MissionDef {
                description: "The Grove blooms in full glory. Complete the final offerings from all four families — two orders, side by side, each harder than the last.",
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
                extra_orders: vec![],
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
// Generator cooldown: 3 ticks. Tighter energy (80 max). Only 3 inventory slots.
//
// Family unlock order: Metal+Ember → Stone (M6) → Crystal (M11) → all four
// Chain orders introduced at M4; multiple simultaneous orders from M8.
// SoftGenerators appear naturally via high-tier merges (soft_gen_chance: 20).

fn foundry_track() -> TrackDef {
    use CellInit::*;
    use Family::*;

    let layout = BoardLayout {
        rows: 10,
        cols: 8,
        cells: vec![
            // row 0: two active generators + starter items
            vec![HardGenerator(Metal,1), Item(Metal,1), Item(Ember,1), Item(Metal,1), Frozen, Frozen, Frozen, Frozen],
            // row 1: Ember gen active
            vec![Item(Ember,1), HardGenerator(Ember,1), Item(Metal,1), Item(Ember,1), FrozenItem(Metal,2), Frozen, Frozen, Frozen],
            // row 2: first frost wall
            vec![Frozen, Frozen, FrozenItem(Ember,2), Frozen, Frozen, Frozen, FrozenItem(Metal,1), Frozen],
            // row 3: Stone gen frozen
            vec![Frozen, FrozenHardGen(Stone), Frozen, Frozen, Frozen, FrozenItem(Ember,2), Frozen, Frozen],
            // row 4: mid-tier frozen items
            vec![Frozen, Frozen, Frozen, FrozenItem(Metal,3), Frozen, Frozen, Frozen, Frozen],
            // row 5: Crystal gen frozen
            vec![Frozen, Frozen, FrozenItem(Ember,1), Frozen, FrozenHardGen(Crystal), Frozen, Frozen, Frozen],
            // row 6
            vec![Frozen, FrozenItem(Metal,2), Frozen, Frozen, Frozen, Frozen, FrozenItem(Stone,2), Frozen],
            // row 7: extra Ember gen frozen; Crystal item locked
            vec![Frozen, Frozen, Frozen, FrozenHardGen(Ember), Frozen, Frozen, Frozen, FrozenItem(Crystal,1)],
            // row 8
            vec![Frozen, Frozen, FrozenItem(Stone,1), Frozen, Frozen, Frozen, Frozen, Frozen],
            // row 9: deep frozen high-tier items
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
            // ── Early: two families, small active zone, no frozen orders ──

            // M1 — warmup, Metal only; mind the cooldown
            MissionDef {
                description: "The Foundry roars awake — but the generators need time to cool. Mind the rhythm and deliver the first rough metal.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 1, 3)],
                    rewards: vec![Reward::Score(200), Reward::Energy(15)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Metal, Ember],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M2 — Ember intro, two-family order
            MissionDef {
                description: "The Ember crucible is lit. Feed the forges: two Ember fragments and two Metal shards before the heat fades.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 1, 2), (Metal, 1, 2)],
                    rewards: vec![Reward::Score(400), Reward::Stars(2)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M3 — first T2 merges
            MissionDef {
                description: "Smelt the raw ore into something worthy. Merge up to T2 and deliver two refined slabs.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 2, 2)],
                    rewards: vec![Reward::Score(600), Reward::Energy(10)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M4 — mixed T2 + first thaw + chain order
            MissionDef {
                description: "Ember and Metal must flow together. Thaw the first frost pocket — a chain order lurks inside.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 2, 2), (Metal, 2, 1)],
                    rewards: vec![Reward::Score(800), Reward::Stars(3)],
                }],
                extra_orders: vec![
                    chain_order(
                        Metal, 2, 1, vec![Reward::Score(400)],
                        Ember, 2, 1,
                        vec![Reward::Score(600), Reward::Energy(15)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(5),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },

            // ── Mid: frozen cells, T3 push, Stone unlocked ───────────────

            // M5 — T3 milestone + Blueprint(Stone) via chain
            MissionDef {
                description: "A Stone blueprint is buried in the deep frost. First prove your forge mastery with T3 Metal — then follow the chain to claim it.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 3, 1)],
                    rewards: vec![Reward::Score(1000)],
                }],
                extra_orders: vec![
                    chain_order(
                        Ember, 2, 2, vec![Reward::Score(600)],
                        Metal, 2, 1,
                        vec![
                            Reward::Score(800),
                            Reward::SpawnPiece(Piece::Blueprint(Stone)),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },
            // M6 — Stone gen unlockable; first Stone T1 orders
            MissionDef {
                description: "Merge the Blueprint into the frozen Stone generator and hear it rumble to life. The first raw stones are already needed.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Stone, 1, 2)],
                    rewards: vec![Reward::Score(500), Reward::Energy(15)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Stone],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M7 — three families at T2-T3; thaw push
            MissionDef {
                description: "Three forges burning at once. The Stone feeds the crucible — a stronger alloy demands your best Ember and freshest Stone.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 3, 1), (Stone, 2, 1)],
                    rewards: vec![Reward::Score(1200), Reward::Stars(4)],
                }],
                extra_orders: vec![
                    chain_order(
                        Stone, 2, 1, vec![Reward::Score(500)],
                        Metal, 3, 1,
                        vec![Reward::Score(900), Reward::Stars(3)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(8),
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M8 — multiple simultaneous orders, three families
            MissionDef {
                description: "Three forges, two orders, one deadline. Balance Metal, Stone and Ember output — every cooldown tick matters.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Metal, 3, 1), (Stone, 2, 2)],
                        rewards: vec![Reward::Score(1400), Reward::Stars(5)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Ember, 3, 1)],
                        rewards: vec![Reward::Score(800), Reward::Energy(20)],
                    },
                ],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },

            // ── Late: T4-T5 push, Crystal unlocked, bubble artefacts, complex chains ──

            // M9 — T4 ore + deep thaw
            MissionDef {
                description: "Push the forge to its limit. T4 ore means four merges deep — the board is wide now, use every unfrozen cell.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Metal, 4, 1)],
                    rewards: vec![Reward::Score(2000), Reward::Stars(6)],
                }],
                extra_orders: vec![
                    chain_order(
                        Ember, 3, 1, vec![Reward::Score(700)],
                        Stone, 3, 1,
                        vec![Reward::Score(1000), Reward::Energy(25)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(12),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M10 — Crystal blueprint reward
            MissionDef {
                description: "Crystal deposits shimmer in the deepest ice. Forge a final chain of Ember and Stone to claim the Crystal blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 4, 1), (Stone, 3, 1)],
                    rewards: vec![Reward::Score(1800)],
                }],
                extra_orders: vec![
                    chain_order(
                        Metal, 3, 1, vec![Reward::Score(600)],
                        Ember, 3, 1,
                        vec![
                            Reward::Score(1200),
                            Reward::SpawnPiece(Piece::Blueprint(Family::Crystal)),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M11 — Crystal gen awakens; first Crystal orders + energy bonus
            MissionDef {
                description: "The Crystal generator blazes to life. Its prismatic light reveals paths through the ice — deliver the first shards and claim extra energy.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 1, 2)],
                    rewards: vec![Reward::Score(600), Reward::Energy(20)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Crystal],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M12 — Crystal T2 + Metal T4; deep thaw; multiple simultaneous orders
            MissionDef {
                description: "The crucible demands crystalline clarity alongside refined metal. Two orders run at once — manage the board carefully.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Crystal, 2, 2), (Metal, 4, 1)],
                        rewards: vec![Reward::Score(2500), Reward::Stars(8)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Stone, 3, 1), (Ember, 3, 1)],
                        rewards: vec![Reward::Score(1500), Reward::Energy(20)],
                    },
                ],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: Some(18),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M13 — high-tier multi-family + inventory reward via chain
            MissionDef {
                description: "The grand alloy demands the finest from every furnace. Complete the chain — Ember flows into Crystal — and earn a new inventory slot.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 4, 1), (Stone, 4, 1), (Crystal, 3, 1)],
                    rewards: vec![Reward::Score(4000), Reward::Stars(12), Reward::InventorySlot],
                }],
                extra_orders: vec![
                    chain_order(
                        Crystal, 2, 1, vec![Reward::Score(800)],
                        Ember, 4, 1,
                        vec![Reward::Score(1600), Reward::InventorySlot],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M14 — Foundry finale; two simultaneous orders at T5
            MissionDef {
                description: "The Foundry's masterwork. Two simultaneous orders at the pinnacle of craft — Metal, Crystal and Ember forged into legend.",
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
                extra_orders: vec![],
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
// Hard. Board 10×8. Only Crystal generator active at start (tiny 2×2 zone).
// Wood gen at (1,0) frozen — Blueprint(Wood) reward at M4.
// Ember gen at (2,5), Stone gen at (5,4), Metal gen at (6,7), Cloth gen at (8,1).
// Energy max 80, cooldown 5, only 1 random order — every tap counts.
//
// Family unlock order: Crystal → Wood (M5) → Ember (M9) → Stone (M11) → Metal+Cloth (M14)
// Chain orders from M3; multiple simultaneous orders from M6.
// Bubble cells accumulate naturally (high merge count with soft_gen_chance: 15).

fn sanctum_track() -> TrackDef {
    use CellInit::*;
    use Family::*;

    let layout = BoardLayout {
        rows: 10,
        cols: 8,
        cells: vec![
            // row 0: all frozen; Crystal shard visible through ice
            vec![Frozen, FrozenItem(Crystal,1), Frozen, Frozen, Frozen, FrozenItem(Wood,1), Frozen, Frozen],
            // row 1: Wood gen frozen col 0
            vec![FrozenHardGen(Wood), Frozen, Frozen, FrozenItem(Crystal,2), Frozen, Frozen, Frozen, FrozenItem(Ember,1)],
            // row 2: Ember gen frozen col 5
            vec![Frozen, Frozen, Frozen, Frozen, Frozen, FrozenHardGen(Ember), Frozen, Frozen],
            // row 3: active 2×2 zone — Crystal gen + two shards + one empty
            vec![Frozen, Frozen, Frozen, HardGenerator(Crystal,1), Item(Crystal,1), Frozen, Frozen, Frozen],
            // row 4
            vec![Frozen, Frozen, Frozen, Item(Crystal,1), Empty, Frozen, Frozen, Frozen],
            // row 5: Stone gen frozen col 4
            vec![Frozen, FrozenItem(Stone,1), Frozen, Frozen, FrozenHardGen(Stone), Frozen, FrozenItem(Crystal,2), Frozen],
            // row 6: Metal gen frozen col 7
            vec![Frozen, Frozen, FrozenItem(Metal,1), Frozen, Frozen, Frozen, Frozen, FrozenHardGen(Metal)],
            // row 7: deep Crystal cache
            vec![FrozenItem(Crystal,3), Frozen, Frozen, FrozenItem(Ember,2), Frozen, Frozen, Frozen, Frozen],
            // row 8: Cloth gen frozen col 1
            vec![Frozen, FrozenHardGen(Cloth), Frozen, Frozen, Frozen, FrozenItem(Stone,2), Frozen, FrozenItem(Cloth,1)],
            // row 9: late-game high-tier cache
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
            // ── Early: Crystal only, tight board, single-family orders ───

            // M1 — Crystal T1 × 2; only two active cells
            MissionDef {
                description: "The Sanctum grants nothing freely. Crystal is all you hold. Tap the generator twice — the long cooldown is the lesson.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 1, 2)],
                    rewards: vec![Reward::Score(300), Reward::Energy(15)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Crystal],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M2 — Crystal T2; first merge on this track
            MissionDef {
                description: "A shard fused with a shard becomes a fragment. Merge two Crystal T1s and deliver the resulting gem.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 2, 1)],
                    rewards: vec![Reward::Score(500), Reward::Stars(2)],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M3 — Crystal T2 × 2 + thaw + first chain order
            MissionDef {
                description: "The permafrost cracks. Two fragments are owed — and a chain order rewards those who crack the ice further.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 2, 2)],
                    rewards: vec![Reward::Score(700), Reward::Energy(10)],
                }],
                extra_orders: vec![
                    chain_order(
                        Crystal, 1, 2, vec![Reward::Score(300)],
                        Crystal, 2, 1,
                        vec![Reward::Score(600), Reward::Energy(15)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(6),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M4 — Crystal T3 + Blueprint(Wood) reward
            MissionDef {
                description: "A Wood blueprint trembles beneath the ice. Offer a Crystal gem — three merges deep — to shake it loose.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 3, 1)],
                    rewards: vec![
                        Reward::Score(1000),
                        Reward::SpawnPiece(Piece::Blueprint(Wood)),
                    ],
                }],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 1,
            },

            // ── Mid: Wood unlocked, two families, thaw deepens ──────────

            // M5 — Wood T1; second family awakens
            MissionDef {
                description: "Wood emerges from the permafrost. Merge the Blueprint into the frozen generator, then deliver the first rough timber.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Wood, 1, 2)],
                    rewards: vec![Reward::Score(400), Reward::Energy(10)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Wood],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M6 — Crystal T3 + Wood T2; two simultaneous orders
            MissionDef {
                description: "Crystal and Wood together reveal the Sanctum's depths. Two orders run in parallel — keep both generators ticking.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Crystal, 3, 1)],
                        rewards: vec![Reward::Score(1200), Reward::Stars(4)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Wood, 2, 1)],
                        rewards: vec![Reward::Score(600), Reward::Energy(15)],
                    },
                ],
                extra_orders: vec![],
                unlock_families: vec![],
                thaw_target: Some(10),
                energy_max_bonus: 5,
                inventory_slot_bonus: 0,
            },
            // M7 — deeper thaw + Crystal T4 + chain
            MissionDef {
                description: "Deeper into the ice. The Sanctum demands a T4 gem — and a chain order tests whether you can sustain the merge chain.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 4, 1)],
                    rewards: vec![Reward::Score(2000), Reward::Stars(5)],
                }],
                extra_orders: vec![
                    chain_order(
                        Wood, 2, 1, vec![Reward::Score(500)],
                        Crystal, 3, 1,
                        vec![Reward::Score(1000), Reward::Stars(4)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(14),
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M8 — Blueprint(Ember) reward; Crystal+Wood chain
            MissionDef {
                description: "Ember stirs in the deep frost. Crystal and Wood together are the price — complete the chain to claim the Ember blueprint.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 3, 1), (Wood, 3, 1)],
                    rewards: vec![Reward::Score(1500)],
                }],
                extra_orders: vec![
                    chain_order(
                        Wood, 2, 1, vec![Reward::Score(400)],
                        Crystal, 3, 1,
                        vec![
                            Reward::Score(1000),
                            Reward::SpawnPiece(Piece::Blueprint(Ember)),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },

            // ── Late: Ember unlocked, three families, complex chains ─────

            // M9 — Ember T1; three-family play begins
            MissionDef {
                description: "Ember flares in the dark. Merge the Blueprint and kindle the first sparks — the Sanctum grows warmer and more perilous.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Ember, 1, 2)],
                    rewards: vec![Reward::Score(600), Reward::Energy(15)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Ember],
                thaw_target: None,
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M10 — Blueprint(Stone) reward; Crystal+Ember chain
            MissionDef {
                description: "Stone waits beneath the permafrost. Three families must offer tribute in a chain — Crystal into Ember — before it yields.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 4, 1), (Ember, 2, 1)],
                    rewards: vec![Reward::Score(2000)],
                }],
                extra_orders: vec![
                    chain_order(
                        Ember, 2, 1, vec![Reward::Score(700)],
                        Crystal, 4, 1,
                        vec![
                            Reward::Score(1500),
                            Reward::SpawnPiece(Piece::Blueprint(Stone)),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(18),
                energy_max_bonus: 10,
                inventory_slot_bonus: 0,
            },
            // M11 — Stone gen awakens; four families
            MissionDef {
                description: "Stone speaks at last. Four families are in your hands — use them wisely. The Sanctum watches.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Stone, 1, 2), (Ember, 2, 1)],
                    rewards: vec![Reward::Score(800), Reward::Stars(6)],
                }],
                extra_orders: vec![],
                unlock_families: vec![Stone],
                thaw_target: None,
                energy_max_bonus: 5,
                inventory_slot_bonus: 1,
            },
            // M12 — four families T3-T5; two simultaneous + chain
            MissionDef {
                description: "The Sanctum's heart pulses. Four offerings of considerable power — two simultaneous orders and a chain that spirals ever upward.",
                story_orders: vec![
                    StoryOrderDef {
                        requirements: vec![(Crystal, 5, 1), (Wood, 4, 1)],
                        rewards: vec![Reward::Score(4000), Reward::Stars(10)],
                    },
                    StoryOrderDef {
                        requirements: vec![(Stone, 3, 1), (Ember, 3, 1)],
                        rewards: vec![Reward::Score(2000), Reward::Energy(30)],
                    },
                ],
                extra_orders: vec![
                    chain_order(
                        Stone, 2, 1, vec![Reward::Score(600)],
                        Ember, 3, 1,
                        vec![Reward::Score(1200), Reward::Stars(5)],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: Some(24),
                energy_max_bonus: 0,
                inventory_slot_bonus: 0,
            },
            // M13 — Metal + Cloth blueprints via chain; penultimate test
            MissionDef {
                description: "Metal and Cloth lie dormant in the deepest ice. A worthy chain sacrifice — Crystal into Ember — unlocks both at once.",
                story_orders: vec![StoryOrderDef {
                    requirements: vec![(Crystal, 5, 1), (Ember, 4, 1)],
                    rewards: vec![Reward::Score(3000), Reward::Stars(8)],
                }],
                extra_orders: vec![
                    chain_order(
                        Crystal, 4, 1, vec![Reward::Score(1000)],
                        Ember, 4, 1,
                        vec![
                            Reward::Score(2000),
                            Reward::SpawnPiece(Piece::Blueprint(Metal)),
                            Reward::Stars(6),
                        ],
                    ),
                ],
                unlock_families: vec![],
                thaw_target: None,
                energy_max_bonus: 10,
                inventory_slot_bonus: 1,
            },
            // M14 — Sanctum finale; all six families; two massive orders
            MissionDef {
                description: "The Sanctum's heart is laid bare. Complete the ultimate offering from all families — two orders, both unforgiving, both glorious.",
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
                extra_orders: vec![],
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

    #[test]
    fn chain_orders_have_follow_up() {
        let order = chain_order(
            Family::Wood, 2, 1, vec![Reward::Score(100)],
            Family::Stone, 1, 1, vec![Reward::Score(200)],
        );
        assert!(order.follow_up.is_some());
        let fu = order.follow_up.unwrap();
        assert_eq!(fu.requirements[0].family, Family::Stone);
        assert_eq!(fu.requirements[0].tier, 1);
    }

    #[test]
    fn late_missions_have_extra_orders() {
        // Grove M9 (index 8) should have a chain order
        let grove = track_def(0);
        assert!(!grove.missions[8].extra_orders.is_empty());
        // Foundry M5 (index 4) should have a chain order
        let foundry = track_def(1);
        assert!(!foundry.missions[4].extra_orders.is_empty());
        // Sanctum M3 (index 2) should have a chain order
        let sanctum = track_def(2);
        assert!(!sanctum.missions[2].extra_orders.is_empty());
    }

    #[test]
    fn each_track_has_at_least_8_missions() {
        for track in 0..TRACK_COUNT {
            let def = track_def(track);
            assert!(def.missions.len() >= 8, "track {} has fewer than 8 missions", track);
        }
    }
}
