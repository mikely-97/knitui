use crossterm::style::Color;
use rand::prelude::*;
use std::collections::HashSet;

#[allow(unused_imports)]
use crate::board::{Board, Cell, CellContent, Orientation, SpecialPiece, TileModifier};
use crate::bonuses::{BonusInventory, BonusState};
use crate::config::Config;
use crate::matches::{self, MatchGroup};
use crate::blessings;
use crate::palette::select_palette;

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

// ── Engine ───────────────────────────────────────────────────────────────

pub struct GameEngine {
    pub board: Board,
    pub palette: Vec<Color>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// First cell of a pending swap, if any.
    pub selected: Option<(usize, usize)>,
    pub score: u32,
    pub moves_used: u32,
    pub move_limit: u32,
    pub phase: GamePhase,
    pub bonuses: BonusInventory,
    pub bonus_state: BonusState,
    /// The pair swapped in the last player move (used to revert on Bouncing).
    pending_swap: Option<((usize, usize), (usize, usize))>,
    pub blessing_flags: BlessingFlags,
}

impl GameEngine {
    pub fn new(config: &Config) -> Self {
        let palette = select_palette(&config.color_mode, config.color_number);
        let board = Board::make_random(
            config.board_height as usize,
            config.board_width as usize,
            &palette,
            config.special_tile_pct,
        );
        Self {
            board,
            palette,
            cursor_row: 0,
            cursor_col: 0,
            selected: None,
            score: 0,
            moves_used: 0,
            move_limit: config.move_limit,
            phase: GamePhase::PlayerInput,
            bonuses: BonusInventory {
                hammer: config.hammer,
                laser: config.laser,
                blaster: config.blaster,
                warp: config.warp,
            },
            bonus_state: BonusState::None,
            pending_swap: None,
            blessing_flags: BlessingFlags::default(),
        }
    }

    /// Populate blessing flags from a list of blessing IDs.
    pub fn set_blessings(&mut self, ids: &[String]) {
        self.blessing_flags = BlessingFlags {
            keen_eye: blessings::has(ids, "keen_eye"),
            lucky_start: blessings::has(ids, "lucky_start"),
            ice_breaker: blessings::has(ids, "ice_breaker"),
            cascade_master: blessings::has(ids, "cascade_master"),
            crate_cracker: blessings::has(ids, "crate_cracker"),
            chain_reaction: blessings::has(ids, "chain_reaction"),
            color_surge: blessings::has(ids, "color_surge"),
            last_stand: blessings::has(ids, "last_stand"),
            last_stand_used: false,
            gem_magnet: blessings::has(ids, "gem_magnet"),
            double_score: blessings::has(ids, "double_score"),
        };
        // Apply lucky_start: seed 1 random special piece on the board
        if self.blessing_flags.lucky_start {
            self.seed_lucky_start();
        }
        // Apply ice_breaker / crate_cracker: reduce starting HP of modifiers
        if self.blessing_flags.ice_breaker || self.blessing_flags.crate_cracker {
            self.apply_modifier_blessings();
        }
    }

    /// Reduce HP of Ice/Crate modifiers on the board per blessings.
    fn apply_modifier_blessings(&mut self) {
        for r in 0..self.board.height {
            for c in 0..self.board.width {
                let remove = match &mut self.board.cells[r][c].modifier {
                    Some(TileModifier::Ice { hp }) if self.blessing_flags.ice_breaker => {
                        *hp = hp.saturating_sub(1);
                        *hp == 0
                    }
                    Some(TileModifier::Crate { hp }) if self.blessing_flags.crate_cracker => {
                        *hp = hp.saturating_sub(1);
                        *hp == 0
                    }
                    _ => false,
                };
                if remove {
                    self.board.cells[r][c].modifier = None;
                }
            }
        }
    }

    /// Place 1 random special piece on an existing gem.
    fn seed_lucky_start(&mut self) {
        let mut rng = rand::rng();
        let mut candidates: Vec<(usize, usize)> = Vec::new();
        for r in 0..self.board.height {
            for c in 0..self.board.width {
                if let CellContent::Gem { special: None, .. } = &self.board.cells[r][c].content {
                    candidates.push((r, c));
                }
            }
        }
        if let Some(&(r, c)) = candidates.choose(&mut rng) {
            let special = match rng.random_range(0u8..3) {
                0 => SpecialPiece::LineBomb(Orientation::Horizontal),
                1 => SpecialPiece::LineBomb(Orientation::Vertical),
                _ => SpecialPiece::AreaBomb { radius: 1 },
            };
            if let CellContent::Gem { special: sp, .. } = &mut self.board.cells[r][c].content {
                *sp = Some(special);
            }
        }
    }

