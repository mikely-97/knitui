// wasm-bindgen entry point driving loom-knit's full Shell<KnitGame> --
// menus, campaign, endless, options, blessings, help -- through a canvas,
// via loom-engine-web. Full parity with the native terminal build:
// same screens, same state machine, same saves (via WebStorage,
// localStorage-backed, instead of FsStorage's real files).

use clap::Parser;
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
use crate::game::KnitGame;

#[wasm_bindgen]
pub struct WebGame {
    shell: Shell<KnitGame>,
    surface: WasmSurface,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(ctx: CanvasRenderingContext2d, cols: u16, rows: u16, font_px: f64) -> WebGame {
        loom_engine_web::init_panic_hook();

        let config_dir = KnitGame.config_dir();
        let user_settings = UserSettings::load_from(&WebStorage, config_dir);
        let mut cli_config = Config::parse_from::<[&str; 0], &str>([]);
        cli_config.scale = user_settings.scale;
        cli_config.color_mode = user_settings.color_mode.clone();
        let campaign_saves = CampaignSaves::<CampaignState>::load_from(&WebStorage, config_dir);
        let endless_hs = EndlessHighScore::load_from(&WebStorage, config_dir);

        let shell = Shell::new(
            KnitGame,
            cli_config,
            user_settings,
            campaign_saves,
            endless_hs,
            Vec::new(), // no ad quotes wired for web yet -- can_watch_ad() gates
                        // the overlay, so an empty quote list is harmless, not
                        // a broken feature (same as knit's native ad_file gap).
            "FREE SCISSORS".to_string(),
            false, // start at the main menu, not straight into play
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

    /// True once the player has selected Quit from the main menu. The web
    /// page can react however it likes (e.g. show a message) -- there's no
    /// forced navigation.
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
