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

impl BlessingFlags {
    /// Return true if the blessing identified by `id` is active.
    pub fn has_id(&self, id: &str) -> bool {
        match id {
            "keen_eye"       => self.keen_eye,
            "lucky_start"    => self.lucky_start,
            "ice_breaker"    => self.ice_breaker,
            "cascade_master" => self.cascade_master,
            "crate_cracker"  => self.crate_cracker,
            "chain_reaction" => self.chain_reaction,
            "color_surge"    => self.color_surge,
            "last_stand"     => self.last_stand,
            "gem_magnet"     => self.gem_magnet,
            "double_score"   => self.double_score,
            _                => false,
        }
    }
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
