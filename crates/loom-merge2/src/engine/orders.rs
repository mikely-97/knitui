use rand::prelude::*;

use crate::board::Cell;
use crate::item::Piece;
use crate::order::{Order, Reward, generate_random_order, generate_timed_order};

use super::{GameEngine, Notification};

impl GameEngine {
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

    pub(super) fn try_deliver_piece(&mut self, piece: &Piece) -> bool {
        let deliver_count = if self.blessing_flags.double_deliver { 2 } else { 1 };

        // Collect delivery result outside the borrow
        let mut found = false;
        let mut fulfilled_rewards: Option<Vec<Reward>> = None;
        let mut follow_up_order: Option<Order> = None;

        for order in &mut self.active_orders {
            if order.accepts(piece) {
                found = true;
                for _ in 0..deliver_count {
                    if !order.try_deliver(piece) {
                        break;
                    }
                }
                if order.is_fulfilled() {
                    fulfilled_rewards = Some(order.rewards.clone());
                    follow_up_order = order.follow_up.take().map(|b| *b);
                }
                break;
            }
        }

        if found {
            if let Piece::Regular(item) = piece {
                self.score += item.score_value() * 2;
            }
            if let Some(rewards) = fulfilled_rewards {
                self.apply_rewards(&rewards);
                self.notifications.push(Notification::OrderCompleted { rewards });
            }
            // Remove fulfilled orders, add follow-up, replenish
            self.active_orders.retain(|o| !o.is_fulfilled());
            if let Some(fu) = follow_up_order {
                self.active_orders.push(fu);
            }
            self.fill_random_orders();
        }
        found
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

    pub(super) fn spawn_piece_anywhere(&mut self, piece: Piece) {
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

    // ── Orders ────────────────────────────────────────────────────────────

    /// Drop all random orders and regenerate them from the current `available_families`.
    /// Call this after overriding `available_families` on a freshly-built engine.
    pub fn regenerate_orders(&mut self) {
        self.active_orders
            .retain(|o| !matches!(o.order_type, crate::order::OrderType::Random));
        self.fill_random_orders();
    }

    pub(super) fn fill_random_orders(&mut self) {
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

    pub(super) fn maybe_spawn_timed_order(&mut self) {
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
}
