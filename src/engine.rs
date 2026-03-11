use crossterm::style::Color;
use serde::{Serialize, Deserialize};
use rand::Rng;

use crate::board_entity::{BoardEntity, Direction, ConveyorData};
use crate::game_board::GameBoard;
use crate::yarn::{Yarn, Stitch};
use crate::spool::Spool;
use crate::config::Config;
use crate::palette::select_palette;
use crate::solvability::{is_solvable, count_solutions};
use crate::color_serde;

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

#[derive(Debug, PartialEq, Clone)]
pub enum BonusState {
    None,
    TweezersActive { saved_row: u16, saved_col: u16 },
}

pub struct BonusInventory {
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
    pub scissors_spools: u16,
    pub balloon_count: u16,
}

// ── GameEngine ─────────────────────────────────────────────────────────────

pub struct GameEngine {
    pub board: GameBoard,
    pub yarn: Yarn,
    pub held_spools: Vec<Spool>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub spool_capacity: u16,
    pub spool_limit: usize,
    pub bonuses: BonusInventory,
    pub bonus_state: BonusState,
    pub ad_limit: Option<u16>,
    pub ads_used: u16,
}

impl GameEngine {
    /// Build a solvable game from config.
    pub fn new(config: &Config) -> Self {
        let color_mode = config.parsed_color_mode();
        let selected_palette = select_palette(color_mode, config.color_number);
        let mut board;
        let mut yarn;
        let mut attempts = 0u32;
        loop {
            board = GameBoard::make_random(
                config.board_height,
                config.board_width,
                &selected_palette,
                config.obstacle_percentage,
                config.spool_capacity,
                config.conveyor_percentage,
                config.conveyor_capacity,
            );
            yarn = Yarn::make_from_color_counter(
                board.count_spools(),
                config.yarn_lines,
                config.visible_stitches,
            );
            if is_solvable(&board, &yarn, config.spool_capacity, config.spool_limit) {
                if let Some(max) = config.max_solutions {
                    if count_solutions(&board, &yarn, config.spool_capacity, config.spool_limit, max) > max {
                        attempts += 1;
                        if attempts >= 100 { break; }
                        continue;
                    }
                }
                break;
            }
            attempts += 1;
            if attempts >= 100 { break; }
        }
        // Lock yarn stitches to match KeySpools on the board
        let mut key_colors: Vec<Color> = Vec::new();
        for row in &board.board {
            for cell in row {
                if let BoardEntity::KeySpool(c) = cell {
                    key_colors.push(*c);
                }
            }
        }
        for key_color in &key_colors {
            // Find the deepest (last) unlocked stitch of this color in any yarn column
            let mut best: Option<(usize, usize)> = None; // (col_idx, stitch_idx)
            for (ci, col) in yarn.board.iter().enumerate() {
                for (si, stitch) in col.iter().enumerate() {
                    if stitch.color == *key_color && !stitch.locked {
                        match best {
                            None => best = Some((ci, si)),
                            Some((_, prev_si)) if si > prev_si => best = Some((ci, si)),
                            _ => {}
                        }
                    }
                }
            }
            if let Some((ci, si)) = best {
                yarn.board[ci][si].locked = true;
            }
        }

        // Find first focusable cell for initial cursor position
        let (mut init_row, mut init_col) = (0u16, 0u16);
        'find_cursor: for r in 0..board.height {
            for c in 0..board.width {
                if board.is_focusable(r as usize, c as usize) {
                    init_row = r;
                    init_col = c;
                    break 'find_cursor;
                }
            }
        }
        Self {
            board,
            yarn,
            held_spools: Vec::new(),
            cursor_row: init_row,
            cursor_col: init_col,
            spool_capacity: config.spool_capacity,
            spool_limit: config.spool_limit,
            bonuses: BonusInventory {
                scissors: config.scissors,
                tweezers: config.tweezers,
                balloons: config.balloons,
                scissors_spools: config.scissors_spools,
                balloon_count: config.balloon_count,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        }
    }

    /// Set the ad limit for campaign levels. Call after `new()`.
    pub fn set_ad_limit(&mut self, limit: u16) {
        self.ad_limit = Some(limit);
    }

    // ── Actions ────────────────────────────────────────────────────────────

    pub fn move_cursor(&mut self, dir: Direction) -> Result<(), MoveError> {
        let (dr, dc) = dir.offset();
        let mut new_row = self.cursor_row as i32 + dr;
        let mut new_col = self.cursor_col as i32 + dc;
        let tweezers = matches!(self.bonus_state, BonusState::TweezersActive { .. });
        loop {
            if new_row < 0 || new_row >= self.board.height as i32
                || new_col < 0 || new_col >= self.board.width as i32
            {
                return Err(MoveError::OutOfBounds);
            }
            if tweezers || self.board.is_focusable(new_row as usize, new_col as usize) {
                self.cursor_row = new_row as u16;
                self.cursor_col = new_col as u16;
                return Ok(());
            }
            new_row += dr;
            new_col += dc;
        }
    }

