pub use loom_engine::settings::UserSettings;

const CONFIG_DIR: &str = "m2tui";

pub const COLOR_MODES: &[&str] = &[
    "dark", "bright", "colorblind",
    "dark-rgb", "bright-rgb", "colorblind-rgb",
];

pub fn load() -> UserSettings {
    UserSettings::load(CONFIG_DIR)
}

pub fn save(settings: &UserSettings) {
    settings.save(CONFIG_DIR);
}

pub fn next_color_mode(current: &str) -> &'static str {
    let idx = COLOR_MODES.iter().position(|&m| m == current).unwrap_or(0);
    COLOR_MODES[(idx + 1) % COLOR_MODES.len()]
}

pub fn prev_color_mode(current: &str) -> &'static str {
    let idx = COLOR_MODES.iter().position(|&m| m == current).unwrap_or(0);
    COLOR_MODES[(idx + COLOR_MODES.len() - 1) % COLOR_MODES.len()]
}
