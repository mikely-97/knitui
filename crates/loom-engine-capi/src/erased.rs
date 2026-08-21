//! Type-erasure boundary for FFI: `Game`/`Shell<G>` are generic (not
//! object-safe -- `Game::Config` differs per game), which is exactly right
//! for the native frontends (compile-time monomorphized, per the pivot's
//! confirmed architecture decisions) but wrong for a C ABI, which needs one
//! concrete entry point selected at runtime by a game id. `ErasedShell` is
//! a small, JSON-only, object-safe trait that every `Shell<G>` satisfies
//! via the blanket impl below; `create_shell` is the one place that knows
//! about all 4 concrete `Game` impls.

use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::game::{Game, GameId};
use loom_engine::input::KeyEvent;
use loom_engine::render::CellGrid;
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;

pub trait ErasedShell {
    fn handle_key(&mut self, key: KeyEvent);
    fn tick(&mut self);
    fn render(&self, width: u16, height: u16) -> CellGrid;
    fn should_quit(&self) -> bool;
    fn save_on_exit(&mut self);
}

impl<G: Game> ErasedShell for Shell<G> {
    fn handle_key(&mut self, key: KeyEvent) {
        Shell::handle_key(self, key);
    }

    fn tick(&mut self) {
        Shell::tick(self);
    }

    fn render(&self, width: u16, height: u16) -> CellGrid {
        let mut grid = CellGrid::new(width, height);
        Shell::render(self, &mut grid);
        grid
    }

    fn should_quit(&self) -> bool {
        Shell::should_quit(self)
    }

    fn save_on_exit(&mut self) {
        Shell::save_on_exit(self);
    }
}

/// Build a fresh `Shell<G>` for the requested game, type-erased. Loads
/// real persisted settings/campaign/high-score state from disk (same
/// `Game::config_dir()` each native frontend uses), matching "host-owns-
/// the-loop": the FFI caller drives the exact same menu/campaign/options
/// state machine a terminal or web frontend would, just through C calls
/// instead of crossterm/canvas events.
pub fn create_shell(game_id: GameId) -> Box<dyn ErasedShell> {
    match game_id {
        GameId::Knit => Box::new(new_shell(knitui::game::KnitGame)),
        GameId::Match3 => Box::new(new_shell(m3tui::game::M3Game)),
        GameId::Merge2 => Box::new(new_shell(m2tui::game::M2Game)),
        GameId::Picross => Box::new(new_shell(pictui::game::PicrossGame)),
    }
}

fn new_shell<G: Game>(game: G) -> Shell<G> {
    let config_dir = game.config_dir();
    let cli_config = game.default_config();
    let user_settings = UserSettings::load(config_dir);
    let campaign_saves = CampaignSaves::<G::CampaignEntry>::load(config_dir);
    let endless_hs = EndlessHighScore::load(config_dir);

    Shell::new(
        game,
        cli_config,
        user_settings,
        campaign_saves,
        endless_hs,
        Vec::new(),
        String::new(),
        false,
    )
}