    pub fn pick_up(&mut self) -> Result<(), PickError> {
        let row = self.cursor_row as usize;
        let col = self.cursor_col as usize;
        let tweezers = matches!(self.bonus_state, BonusState::TweezersActive { .. });

        let spool = match &self.board.board[row][col] {
            BoardEntity::Spool(c)    => Spool { color: *c, fill: 1, has_key: false },
            BoardEntity::KeySpool(c) => Spool { color: *c, fill: 1, has_key: true },
            _ => return Err(PickError::NotASpool),
        };

        if !tweezers && !self.board.is_selectable(row, col) {
            return Err(PickError::NotSelectable);
        }
        if self.held_spools.len() >= self.spool_limit {
            return Err(PickError::ActiveFull);
        }

        self.held_spools.push(spool);
        self.board.board[row][col] = BoardEntity::Void;

        if let Some((gr, gc)) = find_conveyor_for_output(&self.board.board, row, col) {
            advance_conveyor(&mut self.board.board, gr, gc, row, col);
        }

        // Exit tweezers mode after successful pick
        if let BonusState::TweezersActive { saved_row, saved_col } = self.bonus_state {
            self.cursor_row = saved_row;
            self.cursor_col = saved_col;
            self.bonus_state = BonusState::None;
            self.bonuses.tweezers -= 1;
        }

        Ok(())
    }

    /// Process the first held spool one yarn step in place.
    /// Removes the spool only if it has completed `spool_capacity` steps.
    /// Returns true if a spool was processed, false if held list was empty.
    pub fn process_one_active(&mut self) -> bool {
        if self.held_spools.is_empty() {
            return false;
        }
        self.yarn.process_one(&mut self.held_spools[0]);
        if self.held_spools[0].fill > self.spool_capacity {
            self.held_spools.remove(0);
        }
        self.yarn.cleanup_balloon_columns();
        true
    }

    /// Process all held spools one yarn step each (for NI binary).
    pub fn process_all_active(&mut self) {
        let mut i = 0;
        let count = self.held_spools.len();
        for _ in 0..count {
            if i >= self.held_spools.len() { break; }
            self.yarn.process_one(&mut self.held_spools[i]);
            if self.held_spools[i].fill > self.spool_capacity {
                self.held_spools.remove(i);
            } else {
                i += 1;
            }
        }
        self.yarn.cleanup_balloon_columns();
    }

    pub fn is_won(&self) -> bool {
        self.held_spools.is_empty()
            && self.yarn.board.iter().all(|col| col.is_empty())
            && self.board.board.iter().all(|row| {
                row.iter().all(|cell| !matches!(
                    cell,
                    BoardEntity::Spool(_) | BoardEntity::KeySpool(_)
                        | BoardEntity::Conveyor(_)
                ))
            })
    }

    pub fn status(&self) -> GameStatus {
        if self.is_won() {
            return GameStatus::Won;
        }
        if !self.held_spools.is_empty() {
            if !self.can_any_spool_progress()
                && (self.held_spools.len() >= self.spool_limit
                    || !self.board.has_selectable_spool())
            {
                return GameStatus::Stuck;
            }
        } else if !self.board.has_selectable_spool() {
            return GameStatus::Stuck;
        }
        GameStatus::Playing
    }

    /// Check if any held spool can match any yarn column's last stitch.
    fn can_any_spool_progress(&self) -> bool {
        for spool in &self.held_spools {
            for column in &self.yarn.board {
                let Some(last) = column.last() else { continue };
                if last.locked {
                    if last.color == spool.color && spool.has_key {
                        return true;
                    }
                    continue;
                }
                if last.color == spool.color {
                    return true;
                }
            }
        }
        false
    }

    // ── Bonuses ─────────────────────────────────────────────────────────

    /// Check if any bonus is currently active.
    pub fn is_bonus_active(&self) -> bool {
        self.bonus_state != BonusState::None || !self.yarn.balloon_columns.is_empty()
    }

    /// Scissors: deep-scan auto-wind the least-filled spool(s).
    pub fn use_scissors(&mut self) -> Result<(), BonusError> {
        if self.bonuses.scissors == 0 {
            return Err(BonusError::NoneLeft);
        }
        if self.held_spools.is_empty() {
            return Err(BonusError::NoHeldSpools);
        }
        if self.bonus_state != BonusState::None {
            return Err(BonusError::BonusActive);
        }

        self.bonuses.scissors -= 1;

        // Process up to scissors_spools spools, picking lowest fill each time
        for _ in 0..self.bonuses.scissors_spools {
            if self.held_spools.is_empty() { break; }

            // Find the spool with the lowest fill
            let min_idx = self.held_spools.iter()
                .enumerate()
                .min_by_key(|(_, s)| s.fill)
                .map(|(i, _)| i)
                .unwrap();

            // Deep-scan wind until complete or no more matches
            loop {
                if self.held_spools[min_idx].fill > self.spool_capacity {
                    break;
                }
                let prev_fill = self.held_spools[min_idx].fill;
                self.yarn.deep_scan_process(&mut self.held_spools[min_idx]);
                if self.held_spools[min_idx].fill == prev_fill {
                    break; // no match found anywhere
                }
            }

            // Remove if completed
            if self.held_spools[min_idx].fill > self.spool_capacity {
                self.held_spools.remove(min_idx);
            }
        }

        Ok(())
    }

    /// Tweezers: enter free-cursor mode. Cursor can move to any cell
    /// and pick up any spool regardless of selectability.
    pub fn use_tweezers(&mut self) -> Result<(), BonusError> {
        if self.bonuses.tweezers == 0 {
            return Err(BonusError::NoneLeft);
        }
        if self.bonus_state != BonusState::None {
            return Err(BonusError::BonusActive);
        }

        self.bonus_state = BonusState::TweezersActive {
            saved_row: self.cursor_row,
            saved_col: self.cursor_col,
        };
        // Don't decrement yet — only on successful pick
        Ok(())
    }

