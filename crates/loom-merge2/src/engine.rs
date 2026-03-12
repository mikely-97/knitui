use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::board::{Board, Cell};
use crate::config::Config;
use crate::energy::Energy;
use crate::generator::{self, ActivationResult};
use crate::inventory::Inventory;
use crate::item::{Family, Item, Piece, ALL_FAMILIES, MAX_TIER};
use crate::order::{
    Order, Reward, generate_random_order, generate_timed_order,
};

// ── Game status ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
    Stuck,
}

// ── Blessing flags ────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlessingFlags {
    /// 25% chance generator activation is free.
    pub energy_saver: bool,
    /// Energy regen 50% faster.
    pub quick_regen: bool,
    /// Highlight a valid merge pair.
    pub keen_eye: bool,
    /// +2 inventory slots.
    pub bigger_pockets: bool,
    /// Thaw adjacent cells when a frozen cell thaws.
    pub thaw_aura: bool,
    /// Random orders require 1 fewer item (min 1).
    pub lucky_orders: bool,
    /// After merge, auto-merge result with another matching piece (up to 3 chains).
    pub chain_merge: bool,
    /// 15% chance merge skips a tier.
    pub tier_boost: bool,
    /// Hard generators 20% chance to produce T2.
    pub generator_surge: bool,
    /// Each delivery counts as 2.
    pub double_deliver: bool,
    /// Soft generators +3 charges, 30% chance not to consume charge.
    pub soft_gen_master: bool,
    /// Thawed items upgrade +1 tier.
    pub deep_thaw: bool,
}

// ── Pending UI notifications ──────────────────────────────────────────────

/// An event the TUI should display to the player.
#[derive(Clone, Debug)]
pub enum Notification {
    MergeResult { piece: Piece, thawed: bool },
    OrderCompleted { rewards: Vec<Reward> },
    GeneratorActivated { family: Family, pos: (usize, usize) },
    StoreToInventory,
    InventoryFull,
    NoEnergy,
    NoSpace,
    OnCooldown,
    AdWatched,
    Invalid,
}

// ── Deliver target ────────────────────────────────────────────────────────

/// Where a delivered item came from.
pub enum DeliverSource {
    Board { pos: (usize, usize) },
    Inventory { slot: usize },
}

// ── Engine ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameEngine {
    pub board: Board,
    pub inventory: Inventory,
    pub energy: Energy,
    pub active_orders: Vec<Order>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub selected: Option<(usize, usize)>,
    pub score: u32,
    pub stars: u16,
    pub total_merges: u64,
    pub cells_thawed: usize,
    pub scale: u16,
    pub tick_count: u32,
    pub ads_used: u16,
    pub ad_limit: u16,
    pub random_order_count: usize,
    pub max_order_tier: u8,
    pub generator_cost: u16,
    pub generator_cooldown: u32,
    pub soft_gen_chance: u8,
    pub available_families: Vec<Family>,
    /// Ticks until the next time-limited order appears.
    pub timed_order_cooldown: u32,
    pub blessing_flags: BlessingFlags,
    /// Pending notifications for the TUI to display.
    #[serde(skip)]
    pub notifications: Vec<Notification>,
    /// The hint pair for keen_eye blessing (updated each action).
    pub hint_pair: Option<((usize, usize), (usize, usize))>,
}

impl GameEngine {
    /// Create a new engine from a (possibly loaded) board + state for campaign use.
    pub fn from_state(
        board: Board,
        inventory: Inventory,
        energy: Energy,
        active_orders: Vec<Order>,
        score: u32,
        stars: u16,
        total_merges: u64,
        cells_thawed: usize,
        scale: u16,
        ad_limit: u16,
        random_order_count: usize,
        max_order_tier: u8,
        generator_cost: u16,
        generator_cooldown: u32,
        soft_gen_chance: u8,
        available_families: Vec<Family>,
        blessings: &[String],
    ) -> Self {
        let mut engine = Self {
            board,
            inventory,
            energy,
            active_orders,
            cursor_row: 0,
            cursor_col: 0,
            selected: None,
            score,
            stars,
            total_merges,
            cells_thawed,
            scale,
            tick_count: 0,
            ads_used: 0,
            ad_limit,
            random_order_count,
            max_order_tier,
            generator_cost,
            generator_cooldown,
            soft_gen_chance,
            available_families,
            timed_order_cooldown: 400,
            blessing_flags: BlessingFlags::default(),
            notifications: Vec::new(),
            hint_pair: None,
        };
        engine.set_blessings(blessings);
        engine.fill_random_orders();
        engine.update_hint();
        engine
    }

