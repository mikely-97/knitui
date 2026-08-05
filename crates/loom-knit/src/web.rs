// wasm-bindgen entry point wiring loom-knit's GameEngine to a canvas via
// loom-engine-web. Deliberately minimal: drives straight into gameplay
// (mirrors the existing `--skip-menu` CLI path), no menu/campaign/blessing
// screens -- those are Phase 3+ (Shell<G>) territory, not this crate's job.

use clap::Parser;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, KeyboardEvent};

use loom_engine::input::Key;
use loom_engine::render::{Style, Surface};
use loom_engine_web::{key_from_event, WasmSurface};

use crate::board_entity::Direction;
use crate::config::Config;
use crate::engine::GameEngine;
use crate::renderer::{self, COMP_GAP, YARN_VGAP};

#[wasm_bindgen]
pub struct WebGame {
    engine: GameEngine,
    surface: WasmSurface,
    board_y: u16,
    scale: u16,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> WebGame {
        loom_engine_web::init_panic_hook();

        let config = Config::parse_from::<[&str; 0], &str>([]);
        let engine = GameEngine::new(&config);
        let surface = WasmSurface::new(ctx, cols, rows, font_px);

        let sh = config.scale;
        let yarn_h = config.visible_stitches * sh
            + config.visible_stitches.saturating_sub(1) * YARN_VGAP;
        let board_y = yarn_h + COMP_GAP + sh + COMP_GAP;

        WebGame { engine, surface, board_y, scale: config.scale }
    }

    /// Feed a browser `KeyboardEvent` straight to the engine. Unrecognized
    /// keys (per the deliberately minimal `Key` enum) are ignored.
    pub fn handle_key(&mut self, event: KeyboardEvent) {
        let Some(key_event) = key_from_event(&event) else { return };
        match key_event.key {
            Key::Up => { let _ = self.engine.move_cursor(Direction::Up); }
            Key::Down => { let _ = self.engine.move_cursor(Direction::Down); }
            Key::Left => { let _ = self.engine.move_cursor(Direction::Left); }
            Key::Right => { let _ = self.engine.move_cursor(Direction::Right); }
            Key::Enter => { let _ = self.engine.pick_up(); }
            _ => {}
        }
    }

    /// Redraw the full frame. The caller (JS `requestAnimationFrame` loop)
    /// decides when to call this -- the engine itself has no frame timing.
    pub fn render(&mut self) {
        self.surface.clear(Style::default());
        renderer::render_vertical_to_surface(&mut self.surface, &self.engine, self.board_y, self.scale);
    }
}
