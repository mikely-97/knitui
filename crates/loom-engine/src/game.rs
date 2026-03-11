use std::io::{self, Stdout};
use crossterm::event::KeyEvent;
use crossterm::style::Color;

/// Identifies which game is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameId {
    Knit,
    Match3,
    Merge2,
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
    Won { score: u32 },
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
    fn render(&self, stdout: &mut Stdout, area: RenderArea) -> io::Result<()>;

    /// Render the key bar (bottom of screen, shows available keys).
    fn render_keybar(&self, stdout: &mut Stdout, y: u16) -> io::Result<()>;

    /// Score for game-over display.
    fn score(&self) -> u32;

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
}

/// A field definition for the custom-game configuration screen.
pub struct ConfigField {
    pub label: &'static str,
    pub min: i64,
    pub max: i64,
    pub get: fn(&dyn std::any::Any) -> i64,
    pub set: fn(&mut dyn std::any::Any, i64),
}

/// Definition of a game type. Each game crate implements this.
pub trait Game: 'static {
    type Config: GameConfig;

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

    // Endless
    fn endless_wave_config(&self, wave: u32, base: &Self::Config) -> Self::Config;

    // UI metadata
    fn help_lines(&self) -> Vec<(&'static str, &'static str)>;
    fn presets(&self) -> Vec<(&'static str, Self::Config)>;
}
