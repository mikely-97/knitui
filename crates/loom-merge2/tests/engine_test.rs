/// Integration tests for loom-merge2 engine mechanics.
use m2tui::board::{Board, Cell};
use m2tui::energy::Energy;
use m2tui::engine::GameEngine;
use m2tui::inventory::Inventory;
use m2tui::item::{Family, Item, Piece};
use m2tui::order::{Order, OrderRequirement, OrderType, Reward};

// ── Test helper ───────────────────────────────────────────────────────────

fn make_engine(board: Board, inventory: Inventory, energy: Energy, orders: Vec<Order>, stars: u16) -> GameEngine {
    GameEngine::from_state(
        board,
        inventory,
        energy,
        orders,
        0,     // score
        stars, // stars
        0,     // total_merges
        0,     // cells_thawed
        1,     // scale
        5,     // ad_limit
        0,     // random_order_count — 0 so we don't get noise orders
        4,     // max_order_tier
        1,     // generator_cost
        0,     // generator_cooldown
        0,     // soft_gen_chance — 0 so merges don't randomly create generators
        vec![Family::Wood, Family::Stone, Family::Metal],
        &[],
    )
}

fn small_board() -> Board {
    Board::new_empty(4, 4)
}

// ── Sanity tests ──────────────────────────────────────────────────────────

#[test]
fn board_initializes_correctly() {
    let board = Board::new_empty(5, 6);
    assert_eq!(board.rows, 5);
    assert_eq!(board.cols, 6);
    for r in 0..5 {
        for c in 0..6 {
            assert!(board.cells[r][c].is_empty(), "cell ({r},{c}) should be empty");
        }
    }
}

#[test]
fn energy_starts_at_max() {
    let energy = Energy::new(100, 30);
    assert_eq!(energy.current, 100);
    assert_eq!(energy.max, 100);
}

#[test]
fn placing_item_in_inventory_works() {
    let mut inv = Inventory::new(4);
    assert_eq!(inv.used_count(), 0);
    let piece = Piece::Regular(Item::new(Family::Wood, 1));
    let stored = inv.store(piece.clone());
    assert!(stored, "store should succeed when slots are free");
    assert_eq!(inv.used_count(), 1);
    assert_eq!(inv.peek(0), Some(&piece));
}

#[test]
fn inventory_full_rejects_extra_piece() {
    let mut inv = Inventory::new(1);
    assert!(inv.store(Piece::Regular(Item::new(Family::Wood, 1))));
    assert!(!inv.store(Piece::Regular(Item::new(Family::Stone, 1))), "second store should fail");
}

// ── Merge mechanics ───────────────────────────────────────────────────────

/// Place two matching T1 items, merge them, verify source empty & dest is T2.
#[test]
fn merge_two_t1_produces_t2() {
    let mut board = small_board();
    board.cells[0][0] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
    board.cells[0][1] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 0);

    // Select (0,0) then activate (0,1) to merge
    engine.cursor_row = 0;
    engine.cursor_col = 0;
    engine.activate(); // select

    engine.cursor_col = 1;
    let merged = engine.activate(); // merge into (0,1)

    assert!(merged, "merge should succeed");
    assert!(engine.board.cells[0][0].is_empty(), "source cell should be empty after merge");

    match &engine.board.cells[0][1] {
        Cell::Piece(Piece::Regular(item)) => {
            assert_eq!(item.family, Family::Wood);
            assert_eq!(item.tier, 2, "result should be tier-2");
        }
        other => panic!("expected Piece(Regular(Wood T2)) at (0,1), got {:?}", other),
    }
    assert!(engine.total_merges > 0);
}

#[test]
fn merge_mismatched_families_fails() {
    let mut board = small_board();
    board.cells[1][0] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
    board.cells[1][1] = Cell::Piece(Piece::Regular(Item::new(Family::Stone, 1)));

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 0);

    engine.cursor_row = 1;
    engine.cursor_col = 0;
    engine.activate(); // select Wood T1

    engine.cursor_col = 1;
    // Activating on Stone T1: can't merge, engine reselects Stone
    engine.activate();

    // Neither cell should be empty — no merge happened
    assert!(engine.board.cells[1][0].is_piece() || engine.selected.is_some());
}

// ── Chain orders ──────────────────────────────────────────────────────────

