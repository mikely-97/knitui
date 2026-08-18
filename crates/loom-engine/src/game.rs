use crate::blessings::Blessing;
use crate::campaign::CampaignEntry;
use crate::input::KeyEvent;
use crate::render::{Color, Surface};

/// Identifies which game is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameId {
    Knit,
    Match3,
    Merge2,
    Picross,
}

/// Action returned by GameEngine::handle_key to tell the TUI framework what to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing happened or handled internally.
    None,
    /// Screen needs a redraw.
    Redraw,
    /// Switch to help screen.
    ShowHelp,
    /// Return to the main menu.
    QuitToMenu,
    /// Quit the application entirely.
    Quit,
}

/// Game-over status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Playing,
    /// `score` is `None` for games with no numeric scoring concept (e.g. knit,
    /// picross — completion is binary), `Some(_)` for games that track one.
    Won { score: Option<u32> },
    Lost { reason: String },
    /// No valid moves but bonuses may help.
    Stuck,
}

/// Rectangle describing where the game should render.
#[derive(Clone, Copy, Debug)]
pub struct RenderArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Trait for a game engine instance. Each game implements this so the shared
/// TUI framework can drive gameplay without knowing game-specific details.
pub trait GameEngine {
    /// Handle a keyboard event during the Playing state.
    fn handle_key(&mut self, key: KeyEvent) -> Action;

    /// Advance game by one tick (animation, background processing).
    /// Called every frame (~50ms). Returns true if screen needs redraw.
    fn tick(&mut self) -> bool;

    /// Current game status.
    fn status(&self) -> GameStatus;

    /// Render the game area (board + game-specific HUD) at the given origin.
    fn render(&self, surface: &mut dyn Surface, area: RenderArea);

    /// Render the key bar (bottom of screen, shows available keys).
    fn render_keybar(&self, surface: &mut dyn Surface, y: u16);

    /// Render the game-over overlay (status message, optional
    /// campaign/endless-specific text) on top of the last-rendered frame.
    fn render_game_over_overlay(&self, surface: &mut dyn Surface, status: &GameStatus, overlay_msg: Option<&str>);

    /// Render a win-celebration effect on top of the last-rendered frame.
    /// `tick` runs 0..16 over the shell's fixed celebration duration.
    /// Default no-op — not every game has a celebration effect.
    fn render_celebration(&self, surface: &mut dyn Surface, area: RenderArea, tick: u8) {
        let _ = (surface, area, tick);
    }

    /// Render the help screen. Deliberately per-game rather than generic:
    /// a useful help screen shows live state (active blessings, bonus
    /// counts, ...) that only the concrete engine knows about.
    fn render_help(&self, surface: &mut dyn Surface);

    /// Score for game-over display, if this game has a numeric-score concept.
    fn score(&self) -> Option<u32> { None }

    /// Can this game watch ads for bonuses?
    fn can_watch_ad(&self) -> bool { false }
    fn watch_ad(&mut self) {}

    /// Current scale factor.
    fn scale(&self) -> u16;
    fn set_scale(&mut self, scale: u16);

    /// Board dimensions in cells (rows, cols) — used for layout calculations.
    fn board_dims(&self) -> (u16, u16);
}

/// Shared trait for game configuration. Each game has its own Config struct
/// but must expose these common fields for the framework.
pub trait GameConfig: Clone {
    fn board_width(&self) -> usize;
    fn board_height(&self) -> usize;
    fn color_count(&self) -> usize;
    fn scale(&self) -> u16;
    fn color_mode(&self) -> &str;
    fn set_scale(&mut self, scale: u16);
    fn set_color_mode(&mut self, mode: String);

    /// (label, current value) pairs for the custom-game config editor, in
    /// display order. Index 0 in the editor screen is reserved for preset
    /// selection, so these are addressed 1-based by `adjust_custom_field`.
    fn custom_fields(&self) -> Vec<(&'static str, u16)>;
    /// Adjust one custom field by `delta`. `field` is the same 1-based
    /// index scheme as above.
    fn adjust_custom_field(&mut self, field: usize, delta: i16);
}

/// Definition of a game type. Each game crate implements this.
pub trait Game: 'static {
    type Config: GameConfig;
    /// This game's persisted campaign-progress type (e.g. knit's
    /// `CampaignState`). Only the minimal bookkeeping surface
    /// (`CampaignEntry`) is generic; game-specific semantics (what a level
    /// or a blessing actually does to it) live in the methods below, which
    /// take `&Self::CampaignEntry` and are implemented per game.
    type CampaignEntry: CampaignEntry;

    // Identity
    fn id(&self) -> GameId;
    fn name(&self) -> &'static str;
    fn config_dir(&self) -> &'static str;

    // Engine lifecycle
    fn create_engine(&self, config: &Self::Config, palette: &[Color]) -> Box<dyn GameEngine>;
    fn default_config(&self) -> Self::Config;

    // Campaign
    fn track_names(&self) -> &'static [&'static str];
    fn track_count(&self) -> usize;
    fn level_count(&self, track: usize) -> usize;
    fn level_config(&self, track: usize, level: usize, base: &Self::Config) -> Self::Config;
    fn level_intro_lines(&self, track: usize, level: usize) -> Vec<String>;

    /// Build a fresh campaign-progress entry for a track (e.g. `CampaignState::new`).
    fn new_campaign_entry(&self, track: usize) -> Self::CampaignEntry;
    /// Config for a campaign entry's current level — level params, banked
    /// bonuses, and any blessing numeric effects already merged in.
    fn campaign_config(&self, entry: &Self::CampaignEntry, base: &Self::Config) -> Self::Config;
    /// Build a `GameEngine` for a campaign run, applying whatever
    /// entry-specific runtime setup (ad limits, blessing behavior flags,
    /// ...) the plain `create_engine` doesn't know about. Default: no extra
    /// setup, same as a non-campaign game.
    fn create_campaign_engine(
        &self, _entry: &Self::CampaignEntry, config: &Self::Config, palette: &[Color],
    ) -> Box<dyn GameEngine> {
        self.create_engine(config, palette)
    }
    /// Apply a level-completion result to a campaign entry (advance level,
    /// bank rewards, ...). Returns true if the track is now fully complete.
    fn complete_campaign_level(&self, entry: &mut Self::CampaignEntry) -> bool;

    // Blessings — optional; games with no blessing system use the defaults.
    /// This game's full blessing catalog (locked and unlocked alike — the
    /// blessing-selection screen shows locked cards greyed out, it needs
    /// the whole list, not a pre-filtered one). Use
    /// `loom_engine::blessings::is_unlocked` per-card against a
    /// completed-track count to determine lock state.
    fn all_blessings(&self) -> &'static [Blessing] { &[] }
    /// Record chosen blessing IDs on a campaign entry and apply any
    /// one-time banked bonuses they grant.
    fn confirm_blessings(&self, entry: &mut Self::CampaignEntry, ids: &[String]) {
        let _ = (entry, ids);
    }

    // Endless
    fn endless_wave_config(&self, wave: u32, base: &Self::Config) -> Self::Config;

    // UI metadata
    fn help_lines(&self) -> Vec<(&'static str, &'static str)>;
    fn presets(&self) -> Vec<(&'static str, Self::Config)>;
}
