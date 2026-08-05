// wasm-bindgen entry point wiring picross's GameEngine to a canvas via
// loom-engine-web. Deliberately minimal: drives straight into the first
// puzzle, no menu/track-select screens (Phase 3+ Shell<G> territory).

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, KeyboardEvent};

use loom_engine::input::Key;
use loom_engine::render::{Style, Surface};
use loom_engine_web::{key_from_event, WasmSurface};

use crate::engine::GameEngine;
use crate::puzzles::all_puzzles;
use crate::renderer;

#[wasm_bindgen]
pub struct WebGame {
    engine: GameEngine,
    surface: WasmSurface,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> WebGame {
        loom_engine_web::init_panic_hook();

        let puzzle = all_puzzles().into_iter().next().expect("at least one built-in puzzle");
        let engine = GameEngine::new(puzzle);
        let surface = WasmSurface::new(ctx, cols, rows, font_px);

        WebGame { engine, surface }
    }

    pub fn handle_key(&mut self, event: KeyboardEvent) {
        let Some(key_event) = key_from_event(&event) else { return };
        match key_event.key {
            Key::Up => self.engine.move_cursor(-1, 0),
            Key::Down => self.engine.move_cursor(1, 0),
            Key::Left => self.engine.move_cursor(0, -1),
            Key::Right => self.engine.move_cursor(0, 1),
            Key::Enter => self.engine.toggle_fill(),
            Key::Char(' ') => self.engine.toggle_fill(),
            Key::Char('x') | Key::Char('X') => self.engine.toggle_cross(),
            _ => {}
        }
    }

    pub fn render(&mut self) {
        self.surface.clear(Style::default());
        renderer::render_inner(&mut self.surface, &self.engine, 2, 2);
    }
}