    /// Return the two cells involved in the current bounce animation, if any.
    pub fn pending_swap_preview(&self) -> Option<((usize, usize), (usize, usize))> {
        self.pending_swap
    }

    /// Move the cursor by (dr, dc), clamped to board bounds.
    pub fn move_cursor(&mut self, dr: i32, dc: i32) {
        let new_row = (self.cursor_row as i32 + dr)
            .clamp(0, self.board.height as i32 - 1) as usize;
        let new_col = (self.cursor_col as i32 + dc)
            .clamp(0, self.board.width as i32 - 1) as usize;
        self.cursor_row = new_row;
        self.cursor_col = new_col;
    }

    /// Handle Enter key during PlayerInput phase.
    ///
    /// - If nothing selected: select the cursor cell (if swappable).
    /// - If something selected and cursor is adjacent: attempt swap.
    /// - If something selected and cursor is non-adjacent: re-select cursor cell.
    pub fn confirm_selection(&mut self) {
        if !matches!(self.phase, GamePhase::PlayerInput) {
            return;
        }
        let (r, c) = (self.cursor_row, self.cursor_col);

        match self.selected {
            None => {
                if self.board.cells[r][c].is_swappable() {
                    self.selected = Some((r, c));
                }
            }
            Some((sr, sc)) => {
                // Double-click: activate special piece in-place
                if r == sr && c == sc {
                    if let CellContent::Gem { color, special: Some(_) } = &self.board.cells[r][c].content {
                        let group = MatchGroup {
                            cells: vec![(r, c)],
                            color: *color,
                            create_special: None,
                        };
                        self.moves_used += 1;
                        self.selected = None;
                        self.phase = GamePhase::Resolving { match_groups: vec![group], spawn_at: None };
                    } else {
                        self.selected = None;
                    }
                    return;
                }

                let adjacent = (r == sr && c.abs_diff(sc) == 1)
                    || (c == sc && r.abs_diff(sr) == 1);

                if !adjacent {
                    // Re-select current cell (or deselect if not swappable)
                    self.selected = if self.board.cells[r][c].is_swappable() {
                        Some((r, c))
                    } else {
                        None
                    };
                    return;
                }

                self.attempt_swap((sr, sc), (r, c));
            }
        }
    }

    fn attempt_swap(&mut self, a: (usize, usize), b: (usize, usize)) {
        self.board.swap_cells(a, b);
        let groups = matches::find_matches(&self.board);

        if groups.is_empty() {
            // No regular matches — check if either cell has a special piece.
            // Special pieces activate on any swap, even without a match.
            let mut special_groups = Vec::new();
            for &pos in &[a, b] {
                if let CellContent::Gem { color, special: Some(_) } = &self.board.cells[pos.0][pos.1].content {
                    special_groups.push(MatchGroup {
                        cells: vec![pos],
                        color: *color,
                        create_special: None,
                    });
                }
            }

            if special_groups.is_empty() {
                // Truly invalid — revert and bounce
                self.board.swap_cells(a, b);
                self.phase = GamePhase::Bouncing { ticks_left: 6 };
            } else {
                // Special piece activation — keep the swap
                self.moves_used += 1;
                self.pending_swap = Some((a, b));
                self.phase = GamePhase::Resolving { match_groups: special_groups, spawn_at: None };
            }
        } else {
            self.moves_used += 1;
            self.pending_swap = Some((a, b));
            // spawn_at = destination cell (b) — where the player moved to
            self.phase = GamePhase::Resolving { match_groups: groups, spawn_at: Some(b) };
        }
        self.selected = None;
    }

    /// Advance one tick of the non-input phase pipeline.
    /// Returns true if state changed (trigger re-render).
    /// Called unconditionally each event-loop cycle (~50 ms).
    pub fn tick(&mut self) -> bool {
        match &self.phase.clone() {
            GamePhase::PlayerInput => false,

            GamePhase::Bouncing { ticks_left } => {
                let tl = *ticks_left;
                if tl == 0 {
                    self.phase = GamePhase::PlayerInput;
                } else {
                    self.phase = GamePhase::Bouncing { ticks_left: tl - 1 };
                }
                true
            }

            GamePhase::Resolving { match_groups, spawn_at } => {
                let groups = match_groups.clone();
                let spawn = *spawn_at;
                self.execute_resolution(groups, spawn);
                true
            }

            GamePhase::Falling => {
                let moved = self.board.apply_gravity();
                if !moved {
                    self.phase = GamePhase::Refilling;
                }
                true
            }

            GamePhase::Refilling => {
                self.board.refill_top(&self.palette.clone());
                let new_groups = matches::find_matches(&self.board);
                if new_groups.is_empty() {
                    self.phase = GamePhase::PlayerInput;
                } else {
                    // Cascade: new matches from the refill
                    self.phase = GamePhase::Resolving {
                        match_groups: new_groups,
                        spawn_at: None, // no player-initiated spawn during cascade
                    };
                }
                true
            }
        }
    }