/// Create an order with a follow_up. Fulfill the parent; verify follow_up becomes active.
#[test]
fn chain_order_follow_up_activates_on_completion() {
    let follow_up = Order {
        order_type: OrderType::Story,
        requirements: vec![OrderRequirement::new(Family::Stone, 1, 1)],
        rewards: vec![Reward::Score(500)],
        follow_up: None,
    };
    let parent = Order {
        order_type: OrderType::Story,
        requirements: vec![OrderRequirement::new(Family::Wood, 1, 1)],
        rewards: vec![Reward::Score(200)],
        follow_up: Some(Box::new(follow_up)),
    };

    let mut board = small_board();
    board.cells[2][2] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![parent], 0);

    // Before delivery: one story order, no Stone order
    assert_eq!(engine.active_orders.len(), 1);
    assert!(engine.active_orders[0].requirements[0].family == Family::Wood);

    // Select and deliver the Wood T1
    engine.cursor_row = 2;
    engine.cursor_col = 2;
    engine.activate();
    let delivered = engine.deliver_from_board();
    assert!(delivered, "deliver should succeed");

    // After delivery: the Wood order is gone, follow_up (Stone T1) should now be active
    let has_stone_order = engine.active_orders.iter().any(|o| {
        o.requirements.iter().any(|r| r.family == Family::Stone && r.tier == 1)
    });
    assert!(has_stone_order, "follow_up Stone order should be active after parent completion");
}

// ── Generator upgrade ─────────────────────────────────────────────────────

/// Place a generator on board, upgrade it (level 0→1), verify upgrade_level is 1.
#[test]
fn generator_upgrade_level_increments() {
    let mut board = small_board();
    board.cells[0][0] = Cell::HardGenerator {
        family: Family::Wood,
        tier: 1,
        cooldown_remaining: 0,
        upgrade_level: 0,
    };

    // Need 3 stars to upgrade
    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 3);

    engine.cursor_row = 0;
    engine.cursor_col = 0;
    let upgraded = engine.upgrade_generator_at_cursor();

    assert!(upgraded, "upgrade should succeed with enough stars");
    assert_eq!(engine.stars, 0, "should have spent 3 stars");

    match &engine.board.cells[0][0] {
        Cell::HardGenerator { upgrade_level, .. } => {
            assert_eq!(*upgrade_level, 1, "upgrade_level should be 1");
        }
        other => panic!("expected HardGenerator at (0,0), got {:?}", other),
    }
}

#[test]
fn generator_upgrade_fails_without_stars() {
    let mut board = small_board();
    board.cells[0][0] = Cell::HardGenerator {
        family: Family::Wood,
        tier: 1,
        cooldown_remaining: 0,
        upgrade_level: 0,
    };

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 2); // only 2 stars
    engine.cursor_row = 0;
    engine.cursor_col = 0;
    let upgraded = engine.upgrade_generator_at_cursor();

    assert!(!upgraded, "upgrade should fail with insufficient stars");

    match &engine.board.cells[0][0] {
        Cell::HardGenerator { upgrade_level, .. } => {
            assert_eq!(*upgrade_level, 0, "upgrade_level should still be 0");
        }
        other => panic!("expected HardGenerator, got {:?}", other),
    }
}

#[test]
fn generator_upgrade_caps_at_level_2() {
    let mut board = small_board();
    board.cells[0][0] = Cell::HardGenerator {
        family: Family::Wood,
        tier: 1,
        cooldown_remaining: 0,
        upgrade_level: 2, // already at max
    };

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 99);
    engine.cursor_row = 0;
    engine.cursor_col = 0;
    let upgraded = engine.upgrade_generator_at_cursor();

    assert!(!upgraded, "upgrade should fail at max level");
}

// ── Bubble cells ──────────────────────────────────────────────────────────

/// A bubbled item should not be accepted by deliver_from_board.
#[test]
fn bubble_item_cannot_be_delivered_directly() {
    let order = Order {
        order_type: OrderType::Story,
        requirements: vec![OrderRequirement::new(Family::Wood, 1, 1)],
        rewards: vec![Reward::Score(100)],
        follow_up: None,
    };

    let mut board = small_board();
    // Put Wood T1 inside a bubble — it looks like an item but is protected
    board.cells[1][1] = Cell::Bubble(Piece::Regular(Item::new(Family::Wood, 1)));

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![order], 0);

    // Try to select and deliver the bubbled cell
    engine.cursor_row = 1;
    engine.cursor_col = 1;
    engine.activate(); // attempt select — bubble cell is not a free Piece, so won't select
    // selected should remain None since Bubble isn't a free Piece
    assert!(
        engine.selected.is_none(),
        "bubble cell should not be selectable as a free piece"
    );

    // Directly force selection and try deliver
    engine.selected = Some((1, 1));
    let delivered = engine.deliver_from_board();
    assert!(!delivered, "bubble cell should not count for order delivery");
}

