use serde::{Deserialize, Serialize};

use crate::item::{Family, Piece};
use crate::order::Reward;

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

impl BlessingFlags {
    /// Return true if the blessing identified by `id` is active.
    pub fn has_id(&self, id: &str) -> bool {
        match id {
            "energy_saver"   => self.energy_saver,
            "quick_regen"    => self.quick_regen,
            "keen_eye"       => self.keen_eye,
            "bigger_pockets" => self.bigger_pockets,
            "thaw_aura"      => self.thaw_aura,
            "lucky_orders"   => self.lucky_orders,
            "chain_merge"    => self.chain_merge,
            "tier_boost"     => self.tier_boost,
            "generator_surge"=> self.generator_surge,
            "double_deliver" => self.double_deliver,
            "soft_gen_master"=> self.soft_gen_master,
            "deep_thaw"      => self.deep_thaw,
            _                => false,
        }
    }
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
    BubblePopped { piece: Piece },
    GeneratorUpgraded { pos: (usize, usize), level: u8 },
    InventoryExpansion,
}

// ── Deliver source ────────────────────────────────────────────────────────

/// Where a delivered item came from.
pub enum DeliverSource {
    Board { pos: (usize, usize) },
    Inventory { slot: usize },
}
