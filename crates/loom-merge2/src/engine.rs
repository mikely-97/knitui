use crossterm::style::Color;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use loom_engine::palette::{select_palette, ColorMode};

use crate::blessings;
use crate::board::{Board, Cell};
use crate::campaign_levels::levels_for_track;
use crate::config::Config;
use crate::item::Item;
use crate::order::{Order, OrderItem, generate_orders};

// ── Blessing flags ───────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BlessingFlags {
    pub keen_eye: bool,
    pub lucky_start: bool,
    pub generous_orders: bool,
    pub chain_merge: bool,
    pub tier_boost: bool,
    pub double_deliver: bool,
    pub golden_generator: bool,
    pub last_resort: bool,
    pub last_resort_used: bool,
}

// ── Game status ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    Won,
    Lost,
    /// Board is full, no merges, but ads are available.
    Stuck,
}

// ── Engine ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameEngine {
    pub board: Board,
    pub orders: Vec<Order>,
    #[serde(with = "palette_serde")]
    pub palette: Vec<Color>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub selected: Option<(usize, usize)>,
    pub score: u32,
    pub scale: u16,
    pub tick_count: u32,
    pub ads_used: u16,
    pub ad_limit: u16,
    #[serde(default)]
    pub blessing_flags: BlessingFlags,
}

impl GameEngine {
    /// Create a new engine from config.
    pub fn new(config: &Config) -> Self {
        let color_mode = match config.color_mode.as_str() {
            "bright" | "bright-rgb" => ColorMode::Bright,
            "colorblind" | "colorblind-rgb" => ColorMode::Colorblind,
            _ => ColorMode::Dark,
        };
        let palette = select_palette(color_mode, config.color_count);

        let board = Board::make_random(
            config.board_height as usize,
            config.board_width as usize,
            &palette,
            config.generator_count as usize,
            config.generator_charges,
            config.generator_interval,
            config.blocked_cells as usize,
        );

        // Generate random orders based on config
        let orders = Self::generate_random_orders(config, &palette);

        Self {
            board,
            orders,
            palette,
            cursor_row: 0,
            cursor_col: 0,
            selected: None,
            score: 0,
            scale: config.scale,
            tick_count: 0,
            ads_used: 0,
            ad_limit: config.ad_limit,
            blessing_flags: BlessingFlags::default(),
        }
    }

    /// Create engine for a campaign level (uses level-defined orders).
    pub fn new_campaign(config: &Config, track_idx: usize, level_idx: usize) -> Self {
        let color_mode = match config.color_mode.as_str() {
            "bright" | "bright-rgb" => ColorMode::Bright,
            "colorblind" | "colorblind-rgb" => ColorMode::Colorblind,
            _ => ColorMode::Dark,
        };
        let palette = select_palette(color_mode, config.color_count);

        let board = Board::make_random(
            config.board_height as usize,
            config.board_width as usize,
            &palette,
            config.generator_count as usize,
            config.generator_charges,
            config.generator_interval,
            config.blocked_cells as usize,
        );

        let levels = levels_for_track(track_idx);
        let orders = generate_orders(&levels[level_idx].orders, &palette);

        Self {
            board,
            orders,
            palette,
            cursor_row: 0,
            cursor_col: 0,
            selected: None,
            score: 0,
            scale: config.scale,
            tick_count: 0,
            ads_used: 0,
            ad_limit: config.ad_limit,
            blessing_flags: BlessingFlags::default(),
        }
    }

    /// Populate blessing flags from a list of blessing IDs.
    pub fn set_blessings(&mut self, ids: &[String]) {
        self.blessing_flags = BlessingFlags {
            keen_eye: blessings::has(ids, "keen_eye"),
            lucky_start: blessings::has(ids, "lucky_start"),
            generous_orders: blessings::has(ids, "generous_orders"),
            chain_merge: blessings::has(ids, "chain_merge"),
            tier_boost: blessings::has(ids, "tier_boost"),
            double_deliver: blessings::has(ids, "double_deliver"),
            golden_generator: blessings::has(ids, "golden_generator"),
            last_resort: blessings::has(ids, "last_resort"),
            last_resort_used: false,
        };
        // Apply lucky_start: place 1 random T2 item on an empty cell
        if self.blessing_flags.lucky_start {
            let mut rng = rand::rng();
            let mut empties: Vec<(usize, usize)> = Vec::new();
            for r in 0..self.board.height {
                for c in 0..self.board.width {
                    if self.board.cells[r][c].is_empty() {
                        empties.push((r, c));
                    }
                }
            }
            if let Some(&(r, c)) = empties.choose(&mut rng) {
                let color = *self.palette.choose(&mut rng).unwrap();
                self.board.cells[r][c] = Cell::Item(Item::new(color, 2));
            }
        }
        // Apply generous_orders: reduce each order item quantity by 1 (min 1)
        if self.blessing_flags.generous_orders {
            for order in &mut self.orders {
                for oi in &mut order.items {
                    if oi.required > 1 {
                        oi.required -= 1;
                    }
                }
            }
        }
    }

