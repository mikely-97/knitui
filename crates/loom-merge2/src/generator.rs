use rand::prelude::*;

use crate::board::{Board, Cell};
use crate::energy::Energy;
#[allow(unused_imports)]
use crate::item::{Family, Item, Piece, MAX_TIER};

/// Result of trying to activate a generator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActivationResult {
    /// Item spawned at this position.
    Spawned(usize, usize),
    /// Not enough energy.
    NoEnergy,
    /// Generator is on cooldown.
    OnCooldown,
    /// No empty cell on the board to spawn into.
    NoSpace,
    /// Cell at position is not a generator.
    NotAGenerator,
    /// Soft generator exhausted (was already at 0 charges).
    Exhausted,
}

/// Compute the tier of the item to spawn from a generator.
///
/// Base tier equals the generator's tier. Bonus tiers:
///   +1: ~10%  (decreases slightly per generator tier)
///   +2: ~2%
///   +3: ~0.5%
///
/// If `enhanced` is true (player spent 2× energy), the result is bumped +1.
/// If `surge_bonus` is true (generator_surge blessing), effective gen tier is +1.
pub fn spawn_tier(gen_tier: u8, enhanced: bool, surge_bonus: bool) -> u8 {
    let mut rng = rand::rng();
    let t = if surge_bonus { gen_tier.saturating_add(1) } else { gen_tier };
    let t = t.saturating_sub(1) as u32;

    // Thresholds in 1000-scale. They decrease slightly with generator tier
    // so upgrading the generator shifts the base rather than stacking bonuses.
    let thresh3 = 5u32.saturating_sub(t);           // 0.5% → approaches 0 at high tiers
    let thresh2 = 20u32.saturating_sub(t * 2);      // 2%
    let thresh1 = 100u32.saturating_sub(t * 5);     // 10% → approaches 5% at high tiers

    let roll = rng.random_range(0u32..1000);
    let bonus: u8 = if roll < thresh3 { 3 }
        else if roll < thresh2 { 2 }
        else if roll < thresh1 { 1 }
        else { 0 };

    let extra: u8 = if enhanced { 1 } else { 0 };
    let base = if surge_bonus { gen_tier + 1 } else { gen_tier };
    (base + bonus + extra).min(MAX_TIER)
}

/// Try to activate a generator at the given position.
///
/// - `energy_cost`: how much energy to spend (1 = normal, 2 = enhanced)
/// - `cooldown_interval`: what to reset the cooldown to after spawning
/// - `enhanced`: player pressed the enhanced-activation key (2× energy, +1 output tier)
/// - `surge_bonus`: generator_surge blessing active (treats gen as +1 tier for spawn)
pub fn try_activate(
    board: &mut Board,
    r: usize,
    c: usize,
    energy: &mut Energy,
    energy_cost: u16,
    cooldown_interval: u32,
    enhanced: bool,
    surge_bonus: bool,
) -> ActivationResult {
    let (family, gen_tier) = match &board.cells[r][c] {
        Cell::HardGenerator { family, tier, cooldown_remaining } => {
            if *cooldown_remaining > 0 {
                return ActivationResult::OnCooldown;
            }
            (*family, *tier)
        }
        Cell::SoftGenerator { family, tier, charges, cooldown_remaining } => {
            if *charges == 0 {
                return ActivationResult::Exhausted;
            }
            if *cooldown_remaining > 0 {
                return ActivationResult::OnCooldown;
            }
            (*family, *tier)
        }
        _ => return ActivationResult::NotAGenerator,
    };

    // Check for any empty cell before spending energy
    let spawn_pos = match board.find_empty_adjacent(r, c) {
        Some(pos) => pos,
        None => return ActivationResult::NoSpace,
    };

    // Spend energy (enhanced costs 2×)
    let total_cost = if enhanced { energy_cost.saturating_mul(2) } else { energy_cost };
    if !energy.spend(total_cost) {
        return ActivationResult::NoEnergy;
    }

    // Determine spawned tier using probabilistic system
    let tier = spawn_tier(gen_tier, enhanced, surge_bonus);

    // Spawn the item
    board.cells[spawn_pos.0][spawn_pos.1] = Cell::Piece(Piece::Regular(Item::new(family, tier)));

    // Update generator state
    match &mut board.cells[r][c] {
        Cell::HardGenerator { cooldown_remaining, .. } => {
            *cooldown_remaining = cooldown_interval;
        }
        Cell::SoftGenerator { charges, cooldown_remaining, .. } => {
            *cooldown_remaining = cooldown_interval;
            *charges = charges.saturating_sub(1);
            if *charges == 0 {
                board.cells[r][c] = Cell::Empty;
            }
        }
        _ => {}
    }

    ActivationResult::Spawned(spawn_pos.0, spawn_pos.1)
}

