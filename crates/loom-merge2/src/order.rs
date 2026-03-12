use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use crate::item::{Family, Item, Piece};

/// Type of order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderType {
    Story,
    Random,
    TimeLimited { ticks_remaining: u32 },
}

/// Reward for completing an order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reward {
    Score(u32),
    Energy(u16),
    SpawnPiece(Piece),
    InventorySlot,
    Stars(u16),
}

/// A single requirement within an order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderRequirement {
    pub family: Family,
    pub tier: u8,
    pub required: u16,
    pub delivered: u16,
}

impl OrderRequirement {
    pub fn new(family: Family, tier: u8, required: u16) -> Self {
        Self {
            family,
            tier,
            required,
            delivered: 0,
        }
    }

    pub fn is_fulfilled(&self) -> bool {
        self.delivered >= self.required
    }

    pub fn remaining(&self) -> u16 {
        self.required.saturating_sub(self.delivered)
    }

    /// Try to deliver a piece. Returns true if accepted.
    pub fn try_deliver(&mut self, piece: &Piece) -> bool {
        if self.is_fulfilled() {
            return false;
        }
        match piece {
            Piece::Regular(item) => {
                if item.family == self.family && item.tier == self.tier {
                    self.delivered += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// A complete order with requirements and rewards.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_type: OrderType,
    pub requirements: Vec<OrderRequirement>,
    pub rewards: Vec<Reward>,
}

impl Order {
    pub fn is_fulfilled(&self) -> bool {
        self.requirements.iter().all(|r| r.is_fulfilled())
    }

    /// Whether this order accepts a given piece.
    pub fn accepts(&self, piece: &Piece) -> bool {
        match piece {
            Piece::Regular(item) => self.requirements.iter().any(|r| {
                r.family == item.family && r.tier == item.tier && !r.is_fulfilled()
            }),
            _ => false,
        }
    }

    /// Try to deliver a piece to the first matching requirement.
    /// Returns true if accepted.
    pub fn try_deliver(&mut self, piece: &Piece) -> bool {
        for req in &mut self.requirements {
            if req.try_deliver(piece) {
                return true;
            }
        }
        false
    }

    /// Tick time-limited orders. Returns true if expired.
    pub fn tick(&mut self) -> bool {
        if let OrderType::TimeLimited { ticks_remaining } = &mut self.order_type {
            *ticks_remaining = ticks_remaining.saturating_sub(1);
            *ticks_remaining == 0
        } else {
            false
        }
    }

    /// Whether this is a time-limited order.
    pub fn is_time_limited(&self) -> bool {
        matches!(self.order_type, OrderType::TimeLimited { .. })
    }

    /// Remaining ticks for time-limited orders.
    pub fn ticks_remaining(&self) -> Option<u32> {
        match &self.order_type {
            OrderType::TimeLimited { ticks_remaining } => Some(*ticks_remaining),
            _ => None,
        }
    }
}

/// Definition for a story order (used in campaign level definitions).
#[derive(Clone, Debug)]
pub struct StoryOrderDef {
    pub requirements: Vec<(Family, u8, u16)>, // (family, tier, quantity)
    pub rewards: Vec<Reward>,
}

impl StoryOrderDef {
    pub fn build(&self) -> Order {
        Order {
            order_type: OrderType::Story,
            requirements: self
                .requirements
                .iter()
                .map(|&(fam, tier, qty)| OrderRequirement::new(fam, tier, qty))
                .collect(),
            rewards: self.rewards.clone(),
        }
    }
}

/// Generate a random order using the given families and max tier.
pub fn generate_random_order(
    available_families: &[Family],
    max_tier: u8,
    lucky_orders: bool,
) -> Order {
    let mut rng = rand::rng();
    let family = *available_families.choose(&mut rng).unwrap_or(&Family::Wood);
    let tier = rng.random_range(1u8..=max_tier.min(4));
    let mut quantity = rng.random_range(1u16..=3);
    if lucky_orders && quantity > 1 {
        quantity -= 1;
    }

    let score_reward = (tier as u32) * 100 * quantity as u32;

    Order {
        order_type: OrderType::Random,
        requirements: vec![OrderRequirement::new(family, tier, quantity)],
        rewards: vec![
            Reward::Score(score_reward),
            Reward::Energy(5 * tier as u16),
        ],
    }
}

/// Generate a time-limited order.
pub fn generate_timed_order(
    available_families: &[Family],
    max_tier: u8,
    duration_ticks: u32,
) -> Order {
    let mut rng = rand::rng();
    let family = *available_families.choose(&mut rng).unwrap_or(&Family::Wood);
    let tier = rng.random_range(2u8..=max_tier.min(5));
    let quantity = rng.random_range(1u16..=2);

    let score_reward = (tier as u32) * 200 * quantity as u32;
    let star_reward = tier as u16;

    Order {
        order_type: OrderType::TimeLimited {
            ticks_remaining: duration_ticks,
        },
        requirements: vec![OrderRequirement::new(family, tier, quantity)],
        rewards: vec![Reward::Score(score_reward), Reward::Stars(star_reward)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wood_order() -> Order {
        Order {
            order_type: OrderType::Story,
            requirements: vec![
                OrderRequirement::new(Family::Wood, 3, 2),
                OrderRequirement::new(Family::Stone, 2, 1),
            ],
            rewards: vec![Reward::Score(500)],
        }
    }

    #[test]
    fn requirement_delivery() {
        let mut req = OrderRequirement::new(Family::Wood, 3, 2);
        let piece = Piece::Regular(Item::new(Family::Wood, 3));
        assert!(req.try_deliver(&piece));
        assert_eq!(req.remaining(), 1);
        assert!(req.try_deliver(&piece));
        assert!(req.is_fulfilled());
        assert!(!req.try_deliver(&piece)); // already fulfilled
    }

    #[test]
    fn requirement_rejects_wrong_family() {
        let mut req = OrderRequirement::new(Family::Wood, 3, 1);
        let piece = Piece::Regular(Item::new(Family::Stone, 3));
        assert!(!req.try_deliver(&piece));
    }

    #[test]
    fn requirement_rejects_wrong_tier() {
        let mut req = OrderRequirement::new(Family::Wood, 3, 1);
        let piece = Piece::Regular(Item::new(Family::Wood, 2));
        assert!(!req.try_deliver(&piece));
    }

    #[test]
    fn requirement_rejects_blueprint() {
        let mut req = OrderRequirement::new(Family::Wood, 1, 1);
        let piece = Piece::Blueprint(Family::Wood);
        assert!(!req.try_deliver(&piece));
    }

    #[test]
    fn order_fulfillment() {
        let mut order = wood_order();
        assert!(!order.is_fulfilled());
        let wood3 = Piece::Regular(Item::new(Family::Wood, 3));
        let stone2 = Piece::Regular(Item::new(Family::Stone, 2));
        assert!(order.try_deliver(&wood3));
        assert!(order.try_deliver(&wood3));
        assert!(!order.is_fulfilled());
        assert!(order.try_deliver(&stone2));
        assert!(order.is_fulfilled());
    }

    #[test]
    fn order_accepts() {
        let order = wood_order();
        assert!(order.accepts(&Piece::Regular(Item::new(Family::Wood, 3))));
        assert!(order.accepts(&Piece::Regular(Item::new(Family::Stone, 2))));
        assert!(!order.accepts(&Piece::Regular(Item::new(Family::Metal, 1))));
    }

    #[test]
    fn time_limited_tick() {
        let mut order = Order {
            order_type: OrderType::TimeLimited {
                ticks_remaining: 3,
            },
            requirements: vec![],
            rewards: vec![],
        };
        assert!(!order.tick());
        assert!(!order.tick());
        assert!(order.tick()); // expired at 0
    }

    #[test]
    fn generate_random_order_valid() {
        let families = vec![Family::Wood, Family::Stone];
        let order = generate_random_order(&families, 4, false);
        assert_eq!(order.requirements.len(), 1);
        assert!(families.contains(&order.requirements[0].family));
    }

    #[test]
    fn story_order_def_build() {
        let def = StoryOrderDef {
            requirements: vec![(Family::Wood, 3, 2), (Family::Stone, 1, 1)],
            rewards: vec![Reward::Score(1000)],
        };
        let order = def.build();
        assert_eq!(order.requirements.len(), 2);
        assert!(matches!(order.order_type, OrderType::Story));
    }

    #[test]
    fn serde_roundtrip() {
        let order = wood_order();
        let json = serde_json::to_string(&order).unwrap();
        let restored: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.requirements.len(), 2);
    }
}
