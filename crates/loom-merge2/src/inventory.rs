use serde::{Deserialize, Serialize};

use crate::item::Piece;

/// Off-board inventory for storing pieces.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<Piece>>,
    pub max_slots: usize,
}

/// Hard cap on inventory size.
pub const INVENTORY_HARD_CAP: usize = 16;

impl Inventory {
    pub fn new(starting_slots: usize) -> Self {
        let slots = starting_slots.min(INVENTORY_HARD_CAP);
        Self {
            slots: vec![None; slots],
            max_slots: slots,
        }
    }

    /// Number of currently available slots.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Number of occupied slots.
    pub fn used_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Whether the inventory is full (no empty slots).
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    /// Store a piece in the first available slot. Returns false if full.
    pub fn store(&mut self, piece: Piece) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
            *slot = Some(piece);
            true
        } else {
            false
        }
    }

    /// Take a piece from a specific slot index.
    pub fn take(&mut self, slot_idx: usize) -> Option<Piece> {
        self.slots.get_mut(slot_idx).and_then(|s| s.take())
    }

    /// Peek at a slot.
    pub fn peek(&self, slot_idx: usize) -> Option<&Piece> {
        self.slots.get(slot_idx).and_then(|s| s.as_ref())
    }

    /// Expand inventory by `count` slots, up to hard cap.
    pub fn expand(&mut self, count: usize) {
        let new_size = (self.slots.len() + count).min(INVENTORY_HARD_CAP);
        while self.slots.len() < new_size {
            self.slots.push(None);
        }
        self.max_slots = self.slots.len();
    }

    /// Whether expansion is possible.
    pub fn can_expand(&self) -> bool {
        self.slots.len() < INVENTORY_HARD_CAP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::{Family, Item};

    fn wood1() -> Piece {
        Piece::Regular(Item::new(Family::Wood, 1))
    }

    fn stone1() -> Piece {
        Piece::Regular(Item::new(Family::Stone, 1))
    }

    #[test]
    fn store_and_take() {
        let mut inv = Inventory::new(4);
        assert!(inv.store(wood1()));
        assert_eq!(inv.used_count(), 1);
        let p = inv.take(0);
        assert!(p.is_some());
        assert_eq!(inv.used_count(), 0);
    }

    #[test]
    fn store_fills_first_empty() {
        let mut inv = Inventory::new(3);
        inv.store(wood1());
        inv.store(stone1());
        assert_eq!(inv.used_count(), 2);
        inv.take(0);
        inv.store(wood1());
        // Should fill slot 0 again
        assert!(inv.peek(0).is_some());
    }

    #[test]
    fn full_rejects_store() {
        let mut inv = Inventory::new(2);
        assert!(inv.store(wood1()));
        assert!(inv.store(stone1()));
        assert!(inv.is_full());
        assert!(!inv.store(wood1()));
    }

    #[test]
    fn expand_adds_slots() {
        let mut inv = Inventory::new(4);
        inv.expand(2);
        assert_eq!(inv.slot_count(), 6);
    }

    #[test]
    fn expand_capped() {
        let mut inv = Inventory::new(INVENTORY_HARD_CAP);
        inv.expand(5);
        assert_eq!(inv.slot_count(), INVENTORY_HARD_CAP);
        assert!(!inv.can_expand());
    }

    #[test]
    fn serde_roundtrip() {
        let mut inv = Inventory::new(4);
        inv.store(wood1());
        let json = serde_json::to_string(&inv).unwrap();
        let restored: Inventory = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.slot_count(), 4);
        assert_eq!(restored.used_count(), 1);
    }
}