/// Tick all generator cooldowns down by 1.
pub fn tick_cooldowns(board: &mut Board) {
    for r in 0..board.rows {
        for c in 0..board.cols {
            match &mut board.cells[r][c] {
                Cell::HardGenerator { cooldown_remaining, .. } => {
                    *cooldown_remaining = cooldown_remaining.saturating_sub(1);
                }
                Cell::SoftGenerator { cooldown_remaining, .. } => {
                    *cooldown_remaining = cooldown_remaining.saturating_sub(1);
                }
                _ => {}
            }
        }
    }
}

/// Try to create a soft generator from a high-tier merge result.
/// Returns true if the result should become a soft generator instead.
pub fn should_create_soft_generator(merged_tier: u8, chance_percent: u8) -> bool {
    if merged_tier < 7 || chance_percent == 0 {
        return false;
    }
    let mut rng = rand::rng();
    rng.random_range(0u8..100) < chance_percent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    fn board_with_hard_gen() -> (Board, Energy) {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 1,
            cooldown_remaining: 0,
        };
        let energy = Energy::new(100, 30);
        (board, energy)
    }

    #[test]
    fn activate_hard_generator() {
        let (mut board, mut energy) = board_with_hard_gen();
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert!(matches!(result, ActivationResult::Spawned(_, _)));
        assert!(energy.current <= 99); // at most 1 spent (2 if enhanced, but enhanced=false)
    }

    #[test]
    fn activate_no_energy() {
        let (mut board, mut energy) = board_with_hard_gen();
        energy.current = 0;
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert_eq!(result, ActivationResult::NoEnergy);
    }

    #[test]
    fn activate_on_cooldown() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 1,
            cooldown_remaining: 3,
        };
        let mut energy = Energy::new(100, 30);
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert_eq!(result, ActivationResult::OnCooldown);
    }

    #[test]
    fn activate_no_space() {
        let mut board = Board::new_empty(1, 2);
        board.cells[0][0] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 1,
            cooldown_remaining: 0,
        };
        board.cells[0][1] = Cell::Piece(Piece::Regular(Item::new(Family::Stone, 1)));
        let mut energy = Energy::new(100, 30);
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert_eq!(result, ActivationResult::NoSpace);
        assert_eq!(energy.current, 100);
    }

    #[test]
    fn activate_soft_generator_consumes_charge() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::SoftGenerator {
            family: Family::Metal,
            tier: 1,
            charges: 2,
            cooldown_remaining: 0,
        };
        let mut energy = Energy::new(100, 30);
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert!(matches!(result, ActivationResult::Spawned(_, _)));
        if let Cell::SoftGenerator { charges, .. } = &board.cells[0][0] {
            assert_eq!(*charges, 1);
        } else {
            panic!("Expected soft generator");
        }
    }

    #[test]
    fn soft_generator_disappears_on_last_charge() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::SoftGenerator {
            family: Family::Metal,
            tier: 1,
            charges: 1,
            cooldown_remaining: 0,
        };
        let mut energy = Energy::new(100, 30);
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert!(matches!(result, ActivationResult::Spawned(_, _)));
        assert!(board.cells[0][0].is_empty());
    }

    #[test]
    fn activate_not_a_generator() {
        let mut board = Board::new_empty(2, 2);
        let mut energy = Energy::new(100, 30);
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        assert_eq!(result, ActivationResult::NotAGenerator);
    }

    #[test]
    fn tick_cooldowns_decrements() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 1,
            cooldown_remaining: 3,
        };
        board.cells[1][1] = Cell::SoftGenerator {
            family: Family::Metal,
            tier: 1,
            charges: 5,
            cooldown_remaining: 2,
        };
        tick_cooldowns(&mut board);
        if let Cell::HardGenerator { cooldown_remaining, .. } = &board.cells[0][0] {
            assert_eq!(*cooldown_remaining, 2);
        }
        if let Cell::SoftGenerator { cooldown_remaining, .. } = &board.cells[1][1] {
            assert_eq!(*cooldown_remaining, 1);
        }
    }

    #[test]
    fn enhanced_activation_costs_double_energy() {
        let (mut board, mut energy) = board_with_hard_gen();
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, true, false);
        assert!(matches!(result, ActivationResult::Spawned(_, _)));
        assert_eq!(energy.current, 98); // 2 energy spent
    }

    #[test]
    fn t2_generator_spawns_at_least_t2() {
        let mut board = Board::new_empty(2, 2);
        board.cells[0][0] = Cell::HardGenerator {
            family: Family::Wood,
            tier: 2,
            cooldown_remaining: 0,
        };
        let mut energy = Energy::new(100, 30);
        let result = try_activate(&mut board, 0, 0, &mut energy, 1, 0, false, false);
        let (sr, sc) = match result {
            ActivationResult::Spawned(r, c) => (r, c),
            other => panic!("Expected Spawned, got {:?}", other),
        };
        if let Cell::Piece(Piece::Regular(item)) = &board.cells[sr][sc] {
            assert!(item.tier >= 2, "T2 generator should spawn T2+ item, got T{}", item.tier);
        } else {
            panic!("Expected a piece at spawn position");
        }
    }
}
