use serde::{Deserialize, Serialize};

pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub icon: char,
}

pub const ALL_ACHIEVEMENTS: &[AchievementDef] = &[
    AchievementDef { id: "first_win", name: "First Win", desc: "Win any game", icon: '\u{2605}' },
    AchievementDef { id: "knit_complete", name: "Knit Master", desc: "Complete all knit tracks", icon: '\u{1F9F6}' },
    AchievementDef { id: "combo_king", name: "Combo King", desc: "Achieve x5 combo in match-3", icon: '\u{1F4A5}' },
    AchievementDef { id: "merge_mogul", name: "Merge Mogul", desc: "Complete 50 orders in merge-2", icon: '\u{1F3EA}' },
    AchievementDef { id: "daily_devotee", name: "Daily Devotee", desc: "Complete 7 daily challenges", icon: '\u{1F4C5}' },
    AchievementDef { id: "no_bonuses", name: "Perfectionist", desc: "Win a level without using bonuses", icon: '\u{2728}' },
    AchievementDef { id: "speed_run", name: "Speed Run", desc: "Win a knit board in under 30 moves", icon: '\u{26A1}' },
];

#[derive(Default, Serialize, Deserialize)]
pub struct AchievementTracker {
    pub unlocked: Vec<String>,
}

impl AchievementTracker {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let s = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&s).unwrap_or_default()
        } else { Self::default() }
    }
    pub fn save(&self) {
        let path = Self::path();
        if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
        let _ = std::fs::write(path, serde_json::to_string_pretty(self).unwrap_or_default());
    }
    pub fn unlock(&mut self, id: &str) -> bool {
        if !self.unlocked.iter().any(|u| u == id) {
            self.unlocked.push(id.to_string());
            self.save();
            true
        } else { false }
    }
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocked.iter().any(|u| u == id)
    }
    fn path() -> std::path::PathBuf {
        dirs::data_dir().unwrap_or_default().join("loom").join("achievements.json")
    }
}
