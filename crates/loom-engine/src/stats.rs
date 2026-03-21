use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct HighScoreEntry {
    pub score: u64,
    pub mode: String,
    pub date: String, // "YYYY-MM-DD"
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct GameStats {
    pub games_played: u32,
    pub games_won: u32,
    pub high_scores: Vec<HighScoreEntry>, // keep top 10
}

impl GameStats {
    pub fn record_game(&mut self, won: bool) {
        self.games_played += 1;
        if won { self.games_won += 1; }
    }
    pub fn add_score(&mut self, score: u64, mode: &str, date: &str) {
        self.high_scores.push(HighScoreEntry { score, mode: mode.to_string(), date: date.to_string() });
        self.high_scores.sort_by(|a, b| b.score.cmp(&a.score));
        self.high_scores.truncate(10);
    }
    pub fn win_rate(&self) -> f32 {
        if self.games_played == 0 { 0.0 } else { self.games_won as f32 / self.games_played as f32 * 100.0 }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct AllStats {
    pub knit: GameStats,
    pub match3: GameStats,
    pub merge2: GameStats,
}

impl AllStats {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let s = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            Self::default()
        }
    }
    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
        let _ = std::fs::write(path, serde_json::to_string_pretty(self).unwrap_or_default());
    }
    fn path() -> std::path::PathBuf {
        dirs::data_dir().unwrap_or_default().join("loom").join("stats.json")
    }
}