    /// Cancel tweezers mode without consuming the bonus.
    pub fn cancel_tweezers(&mut self) {
        if let BonusState::TweezersActive { saved_row, saved_col } = self.bonus_state {
            self.cursor_row = saved_row;
            self.cursor_col = saved_col;
            self.bonus_state = BonusState::None;
        }
    }

    /// Balloons: lift the front N stitches from each yarn column into
    /// separate pseudo-columns, exposing the stitches behind them.
    pub fn use_balloons(&mut self) -> Result<(), BonusError> {
        if self.bonuses.balloons == 0 {
            return Err(BonusError::NoneLeft);
        }
        if !self.yarn.balloon_columns.is_empty() {
            return Err(BonusError::BalloonColumnsNotEmpty);
        }
        if self.bonus_state != BonusState::None {
            return Err(BonusError::BonusActive);
        }

        self.bonuses.balloons -= 1;

        // Lift individual stitches into fixed balloon slots.
        // Left side: pop from leftmost non-empty column(s)
        let left_count = (self.bonuses.balloon_count / 2) as usize;
        for _ in 0..left_count {
            if let Some(idx) = self.yarn.board.iter().position(|c| !c.is_empty()) {
                if let Some(stitch) = self.yarn.board[idx].pop() {
                    self.yarn.balloon_columns.push(Some(stitch));
                }
            }
        }

        // Right side: pop from rightmost non-empty column(s)
        let right_count = ((self.bonuses.balloon_count + 1) / 2) as usize;
        for _ in 0..right_count {
            if let Some(idx) = self.yarn.board.iter().rposition(|c| !c.is_empty()) {
                if let Some(stitch) = self.yarn.board[idx].pop() {
                    self.yarn.balloon_columns.push(Some(stitch));
                }
            }
        }

        Ok(())
    }

    /// Return true if the player is allowed to watch an ad right now.
    /// Allowed when: no ad limit set, or ads_used < ad_limit.
    pub fn can_watch_ad(&self) -> bool {
        match self.ad_limit {
            None => true,
            Some(limit) => self.ads_used < limit,
        }
    }

    /// Grant one free scissors bonus as the reward for watching an ad.
    /// Increments ads_used.
    pub fn watch_ad(&mut self) {
        self.bonuses.scissors += 1;
        self.ads_used += 1;
    }

    // ── Serialisation ──────────────────────────────────────────────────────

    pub fn to_json(&self) -> String {
        serde_json::to_string(&GameStateSnapshot::from_engine(self))
            .expect("snapshot serialisation failed")
    }

    pub fn from_json(s: &str) -> Result<Self, String> {
        let snap: GameStateSnapshot =
            serde_json::from_str(s).map_err(|e| e.to_string())?;
        snap.into_engine()
    }

    /// Generate a random 8-char alphanumeric game hash.
    pub fn generate_hash() -> String {
        let mut rng = rand::rng();
        (0..8)
            .map(|_| {
                let idx = rng.random_range(0..36u8);
                (if idx < 10 { b'0' + idx } else { b'a' + idx - 10 }) as char
            })
            .collect()
    }
}

// ── Conveyor helpers ────────────────────────────────────────────────────────

fn find_conveyor_for_output(
    board: &Vec<Vec<BoardEntity>>,
    out_row: usize,
    out_col: usize,
) -> Option<(usize, usize)> {
    for r in 0..board.len() {
        for c in 0..board[r].len() {
            if let BoardEntity::Conveyor(ref data) = board[r][c] {
                let (dr, dc) = data.output_dir.offset();
                if r as i32 + dr == out_row as i32 && c as i32 + dc == out_col as i32 {
                    return Some((r, c));
                }
            }
        }
    }
    None
}

fn advance_conveyor(
    board: &mut Vec<Vec<BoardEntity>>,
    conv_row: usize,
    conv_col: usize,
    out_row: usize,
    out_col: usize,
) {
    enum Action { Spawn(Color), Deplete }

    let action = if let BoardEntity::Conveyor(ref mut data) = board[conv_row][conv_col] {
        if data.queue.is_empty() { Action::Deplete }
        else { Action::Spawn(data.queue.remove(0)) }
    } else {
        return;
    };

    match action {
        Action::Spawn(color) => board[out_row][out_col] = BoardEntity::Spool(color),
        Action::Deplete      => board[conv_row][conv_col] = BoardEntity::EmptyConveyor,
    }
}

// ── Snapshot types (serde mirror of engine state) ──────────────────────────

#[derive(Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub spool_capacity: u16,
    pub spool_limit: usize,
    pub board_height: u16,
    pub board_width: u16,
    pub board: Vec<Vec<String>>,
    pub yarn_lines: u16,
    pub visible_stitches: u16,
    pub yarn: Vec<Vec<YarnStitchSnap>>,
    pub held_spools: Vec<SpoolSnap>,
    #[serde(default)]
    pub scissors: u16,
    #[serde(default)]
    pub tweezers: u16,
    #[serde(default)]
    pub balloons: u16,
    #[serde(default)]
    pub scissors_spools: u16,
    #[serde(default)]
    pub balloon_count: u16,
    #[serde(default)]
    pub balloon_columns: Vec<Option<YarnStitchSnap>>,
    #[serde(default)]
    pub ad_limit: Option<u16>,
    #[serde(default)]
    pub ads_used: u16,
}