/// Merging a free identical piece into a bubble cell pops the bubble.
#[test]
fn merging_into_bubble_pops_it() {
    let mut board = small_board();
    // Bubble contains Wood T1
    board.cells[0][0] = Cell::Bubble(Piece::Regular(Item::new(Family::Wood, 1)));
    // Free Wood T1 as the merge source
    board.cells[0][1] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 0);

    // Select (0,1) then merge into bubble at (0,0)
    engine.cursor_row = 0;
    engine.cursor_col = 1;
    engine.activate(); // select free piece
    assert_eq!(engine.selected, Some((0, 1)));

    engine.cursor_col = 0;
    let result = engine.activate(); // merge free piece into bubble

    assert!(result, "merge into bubble should succeed");
    assert!(engine.board.cells[0][1].is_empty(), "source should be empty");

    // Destination should be a free Piece (bubble popped), tier 2
    match &engine.board.cells[0][0] {
        Cell::Piece(Piece::Regular(item)) => {
            assert_eq!(item.family, Family::Wood);
            assert_eq!(item.tier, 2, "merged result should be tier 2");
        }
        other => panic!("expected free Piece(Wood T2) after bubble pop, got {:?}", other),
    }

    // The BubblePopped notification should have been pushed
    let bubble_popped = engine.notifications.iter().any(|n| {
        matches!(n, m2tui::engine::Notification::BubblePopped { .. })
    });
    assert!(bubble_popped, "BubblePopped notification should be present");
}

// ── Inventory expansion event ─────────────────────────────────────────────

/// Manually trigger the inv expansion event, accept it, verify slot count increased by 1.
#[test]
fn accept_inv_expansion_increases_slots_by_one() {
    let board = small_board();
    let inv = Inventory::new(4);
    let initial_slots = inv.slot_count();
    let mut engine = make_engine(board, inv, Energy::new(100, 30), vec![], 0);

    assert_eq!(engine.inventory.slot_count(), initial_slots);
    assert!(!engine.inv_expansion_pending);

    // Manually trigger the popup
    engine.inv_expansion_pending = true;
    engine.accept_inv_expansion();

    assert!(!engine.inv_expansion_pending, "pending flag should be cleared");
    assert_eq!(
        engine.inventory.slot_count(),
        initial_slots + 1,
        "inventory should have grown by 1 slot"
    );
}

#[test]
fn dismiss_inv_expansion_does_not_add_slot() {
    let board = small_board();
    let inv = Inventory::new(4);
    let initial_slots = inv.slot_count();
    let mut engine = make_engine(board, inv, Energy::new(100, 30), vec![], 0);

    engine.inv_expansion_pending = true;
    engine.dismiss_inv_expansion();

    assert!(!engine.inv_expansion_pending);
    assert_eq!(engine.inventory.slot_count(), initial_slots, "slots should be unchanged after dismiss");
}

// ── Additional sanity: inventory store/retrieve ───────────────────────────

#[test]
fn store_to_inventory_and_retrieve() {
    let mut board = small_board();
    board.cells[3][3] = Cell::Piece(Piece::Regular(Item::new(Family::Stone, 2)));

    let mut engine = make_engine(board, Inventory::new(4), Energy::new(100, 30), vec![], 0);

    engine.cursor_row = 3;
    engine.cursor_col = 3;
    engine.activate(); // select

    let stored = engine.store_selected_to_inventory();
    assert!(stored, "store should succeed");
    assert!(engine.board.cells[3][3].is_empty());
    assert_eq!(engine.inventory.used_count(), 1);

    // Place it back on an empty cell
    engine.cursor_row = 0;
    engine.cursor_col = 0;
    let placed = engine.place_from_inventory(0);
    assert!(placed);
    assert!(engine.board.cells[0][0].is_piece());
    assert_eq!(engine.inventory.used_count(), 0);
}
