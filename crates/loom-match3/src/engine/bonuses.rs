use crossterm::style::Color;
use rand::prelude::*;

use crate::board::{CellContent, TileModifier};
use crate::bonuses::BonusState;
use crate::matches;

use super::GameEngine;
use super::types::GamePhase;

impl GameEngine {
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
            BonusState::ColorBombActive { saved_row, saved_col } => {
                self.cursor_row = saved_row;
                self.cursor_col = saved_col;
                self.bonuses.color_bomb += 1; // refund
                self.bonus_state = BonusState::None;
            }
            BonusState::None => {}
        }
    }

    /// Activate Color Bomb: enter targeting mode. No-op if inventory empty or bonus active.
    pub fn activate_color_bomb(&mut self) {
        if !matches!(self.bonus_state, BonusState::None) { return; }
        if !self.bonuses.consume_color_bomb() { return; }
        self.bonus_state = BonusState::ColorBombActive {
            saved_row: self.cursor_row,
            saved_col: self.cursor_col,
        };
    }

    /// Confirm Color Bomb: clear all gems on the board matching the cursor cell's color.
    pub fn confirm_color_bomb(&mut self) {
        if !matches!(self.bonus_state, BonusState::ColorBombActive { .. }) { return; }
        let (r, c) = (self.cursor_row, self.cursor_col);
        if let Some(target_color) = self.board.cells[r][c].color() {
            for rr in 0..self.board.height {
                for cc in 0..self.board.width {
                    if self.board.cells[rr][cc].color() == Some(target_color) {
                        self.damage_modifier(rr, cc, true);
                        self.board.cells[rr][cc].content = CellContent::Empty;
                        self.score += 10;
                    }
                }
            }
        }
        self.bonus_state = BonusState::None;
        self.phase = GamePhase::Falling;
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
}
