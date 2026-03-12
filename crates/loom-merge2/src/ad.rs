use crate::item::Family;

/// Reward from watching an ad.
#[derive(Clone, Debug)]
pub enum AdReward {
    /// Restore a fixed amount of energy.
    Energy(u16),
    /// Fill energy to max (once per session).
    FullEnergy,
    /// Spawn a rare item of the given family and tier.
    RareItem(Family, u8),
    /// Expand inventory by 1 slot (capped at 3 times total via ads).
    InventoryExpand,
    /// Replace all active random orders with fresh ones.
    OrderRefresh,
}

impl AdReward {
    pub fn label(&self) -> String {
        match self {
            AdReward::Energy(n) => format!("+{}⚡", n),
            AdReward::FullEnergy => "Full⚡".to_string(),
            AdReward::RareItem(fam, tier) => {
                format!("+{} T{}", fam.name(), tier)
            }
            AdReward::InventoryExpand => "+1 Inv Slot".to_string(),
            AdReward::OrderRefresh => "Refresh Orders".to_string(),
        }
    }
}

/// Cycle of ad rewards (rotates each use).
pub const AD_REWARD_CYCLE: &[(AdReward, &str)] = &[
    // tuple: (reward, hint shown in HUD)
];

/// Generate the ad reward for a given use index (0-based, cycles).
pub fn reward_for_use(use_idx: u16, available_families: &[Family]) -> AdReward {
    use rand::prelude::*;
    let mut rng = rand::rng();

    match use_idx % 5 {
        0 => AdReward::Energy(20),
        1 => {
            // Rare item: pick a random available family, tier 3
            let family = available_families
                .choose(&mut rng)
                .copied()
                .unwrap_or(Family::Wood);
            AdReward::RareItem(family, 3)
        }
        2 => AdReward::OrderRefresh,
        3 => AdReward::InventoryExpand,
        _ => AdReward::Energy(30),
    }
}

/// Notification label shown in the HUD when an ad is available.
pub fn hud_label(reward: &AdReward) -> String {
    format!("[AD] {} — press A", reward.label())
}