    /// Create a fresh engine for endless mode from config.
    pub fn new_endless(config: &Config, blessings: &[String]) -> Self {
        let families: Vec<Family> = ALL_FAMILIES
            .iter()
            .take(config.family_count as usize)
            .copied()
            .collect();

        let mut board = Board::new_empty(config.board_rows as usize, config.board_cols as usize);

        // Place 2 random hard generators
        let gen_positions = [(0, 0), (config.board_rows as usize - 1, config.board_cols as usize - 1)];
        for (i, &(r, c)) in gen_positions.iter().enumerate() {
            let family = families[i % families.len()];
            board.cells[r][c] = Cell::HardGenerator {
                family,
                tier: 1,
                cooldown_remaining: 0,
            };
        }

        let energy = Energy::new(config.energy_max, config.energy_regen_secs);
        let inventory = Inventory::new(config.inventory_slots as usize);

        Self::from_state(
            board,
            inventory,
            energy,
            Vec::new(),
            0,
            0,
            0,
            0,
            config.scale,
            config.ad_limit,
            config.random_order_count as usize,
            config.max_order_tier,
            config.generator_cost,
            config.generator_cooldown,
            config.soft_gen_chance,
            families,
            blessings,
        )
    }

    pub fn set_blessings(&mut self, ids: &[String]) {
        self.blessing_flags = BlessingFlags {
            energy_saver: blessings_has(ids, "energy_saver"),
            quick_regen: blessings_has(ids, "quick_regen"),
            keen_eye: blessings_has(ids, "keen_eye"),
            bigger_pockets: blessings_has(ids, "bigger_pockets"),
            thaw_aura: blessings_has(ids, "thaw_aura"),
            lucky_orders: blessings_has(ids, "lucky_orders"),
            chain_merge: blessings_has(ids, "chain_merge"),
            tier_boost: blessings_has(ids, "tier_boost"),
            generator_surge: blessings_has(ids, "generator_surge"),
            double_deliver: blessings_has(ids, "double_deliver"),
            soft_gen_master: blessings_has(ids, "soft_gen_master"),
            deep_thaw: blessings_has(ids, "deep_thaw"),
        };

        // Apply bigger_pockets immediately
        if self.blessing_flags.bigger_pockets && self.inventory.slot_count() < 6 {
            self.inventory.expand(2);
        }

        // Apply quick_regen
        if self.blessing_flags.quick_regen && self.energy.regen_rate_secs > 10 {
            self.energy.regen_rate_secs = (self.energy.regen_rate_secs * 2) / 3;
        }
    }

    // ── Cursor movement ───────────────────────────────────────────────────

    pub fn move_cursor(&mut self, dr: i32, dc: i32) -> bool {
        let nr = self.cursor_row as i32 + dr;
        let nc = self.cursor_col as i32 + dc;
        if nr >= 0
            && (nr as usize) < self.board.rows
            && nc >= 0
            && (nc as usize) < self.board.cols
        {
            self.cursor_row = nr as usize;
            self.cursor_col = nc as usize;
            true
        } else {
            false
        }
    }

    // ── Primary action (Enter) ────────────────────────────────────────────