    fn generate_random_orders(config: &Config, palette: &[Color]) -> Vec<Order> {
        let mut rng = rand::rng();
        let mut orders = Vec::new();
        for _ in 0..config.order_count {
            let color = *palette.choose(&mut rng).unwrap();
            let tier = rng.random_range(2..=config.max_order_tier);
            let quantity = rng.random_range(1..=2);
            orders.push(Order {
                items: vec![OrderItem::new(color, tier, quantity)],
            });
        }
        orders
    }

    /// Current game status.
    pub fn status(&mut self) -> GameStatus {
        if self.orders.iter().all(|o| o.is_fulfilled()) {
            return GameStatus::Won;
        }
        if self.board.is_full() && !self.board.has_any_merge() {
            if self.can_watch_ad() {
                return GameStatus::Stuck;
            }
            // last_resort: when stuck with no ads, clear 2 items once
            if self.blessing_flags.last_resort && !self.blessing_flags.last_resort_used {
                self.blessing_flags.last_resort_used = true;
                self.board.clear_random_items(2);
                return GameStatus::Playing;
            }
            return GameStatus::Lost;
        }
        GameStatus::Playing
    }

    /// Whether the player can watch an ad.
    pub fn can_watch_ad(&self) -> bool {
        self.ad_limit > 0 && self.ads_used < self.ad_limit
    }

    /// Watch an ad: clear 3 random items from the board.
    pub fn watch_ad(&mut self) {
        if !self.can_watch_ad() { return; }
        self.board.clear_random_items(3);
        self.ads_used += 1;
    }

    /// Move cursor in a direction. Returns true if moved.
    pub fn move_cursor(&mut self, dr: i32, dc: i32) -> bool {
        let nr = self.cursor_row as i32 + dr;
        let nc = self.cursor_col as i32 + dc;
        if nr >= 0 && nr < self.board.height as i32 && nc >= 0 && nc < self.board.width as i32 {
            self.cursor_row = nr as usize;
            self.cursor_col = nc as usize;
            true
        } else {
            false
        }
    }

    /// Handle Enter/Space press — select, merge, or reselect.
    pub fn activate(&mut self) -> bool {
        let pos = (self.cursor_row, self.cursor_col);

        if let Some(sel) = self.selected {
            if sel == pos {
                // Deselect
                self.selected = None;
                return true;
            }
            if self.board.can_merge(sel, pos) {
                // Merge!
                if let Some(mut merged) = self.board.do_merge(sel, pos) {
                    // tier_boost: 15% chance to skip a tier
                    if self.blessing_flags.tier_boost && merged.tier < crate::item::MAX_TIER {
                        let mut rng = rand::rng();
                        if rng.random_range(0u8..100) < 15 {
                            merged.tier += 1;
                            // Update the item on the board
                            self.board.cells[pos.0][pos.1] = Cell::Item(merged.clone());
                        }
                    }
                    self.score += merged.score_value();
                    // chain_merge: auto-merge result if it matches an adjacent item
                    if self.blessing_flags.chain_merge {
                        self.try_chain_merge(pos);
                    }
                    self.selected = None;
                    return true;
                }
            }
            // Not mergeable — reselect if cursor is on an item
            if self.board.item_at(pos.0, pos.1).is_some() {
                self.selected = Some(pos);
                return true;
            }
            return false;
        }

        // Nothing selected — select if on an item
        if self.board.item_at(pos.0, pos.1).is_some() {
            self.selected = Some(pos);
            return true;
        }
        false
    }

    /// Deliver the selected item to the first matching unfulfilled order.
    pub fn deliver(&mut self) -> bool {
        let Some(sel) = self.selected else { return false; };
        let Some(item) = self.board.item_at(sel.0, sel.1) else { return false; };
        let color = item.color;
        let tier = item.tier;

        // Check if any order accepts this item
        let deliver_count = if self.blessing_flags.double_deliver { 2 } else { 1 };
        for order in &mut self.orders {
            if order.try_deliver(color, tier) {
                // double_deliver: deliver counts as 2
                if deliver_count > 1 {
                    order.try_deliver(color, tier);
                }
                let score = Item::new(color, tier).score_value() * 2;
                self.score += score;
                self.board.take_item(sel.0, sel.1);
                self.selected = None;

                // Check for order completion bonus
                if order.is_fulfilled() {
                    self.score += 500;
                }
                // Check for all-orders bonus
                if self.orders.iter().all(|o| o.is_fulfilled()) {
                    self.score += 2000;
                }
                return true;
            }
        }
        false
    }

    /// Tick the board (generators spawn). Returns true if anything changed.
    pub fn tick(&mut self) -> bool {
        self.tick_count += 1;
        self.board.tick_generators_with_golden(self.blessing_flags.golden_generator)
    }

