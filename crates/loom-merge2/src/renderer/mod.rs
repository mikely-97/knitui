mod board;
mod panels;
mod popups;

pub use board::render_board;
pub use panels::{render_hud, render_orders, render_inventory, render_key_bar, render_game_over};
pub use popups::{
    render_help, render_main_menu, render_campaign_select, render_level_intro,
    render_ad_overlay, render_options, render_custom_game, render_blessing_selection,
    render_inv_expansion_popup, render_celebration, render_mission_summary,
};

use crossterm::terminal::size as term_size;
use crate::engine::GameEngine;
use crate::glyphs::cell_dims;

// ── Layout ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum Layout { Horizontal, Vertical }

pub struct LayoutGeometry {
    pub layout: Layout,
    pub board_x: u16,
    pub board_y: u16,
    pub order_x: u16,
    pub order_y: u16,
    pub scale: u16,
}

impl LayoutGeometry {
    pub fn compute(engine: &GameEngine) -> Self {
        let scale = engine.scale;
        let (cw, _) = cell_dims(scale);
        let board_w = (engine.board.cols * (cw + 1) + 1) as u16;
        let (term_w, _) = term_size().unwrap_or((80, 24));
        let order_panel_w = 24u16;

        if term_w >= board_w + 3 + order_panel_w {
            LayoutGeometry {
                layout: Layout::Horizontal,
                board_x: 1,
                board_y: 2,
                order_x: board_w + 3,
                order_y: 2,
                scale,
            }
        } else {
            // vertical: orders above board
            let order_rows = 10u16;
            LayoutGeometry {
                layout: Layout::Vertical,
                board_x: 1,
                board_y: 2 + order_rows,
                order_x: 1,
                order_y: 2,
                scale,
            }
        }
    }

    /// Y coordinate of the row below the board (for inventory strip).
    pub fn inventory_y(&self, engine: &GameEngine) -> u16 {
        let (_, ch) = cell_dims(self.scale);
        let board_h = (engine.board.rows * (ch + 1) + 1) as u16;
        self.board_y + board_h
    }

    /// Y coordinate of the key bar (bottom hint line).
    pub fn key_bar_y(&self, engine: &GameEngine) -> u16 {
        self.inventory_y(engine) + 1
    }
}