    /// Handles Enter/Space on the board:
    /// - If cursor is on a generator: try to activate it.
    /// - If something is selected: try to merge with cursor cell.
    /// - Otherwise: try to select cursor cell.
    pub fn activate(&mut self) -> bool {
        let pos = (self.cursor_row, self.cursor_col);

        // Generator activation
        if self.board.cells[pos.0][pos.1].is_any_generator() {
            // If another generator of same family+tier is selected → merge them
            if let Some(sel) = self.selected {
                if sel != pos {
                    if let (
                        Cell::HardGenerator { family: f1, tier: t1, .. },
                        Cell::HardGenerator { family: f2, tier: t2, .. },
                    ) = (self.board.cells[sel.0][sel.1].clone(), self.board.cells[pos.0][pos.1].clone()) {
                        if f1 == f2 && t1 == t2 && t1 < MAX_TIER {
                            let new_tier = t1 + 1;
                            self.board.cells[sel.0][sel.1] = Cell::Empty;
                            self.board.cells[pos.0][pos.1] = Cell::HardGenerator {
                                family: f1,
                                tier: new_tier,
                                cooldown_remaining: 0,
                            };
                            self.selected = None;
                            self.score += 50 * new_tier as u32;
                            self.update_hint();
                            return true;
                        }
                    }
                }
            }
            return self.activate_generator_inner(pos, false);
        }

        if let Some(sel) = self.selected {
            if sel == pos {
                // Deselect
                self.selected = None;
                return true;
            }

            if self.board.can_merge(sel, pos) {
                return self.do_merge(sel, pos);
            }

            // Not mergeable: reselect if cursor is on a free piece
            if self.board.cells[pos.0][pos.1].is_piece() {
                self.selected = Some(pos);
                return true;
            }
            // Clicked on empty or frozen (not matching) — deselect
            self.selected = None;
            return true;
        }

        // Select free piece
        if self.board.cells[pos.0][pos.1].is_piece() {
            self.selected = Some(pos);
            self.update_hint();
            return true;
        }

        false
    }

    /// Enhanced activation: spend 2× energy, spawn tier+1 item.
    /// Only works if cursor is on a generator.
    pub fn activate_enhanced(&mut self) -> bool {
        let pos = (self.cursor_row, self.cursor_col);
        if self.board.cells[pos.0][pos.1].is_any_generator() {
            return self.activate_generator_inner(pos, true);
        }
        false
    }

    fn activate_generator_inner(&mut self, pos: (usize, usize), enhanced: bool) -> bool {
        // energy_saver: 25% chance free
        let mut rng = rand::rng();
        let cost = if self.blessing_flags.energy_saver && rng.random_range(0u8..100) < 25 {
            0
        } else {
            self.generator_cost
        };

        let surge = self.blessing_flags.generator_surge;

        let result = generator::try_activate(
            &mut self.board,
            pos.0,
            pos.1,
            &mut self.energy,
            cost,
            self.generator_cooldown,
            enhanced,
            surge,
        );

        match result {
            ActivationResult::Spawned(r, c) => {
                let family = self.board.cells[pos.0][pos.1]
                    .family()
                    .or_else(|| {
                        // Generator was just deleted (soft gen exhausted)
                        self.board.cells[r][c].family()
                    })
                    .unwrap_or(Family::Wood);
                self.notifications.push(Notification::GeneratorActivated {
                    family,
                    pos: (r, c),
                });
                self.update_hint();
                true
            }
            ActivationResult::NoEnergy => {
                self.notifications.push(Notification::NoEnergy);
                false
            }
            ActivationResult::OnCooldown => {
                self.notifications.push(Notification::OnCooldown);
                false
            }
            ActivationResult::NoSpace => {
                self.notifications.push(Notification::NoSpace);
                false
            }
            ActivationResult::NotAGenerator | ActivationResult::Exhausted => false,
        }
    }

