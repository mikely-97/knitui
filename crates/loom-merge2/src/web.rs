// wasm-bindgen entry point wiring merge2's GameEngine to a canvas via
// loom-engine-web. Deliberately minimal: drives straight into an Endless
// run with no blessings, no menu/campaign/blessing-selection screens
// (Phase 3+ Shell<G> territory).

use clap::Parser;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, KeyboardEvent};

use loom_engine::input::Key;
use loom_engine::render::{Style, Surface};
use loom_engine_web::{key_from_event, WasmSurface};

use crate::config::Config;
use crate::engine::GameEngine;
use crate::renderer::{self, LayoutGeometry};

#[wasm_bindgen]
pub struct WebGame {
    engine: GameEngine,
    surface: WasmSurface,
    geo: LayoutGeometry,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> WebGame {
        loom_engine_web::init_panic_hook();

        let config = Config::parse_from::<[&str; 0], &str>([]);
        let engine = GameEngine::new_endless(&config, &[]);
        let surface = WasmSurface::new(ctx, cols, rows, font_px);
        let geo = LayoutGeometry::compute(&engine);

        WebGame { engine, surface, geo }
    }

    pub fn handle_key(&mut self, event: KeyboardEvent) {
        let Some(key_event) = key_from_event(&event) else { return };
        match key_event.key {
            Key::Up => { self.engine.move_cursor(-1, 0); }
            Key::Down => { self.engine.move_cursor(1, 0); }
            Key::Left => { self.engine.move_cursor(0, -1); }
            Key::Right => { self.engine.move_cursor(0, 1); }
            Key::Enter => { self.engine.activate(); }
            _ => {}
        }
    }

    pub fn render(&mut self) {
        self.engine.tick_anims();
        self.surface.clear(Style::default());
        renderer::do_render_to_surface(&mut self.surface, &self.engine, &self.geo);
    }
}
