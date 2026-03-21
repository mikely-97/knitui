use crate::blessings;

// ── Error / result types ───────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum MoveError {
    OutOfBounds,
}

#[derive(Debug, PartialEq)]
pub enum PickError {
    NotSelectable,
    NotASpool,
    ActiveFull,
}

#[derive(Debug, PartialEq)]
pub enum GameStatus {
    Playing,
    Won,
    Stuck,
}

#[derive(Debug, PartialEq)]
pub enum BonusError {
    NoneLeft,
    BonusActive,
    NoHeldSpools,
    BalloonColumnsNotEmpty,
}

/// Runtime flags for active blessings that affect gameplay behavior.
#[derive(Debug, Clone, Default)]
pub struct BlessingFlags {
    pub scouts_eye: bool,
    pub wrap_around: bool,
    pub tidy_workspace: bool,
    pub conveyor_peek: bool,
    pub color_count: bool,
    pub match_hint: bool,
}

impl BlessingFlags {
    pub fn from_ids(ids: &[String]) -> Self {
        Self {
            scouts_eye:     blessings::has(ids, "scouts_eye"),
            wrap_around:    blessings::has(ids, "wrap_around"),
            tidy_workspace: blessings::has(ids, "tidy_workspace"),
            conveyor_peek:  blessings::has(ids, "conveyor_peek"),
            color_count:    blessings::has(ids, "color_count"),
            match_hint:     blessings::has(ids, "match_hint"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BonusState {
    None,
    TweezersActive { saved_row: u16, saved_col: u16 },
}
