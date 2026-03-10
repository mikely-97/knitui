use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub scale: u16,
    pub color_mode: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            scale: 1,
            color_mode: "dark".to_string(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("knitui").join(SETTINGS_FILE))
}

impl UserSettings {
    /// Load settings from disk. Returns defaults if file is missing or corrupt.
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to disk. Creates the config directory if needed.
    pub fn save(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }
}

pub const COLOR_MODES: &[&str] = &[
    "dark", "bright", "colorblind", "dark-rgb", "bright-rgb", "colorblind-rgb",
];

/// Cycle to the next color mode in the list. Returns the new mode string.
pub fn next_color_mode(current: &str) -> &'static str {
    let idx = COLOR_MODES.iter().position(|&m| m == current).unwrap_or(0);
    COLOR_MODES[(idx + 1) % COLOR_MODES.len()]
}

/// Cycle to the previous color mode in the list. Returns the new mode string.
pub fn prev_color_mode(current: &str) -> &'static str {
    let idx = COLOR_MODES.iter().position(|&m| m == current).unwrap_or(0);
    COLOR_MODES[(idx + COLOR_MODES.len() - 1) % COLOR_MODES.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = UserSettings::default();
        assert_eq!(s.scale, 1);
        assert_eq!(s.color_mode, "dark");
    }

    #[test]
    fn color_mode_cycling_forward() {
        assert_eq!(next_color_mode("dark"), "bright");
        assert_eq!(next_color_mode("colorblind-rgb"), "dark");
    }

    #[test]
    fn color_mode_cycling_backward() {
        assert_eq!(prev_color_mode("dark"), "colorblind-rgb");
        assert_eq!(prev_color_mode("bright"), "dark");
    }

    #[test]
    fn unknown_color_mode_defaults_to_first() {
        assert_eq!(next_color_mode("unknown"), "bright");
        assert_eq!(prev_color_mode("unknown"), "colorblind-rgb");
    }

    #[test]
    fn roundtrip_serialization() {
        let s = UserSettings { scale: 3, color_mode: "bright-rgb".to_string() };
        let json = serde_json::to_string(&s).unwrap();
        let loaded: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.scale, 3);
        assert_eq!(loaded.color_mode, "bright-rgb");
    }
}