    /// Clear all matched cells, apply modifier damage to adjacent cells,
    /// trigger any special pieces within the matched set, score points,
    /// and place a new special piece at spawn_at if applicable.
    fn execute_resolution(
        &mut self,
        match_groups: Vec<MatchGroup>,
        spawn_at: Option<(usize, usize)>,
    ) {
        // 1. Collect all cells to clear (iteratively expand for special pieces)
        let mut to_clear: HashSet<(usize, usize)> = match_groups
            .iter()
            .flat_map(|g| g.cells.iter().copied())
            .collect();

        // 2. Trigger special pieces (chain reaction loop)
        loop {
            let mut added: HashSet<(usize, usize)> = HashSet::new();
            for &(r, c) in &to_clear {
                if let CellContent::Gem { special: Some(ref sp), .. } = self.board.cells[r][c].content.clone() {
                    self.collect_explosion(sp, r, c, &mut added);
                }
            }
            // chain_reaction blessing: also trigger specials adjacent to cleared cells
            if self.blessing_flags.chain_reaction {
                let border: Vec<(usize, usize)> = to_clear
                    .iter()
                    .flat_map(|&(r, c)| {
                        [(r.wrapping_sub(1), c), (r + 1, c), (r, c.wrapping_sub(1)), (r, c + 1)]
                            .into_iter()
                            .filter(|&(nr, nc)| nr < self.board.height && nc < self.board.width)
                            .filter(|pos| !to_clear.contains(pos))
                    })
                    .collect();
                for (nr, nc) in border {
                    if let CellContent::Gem { special: Some(ref sp), .. } = self.board.cells[nr][nc].content.clone() {
                        added.insert((nr, nc));
                        self.collect_explosion(sp, nr, nc, &mut added);
                    }
                }
            }
            let before = to_clear.len();
            to_clear.extend(added);
            if to_clear.len() == before { break; } // no new cells added
        }

        // 3. Score: 10 pts per gem cleared (with blessing modifiers)
        let mut pts = (to_clear.len() as u32) * 10;
        // cascade_master: cascades (non-player-initiated resolves) score 50% more
        if self.blessing_flags.cascade_master && spawn_at.is_none() {
            pts = pts * 3 / 2;
        }
        if self.blessing_flags.double_score {
            pts *= 2;
        }
        self.score += pts;

        // 4. Damage modifiers adjacent to cleared cells (non-direct: Ice/Crate only)
        let adjacent: Vec<(usize, usize)> = to_clear
            .iter()
            .flat_map(|&(r, c)| {
                [(r.wrapping_sub(1), c), (r + 1, c), (r, c.wrapping_sub(1)), (r, c + 1)]
                    .into_iter()
                    .filter(|&(nr, nc)| nr < self.board.height && nc < self.board.width)
                    .filter(|pos| !to_clear.contains(pos))
            })
            .collect();
        for (nr, nc) in adjacent {
            self.damage_modifier(nr, nc, false);
        }

        // 5. Damage modifiers on cleared cells themselves (direct hit: all types)
        for &(r, c) in &to_clear {
            self.damage_modifier(r, c, true);
        }

        // 6. Clear the cells
        for &(r, c) in &to_clear {
            self.board.cells[r][c].content = CellContent::Empty;
        }

        // 6b. color_surge: if any match group has 5+ cells, clear 2 extra random gems of that color
        if self.blessing_flags.color_surge {
            let mut rng = rand::rng();
            for group in &match_groups {
                if group.cells.len() >= 5 {
                    let mut extras: Vec<(usize, usize)> = Vec::new();
                    for r in 0..self.board.height {
                        for c in 0..self.board.width {
                            if !to_clear.contains(&(r, c)) {
                                if self.board.cells[r][c].color() == Some(group.color) {
                                    extras.push((r, c));
                                }
                            }
                        }
                    }
                    extras.shuffle(&mut rng);
                    for &(r, c) in extras.iter().take(2) {
                        self.board.cells[r][c].content = CellContent::Empty;
                        self.score += if self.blessing_flags.double_score { 20 } else { 10 };
                    }
                }
            }
        }

        // 7. Place special piece at spawn position (from the matching group that owns it)
        if let Some(pos) = spawn_at {
            for group in &match_groups {
                if let Some(ref sp) = group.create_special {
                    if group.cells.contains(&pos) {
                        // gem_magnet: 4-matches always produce AreaBombs
                        let actual_sp = if self.blessing_flags.gem_magnet {
                            if matches!(sp, SpecialPiece::LineBomb(_)) {
                                SpecialPiece::AreaBomb { radius: 1 }
                            } else {
                                sp.clone()
                            }
                        } else {
                            sp.clone()
                        };
                        // Restore the cell with the special piece gem
                        self.board.cells[pos.0][pos.1].content = CellContent::Gem {
                            color: group.color,
                            special: Some(actual_sp),
                        };
                        break;
                    }
                }
            }
        }

        self.phase = GamePhase::Falling;
    }