    /// Try to chain-merge the item at `pos` with an adjacent matching item.
    fn try_chain_merge(&mut self, pos: (usize, usize)) {
        let (r, c) = pos;
        let neighbors = [
            (r.wrapping_sub(1), c),
            (r + 1, c),
            (r, c.wrapping_sub(1)),
            (r, c + 1),
        ];
        for n in neighbors {
            if n.0 < self.board.height && n.1 < self.board.width {
                if self.board.can_merge(pos, n) {
                    if let Some(merged) = self.board.do_merge(pos, n) {
                        self.score += merged.score_value();
                        // Recurse once more (but avoid infinite chains)
                        return;
                    }
                }
            }
        }
    }

    /// Total score.
    pub fn score(&self) -> u32 { self.score }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Serde helper for Vec<Color> using loom_engine::color_serde.
mod palette_serde {
    use crossterm::style::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct ColorWrapper(#[serde(with = "loom_engine::color_serde")] Color);

    pub fn serialize<S: Serializer>(colors: &[Color], s: S) -> Result<S::Ok, S::Error> {
        let wrappers: Vec<ColorWrapper> = colors.iter().map(|&c| ColorWrapper(c)).collect();
        wrappers.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Color>, D::Error> {
        let wrappers: Vec<ColorWrapper> = Vec::deserialize(d)?;
        Ok(wrappers.into_iter().map(|w| w.0).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        use clap::Parser;
        let mut cfg = Config::parse_from::<[&str; 0], &str>([]);
        cfg.board_height = 3;
        cfg.board_width = 3;
        cfg.color_count = 1;
        cfg.generator_count = 1;
        cfg.generator_charges = 0;
        cfg.blocked_cells = 0;
        cfg.order_count = 1;
        cfg.max_order_tier = 2;
        cfg.ad_limit = 3;
        cfg
    }

    #[test]
    fn new_creates_valid_engine() {
        let mut e = GameEngine::new(&test_config());
        assert_eq!(e.board.height, 3);
        assert_eq!(e.board.width, 3);
        assert!(!e.orders.is_empty());
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn cursor_movement() {
        let mut e = GameEngine::new(&test_config());
        assert!(e.move_cursor(0, 1));
        assert_eq!(e.cursor_col, 1);
        assert!(e.move_cursor(1, 0));
        assert_eq!(e.cursor_row, 1);
        // Out of bounds
        e.cursor_row = 0; e.cursor_col = 0;
        assert!(!e.move_cursor(-1, 0));
        assert!(!e.move_cursor(0, -1));
    }

    #[test]
    fn select_and_deselect() {
        let mut e = GameEngine::new(&test_config());
        // Find a cell with an item
        let mut found = false;
        for r in 0..e.board.height {
            for c in 0..e.board.width {
                if e.board.item_at(r, c).is_some() {
                    e.cursor_row = r;
                    e.cursor_col = c;
                    found = true;
                    break;
                }
            }
            if found { break; }
        }
        assert!(found);
        assert!(e.activate()); // select
        assert!(e.selected.is_some());
        assert!(e.activate()); // deselect (same pos)
        assert!(e.selected.is_none());
    }

    #[test]
    fn can_watch_ad_respects_limit() {
        let mut e = GameEngine::new(&test_config());
        assert!(e.can_watch_ad());
        for _ in 0..3 {
            e.watch_ad();
        }
        assert!(!e.can_watch_ad());
    }

    #[test]
    fn deliver_scores_and_removes_item() {
        let palette = vec![Color::Red];
        let mut e = GameEngine {
            board: Board {
                cells: vec![vec![Cell::Item(Item::new(Color::Red, 2)), Cell::Empty]],
                height: 1, width: 2,
            },
            orders: vec![Order {
                items: vec![OrderItem::new(Color::Red, 2, 1)],
            }],
            palette,
            cursor_row: 0, cursor_col: 0,
            selected: Some((0, 0)),
            score: 0, scale: 1, tick_count: 0,
            ads_used: 0, ad_limit: 3,
            blessing_flags: BlessingFlags::default(),
        };
        assert!(e.deliver());
        assert!(e.board.cells[0][0].is_empty());
        assert!(e.score > 0);
        assert!(e.orders[0].is_fulfilled());
    }

    #[test]
    fn won_when_all_orders_fulfilled() {
        let palette = vec![Color::Red];
        let mut e = GameEngine {
            board: Board {
                cells: vec![vec![Cell::Item(Item::new(Color::Red, 2)), Cell::Empty]],
                height: 1, width: 2,
            },
            orders: vec![Order {
                items: vec![OrderItem::new(Color::Red, 2, 1)],
            }],
            palette,
            cursor_row: 0, cursor_col: 0,
            selected: Some((0, 0)),
            score: 0, scale: 1, tick_count: 0,
            ads_used: 0, ad_limit: 0,
            blessing_flags: BlessingFlags::default(),
        };
        e.deliver();
        assert_eq!(e.status(), GameStatus::Won);
    }

    #[test]
    fn json_roundtrip() {
        let e = GameEngine::new(&test_config());
        let json = e.to_json();
        let restored = GameEngine::from_json(&json).unwrap();
        assert_eq!(restored.board.height, e.board.height);
        assert_eq!(restored.score, e.score);
    }
}