    fn do_merge(&mut self, src: (usize, usize), dst: (usize, usize)) -> bool {
        let Some(result) = self.board.do_merge(src, dst) else {
            return false;
        };

        self.selected = None;
        self.total_merges += 1;

        // Handle blueprint merge → hard generator (always T1)
        if let Piece::Blueprint(family) = &result.piece {
            let fam = *family;
            self.board.cells[dst.0][dst.1] = Cell::HardGenerator {
                family: fam,
                tier: 1,
                cooldown_remaining: 0,
            };
            self.notifications.push(Notification::MergeResult {
                piece: Piece::Blueprint(fam),
                thawed: result.thawed,
            });
            self.update_hint();
            return true;
        }

        // Regular item merge
        if let Piece::Regular(ref item) = result.piece {
            let mut final_item = item.clone();

            // tier_boost: 15% chance to skip a tier
            if self.blessing_flags.tier_boost && final_item.tier < MAX_TIER {
                let mut rng = rand::rng();
                if rng.random_range(0u8..100) < 15 {
                    final_item.tier = (final_item.tier + 1).min(MAX_TIER);
                    self.board.cells[dst.0][dst.1] =
                        Cell::Piece(Piece::Regular(final_item.clone()));
                }
            }

            // deep_thaw: thawed items upgrade +1 tier
            if result.thawed && self.blessing_flags.deep_thaw {
                final_item.tier = (final_item.tier + 1).min(MAX_TIER);
                self.board.cells[dst.0][dst.1] =
                    Cell::Piece(Piece::Regular(final_item.clone()));
                self.cells_thawed += 1;
            } else if result.thawed {
                self.cells_thawed += 1;
            }

            // thaw_aura: unfreeze adjacent frozen cells
            if result.thawed && self.blessing_flags.thaw_aura {
                self.board.thaw_adjacent(dst.0, dst.1);
            }

            self.score += final_item.score_value();

            // Soft generator creation from high-tier merge
            if generator::should_create_soft_generator(final_item.tier, self.soft_gen_chance) {
                let charges = if self.blessing_flags.soft_gen_master { 8 } else { 5 };
                self.board.cells[dst.0][dst.1] = Cell::SoftGenerator {
                    family: final_item.family,
                    tier: 1,
                    charges,
                    cooldown_remaining: 0,
                };
            }

            // chain_merge: auto-merge result with another matching piece
            if self.blessing_flags.chain_merge {
                self.try_chain_merge(dst, 3);
            }

            self.notifications.push(Notification::MergeResult {
                piece: Piece::Regular(final_item),
                thawed: result.thawed,
            });
        }

        self.update_hint();
        true
    }

    /// Try to auto-merge the piece at `pos` with any matching piece (up to `depth` times).
    fn try_chain_merge(&mut self, pos: (usize, usize), depth: u8) {
        if depth == 0 {
            return;
        }
        let current_piece = match &self.board.cells[pos.0][pos.1] {
            Cell::Piece(p) => p.clone(),
            _ => return,
        };

        // Find any matching free or frozen piece to merge into
        let target = self.find_merge_target_for(&current_piece, pos);
        if let Some(tgt) = target {
            if let Some(result) = self.board.do_merge(pos, tgt) {
                self.total_merges += 1;
                if let Piece::Regular(ref item) = result.piece {
                    self.score += item.score_value();
                    if result.thawed {
                        self.cells_thawed += 1;
                    }
                }
                self.try_chain_merge(tgt, depth - 1);
            }
        }
    }

    fn find_merge_target_for(&self, piece: &Piece, exclude: (usize, usize)) -> Option<(usize, usize)> {
        for r in 0..self.board.rows {
            for c in 0..self.board.cols {
                if (r, c) == exclude {
                    continue;
                }
                let target_piece = match &self.board.cells[r][c] {
                    Cell::Piece(p) | Cell::Frozen(p) => p,
                    _ => continue,
                };
                if piece.can_merge(target_piece) {
                    return Some((r, c));
                }
            }
        }
        None
    }

    // ── Delivery ──────────────────────────────────────────────────────────

    /// Deliver a piece from a source to the first matching order.
    /// Returns the rewards collected if successful.
    pub fn deliver_from_board(&mut self) -> bool {
        let Some(sel) = self.selected else {
            return false;
        };
        let piece = match &self.board.cells[sel.0][sel.1] {
            Cell::Piece(p) => p.clone(),
            _ => return false,
        };

        if self.try_deliver_piece(&piece) {
            self.board.cells[sel.0][sel.1] = Cell::Empty;
            self.selected = None;
            self.update_hint();
            true
        } else {
            false
        }
    }