    /// Expand `added` with cells destroyed by special piece `sp` at (r, c).
    fn collect_explosion(
        &self,
        sp: &SpecialPiece,
        r: usize,
        c: usize,
        added: &mut HashSet<(usize, usize)>,
    ) {
        match sp {
            SpecialPiece::LineBomb(Orientation::Horizontal) => {
                for cc in 0..self.board.width {
                    added.insert((r, cc));
                }
            }
            SpecialPiece::LineBomb(Orientation::Vertical) => {
                for rr in 0..self.board.height {
                    added.insert((rr, c));
                }
            }
            SpecialPiece::ColorBomb => {
                if let Some(color) = self.board.cells[r][c].color() {
                    for rr in 0..self.board.height {
                        for cc in 0..self.board.width {
                            if self.board.cells[rr][cc].color() == Some(color) {
                                added.insert((rr, cc));
                            }
                        }
                    }
                }
            }
            SpecialPiece::AreaBomb { radius } => {
                let rad = *radius as i32;
                for dr in -rad..=rad {
                    for dc in -rad..=rad {
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        if nr >= 0 && nr < self.board.height as i32
                            && nc >= 0 && nc < self.board.width as i32
                        {
                            added.insert((nr as usize, nc as usize));
                        }
                    }
                }
            }
        }
    }

    /// Apply one unit of damage to the modifier at (r, c).
    ///
    /// `direct` = true  → cell is directly in the explosion/clear zone.
    /// `direct` = false → cell is adjacent to a cleared cell.
    ///
    /// Stone is only damaged by direct hits.
    /// Ice/Crate are damaged by both direct and adjacent.
    /// Locked is removed on direct clear (the gem gets matched).
    fn damage_modifier(&mut self, r: usize, c: usize, direct: bool) {
        // Read the modifier kind first to avoid simultaneous borrow conflicts.
        let modifier_kind = match &self.board.cells[r][c].modifier {
            None => return,
            Some(m) => m.clone(),
        };
        match modifier_kind {
            TileModifier::Ice { hp } | TileModifier::Crate { hp } => {
                let new_hp = hp.saturating_sub(1);
                if new_hp == 0 {
                    self.board.cells[r][c].modifier = None;
                } else {
                    // Write back with decremented hp — must reconstruct the variant.
                    match self.board.cells[r][c].modifier {
                        Some(TileModifier::Ice { ref mut hp }) => *hp = new_hp,
                        Some(TileModifier::Crate { ref mut hp }) => *hp = new_hp,
                        _ => {}
                    }
                }
            }
            TileModifier::Stone => {
                if direct {
                    self.board.cells[r][c].modifier = None;
                }
                // Stone ignores adjacent damage
            }
            TileModifier::Locked => {
                if direct {
                    self.board.cells[r][c].modifier = None;
                }
            }
        }
    }

    // ── Bonus actions ─────────────────────────────────────────────────────

    /// Activate Hammer: enter targeting mode. No-op if inventory empty or bonus active.
    pub fn activate_hammer(&mut self) {
        if !matches!(self.bonus_state, BonusState::None) { return; }
        if !self.bonuses.consume_hammer() { return; }
        self.bonus_state = BonusState::HammerActive {
            saved_row: self.cursor_row,
            saved_col: self.cursor_col,
        };
    }

