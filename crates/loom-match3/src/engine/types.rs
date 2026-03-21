use crate::matches::MatchGroup;

// ── Blessing flags ───────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct BlessingFlags {
    pub keen_eye: bool,
    pub lucky_start: bool,
    pub ice_breaker: bool,
    pub cascade_master: bool,
    pub crate_cracker: bool,
    pub chain_reaction: bool,
    pub color_surge: bool,
    pub last_stand: bool,
    pub last_stand_used: bool,
    pub gem_magnet: bool,
    pub double_score: bool,
}

// ── Phase ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum GamePhase {
    /// Waiting for player input.
    PlayerInput,
    /// Invalid swap: visual bounce lasts `ticks_left` ticks, then reverts.
    Bouncing { ticks_left: u8 },
    /// Matches have been found. On next tick: clear cells, trigger specials, → Falling.
    Resolving {
        match_groups: Vec<MatchGroup>,
        /// Where to place the created special piece (swap destination).
        spawn_at: Option<(usize, usize)>,
    },
    /// Applying gravity tick-by-tick until nothing moves.
    Falling,
    /// Refill empty cells from top; then check for cascade.
    Refilling,
}

// ── Status ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    Playing,
    Won,
    OutOfMoves,
    Stuck,
}