#[derive(Serialize, Deserialize)]
pub struct YarnStitchSnap { pub color: String, pub locked: bool }

#[derive(Serialize, Deserialize)]
pub struct SpoolSnap { pub color: String, pub fill: u16, pub has_key: bool }

impl GameStateSnapshot {
    fn from_engine(e: &GameEngine) -> Self {
        Self {
            cursor_row: e.cursor_row,
            cursor_col: e.cursor_col,
            spool_capacity: e.spool_capacity,
            spool_limit: e.spool_limit,
            board_height: e.board.height,
            board_width: e.board.width,
            board: e.board.board.iter()
                .map(|row| row.iter().map(cell_to_str).collect())
                .collect(),
            yarn_lines: e.yarn.yarn_lines,
            visible_stitches: e.yarn.visible_stitches,
            yarn: e.yarn.board.iter()
                .map(|col| col.iter().map(|s| YarnStitchSnap {
                    color: color_serde::color_to_str(&s.color),
                    locked: s.locked,
                }).collect())
                .collect(),
            held_spools: e.held_spools.iter()
                .map(|s| SpoolSnap {
                    color: color_serde::color_to_str(&s.color),
                    fill: s.fill,
                    has_key: s.has_key,
                })
                .collect(),
            scissors: e.bonuses.scissors,
            tweezers: e.bonuses.tweezers,
            balloons: e.bonuses.balloons,
            scissors_spools: e.bonuses.scissors_spools,
            balloon_count: e.bonuses.balloon_count,
            balloon_columns: e.yarn.balloon_columns.iter()
                .map(|opt| opt.as_ref().map(|s| YarnStitchSnap {
                    color: color_serde::color_to_str(&s.color),
                    locked: s.locked,
                }))
                .collect(),
            ad_limit: e.ad_limit,
            ads_used: e.ads_used,
        }
    }

    fn into_engine(self) -> Result<GameEngine, String> {
        let board_cells: Result<Vec<Vec<BoardEntity>>, String> = self.board.iter()
            .map(|row| row.iter().map(|s| str_to_cell(s)).collect())
            .collect();
        let board_cells = board_cells?;

        let yarn_cols: Result<Vec<Vec<Stitch>>, String> = self.yarn.iter()
            .map(|col| col.iter().map(|s| {
                let color = color_serde::str_to_color(&s.color)
                    .ok_or_else(|| format!("bad color: {}", s.color))?;
                Ok(Stitch { color, locked: s.locked })
            }).collect())
            .collect();
        let yarn_cols = yarn_cols?;

        let spools: Result<Vec<Spool>, String> = self.held_spools.iter()
            .map(|s| {
                let color = color_serde::str_to_color(&s.color)
                    .ok_or_else(|| format!("bad color: {}", s.color))?;
                Ok(Spool { color, fill: s.fill, has_key: s.has_key })
            })
            .collect();
        let spools = spools?;

        let balloon_cols: Result<Vec<Option<Stitch>>, String> = self.balloon_columns.iter()
            .map(|opt| opt.as_ref().map(|s| {
                let color = color_serde::str_to_color(&s.color)
                    .ok_or_else(|| format!("bad color: {}", s.color))?;
                Ok(Stitch { color, locked: s.locked })
            }).transpose())
            .collect();
        let balloon_cols = balloon_cols?;

        Ok(GameEngine {
            board: GameBoard {
                board: board_cells,
                height: self.board_height,
                width: self.board_width,
                spool_capacity: self.spool_capacity,
            },
            yarn: Yarn {
                board: yarn_cols,
                yarn_lines: self.yarn_lines,
                visible_stitches: self.visible_stitches,
                balloon_columns: balloon_cols,
            },
            held_spools: spools,
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            spool_capacity: self.spool_capacity,
            spool_limit: self.spool_limit,
            bonuses: BonusInventory {
                scissors: self.scissors,
                tweezers: self.tweezers,
                balloons: self.balloons,
                scissors_spools: if self.scissors_spools == 0 { 1 } else { self.scissors_spools },
                balloon_count: if self.balloon_count == 0 { 2 } else { self.balloon_count },
            },
            bonus_state: BonusState::None,
            ad_limit: self.ad_limit,
            ads_used: self.ads_used,
        })
    }
}

fn cell_to_str(cell: &BoardEntity) -> String {
    match cell {
        BoardEntity::Spool(c)    => format!("T:{}", color_serde::color_to_str(c)),
        BoardEntity::KeySpool(c) => format!("K:{}", color_serde::color_to_str(c)),
        BoardEntity::Obstacle    => "X".into(),
        BoardEntity::Void        => "V".into(),
        BoardEntity::Conveyor(d) => {
            let dir = match d.output_dir {
                Direction::Up    => "up",
                Direction::Down  => "down",
                Direction::Left  => "left",
                Direction::Right => "right",
            };
            let queue: Vec<String> = d.queue.iter().map(|c| color_serde::color_to_str(c)).collect();
            format!("G:{}:{}:{}", color_serde::color_to_str(&d.color), dir, queue.join(","))
        }
        BoardEntity::EmptyConveyor => "#".into(),
    }
}