    pub fn deliver_from_inventory(&mut self, slot: usize) -> bool {
        let Some(piece) = self.inventory.peek(slot).cloned() else {
            return false;
        };

        if self.try_deliver_piece(&piece) {
            self.inventory.take(slot);
            true
        } else {
            false
        }
    }

    fn try_deliver_piece(&mut self, piece: &Piece) -> bool {
        let deliver_count = if self.blessing_flags.double_deliver { 2 } else { 1 };

        for order in &mut self.active_orders {
            if order.accepts(piece) {
                for _ in 0..deliver_count {
                    if !order.try_deliver(piece) {
                        break;
                    }
                }
                if let Piece::Regular(item) = piece {
                    self.score += item.score_value() * 2;
                }
                if order.is_fulfilled() {
                    let rewards = order.rewards.clone();
                    self.apply_rewards(&rewards);
                    self.notifications.push(Notification::OrderCompleted { rewards });
                }
                return true;
            }
        }

        // Remove fulfilled orders and replenish
        self.active_orders.retain(|o| !o.is_fulfilled());
        self.fill_random_orders();
        false
    }

    pub fn apply_rewards(&mut self, rewards: &[Reward]) {
        for reward in rewards {
            match reward {
                Reward::Score(n) => self.score += n,
                Reward::Energy(n) => self.energy.add(*n),
                Reward::SpawnPiece(piece) => {
                    self.spawn_piece_anywhere(piece.clone());
                }
                Reward::InventorySlot => {
                    self.inventory.expand(1);
                }
                Reward::Stars(n) => self.stars += n,
            }
        }
        // Remove fulfilled orders and refill
        self.active_orders.retain(|o| !o.is_fulfilled());
        self.fill_random_orders();
    }

    fn spawn_piece_anywhere(&mut self, piece: Piece) {
        let mut rng = rand::rng();
        let mut empties: Vec<(usize, usize)> = Vec::new();
        for r in 0..self.board.rows {
            for c in 0..self.board.cols {
                if self.board.cells[r][c].is_empty() {
                    empties.push((r, c));
                }
            }
        }
        if let Some(&(r, c)) = empties.choose(&mut rng) {
            self.board.cells[r][c] = Cell::Piece(piece);
        } else {
            // Board full — try inventory
            let _ = self.inventory.store(piece);
        }
    }

    // ── Inventory ─────────────────────────────────────────────────────────

    /// Move the selected board piece to inventory.
    pub fn store_selected_to_inventory(&mut self) -> bool {
        let Some(sel) = self.selected else {
            return false;
        };
        let piece = match &self.board.cells[sel.0][sel.1] {
            Cell::Piece(p) => p.clone(),
            _ => return false,
        };

        if self.inventory.store(piece) {
            self.board.cells[sel.0][sel.1] = Cell::Empty;
            self.selected = None;
            self.notifications.push(Notification::StoreToInventory);
            self.update_hint();
            true
        } else {
            self.notifications.push(Notification::InventoryFull);
            false
        }
    }

    /// Place a piece from inventory onto an empty board cell at cursor.
    pub fn place_from_inventory(&mut self, slot: usize) -> bool {
        let pos = (self.cursor_row, self.cursor_col);
        if !self.board.cells[pos.0][pos.1].is_empty() {
            return false;
        }
        let Some(piece) = self.inventory.take(slot) else {
            return false;
        };
        self.board.cells[pos.0][pos.1] = Cell::Piece(piece);
        self.update_hint();
        true
    }

    /// Try to merge an inventory piece with a board piece at cursor.
    pub fn merge_from_inventory(&mut self, slot: usize) -> bool {
        let pos = (self.cursor_row, self.cursor_col);
        let inv_piece = match self.inventory.peek(slot) {
            Some(p) => p.clone(),
            None => return false,
        };
        let board_piece = match &self.board.cells[pos.0][pos.1] {
            Cell::Piece(p) | Cell::Frozen(p) => p.clone(),
            _ => return false,
        };
        if !inv_piece.can_merge(&board_piece) {
            return false;
        }

        // Create a temporary board cell for the inventory piece so do_merge can work
        // We find any empty cell, place inv piece there, then merge
        let temp_pos = self.find_any_empty_cell();
        let Some(temp) = temp_pos else {
            return false;
        };

        self.inventory.take(slot);
        self.board.cells[temp.0][temp.1] = Cell::Piece(inv_piece);

        if self.do_merge(temp, pos) {
            self.update_hint();
            true
        } else {
            // Rollback: put piece back in inventory, clear temp
            if let Cell::Piece(p) = self.board.cells[temp.0][temp.1].clone() {
                self.inventory.store(p);
            }
            self.board.cells[temp.0][temp.1] = Cell::Empty;
            false
        }
    }

