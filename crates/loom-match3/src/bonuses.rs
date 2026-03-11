#[derive(Debug, Clone)]
pub struct BonusInventory {
    pub hammer:  u16,
    pub laser:   u16,
    pub blaster: u16,
    pub warp:    u16,
}

impl BonusInventory {
    pub fn consume_hammer(&mut self) -> bool {
        if self.hammer > 0 { self.hammer -= 1; true } else { false }
    }
    pub fn consume_laser(&mut self) -> bool {
        if self.laser > 0 { self.laser -= 1; true } else { false }
    }
    pub fn consume_blaster(&mut self) -> bool {
        if self.blaster > 0 { self.blaster -= 1; true } else { false }
    }
    pub fn consume_warp(&mut self) -> bool {
        if self.warp > 0 { self.warp -= 1; true } else { false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BonusState {
    None,
    /// Hammer is active: cursor moves freely to select a target cell.
    /// `saved_*` is the cursor position before hammer was activated,
    /// restored if the player cancels with Esc.
    HammerActive { saved_row: usize, saved_col: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_hammer_decrements() {
        let mut inv = BonusInventory { hammer: 2, laser: 1, blaster: 1, warp: 1 };
        assert!(inv.consume_hammer());
        assert_eq!(inv.hammer, 1);
    }

    #[test]
    fn consume_hammer_fails_at_zero() {
        let mut inv = BonusInventory { hammer: 0, laser: 0, blaster: 0, warp: 0 };
        assert!(!inv.consume_hammer());
    }

    #[test]
    fn consume_laser_decrements() {
        let mut inv = BonusInventory { hammer: 0, laser: 3, blaster: 0, warp: 0 };
        assert!(inv.consume_laser());
        assert_eq!(inv.laser, 2);
    }

    #[test]
    fn consume_blaster_decrements() {
        let mut inv = BonusInventory { hammer: 0, laser: 0, blaster: 2, warp: 0 };
        assert!(inv.consume_blaster());
        assert_eq!(inv.blaster, 1);
    }

    #[test]
    fn consume_warp_decrements() {
        let mut inv = BonusInventory { hammer: 0, laser: 0, blaster: 0, warp: 5 };
        assert!(inv.consume_warp());
        assert_eq!(inv.warp, 4);
    }

    #[test]
    fn bonus_state_hammer_active() {
        let s = BonusState::HammerActive { saved_row: 3, saved_col: 2 };
        assert!(matches!(s, BonusState::HammerActive { .. }));
    }
}
