// wasm-bindgen entry point driving merge2's full Shell<M2Game> -- menus,
// campaign, endless, options, blessings, help -- through a canvas, via
// loom-engine-web. Full parity with the native terminal build: same
// screens, same state machine, same saves (via WebStorage,
// localStorage-backed, instead of FsStorage's real files).

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, KeyboardEvent};

use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::game::Game;
use loom_engine::render::{Style, Surface};
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;
use loom_engine_web::{key_from_event, WasmSurface, WebStorage};

use crate::campaign::CampaignState;
use crate::config::Config;
use crate::game::M2Game;

#[wasm_bindgen]
pub struct WebGame {
    shell: Shell<M2Game>,
    surface: WasmSurface,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> WebGame {
        loom_engine_web::init_panic_hook();

        let config_dir = M2Game.config_dir();
        let user_settings = UserSettings::load_from(&WebStorage, config_dir);
        let mut cli_config = Config::default();
        cli_config.scale = user_settings.scale;
        cli_config.color_mode = user_settings.color_mode.clone();
        let campaign_saves = CampaignSaves::<CampaignState>::load_from(&WebStorage, config_dir);
        let endless_hs = EndlessHighScore::load_from(&WebStorage, config_dir);

        let shell = Shell::new(
            M2Game,
            cli_config,
            user_settings,
            campaign_saves,
            endless_hs,
            Vec::new(), // no ad quotes wired yet (matches an already-disclosed gap)
            String::new(),
            false, // start at the main menu
            Box::new(WebStorage),
        );

        let surface = WasmSurface::new(ctx, cols, rows, font_px);
        WebGame { shell, surface }
    }

    /// Feed a browser `KeyboardEvent` to the shell. Unrecognized keys (per
    /// the deliberately minimal `Key` enum) are ignored.
    pub fn handle_key(&mut self, event: KeyboardEvent) {
        let Some(key_event) = key_from_event(&event) else { return };
        self.shell.handle_key(key_event);
    }

    /// Advance background/animation state by one tick. The caller (JS
    /// `requestAnimationFrame` loop) decides the cadence.
    pub fn tick(&mut self) {
        self.shell.tick();
    }

    /// True once the player has selected Quit from the main menu.
    pub fn should_quit(&self) -> bool {
        self.shell.should_quit()
    }

    /// Persist any in-progress campaign run. Call before the page
    /// unloads, mirroring the native build's exit sequence.
    pub fn save_on_exit(&mut self) {
        self.shell.save_on_exit();
    }

    /// Redraw the full frame.
    pub fn render(&mut self) {
        self.surface.clear(Style::default());
        self.shell.render(&mut self.surface);
    }
}
