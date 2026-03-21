use rand::prelude::*;

use crate::board::Cell;
use crate::generator::{self, ActivationResult};
use crate::item::Family;

use super::{GameEngine, Notification};

impl GameEngine {
    pub(super) fn activate_generator_inner(&mut self, pos: (usize, usize), enhanced: bool) -> bool {
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
                // Spawn rise animation (starts at frame 2 — shorter than merge)
                self.anim_cells.rise_brief((r, c));
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

    // ── Generator upgrade ─────────────────────────────────────────────────

    /// Upgrade the generator at the cursor position (costs stars).
    /// Returns true if upgraded successfully.
    pub fn upgrade_generator_at_cursor(&mut self) -> bool {
        let pos = (self.cursor_row, self.cursor_col);
        let upgrade_cost: u16 = 3;

        match &self.board.cells[pos.0][pos.1] {
            Cell::HardGenerator { family, tier, cooldown_remaining, upgrade_level } => {
                if *upgrade_level >= 2 { return false; }
                if self.stars < upgrade_cost { return false; }
                let new_level = upgrade_level + 1;
                let (fam, t, cd) = (*family, *tier, *cooldown_remaining);
                self.stars -= upgrade_cost;
                self.board.cells[pos.0][pos.1] = Cell::HardGenerator {
                    family: fam, tier: t, cooldown_remaining: cd, upgrade_level: new_level,
                };
                self.notifications.push(Notification::GeneratorUpgraded { pos, level: new_level });
                true
            }
            Cell::SoftGenerator { family, tier, charges, cooldown_remaining, upgrade_level } => {
                if *upgrade_level >= 2 { return false; }
                if self.stars < upgrade_cost { return false; }
                let new_level = upgrade_level + 1;
                let (fam, t, ch, cd) = (*family, *tier, *charges, *cooldown_remaining);
                self.stars -= upgrade_cost;
                self.board.cells[pos.0][pos.1] = Cell::SoftGenerator {
                    family: fam, tier: t, charges: ch, cooldown_remaining: cd,
                    upgrade_level: new_level,
                };
                self.notifications.push(Notification::GeneratorUpgraded { pos, level: new_level });
                true
            }
            _ => false,
        }
    }
}