    /// Confirm Hammer target: destroy cell at cursor, enter Falling.
    pub fn confirm_hammer(&mut self) {
        if !matches!(self.bonus_state, BonusState::HammerActive { .. }) { return; }
        let (r, c) = (self.cursor_row, self.cursor_col);
        // Damage modifier (direct) then clear content
        self.damage_modifier(r, c, true);
        self.board.cells[r][c].content = CellContent::Empty;
        self.bonus_state = BonusState::None;
        self.phase = GamePhase::Falling;
    }

    /// Cancel an active bonus, restoring saved state and refunding the charge.
    pub fn cancel_bonus(&mut self) {
        match self.bonus_state.clone() {
            BonusState::HammerActive { saved_row, saved_col } => {
                self.cursor_row = saved_row;
                self.cursor_col = saved_col;
                self.bonuses.hammer += 1; // refund
                self.bonus_state = BonusState::None;
            }
            BonusState::None => {}
        }
    }

    /// Laser: destroy entire cursor row immediately.
    pub fn activate_laser(&mut self) {
        if !self.bonuses.consume_laser() { return; }
        let r = self.cursor_row;
        for c in 0..self.board.width {
            self.damage_modifier(r, c, true);
            self.board.cells[r][c].content = CellContent::Empty;
        }
        self.phase = GamePhase::Falling;
    }

    /// Blaster: destroy entire cursor column immediately.
    pub fn activate_blaster(&mut self) {
        if !self.bonuses.consume_blaster() { return; }
        let c = self.cursor_col;
        for r in 0..self.board.height {
            self.damage_modifier(r, c, true);
            self.board.cells[r][c].content = CellContent::Empty;
        }
        self.phase = GamePhase::Falling;
    }

    /// Warp: collect all gem colors, shuffle them, re-place without pre-existing matches.
    pub fn activate_warp(&mut self) {
        if !self.bonuses.consume_warp() { return; }
        let mut rng = rand::rng();
        // Collect all gem positions (non-stone, non-empty)
        let positions: Vec<(usize, usize)> = (0..self.board.height)
            .flat_map(|r| (0..self.board.width).map(move |c| (r, c)))
            .filter(|&(r, c)| {
                !matches!(self.board.cells[r][c].modifier, Some(TileModifier::Stone))
                    && matches!(self.board.cells[r][c].content, CellContent::Gem { .. })
            })
            .collect();

        let mut colors: Vec<Color> = positions
            .iter()
            .filter_map(|&(r, c)| self.board.cells[r][c].color())
            .collect();

        // Shuffle and re-place, fixing pre-existing matches
        colors.shuffle(&mut rng);
        for (&(r, c), color) in positions.iter().zip(colors.iter()) {
            self.board.cells[r][c].content = CellContent::Gem { color: *color, special: None };
        }
        // Fix any pre-existing matches created by the shuffle.
        // Try up to 10 re-rolls (overwhelmingly sufficient in practice).
        for _ in 0..10 {
            let gs = matches::find_matches(&self.board);
            if gs.is_empty() { break; }
            for group in &gs {
                let &(r, c) = group.cells.first().unwrap();
                let new_color = *self.palette.choose(&mut rng).unwrap();
                self.board.cells[r][c].content = CellContent::Gem { color: new_color, special: None };
            }
        }
        // If matches still remain after re-rolling (astronomically unlikely), cascade-resolve
        // them instead of leaving them silently in place.
        let new_groups = matches::find_matches(&self.board);
        if new_groups.is_empty() {
            self.phase = GamePhase::PlayerInput;
        } else {
            self.phase = GamePhase::Resolving { match_groups: new_groups, spawn_at: None };
        }
    }

    // ── Game status ───────────────────────────────────────────────────────

    /// Check overall game status. Called after every Refilling → PlayerInput transition.
    ///
    /// **NOTE:** `GameStatus::Won` is intentionally NOT returned here — win conditions
    /// depend on campaign objectives (score target, gem quota, clear-all-specials) which
    /// the engine has no knowledge of. The main loop in `main.rs` checks `objective_met()`
    /// directly and handles the "Won" case before calling this method.
    pub fn game_status(&mut self) -> GameStatus {
        if self.moves_used >= self.move_limit {
            // last_stand: grant 3 bonus moves once
            if self.blessing_flags.last_stand && !self.blessing_flags.last_stand_used {
                self.blessing_flags.last_stand_used = true;
                self.move_limit += 3;
                return GameStatus::Playing;
            }
            return GameStatus::OutOfMoves;
        }
        if !self.has_valid_swap() {
            return GameStatus::Stuck;
        }
        GameStatus::Playing
    }

