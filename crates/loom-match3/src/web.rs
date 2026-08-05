// wasm-bindgen entry point wiring match3's GameEngine to a canvas via
// loom-engine-web. Deliberately minimal: drives straight into gameplay
// (mirrors the CLI's default Quick Game config), no menu/campaign/
// blessing screens (Phase 3+ Shell<G> territory).

use clap::Parser;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, KeyboardEvent};

use loom_engine::input::Key;
use loom_engine::render::{Style, Surface};
use loom_engine_web::{key_from_event, WasmSurface};

use crate::config::Config;
use crate::engine::{GameEngine, GamePhase};
use crate::renderer::{self, LayoutGeometry};

#[wasm_bindgen]
pub struct WebGame {
    engine: GameEngine,
    surface: WasmSurface,
    geo: LayoutGeometry,
    frame_count: u32,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> WebGame {
        loom_engine_web::init_panic_hook();

        let config = Config::parse_from::<[&str; 0], &str>([]);
        let engine = GameEngine::new(&config);
        let surface = WasmSurface::new(ctx, cols, rows, font_px);
        let geo = LayoutGeometry::compute(config.board_height as usize, config.board_width as usize, config.scale);

        WebGame { engine, surface, geo, frame_count: 0 }
    }

    pub fn handle_key(&mut self, event: KeyboardEvent) {
        let Some(key_event) = key_from_event(&event) else { return };
        match key_event.key {
            Key::Up => self.engine.move_cursor(-1, 0),
            Key::Down => self.engine.move_cursor(1, 0),
            Key::Left => self.engine.move_cursor(0, -1),
            Key::Right => self.engine.move_cursor(0, 1),
            Key::Enter => {
                if matches!(self.engine.phase, GamePhase::PlayerInput) {
                    self.engine.confirm_selection();
                }
            }
            _ => {}
        }
    }

    /// Redraw the current frame. Also advances match/cascade animation
    /// ticks -- throttled to roughly every 4th call, approximating the
    /// ~50-80ms poll cadence the terminal frontend's tick() calls run at
    /// (this is a rough approximation, not calibrated against real elapsed
    /// time; a future pass could thread requestAnimationFrame's timestamp
    /// through instead).
    pub fn render(&mut self) {
        self.frame_count += 1;
        if self.frame_count % 4 == 0 {
            self.engine.tick();
        }

        self.surface.clear(Style::default());
        renderer::do_render_to_surface(&mut self.surface, &self.engine, &self.geo, "");
    }
}
