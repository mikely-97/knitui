use rand::prelude::*;
use std::collections::HashSet;

use crate::board::{CellContent, Orientation, SpecialPiece, TileModifier};
use crate::matches::MatchGroup;

use super::{GameEngine, GamePhase};

impl GameEngine {
    /// Clear all matched cells, apply modifier damage to adjacent cells,
    /// trigger any special pieces within the matched set, score points,
    /// and place a new special piece at spawn_at if applicable.
    pub(super) fn execute_resolution(
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

        // 6. Clear the cells (skip cells where Ice modifier still present — first hit removes ice,
        //    second hit clears the gem. The damage_modifier call above already decremented Ice hp.)
        for &(r, c) in &to_clear {
            if matches!(self.board.cells[r][c].modifier, Some(TileModifier::Ice { .. })) {
                // Ice was hit but not yet removed; leave the gem in place.
                continue;
            }
            self.anim_cells.dissolve((r, c));
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
                        // Spawn flash: cyan burst to distinguish creation from destruction
                        self.anim_cells.dissolve(pos);
                        break;
                    }
                }
            }
        }

        self.phase = GamePhase::Falling;
    }

    /// Expand `added` with cells destroyed by special piece `sp` at (r, c).
    pub(super) fn collect_explosion(
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
    pub(crate) fn damage_modifier(&mut self, r: usize, c: usize, direct: bool) {
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
}