    fn find_any_empty_cell(&self) -> Option<(usize, usize)> {
        for r in 0..self.board.rows {
            for c in 0..self.board.cols {
                if self.board.cells[r][c].is_empty() {
                    return Some((r, c));
                }
            }
        }
        None
    }

    // ── Ads ───────────────────────────────────────────────────────────────

    pub fn can_watch_ad(&self) -> bool {
        self.ad_limit > 0 && self.ads_used < self.ad_limit
    }

    pub fn watch_ad_reward(&mut self, reward: crate::ad::AdReward) {
        use crate::ad::AdReward;
        match reward {
            AdReward::Energy(n) => {
                self.energy.add(n);
            }
            AdReward::FullEnergy => {
                self.energy.fill();
            }
            AdReward::RareItem(family, tier) => {
                self.spawn_piece_anywhere(Piece::Regular(Item::new(family, tier)));
            }
            AdReward::InventoryExpand => {
                self.inventory.expand(1);
            }
            AdReward::OrderRefresh => {
                self.active_orders.retain(|o| !matches!(o.order_type, crate::order::OrderType::Random));
                self.fill_random_orders();
            }
        }
        self.ads_used += 1;
        self.notifications.push(Notification::AdWatched);
    }

    // ── Orders ────────────────────────────────────────────────────────────

    /// Drop all random orders and regenerate them from the current `available_families`.
    /// Call this after overriding `available_families` on a freshly-built engine.
    pub fn regenerate_orders(&mut self) {
        self.active_orders
            .retain(|o| !matches!(o.order_type, crate::order::OrderType::Random));
        self.fill_random_orders();
    }

    fn fill_random_orders(&mut self) {
        let random_count = self
            .active_orders
            .iter()
            .filter(|o| matches!(o.order_type, crate::order::OrderType::Random))
            .count();

        for _ in random_count..self.random_order_count {
            let order = generate_random_order(
                &self.available_families,
                self.max_order_tier,
                self.blessing_flags.lucky_orders,
            );
            self.active_orders.push(order);
        }
    }

    fn maybe_spawn_timed_order(&mut self) {
        if self.timed_order_cooldown > 0 {
            self.timed_order_cooldown -= 1;
            return;
        }
        // Only one active timed order at a time
        if self
            .active_orders
            .iter()
            .any(|o| o.is_time_limited())
        {
            self.timed_order_cooldown = 600;
            return;
        }
        let order = generate_timed_order(&self.available_families, self.max_order_tier + 1, 600);
        self.active_orders.push(order);
        self.timed_order_cooldown = 1200;
    }

    // ── Hint pair ─────────────────────────────────────────────────────────