    /// True if at least one adjacent swap would produce a match.
    pub fn has_valid_swap(&self) -> bool {
        if self.board.width < 2 || self.board.height < 2 {
            return false;
        }
        let h = self.board.height;
        let w = self.board.width;
        let mut test_board = self.board.clone();

        // Try every adjacent horizontal pair
        for r in 0..h {
            for c in 0..w - 1 {
                if !test_board.cells[r][c].is_swappable() { continue; }
                if !test_board.cells[r][c + 1].is_swappable() { continue; }
                test_board.swap_cells((r, c), (r, c + 1));
                let found = !matches::find_matches(&test_board).is_empty();
                test_board.swap_cells((r, c), (r, c + 1));
                if found { return true; }
            }
        }
        // Try every adjacent vertical pair
        for r in 0..h - 1 {
            for c in 0..w {
                if !test_board.cells[r][c].is_swappable() { continue; }
                if !test_board.cells[r + 1][c].is_swappable() { continue; }
                test_board.swap_cells((r, c), (r + 1, c));
                let found = !matches::find_matches(&test_board).is_empty();
                test_board.swap_cells((r, c), (r + 1, c));
                if found { return true; }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use clap::Parser;

    fn default_config() -> Config {
        Config::parse_from::<[&str; 0], &str>([])
    }

    fn engine() -> GameEngine {
        GameEngine::new(&default_config())
    }

    // ── Constructor ─────────────────────────────────────────────────────────

    #[test]
    fn new_engine_has_correct_dimensions() {
        let e = engine();
        assert_eq!(e.board.height, 8);
        assert_eq!(e.board.width, 8);
    }

    #[test]
    fn new_engine_starts_at_player_input() {
        let e = engine();
        assert!(matches!(e.phase, GamePhase::PlayerInput));
    }

    #[test]
    fn new_engine_cursor_in_bounds() {
        let e = engine();
        assert!(e.cursor_row < e.board.height);
        assert!(e.cursor_col < e.board.width);
    }

    #[test]
    fn new_engine_no_selection() {
        let e = engine();
        assert!(e.selected.is_none());
    }

    // ── move_cursor ─────────────────────────────────────────────────────────

    #[test]
    fn move_cursor_down() {
        let mut e = engine();
        e.cursor_row = 0;
        e.move_cursor(1, 0);
        assert_eq!(e.cursor_row, 1);
    }

    #[test]
    fn move_cursor_clamps_at_top() {
        let mut e = engine();
        e.cursor_row = 0;
        e.move_cursor(-1, 0);
        assert_eq!(e.cursor_row, 0);
    }

    #[test]
    fn move_cursor_clamps_at_bottom() {
        let mut e = engine();
        e.cursor_row = e.board.height - 1;
        e.move_cursor(1, 0);
        assert_eq!(e.cursor_row, e.board.height - 1);
    }

    #[test]
    fn move_cursor_clamps_at_left() {
        let mut e = engine();
        e.cursor_col = 0;
        e.move_cursor(0, -1);
        assert_eq!(e.cursor_col, 0);
    }

    #[test]
    fn move_cursor_clamps_at_right() {
        let mut e = engine();
        e.cursor_col = e.board.width - 1;
        e.move_cursor(0, 1);
        assert_eq!(e.cursor_col, e.board.width - 1);
    }

    // ── confirm_selection (first click = select) ────────────────────────────

    #[test]
    fn first_confirm_on_gem_sets_selection() {
        let mut e = engine();
        // Ensure the cursor is on a swappable gem
        let (r, c) = find_swappable(&e);
        e.cursor_row = r;
        e.cursor_col = c;
        e.confirm_selection();
        assert_eq!(e.selected, Some((r, c)));
    }

    fn find_swappable(e: &GameEngine) -> (usize, usize) {
        for r in 0..e.board.height {
            for c in 0..e.board.width {
                if e.board.cells[r][c].is_swappable() {
                    return (r, c);
                }
            }
        }
        (0, 0) // fallback — board has no swappable cells
    }

    // ── Phase pipeline ──────────────────────────────────────────────────────

    #[test]
    fn bouncing_decrements_then_returns_to_player_input() {
        let mut e = engine();
        e.phase = GamePhase::Bouncing { ticks_left: 1 };
        e.tick(); // ticks_left → 0
        assert!(matches!(e.phase, GamePhase::Bouncing { ticks_left: 0 }));
        e.tick(); // 0 → PlayerInput
        assert!(matches!(e.phase, GamePhase::PlayerInput));
    }

    #[test]
    fn player_input_tick_returns_false() {
        let mut e = engine();
        assert!(!e.tick());
    }

    #[test]
    fn resolving_clears_matched_cells_and_transitions_to_falling() {
        use crate::board::Cell;
        use crossterm::style::Color;
        // Build a board with a guaranteed 3-match in row 0
        let mut e = engine();
        // Force a 3-Red row at top
        e.board.cells[0][0] = Cell::gem(Color::Red);
        e.board.cells[0][1] = Cell::gem(Color::Red);
        e.board.cells[0][2] = Cell::gem(Color::Red);
        // Avoid vertical matches with row 1
        e.board.cells[1][0] = Cell::gem(Color::Blue);
        e.board.cells[1][1] = Cell::gem(Color::Blue);
        e.board.cells[1][2] = Cell::gem(Color::Blue);
        // But that creates a Blue match — fine for this test, just overwrite row 2
        e.board.cells[2][0] = Cell::gem(Color::Green);
        e.board.cells[2][1] = Cell::gem(Color::Green);
        e.board.cells[2][2] = Cell::gem(Color::Green);
        // Manually enter Resolving with the red match group
        let groups = matches::find_matches(&e.board);
        // Take only the first group (should be the 3-Red one)
        let red_group = groups.into_iter().find(|g| g.color == Color::Red).unwrap();
        e.phase = GamePhase::Resolving {
            match_groups: vec![red_group],
            spawn_at: None,
        };
        e.tick(); // execute resolution
        // Cells [0][0..2] should now be Empty
        assert_eq!(e.board.cells[0][0].content, CellContent::Empty);
        assert_eq!(e.board.cells[0][1].content, CellContent::Empty);
        assert_eq!(e.board.cells[0][2].content, CellContent::Empty);
        // Score should be > 0
        assert!(e.score > 0);
        assert!(matches!(e.phase, GamePhase::Falling));
    }

    #[test]
    fn falling_applies_gravity_and_transitions_to_refilling() {
        use crate::board::Cell;
        use crossterm::style::Color;
        let mut e = engine();
        e.board.cells[0][0] = Cell::gem(Color::Red);
        e.board.cells[1][0] = Cell::empty();
        e.phase = GamePhase::Falling;
        e.tick(); // gravity step
        // Gem should have fallen
        assert!(matches!(e.board.cells[1][0].content, CellContent::Gem { .. }));
        assert_eq!(e.board.cells[0][0].content, CellContent::Empty);
        e.tick(); // gravity: nothing moved → Refilling
        assert!(matches!(e.phase, GamePhase::Refilling));
    }

    #[test]
    fn refilling_fills_empty_cells_and_transitions_to_player_input() {
        use crate::board::Cell;
        let mut e = engine();
        // Clear one cell
        e.board.cells[0][0] = Cell::empty();
        e.phase = GamePhase::Refilling;
        e.tick();
        // After refill + no new matches → PlayerInput (usually)
        // The cell should be filled
        assert!(matches!(e.board.cells[0][0].content, CellContent::Gem { .. }));
    }

    #[test]
    fn score_increases_on_resolution() {
        use crate::board::Cell;
        use crossterm::style::Color;
        let mut e = engine();
        e.board.cells[0][0] = Cell::gem(Color::Red);
        e.board.cells[0][1] = Cell::gem(Color::Red);
        e.board.cells[0][2] = Cell::gem(Color::Red);
        e.board.cells[1][0] = Cell::gem(Color::Blue);
        e.board.cells[1][1] = Cell::gem(Color::Blue);
        e.board.cells[1][2] = Cell::gem(Color::Blue);
        e.board.cells[2][0] = Cell::gem(Color::Green);
        e.board.cells[2][1] = Cell::gem(Color::Green);
        e.board.cells[2][2] = Cell::gem(Color::Green);
        let groups = matches::find_matches(&e.board);
        let red_group = groups.into_iter().find(|g| g.color == Color::Red).unwrap();
        e.phase = GamePhase::Resolving { match_groups: vec![red_group], spawn_at: None };
        let before = e.score;
        e.tick();
        assert!(e.score > before);
    }

    // ── confirm_selection (second click = swap attempt) ─────────────────────

    #[test]
    fn second_confirm_non_adjacent_reselects() {
        let mut e = engine();
        // Force cells (0,0) and (0,3) to be plain swappable gems
        e.board.cells[0][0] = Cell { content: CellContent::Gem { color: Color::Red, special: None }, modifier: None };
        e.board.cells[0][3] = Cell { content: CellContent::Gem { color: Color::Blue, special: None }, modifier: None };
        e.cursor_row = 0;
        e.cursor_col = 0;
        e.confirm_selection(); // select (0,0)
        e.cursor_row = 0;
        e.cursor_col = 3;     // not adjacent
        e.confirm_selection();
        // Should have re-selected (0,3) instead of attempting swap
        assert_eq!(e.selected, Some((0, 3)));
        assert!(matches!(e.phase, GamePhase::PlayerInput));
    }

    // ── Bonus: Hammer ───────────────────────────────────────────────────────

    #[test]
    fn activate_hammer_enters_hammer_active() {
        let mut e = engine();
        e.bonuses.hammer = 1;
        e.activate_hammer();
        assert!(matches!(e.bonus_state, BonusState::HammerActive { .. }));
        assert_eq!(e.bonuses.hammer, 0);
    }

    #[test]
    fn activate_hammer_at_zero_does_nothing() {
        let mut e = engine();
        e.bonuses.hammer = 0;
        e.activate_hammer();
        assert_eq!(e.bonus_state, BonusState::None);
    }

    #[test]
    fn confirm_hammer_destroys_cell_and_resolves() {
        let mut e = engine();
        e.bonuses.hammer = 1;
        e.activate_hammer();
        e.cursor_row = 2;
        e.cursor_col = 2;
        e.confirm_hammer();
        // Cell should be empty (cleared by hammer)
        assert_eq!(e.board.cells[2][2].content, CellContent::Empty);
        assert_eq!(e.bonus_state, BonusState::None);
        assert!(matches!(e.phase, GamePhase::Falling));
    }

    #[test]
    fn cancel_hammer_restores_cursor() {
        let mut e = engine();
        e.bonuses.hammer = 1;
        e.cursor_row = 3;
        e.cursor_col = 3;
        e.activate_hammer();
        e.cursor_row = 1;
        e.cursor_col = 1;
        e.cancel_bonus();
        // Cursor restored
        assert_eq!(e.cursor_row, 3);
        assert_eq!(e.cursor_col, 3);
        assert_eq!(e.bonus_state, BonusState::None);
        assert_eq!(e.bonuses.hammer, 1); // refunded
    }

    // ── Bonus: Laser ────────────────────────────────────────────────────────

    #[test]
    fn laser_destroys_entire_row() {
        let mut e = engine();
        e.bonuses.laser = 1;
        e.cursor_row = 2;
        e.activate_laser();
        // All cells in row 2 should be empty
        for c in 0..e.board.width {
            assert_eq!(e.board.cells[2][c].content, CellContent::Empty,
                "Cell (2,{c}) should be empty");
        }
        assert!(matches!(e.phase, GamePhase::Falling));
    }

    // ── Bonus: Blaster ──────────────────────────────────────────────────────

    #[test]
    fn blaster_destroys_entire_column() {
        let mut e = engine();
        e.bonuses.blaster = 1;
        e.cursor_col = 3;
        e.activate_blaster();
        for r in 0..e.board.height {
            assert_eq!(e.board.cells[r][3].content, CellContent::Empty,
                "Cell ({r},3) should be empty");
        }
        assert!(matches!(e.phase, GamePhase::Falling));
    }

    // ── Bonus: Warp ─────────────────────────────────────────────────────────

    #[test]
    fn warp_shuffles_board_no_pre_existing_matches() {
        let mut e = engine();
        e.bonuses.warp = 1;
        e.activate_warp();
        // After warp: phase is PlayerInput (typical) or Resolving (extremely rare: a match
        // survived 10 re-roll attempts). Both are valid outcomes.
        assert!(
            matches!(e.phase, GamePhase::PlayerInput) || matches!(e.phase, GamePhase::Resolving { .. }),
            "Phase after warp must be PlayerInput or Resolving, got {:?}", e.phase,
        );
        // If PlayerInput, board must have no pre-existing matches
        if matches!(e.phase, GamePhase::PlayerInput) {
            let groups = matches::find_matches(&e.board);
            assert_eq!(groups.len(), 0, "PlayerInput after warp must have no pre-existing matches");
        }
    }

    // ── GameStatus ──────────────────────────────────────────────────────────

    #[test]
    fn out_of_moves_when_move_limit_reached() {
        let mut e = engine();
        e.moves_used = e.move_limit;
        assert_eq!(e.game_status(), GameStatus::OutOfMoves);
    }

    #[test]
    fn playing_when_moves_remain() {
        let mut e = engine();
        e.moves_used = 0;
        // Not stuck, not out of moves → Playing
        // (Won requires campaign objectives which aren't set here)
        let status = e.game_status();
        assert!(status == GameStatus::Playing || status == GameStatus::Stuck);
    }
}
