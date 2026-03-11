use crossterm::style::Color;
use serde::{Deserialize, Serialize};

/// A single required item within an order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderItem {
    #[serde(with = "loom_engine::color_serde")]
    pub color: Color,
    pub tier: u8,
    pub required: u16,
    pub delivered: u16,
}

impl OrderItem {
    pub fn new(color: Color, tier: u8, required: u16) -> Self {
        Self { color, tier, required, delivered: 0 }
    }

    pub fn is_fulfilled(&self) -> bool {
        self.delivered >= self.required
    }

    pub fn remaining(&self) -> u16 {
        self.required.saturating_sub(self.delivered)
    }

    /// Try to deliver an item. Returns true if accepted.
    pub fn try_deliver(&mut self, color: Color, tier: u8) -> bool {
        if self.color == color && self.tier == tier && !self.is_fulfilled() {
            self.delivered += 1;
            true
        } else {
            false
        }
    }
}

/// A client order consisting of one or more required items.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub items: Vec<OrderItem>,
}

impl Order {
    pub fn is_fulfilled(&self) -> bool {
        self.items.iter().all(|i| i.is_fulfilled())
    }

    /// Whether this order accepts an item of the given color and tier.
    pub fn accepts(&self, color: Color, tier: u8) -> bool {
        self.items.iter().any(|i| i.color == color && i.tier == tier && !i.is_fulfilled())
    }

    /// Try to deliver an item to this order. Returns true if accepted.
    pub fn try_deliver(&mut self, color: Color, tier: u8) -> bool {
        for item in &mut self.items {
            if item.try_deliver(color, tier) {
                return true;
            }
        }
        false
    }
}

/// Level definition for an order item (uses color index into palette).
#[derive(Clone, Debug)]
pub struct OrderDef {
    pub color_idx: usize,
    pub tier: u8,
    pub quantity: u16,
}

/// Convert level definitions into runtime orders, mapping color indices to palette colors.
pub fn generate_orders(defs: &[Vec<OrderDef>], palette: &[Color]) -> Vec<Order> {
    defs.iter().map(|order_defs| {
        Order {
            items: order_defs.iter().map(|d| {
                let color = palette[d.color_idx % palette.len()];
                OrderItem::new(color, d.tier, d.quantity)
            }).collect(),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_item_delivery() {
        let mut oi = OrderItem::new(Color::Red, 3, 2);
        assert!(!oi.is_fulfilled());
        assert_eq!(oi.remaining(), 2);
        assert!(oi.try_deliver(Color::Red, 3));
        assert_eq!(oi.remaining(), 1);
        assert!(oi.try_deliver(Color::Red, 3));
        assert!(oi.is_fulfilled());
        assert_eq!(oi.remaining(), 0);
    }

    #[test]
    fn order_item_rejects_wrong_color() {
        let mut oi = OrderItem::new(Color::Red, 2, 1);
        assert!(!oi.try_deliver(Color::Blue, 2));
    }

    #[test]
    fn order_item_rejects_wrong_tier() {
        let mut oi = OrderItem::new(Color::Red, 2, 1);
        assert!(!oi.try_deliver(Color::Red, 3));
    }

    #[test]
    fn order_item_rejects_when_fulfilled() {
        let mut oi = OrderItem::new(Color::Red, 1, 1);
        assert!(oi.try_deliver(Color::Red, 1));
        assert!(!oi.try_deliver(Color::Red, 1));
    }

    #[test]
    fn order_fulfillment() {
        let mut order = Order {
            items: vec![
                OrderItem::new(Color::Red, 2, 1),
                OrderItem::new(Color::Blue, 3, 1),
            ],
        };
        assert!(!order.is_fulfilled());
        assert!(order.accepts(Color::Red, 2));
        assert!(order.try_deliver(Color::Red, 2));
        assert!(!order.is_fulfilled());
        assert!(order.try_deliver(Color::Blue, 3));
        assert!(order.is_fulfilled());
    }

    #[test]
    fn order_does_not_accept_unneeded() {
        let order = Order {
            items: vec![OrderItem::new(Color::Red, 2, 1)],
        };
        assert!(!order.accepts(Color::Green, 2));
        assert!(!order.accepts(Color::Red, 1));
    }

    #[test]
    fn generate_orders_maps_colors() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let defs = vec![
            vec![OrderDef { color_idx: 0, tier: 2, quantity: 1 }],
            vec![OrderDef { color_idx: 1, tier: 3, quantity: 2 }],
        ];
        let orders = generate_orders(&defs, &palette);
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].items[0].color, Color::Red);
        assert_eq!(orders[1].items[0].color, Color::Blue);
        assert_eq!(orders[1].items[0].required, 2);
    }

    #[test]
    fn serde_roundtrip() {
        let order = Order {
            items: vec![OrderItem::new(Color::Red, 3, 2)],
        };
        let json = serde_json::to_string(&order).unwrap();
        let restored: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.items.len(), 1);
        assert_eq!(restored.items[0].color, Color::Red);
        assert_eq!(restored.items[0].tier, 3);
    }
}