    fn update_hint(&mut self) {
        if !self.blessing_flags.keen_eye {
            self.hint_pair = None;
            return;
        }
        'outer: for r1 in 0..self.board.rows {
            for c1 in 0..self.board.cols {
                if let Cell::Piece(p1) = &self.board.cells[r1][c1] {
                    let p1 = p1.clone();
                    for r2 in 0..self.board.rows {
                        for c2 in 0..self.board.cols {
                            if (r1, c1) == (r2, c2) {
                                continue;
                            }
                            let target = match &self.board.cells[r2][c2] {
                                Cell::Piece(p) | Cell::Frozen(p) => p,
                                _ => continue,
                            };
                            if p1.can_merge(target) {
                                self.hint_pair = Some(((r1, c1), (r2, c2)));
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        if self.hint_pair.is_none() {
            // No merge possible
        }
    }

    // ── Tick ─────────────────────────────────────────────────────────────

    /// Advance game state by one tick. Returns true if the screen should redraw.
    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;
        let mut changed = false;

        // Energy regen
        if self.energy.regen_tick() > 0 {
            changed = true;
        }

        // Generator cooldowns
        generator::tick_cooldowns(&mut self.board);

        // Tick time-limited orders (expire them)
        let before = self.active_orders.len();
        self.active_orders.retain_mut(|o| !o.tick());
        if self.active_orders.len() != before {
            self.fill_random_orders();
            changed = true;
        }

        // Maybe spawn a new timed order
        self.maybe_spawn_timed_order();

        changed
    }

    // ── Status ────────────────────────────────────────────────────────────

    /// Whether the game is stuck (board full, no merges, no energy for generators).
    pub fn is_stuck(&self) -> bool {
        let board_full = self.board.is_full() && !self.board.has_any_merge();
        // Check if any generator could be activated if we had energy/space
        let has_generator = (0..self.board.rows)
            .any(|r| (0..self.board.cols).any(|c| self.board.cells[r][c].is_any_generator()));
        let has_empty = self.board.empty_count() > 0;

        board_full && !(has_generator && has_empty && self.energy.current > 0)
    }

    pub fn total_score(&self) -> u32 {
        self.score
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

pub fn blessings_has(ids: &[String], id: &str) -> bool {
    ids.iter().any(|s| s == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardLayout;
    use crate::board::CellInit;

    fn test_engine() -> GameEngine {
        let mut board = Board::new_empty(4, 4);
        board.cells[0][0] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 1,
            cooldown_remaining: 0,
        };
        board.cells[1][1] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[1][2] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 1)));
        board.cells[2][2] = Cell::Frozen(Piece::Regular(Item::new(Family::Stone, 2)));
        board.cells[2][3] = Cell::Piece(Piece::Regular(Item::new(Family::Stone, 2)));

        GameEngine::from_state(
            board,
            Inventory::new(4),
            Energy::new(100, 30),
            Vec::new(),
            0,
            0,
            0,
            0,
            1,
            5,
            2,
            4,
            1,
            0,
            25,
            vec![Family::Wood, Family::Stone, Family::Metal],
            &[],
        )
    }

    #[test]
    fn cursor_movement() {
        let mut e = test_engine();
        assert!(e.move_cursor(0, 1));
        assert_eq!(e.cursor_col, 1);
        assert!(!e.move_cursor(-1, 0)); // out of bounds
    }

    #[test]
    fn select_and_deselect() {
        let mut e = test_engine();
        e.cursor_row = 1;
        e.cursor_col = 1;
        assert!(e.activate()); // select
        assert_eq!(e.selected, Some((1, 1)));
        assert!(e.activate()); // deselect
        assert_eq!(e.selected, None);
    }

    #[test]
    fn merge_regular_pieces() {
        let mut e = test_engine();
        e.cursor_row = 1;
        e.cursor_col = 1;
        e.activate(); // select (1,1)
        e.cursor_row = 1;
        e.cursor_col = 2;
        e.activate(); // merge into (1,2)
        assert!(e.board.cells[1][1].is_empty());
        if let Cell::Piece(Piece::Regular(item)) = &e.board.cells[1][2] {
            assert_eq!(item.tier, 2);
            assert_eq!(item.family, Family::Wood);
        } else {
            panic!("Expected tier-2 Wood at (1,2)");
        }
        assert!(e.total_merges > 0);
    }

    #[test]
    fn merge_into_frozen_thaws() {
        let mut e = test_engine();
        e.cursor_row = 2;
        e.cursor_col = 3;
        e.activate(); // select (2,3) Stone T2
        e.cursor_row = 2;
        e.cursor_col = 2;
        e.activate(); // merge into frozen (2,2) Stone T2
        assert!(e.board.cells[2][3].is_empty());
        assert!(e.board.cells[2][2].is_piece()); // thawed
        assert!(e.cells_thawed > 0);
    }

    #[test]
    fn non_adjacent_merge_works() {
        let mut board = Board::new_empty(5, 5);
        board.cells[0][0] = Cell::Piece(Piece::Regular(Item::new(Family::Crystal, 3)));
        board.cells[4][4] = Cell::Piece(Piece::Regular(Item::new(Family::Crystal, 3)));
        let mut e = GameEngine::from_state(
            board, Inventory::new(4), Energy::new(100, 30),
            Vec::new(), 0, 0, 0, 0, 1, 5, 0, 4, 1, 0, 0,
            vec![Family::Crystal], &[],
        );
        e.cursor_row = 0; e.cursor_col = 0;
        e.activate();
        e.cursor_row = 4; e.cursor_col = 4;
        e.activate(); // merge non-adjacent
        assert!(e.board.cells[0][0].is_empty());
        if let Cell::Piece(Piece::Regular(item)) = &e.board.cells[4][4] {
            assert_eq!(item.tier, 4);
        } else {
            panic!("Expected Crystal T4 at (4,4)");
        }
    }

    #[test]
    fn generator_activation() {
        let mut e = test_engine();
        e.cursor_row = 0;
        e.cursor_col = 0;
        e.activate(); // activate generator
        assert_eq!(e.energy.current, 99);
    }

    #[test]
    fn generator_no_energy_fails() {
        let mut e = test_engine();
        e.energy.current = 0;
        e.cursor_row = 0;
        e.cursor_col = 0;
        let changed = e.activate();
        assert!(!changed);
        assert!(e.notifications.iter().any(|n| matches!(n, Notification::NoEnergy)));
    }

    #[test]
    fn store_to_inventory() {
        let mut e = test_engine();
        e.cursor_row = 1;
        e.cursor_col = 1;
        e.activate(); // select
        let stored = e.store_selected_to_inventory();
        assert!(stored);
        assert!(e.board.cells[1][1].is_empty());
        assert_eq!(e.inventory.used_count(), 1);
    }

    #[test]
    fn delivery() {
        let mut e = test_engine();
        // Add a story order for Wood T2
        e.active_orders.push(crate::order::Order {
            order_type: crate::order::OrderType::Story,
            requirements: vec![crate::order::OrderRequirement::new(Family::Wood, 2, 1)],
            rewards: vec![crate::order::Reward::Score(100)],
        });
        // Create a Wood T2 on board
        e.board.cells[3][3] = Cell::Piece(Piece::Regular(Item::new(Family::Wood, 2)));
        e.cursor_row = 3;
        e.cursor_col = 3;
        e.activate(); // select
        let ok = e.deliver_from_board();
        assert!(ok);
        assert!(e.board.cells[3][3].is_empty());
    }

    #[test]
    fn blueprint_merge_creates_generator() {
        let mut board = Board::new_empty(3, 3);
        board.cells[0][0] = Cell::Piece(Piece::Blueprint(Family::Metal));
        board.cells[2][2] = Cell::Piece(Piece::Blueprint(Family::Metal));
        let mut e = GameEngine::from_state(
            board, Inventory::new(4), Energy::new(100, 30),
            Vec::new(), 0, 0, 0, 0, 1, 5, 0, 4, 1, 0, 0,
            vec![Family::Metal], &[],
        );
        e.cursor_row = 0; e.cursor_col = 0;
        e.activate();
        e.cursor_row = 2; e.cursor_col = 2;
        e.activate();
        assert!(e.board.cells[0][0].is_empty());
        assert!(e.board.cells[2][2].is_hard_generator());
    }

    #[test]
    fn tick_regens_energy_over_time() {
        let mut e = test_engine();
        e.energy.current = 50;
        // Force last_regen_epoch to be in the past by many intervals
        e.energy.last_regen_epoch = 0;
        e.tick();
        assert!(e.energy.current > 50);
    }

    #[test]
    fn random_orders_filled_on_start() {
        let e = test_engine();
        // Should have random_order_count random orders
        let random_count = e
            .active_orders
            .iter()
            .filter(|o| matches!(o.order_type, crate::order::OrderType::Random))
            .count();
        assert_eq!(random_count, e.random_order_count);
    }
}