fn str_to_cell(s: &str) -> Result<BoardEntity, String> {
    if s == "X" { return Ok(BoardEntity::Obstacle); }
    if s == "V" { return Ok(BoardEntity::Void); }
    if s == "#" { return Ok(BoardEntity::EmptyConveyor); }

    let parts: Vec<&str> = s.splitn(4, ':').collect();
    match parts.as_slice() {
        ["T", color] => {
            let c = color_serde::str_to_color(color)
                .ok_or_else(|| format!("bad color: {color}"))?;
            Ok(BoardEntity::Spool(c))
        }
        ["K", color] => {
            let c = color_serde::str_to_color(color)
                .ok_or_else(|| format!("bad color: {color}"))?;
            Ok(BoardEntity::KeySpool(c))
        }
        ["G", color, dir_str, queue_str] => {
            let color = color_serde::str_to_color(color)
                .ok_or_else(|| format!("bad conveyor color: {color}"))?;
            let output_dir = match *dir_str {
                "up"    => Direction::Up,
                "down"  => Direction::Down,
                "left"  => Direction::Left,
                "right" => Direction::Right,
                d       => return Err(format!("bad direction: {d}")),
            };
            let queue: Result<Vec<Color>, String> = if queue_str.is_empty() {
                Ok(vec![])
            } else {
                queue_str.split(',')
                    .map(|c| color_serde::str_to_color(c)
                        .ok_or_else(|| format!("bad queue color: {c}")))
                    .collect()
            };
            Ok(BoardEntity::Conveyor(ConveyorData { color, output_dir, queue: queue? }))
        }
        _ => Err(format!("cannot parse cell: {s}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_engine() -> GameEngine {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red),  BoardEntity::Spool(Color::Blue), BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Spool(Color::Blue), BoardEntity::Obstacle,            BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Spool(Color::Red),  BoardEntity::Spool(Color::Blue), BoardEntity::Spool(Color::Red)],
            ],
            height: 3,
            width: 3,
            spool_capacity: 1,
        };
        let yarn = Yarn {
            board: vec![
                vec![Stitch { color: Color::Red, locked: false }, Stitch { color: Color::Blue, locked: false }],
                vec![Stitch { color: Color::Red, locked: false }, Stitch { color: Color::Red, locked: false }],
            ],
            yarn_lines: 2,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };
        GameEngine {
            board,
            yarn,
            held_spools: vec![],
            cursor_row: 0,
            cursor_col: 0,
            spool_capacity: 1,
            spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        }
    }

    #[test]
    fn move_cursor_right_succeeds() {
        let mut e = default_engine();
        assert!(e.move_cursor(Direction::Right).is_ok());
        assert_eq!(e.cursor_col, 1);
    }
    #[test]
    fn move_cursor_left_at_edge_fails() {
        let mut e = default_engine();
        assert_eq!(e.move_cursor(Direction::Left), Err(MoveError::OutOfBounds));
    }
    #[test]
    fn move_cursor_up_at_edge_fails() {
        let mut e = default_engine();
        assert_eq!(e.move_cursor(Direction::Up), Err(MoveError::OutOfBounds));
    }
    #[test]
    fn pick_up_top_row_succeeds() {
        let mut e = default_engine();
        assert!(e.pick_up().is_ok());
        assert_eq!(e.held_spools.len(), 1);
    }
    #[test]
    fn pick_up_obstacle_fails() {
        let mut e = default_engine();
        e.cursor_row = 1; e.cursor_col = 1;
        assert_eq!(e.pick_up(), Err(PickError::NotASpool));
    }
    #[test]
    fn pick_up_non_exposed_fails() {
        let mut e = default_engine();
        e.cursor_row = 2; e.cursor_col = 0;
        assert_eq!(e.pick_up(), Err(PickError::NotSelectable));
    }
    #[test]
    fn pick_up_active_full_fails() {
        let mut e = default_engine();
        e.spool_limit = 0;
        assert_eq!(e.pick_up(), Err(PickError::ActiveFull));
    }
    #[test]
    fn process_one_active_removes_when_done() {
        let mut e = default_engine();
        e.held_spools.push(Spool { color: Color::Red, fill: 1, has_key: false });
        e.process_one_active();
        // spool_capacity=1: after one successful process, fill becomes 2 > 1, discarded
        assert_eq!(e.held_spools.len(), 0);
    }
    #[test]
    fn is_won_false_while_board_has_spools() {
        assert!(!default_engine().is_won());
    }
    #[test]
    fn snapshot_roundtrip() {
        let e = default_engine();
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.cursor_row, e.cursor_row);
        assert_eq!(e2.spool_capacity, e.spool_capacity);
        assert_eq!(e2.board.height, e.board.height);
        assert_eq!(e2.yarn.yarn_lines, e.yarn.yarn_lines);
    }
    #[test]
    fn cell_roundtrip_all_variants() {
        let cells = vec![
            BoardEntity::Spool(Color::Red),
            BoardEntity::KeySpool(Color::Blue),
            BoardEntity::Obstacle,
            BoardEntity::Void,
            BoardEntity::EmptyConveyor,
            BoardEntity::Conveyor(ConveyorData {
                color: Color::Green,
                output_dir: Direction::Down,
                queue: vec![Color::Red, Color::Blue],
            }),
        ];
        for cell in &cells {
            let s = cell_to_str(cell);
            str_to_cell(&s).unwrap_or_else(|_| panic!("failed to parse: {s}"));
        }
    }

    #[test]
    fn move_cursor_down_succeeds() {
        let mut e = default_engine();
        e.board.board[0][0] = BoardEntity::Void;
        assert!(e.move_cursor(Direction::Down).is_ok());
        assert_eq!(e.cursor_row, 1);
    }
    #[test]
    fn move_cursor_down_at_edge_fails() {
        let mut e = default_engine();
        e.cursor_row = 2;
        assert_eq!(e.move_cursor(Direction::Down), Err(MoveError::OutOfBounds));
    }
    #[test]
    fn move_cursor_right_at_edge_fails() {
        let mut e = default_engine();
        e.cursor_col = 2;
        assert_eq!(e.move_cursor(Direction::Right), Err(MoveError::OutOfBounds));
    }
    #[test]
    fn move_cursor_skips_non_focusable_spools() {
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red),  BoardEntity::Void,     BoardEntity::Void],
                vec![BoardEntity::Spool(Color::Blue), BoardEntity::Obstacle, BoardEntity::Void],
                vec![BoardEntity::Spool(Color::Red),  BoardEntity::Void,     BoardEntity::Void],
            ],
            height: 3,
            width: 3,
            spool_capacity: 1,
        };
        let mut e = GameEngine {
            board,
            yarn: Yarn {
                board: vec![vec![Stitch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_stitches: 3,
                balloon_columns: Vec::new(),
            },
            held_spools: vec![],
            cursor_row: 0, cursor_col: 0,
            spool_capacity: 1, spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert!(e.move_cursor(Direction::Down).is_ok());
        assert_eq!(e.cursor_row, 2); // skipped row 1
        assert_eq!(e.cursor_col, 0);
    }
    #[test]
    fn move_cursor_down_into_all_buried_fails() {
        let mut e = default_engine();
        assert_eq!(e.move_cursor(Direction::Down), Err(MoveError::OutOfBounds));
    }

    #[test]
    fn pick_up_makes_cell_void() {
        let mut e = default_engine();
        e.pick_up().unwrap();
        assert!(matches!(e.board.board[0][0], BoardEntity::Void));
    }
    #[test]
    fn pick_up_key_spool_sets_has_key() {
        let mut e = default_engine();
        e.board.board[0][0] = BoardEntity::KeySpool(Color::Red);
        e.pick_up().unwrap();
        assert!(e.held_spools[0].has_key);
        assert_eq!(e.held_spools[0].color, Color::Red);
    }

    #[test]
    fn process_all_active_processes_each_spool() {
        let mut e = default_engine();
        e.spool_capacity = 2;
        e.held_spools = vec![
            Spool { color: Color::Red,  fill: 1, has_key: false },
            Spool { color: Color::Blue, fill: 1, has_key: false },
            Spool { color: Color::Red,  fill: 1, has_key: false },
        ];
        e.process_all_active();
        assert_eq!(e.held_spools.len(), 3);
        for s in &e.held_spools {
            assert_eq!(s.fill, 2);
        }
    }
    #[test]
    fn process_all_active_removes_completed() {
        let mut e = default_engine(); // spool_capacity=1
        e.held_spools = vec![
            Spool { color: Color::Red, fill: 1, has_key: false },
            Spool { color: Color::Red, fill: 1, has_key: false },
        ];
        e.process_all_active();
        assert_eq!(e.held_spools.len(), 0);
    }
    #[test]
    fn process_one_active_returns_false_when_empty() {
        let mut e = default_engine();
        assert!(!e.process_one_active());
    }

    #[test]
    fn is_won_true_when_board_cleared() {
        let e = GameEngine {
            board: GameBoard {
                board: vec![
                    vec![BoardEntity::Void, BoardEntity::Obstacle],
                    vec![BoardEntity::EmptyConveyor, BoardEntity::Void],
                ],
                height: 2, width: 2, spool_capacity: 1,
            },
            yarn: Yarn { board: vec![vec![], vec![]], yarn_lines: 2, visible_stitches: 3, balloon_columns: Vec::new() },
            held_spools: vec![],
            cursor_row: 0, cursor_col: 0,
            spool_capacity: 1, spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert!(e.is_won());
    }
    #[test]
    fn is_won_false_with_held_spools() {
        let mut e = default_engine();
        e.board.board = vec![vec![BoardEntity::Void]];
        e.board.height = 1; e.board.width = 1;
        e.yarn.board = vec![vec![]];
        e.held_spools = vec![Spool { color: Color::Red, fill: 1, has_key: false }];
        assert!(!e.is_won());
    }
    #[test]
    fn is_won_false_with_remaining_yarn() {
        let mut e = default_engine();
        e.board.board = vec![vec![BoardEntity::Void]];
        e.board.height = 1; e.board.width = 1;
        e.held_spools = vec![];
        assert!(!e.is_won());
    }

    #[test]
    fn generate_hash_format() {
        let h = GameEngine::generate_hash();
        assert_eq!(h.len(), 8);
        assert!(h.chars().all(|c| c.is_ascii_alphanumeric()));
    }
    #[test]
    fn generate_hash_uniqueness() {
        let hashes: Vec<String> = (0..100).map(|_| GameEngine::generate_hash()).collect();
        let mut deduped = hashes.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(hashes.len(), deduped.len());
    }

    #[test]
    fn new_from_config_produces_solvable_game() {
        let config = Config {
            board_height: 4, board_width: 4, color_number: 3,
            color_mode: "dark".into(), spool_limit: 7,
            spool_capacity: 2, yarn_lines: 3, obstacle_percentage: 5,
            visible_stitches: 4, conveyor_capacity: 3, conveyor_percentage: 5,
            layout: "auto".into(),
            scale: 1,
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2,
            ad_file: None,
            max_solutions: None,
        };
        let e = GameEngine::new(&config);
        assert_eq!(e.board.height, 4);
        assert_eq!(e.board.width, 4);
        assert_eq!(e.spool_capacity, 2);
        assert!(e.held_spools.is_empty());
        let total_stitches: usize = e.yarn.board.iter().map(|c| c.len()).sum();
        assert!(total_stitches > 0);
    }

    #[test]
    fn snapshot_roundtrip_with_conveyor() {
        let mut e = default_engine();
        e.board.board[1][0] = BoardEntity::Conveyor(ConveyorData {
            color: Color::Cyan,
            output_dir: Direction::Right,
            queue: vec![Color::Red, Color::Blue, Color::Green],
        });
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        match &e2.board.board[1][0] {
            BoardEntity::Conveyor(d) => {
                assert_eq!(d.color, Color::Cyan);
                assert_eq!(d.output_dir, Direction::Right);
                assert_eq!(d.queue, vec![Color::Red, Color::Blue, Color::Green]);
            }
            other => panic!("expected Conveyor, got {:?}", cell_to_str(other)),
        }
    }
    #[test]
    fn snapshot_roundtrip_with_locked_stitches() {
        let mut e = default_engine();
        e.yarn.board[0].push(Stitch { color: Color::Magenta, locked: true });
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        let last = e2.yarn.board[0].last().unwrap();
        assert!(last.locked);
        assert_eq!(last.color, Color::Magenta);
    }
    #[test]
    fn snapshot_roundtrip_with_key_spools() {
        let mut e = default_engine();
        e.board.board[0][1] = BoardEntity::KeySpool(Color::Yellow);
        e.held_spools.push(Spool { color: Color::Yellow, fill: 2, has_key: true });
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        match &e2.board.board[0][1] {
            BoardEntity::KeySpool(c) => assert_eq!(*c, Color::Yellow),
            other => panic!("expected KeySpool, got {:?}", cell_to_str(other)),
        }
        assert!(e2.held_spools[0].has_key);
        assert_eq!(e2.held_spools[0].fill, 2);
    }
    #[test]
    fn snapshot_roundtrip_with_held_spools() {
        let mut e = default_engine();
        e.held_spools = vec![
            Spool { color: Color::Red,  fill: 1, has_key: false },
            Spool { color: Color::Blue, fill: 3, has_key: true },
        ];
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.held_spools.len(), 2);
        assert_eq!(e2.held_spools[0].color, Color::Red);
        assert_eq!(e2.held_spools[0].fill, 1);
        assert!(!e2.held_spools[0].has_key);
        assert_eq!(e2.held_spools[1].color, Color::Blue);
        assert_eq!(e2.held_spools[1].fill, 3);
        assert!(e2.held_spools[1].has_key);
    }

    #[test]
    fn from_json_rejects_bad_json() {
        assert!(GameEngine::from_json("not json at all").is_err());
    }
    #[test]
    fn from_json_rejects_bad_color() {
        let e = default_engine();
        let mut json = e.to_json();
        json = json.replace("\"red\"", "\"neonpink\"");
        assert!(GameEngine::from_json(&json).is_err());
    }
    #[test]
    fn from_json_rejects_bad_cell() {
        let e = default_engine();
        let mut json = e.to_json();
        json = json.replace("\"T:red\"", "\"Z:invalid\"");
        assert!(GameEngine::from_json(&json).is_err());
    }

    #[test]
    fn status_playing_at_start() {
        let e = default_engine();
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn status_won_when_cleared() {
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void, BoardEntity::Obstacle]],
                height: 1, width: 2, spool_capacity: 1,
            },
            yarn: Yarn { board: vec![vec![], vec![]], yarn_lines: 2, visible_stitches: 3, balloon_columns: Vec::new() },
            held_spools: vec![],
            cursor_row: 0, cursor_col: 0,
            spool_capacity: 1, spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Won);
    }

    #[test]
    fn status_stuck_front_spool_blocked() {
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, spool_capacity: 1,
            },
            yarn: Yarn {
                board: vec![vec![Stitch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_stitches: 3,
                balloon_columns: Vec::new(),
            },
            held_spools: vec![Spool { color: Color::Green, fill: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            spool_capacity: 3, spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    #[test]
    fn status_stuck_no_selectable_spools_on_board() {
        let e = GameEngine {
            board: GameBoard {
                board: vec![
                    vec![BoardEntity::Obstacle, BoardEntity::Obstacle],
                    vec![BoardEntity::Spool(Color::Red), BoardEntity::Spool(Color::Blue)],
                ],
                height: 2, width: 2, spool_capacity: 1,
            },
            yarn: Yarn {
                board: vec![vec![Stitch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_stitches: 3,
                balloon_columns: Vec::new(),
            },
            held_spools: vec![],
            cursor_row: 0, cursor_col: 0,
            spool_capacity: 1, spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    #[test]
    fn status_playing_when_front_spool_can_match() {
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, spool_capacity: 1,
            },
            yarn: Yarn {
                board: vec![vec![Stitch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_stitches: 3,
                balloon_columns: Vec::new(),
            },
            held_spools: vec![Spool { color: Color::Red, fill: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            spool_capacity: 3, spool_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_spools: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn use_scissors_completes_spool() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        e.bonuses.scissors_spools = 1;
        e.spool_capacity = 2;
        e.held_spools = vec![
            Spool { color: Color::Red, fill: 1, has_key: false },
        ];
        let result = e.use_scissors();
        assert!(result.is_ok());
        assert_eq!(e.bonuses.scissors, 0);
        assert_eq!(e.held_spools.len(), 0);
    }

    #[test]
    fn use_scissors_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.scissors = 0;
        e.held_spools = vec![Spool { color: Color::Red, fill: 1, has_key: false }];
        assert_eq!(e.use_scissors(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn use_scissors_no_held_spools_fails() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        assert_eq!(e.use_scissors(), Err(BonusError::NoHeldSpools));
    }

    #[test]
    fn use_scissors_picks_least_filled_spool() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        e.bonuses.scissors_spools = 1;
        e.spool_capacity = 1;
        e.held_spools = vec![
            Spool { color: Color::Red,  fill: 2, has_key: false },
            Spool { color: Color::Blue, fill: 1, has_key: false },
        ];
        let _ = e.use_scissors();
        assert_eq!(e.held_spools.len(), 1);
        assert_eq!(e.held_spools[0].color, Color::Red);
    }

    #[test]
    fn use_tweezers_enters_mode() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.cursor_row = 0;
        e.cursor_col = 0;
        let result = e.use_tweezers();
        assert!(result.is_ok());
        assert_eq!(e.bonus_state, BonusState::TweezersActive { saved_row: 0, saved_col: 0 });
        assert_eq!(e.bonuses.tweezers, 1);
    }

    #[test]
    fn use_tweezers_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.tweezers = 0;
        assert_eq!(e.use_tweezers(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn tweezers_mode_cursor_moves_anywhere() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        let result = e.move_cursor(Direction::Down);
        assert!(result.is_ok());
        assert_eq!(e.cursor_row, 1);
    }

    #[test]
    fn tweezers_pick_up_ignores_selectability() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        e.cursor_row = 2;
        e.cursor_col = 0;
        let result = e.pick_up();
        assert!(result.is_ok());
        assert_eq!(e.held_spools.len(), 1);
        assert_eq!(e.cursor_row, 0);
        assert_eq!(e.cursor_col, 0);
        assert_eq!(e.bonuses.tweezers, 0);
        assert_eq!(e.bonus_state, BonusState::None);
    }

    #[test]
    fn cancel_tweezers_restores_cursor() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        e.cursor_row = 2;
        e.cursor_col = 1;
        e.cancel_tweezers();
        assert_eq!(e.cursor_row, 0);
        assert_eq!(e.cursor_col, 0);
        assert_eq!(e.bonus_state, BonusState::None);
        assert_eq!(e.bonuses.tweezers, 1);
    }

    #[test]
    fn use_balloons_lifts_stitches() {
        let mut e = default_engine();
        e.bonuses.balloons = 1;
        e.bonuses.balloon_count = 1;
        let total_before: usize = e.yarn.board.iter().map(|c| c.len()).sum();
        let result = e.use_balloons();
        assert!(result.is_ok());
        assert_eq!(e.bonuses.balloons, 0);
        assert!(!e.yarn.balloon_columns.is_empty());
        let total_regular: usize = e.yarn.board.iter().map(|c| c.len()).sum();
        let total_balloon = e.yarn.balloon_columns.iter().filter(|s| s.is_some()).count();
        assert_eq!(total_before, total_regular + total_balloon);
    }

    #[test]
    fn use_balloons_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.balloons = 0;
        assert_eq!(e.use_balloons(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn use_balloons_while_columns_exist_fails() {
        let mut e = default_engine();
        e.bonuses.balloons = 2;
        e.bonuses.balloon_count = 1;
        e.use_balloons().unwrap();
        assert_eq!(e.use_balloons(), Err(BonusError::BalloonColumnsNotEmpty));
    }

    #[test]
    fn snapshot_roundtrip_with_bonuses() {
        let mut e = default_engine();
        e.bonuses.scissors = 3;
        e.bonuses.tweezers = 2;
        e.bonuses.balloons = 1;
        e.yarn.balloon_columns = vec![Some(Stitch { color: Color::Red, locked: false })];
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.bonuses.scissors, 3);
        assert_eq!(e2.bonuses.tweezers, 2);
        assert_eq!(e2.bonuses.balloons, 1);
        assert_eq!(e2.yarn.balloon_columns.len(), 1);
        assert_eq!(e2.yarn.balloon_columns[0].as_ref().unwrap().color, Color::Red);
    }

    #[test]
    fn new_engine_has_locked_stitches_when_keys_present() {
        let config = Config {
            board_height: 6, board_width: 6, color_number: 4,
            color_mode: "dark".into(), spool_limit: 7,
            spool_capacity: 2, yarn_lines: 4, obstacle_percentage: 0,
            visible_stitches: 6, conveyor_capacity: 0, conveyor_percentage: 0,
            layout: "auto".into(), scale: 1,
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2, ad_file: None,
            max_solutions: None,
        };
        let e = GameEngine::new(&config);

        let key_count: usize = e.board.board.iter().flatten().filter(|c|
            matches!(c, BoardEntity::KeySpool(_))
        ).count();

        let lock_count: usize = e.yarn.board.iter()
            .flat_map(|col| col.iter())
            .filter(|s| s.locked)
            .count();

        assert_eq!(key_count, lock_count,
            "keys={} but locked_stitches={}", key_count, lock_count);
    }
}
