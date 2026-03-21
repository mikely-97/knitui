pub use loom_engine::settings::{COLOR_MODES, next_color_mode, prev_color_mode};
pub use loom_engine::settings::UserSettings as _EngineUserSettings;

const CONFIG_DIR: &str = "m3tui";

/// Per-session settings (scale, color mode) persisted to
/// `~/.config/m3tui/settings.json`. Delegates load/save to loom-engine.
#[derive(Clone)]
pub struct UserSettings(loom_engine::settings::UserSettings);

impl Default for UserSettings {
    fn default() -> Self {
        Self(loom_engine::settings::UserSettings::default())
    }
}

impl UserSettings {
    pub fn load() -> Self {
        Self(loom_engine::settings::UserSettings::load(CONFIG_DIR))
    }

    pub fn save(&self) {
        self.0.save(CONFIG_DIR);
    }
}

impl std::ops::Deref for UserSettings {
    type Target = loom_engine::settings::UserSettings;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::ops::DerefMut for UserSettings {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
