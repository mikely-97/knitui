use crossterm::style::Color;
use serde::{Serialize, Deserialize};
use rand::Rng;

use crate::board_entity::{BoardEntity, Direction, GeneratorData};
use crate::game_board::GameBoard;
use crate::yarn::{Yarn, Patch};
use crate::active_threads::Thread;
use crate::config::Config;
use crate::palette::select_palette;
use crate::solvability::is_solvable;
use crate::color_serde;

// ── Error / result types ───────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum MoveError {
    OutOfBounds,
}

#[derive(Debug, PartialEq)]
pub enum PickError {
    NotSelectable,
    NotAThread,
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
    NoActiveThreads,
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
    pub scissors_threads: u16,
    pub balloon_count: u16,
}

// ── GameEngine ─────────────────────────────────────────────────────────────

pub struct GameEngine {
    pub board: GameBoard,
    pub yarn: Yarn,
    pub active_threads: Vec<Thread>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub knit_volume: u16,
    pub active_threads_limit: usize,
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
                config.knit_volume,
                config.generator_percentage,
                config.generator_capacity,
            );
            yarn = Yarn::make_from_color_counter(
                board.count_knits(),
                config.yarn_lines,
                config.visible_patches,
            );
            if is_solvable(&board, &yarn, config.knit_volume, config.active_threads_limit) {
                break;
            }
            attempts += 1;
            if attempts >= 100 { break; }
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
            active_threads: Vec::new(),
            cursor_row: init_row,
            cursor_col: init_col,
            knit_volume: config.knit_volume,
            active_threads_limit: config.active_threads_limit,
            bonuses: BonusInventory {
                scissors: config.scissors,
                tweezers: config.tweezers,
                balloons: config.balloons,
                scissors_threads: config.scissors_threads,
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

        let thread = match &self.board.board[row][col] {
            BoardEntity::Thread(c)    => Thread { color: *c, status: 1, has_key: false },
            BoardEntity::KeyThread(c) => Thread { color: *c, status: 1, has_key: true },
            _ => return Err(PickError::NotAThread),
        };

        if !tweezers && !self.board.is_selectable(row, col) {
            return Err(PickError::NotSelectable);
        }
        if self.active_threads.len() >= self.active_threads_limit {
            return Err(PickError::ActiveFull);
        }

        self.active_threads.push(thread);
        self.board.board[row][col] = BoardEntity::Void;

        if let Some((gr, gc)) = find_generator_for_output(&self.board.board, row, col) {
            advance_generator(&mut self.board.board, gr, gc, row, col);
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

    /// Process the first active thread one yarn step in place.
    /// Removes the thread only if it has completed `knit_volume` steps.
    /// Returns true if a thread was processed, false if active list was empty.
    pub fn process_one_active(&mut self) -> bool {
        if self.active_threads.is_empty() {
            return false;
        }
        self.yarn.process_one(&mut self.active_threads[0]);
        if self.active_threads[0].status > self.knit_volume {
            self.active_threads.remove(0);
        }
        self.yarn.cleanup_balloon_columns();
        true
    }

    /// Process all active threads one yarn step each (for NI binary).
    pub fn process_all_active(&mut self) {
        let mut i = 0;
        let count = self.active_threads.len();
        for _ in 0..count {
            if i >= self.active_threads.len() { break; }
            self.yarn.process_one(&mut self.active_threads[i]);
            if self.active_threads[i].status > self.knit_volume {
                self.active_threads.remove(i);
            } else {
                i += 1;
            }
        }
        self.yarn.cleanup_balloon_columns();
    }

    pub fn is_won(&self) -> bool {
        self.active_threads.is_empty()
            && self.yarn.board.iter().all(|col| col.is_empty())
            && self.board.board.iter().all(|row| {
                row.iter().all(|cell| !matches!(
                    cell,
                    BoardEntity::Thread(_) | BoardEntity::KeyThread(_)
                        | BoardEntity::Generator(_)
                ))
            })
    }

    pub fn status(&self) -> GameStatus {
        if self.is_won() {
            return GameStatus::Won;
        }
        if !self.active_threads.is_empty() {
            if !self.can_any_thread_progress()
                && (self.active_threads.len() >= self.active_threads_limit
                    || !self.board.has_selectable_thread())
            {
                return GameStatus::Stuck;
            }
        } else if !self.board.has_selectable_thread() {
            return GameStatus::Stuck;
        }
        GameStatus::Playing
    }

    /// Check if any active thread can match any yarn column's last patch.
    fn can_any_thread_progress(&self) -> bool {
        for thread in &self.active_threads {
            for column in &self.yarn.board {
                let Some(last) = column.last() else { continue };
                if last.locked {
                    if last.color == thread.color && thread.has_key {
                        return true;
                    }
                    continue;
                }
                if last.color == thread.color {
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

    /// Scissors: deep-scan auto-knit the least-progressed thread(s).
    pub fn use_scissors(&mut self) -> Result<(), BonusError> {
        if self.bonuses.scissors == 0 {
            return Err(BonusError::NoneLeft);
        }
        if self.active_threads.is_empty() {
            return Err(BonusError::NoActiveThreads);
        }
        if self.is_bonus_active() {
            return Err(BonusError::BonusActive);
        }

        self.bonuses.scissors -= 1;

        // Process up to scissors_threads threads, picking lowest status each time
        for _ in 0..self.bonuses.scissors_threads {
            if self.active_threads.is_empty() { break; }

            // Find the thread with the lowest status
            let min_idx = self.active_threads.iter()
                .enumerate()
                .min_by_key(|(_, t)| t.status)
                .map(|(i, _)| i)
                .unwrap();

            // Deep-scan knit until complete or no more matches
            loop {
                if self.active_threads[min_idx].status > self.knit_volume {
                    break;
                }
                let prev_status = self.active_threads[min_idx].status;
                self.yarn.deep_scan_process(&mut self.active_threads[min_idx]);
                if self.active_threads[min_idx].status == prev_status {
                    break; // no match found anywhere
                }
            }

            // Remove if completed
            if self.active_threads[min_idx].status > self.knit_volume {
                self.active_threads.remove(min_idx);
            }
        }

        Ok(())
    }

    /// Tweezers: enter free-cursor mode. Cursor can move to any cell
    /// and pick up any thread regardless of selectability.
    pub fn use_tweezers(&mut self) -> Result<(), BonusError> {
        if self.bonuses.tweezers == 0 {
            return Err(BonusError::NoneLeft);
        }
        if self.is_bonus_active() {
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

    /// Balloons: lift the front N patches from each yarn column into
    /// separate pseudo-columns, exposing the patches behind them.
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

        // Lift individual patches into fixed balloon slots.
        // Left side: pop from leftmost non-empty column(s)
        let left_count = (self.bonuses.balloon_count / 2) as usize;
        for _ in 0..left_count {
            if let Some(idx) = self.yarn.board.iter().position(|c| !c.is_empty()) {
                if let Some(patch) = self.yarn.board[idx].pop() {
                    self.yarn.balloon_columns.push(Some(patch));
                }
            }
        }

        // Right side: pop from rightmost non-empty column(s)
        let right_count = ((self.bonuses.balloon_count + 1) / 2) as usize;
        for _ in 0..right_count {
            if let Some(idx) = self.yarn.board.iter().rposition(|c| !c.is_empty()) {
                if let Some(patch) = self.yarn.board[idx].pop() {
                    self.yarn.balloon_columns.push(Some(patch));
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

// ── Generator helpers (moved from main.rs) ─────────────────────────────────

fn find_generator_for_output(
    board: &Vec<Vec<BoardEntity>>,
    out_row: usize,
    out_col: usize,
) -> Option<(usize, usize)> {
    for r in 0..board.len() {
        for c in 0..board[r].len() {
            if let BoardEntity::Generator(ref data) = board[r][c] {
                let (dr, dc) = data.output_dir.offset();
                if r as i32 + dr == out_row as i32 && c as i32 + dc == out_col as i32 {
                    return Some((r, c));
                }
            }
        }
    }
    None
}

fn advance_generator(
    board: &mut Vec<Vec<BoardEntity>>,
    gen_row: usize,
    gen_col: usize,
    out_row: usize,
    out_col: usize,
) {
    enum Action { Spawn(Color), Deplete }

    let action = if let BoardEntity::Generator(ref mut data) = board[gen_row][gen_col] {
        if data.queue.is_empty() { Action::Deplete }
        else { Action::Spawn(data.queue.remove(0)) }
    } else {
        return;
    };

    match action {
        Action::Spawn(color) => board[out_row][out_col] = BoardEntity::Thread(color),
        Action::Deplete      => board[gen_row][gen_col] = BoardEntity::DepletedGenerator,
    }
}

// ── Snapshot types (serde mirror of engine state) ──────────────────────────

#[derive(Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub knit_volume: u16,
    pub active_threads_limit: usize,
    pub board_height: u16,
    pub board_width: u16,
    pub board: Vec<Vec<String>>,
    pub yarn_lines: u16,
    pub visible_patches: u16,
    pub yarn: Vec<Vec<YarnPatchSnap>>,
    pub active_threads: Vec<ThreadSnap>,
    #[serde(default)]
    pub scissors: u16,
    #[serde(default)]
    pub tweezers: u16,
    #[serde(default)]
    pub balloons: u16,
    #[serde(default)]
    pub scissors_threads: u16,
    #[serde(default)]
    pub balloon_count: u16,
    #[serde(default)]
    pub balloon_columns: Vec<Option<YarnPatchSnap>>,
    #[serde(default)]
    pub ad_limit: Option<u16>,
    #[serde(default)]
    pub ads_used: u16,
}

#[derive(Serialize, Deserialize)]
pub struct YarnPatchSnap { pub color: String, pub locked: bool }

#[derive(Serialize, Deserialize)]
pub struct ThreadSnap { pub color: String, pub status: u16, pub has_key: bool }

impl GameStateSnapshot {
    fn from_engine(e: &GameEngine) -> Self {
        Self {
            cursor_row: e.cursor_row,
            cursor_col: e.cursor_col,
            knit_volume: e.knit_volume,
            active_threads_limit: e.active_threads_limit,
            board_height: e.board.height,
            board_width: e.board.width,
            board: e.board.board.iter()
                .map(|row| row.iter().map(cell_to_str).collect())
                .collect(),
            yarn_lines: e.yarn.yarn_lines,
            visible_patches: e.yarn.visible_patches,
            yarn: e.yarn.board.iter()
                .map(|col| col.iter().map(|p| YarnPatchSnap {
                    color: color_serde::color_to_str(&p.color),
                    locked: p.locked,
                }).collect())
                .collect(),
            active_threads: e.active_threads.iter()
                .map(|t| ThreadSnap {
                    color: color_serde::color_to_str(&t.color),
                    status: t.status,
                    has_key: t.has_key,
                })
                .collect(),
            scissors: e.bonuses.scissors,
            tweezers: e.bonuses.tweezers,
            balloons: e.bonuses.balloons,
            scissors_threads: e.bonuses.scissors_threads,
            balloon_count: e.bonuses.balloon_count,
            balloon_columns: e.yarn.balloon_columns.iter()
                .map(|opt| opt.as_ref().map(|p| YarnPatchSnap {
                    color: color_serde::color_to_str(&p.color),
                    locked: p.locked,
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

        let yarn_cols: Result<Vec<Vec<Patch>>, String> = self.yarn.iter()
            .map(|col| col.iter().map(|p| {
                let color = color_serde::str_to_color(&p.color)
                    .ok_or_else(|| format!("bad color: {}", p.color))?;
                Ok(Patch { color, locked: p.locked })
            }).collect())
            .collect();
        let yarn_cols = yarn_cols?;

        let threads: Result<Vec<Thread>, String> = self.active_threads.iter()
            .map(|t| {
                let color = color_serde::str_to_color(&t.color)
                    .ok_or_else(|| format!("bad color: {}", t.color))?;
                Ok(Thread { color, status: t.status, has_key: t.has_key })
            })
            .collect();
        let threads = threads?;

        let balloon_cols: Result<Vec<Option<Patch>>, String> = self.balloon_columns.iter()
            .map(|opt| opt.as_ref().map(|p| {
                let color = color_serde::str_to_color(&p.color)
                    .ok_or_else(|| format!("bad color: {}", p.color))?;
                Ok(Patch { color, locked: p.locked })
            }).transpose())
            .collect();
        let balloon_cols = balloon_cols?;

        Ok(GameEngine {
            board: GameBoard {
                board: board_cells,
                height: self.board_height,
                width: self.board_width,
                knit_volume: self.knit_volume,
            },
            yarn: Yarn {
                board: yarn_cols,
                yarn_lines: self.yarn_lines,
                visible_patches: self.visible_patches,
                balloon_columns: balloon_cols,
            },
            active_threads: threads,
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            knit_volume: self.knit_volume,
            active_threads_limit: self.active_threads_limit,
            bonuses: BonusInventory {
                scissors: self.scissors,
                tweezers: self.tweezers,
                balloons: self.balloons,
                scissors_threads: if self.scissors_threads == 0 { 1 } else { self.scissors_threads },
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
        BoardEntity::Thread(c)    => format!("T:{}", color_serde::color_to_str(c)),
        BoardEntity::KeyThread(c) => format!("K:{}", color_serde::color_to_str(c)),
        BoardEntity::Obstacle     => "X".into(),
        BoardEntity::Void         => "V".into(),
        BoardEntity::Generator(d) => {
            let dir = match d.output_dir {
                Direction::Up    => "up",
                Direction::Down  => "down",
                Direction::Left  => "left",
                Direction::Right => "right",
            };
            let queue: Vec<String> = d.queue.iter().map(|c| color_serde::color_to_str(c)).collect();
            format!("G:{}:{}:{}", color_serde::color_to_str(&d.color), dir, queue.join(","))
        }
        BoardEntity::DepletedGenerator => "#".into(),
    }
}

fn str_to_cell(s: &str) -> Result<BoardEntity, String> {
    if s == "X" { return Ok(BoardEntity::Obstacle); }
    if s == "V" { return Ok(BoardEntity::Void); }
    if s == "#" { return Ok(BoardEntity::DepletedGenerator); }

    let parts: Vec<&str> = s.splitn(4, ':').collect();
    match parts.as_slice() {
        ["T", color] => {
            let c = color_serde::str_to_color(color)
                .ok_or_else(|| format!("bad color: {color}"))?;
            Ok(BoardEntity::Thread(c))
        }
        ["K", color] => {
            let c = color_serde::str_to_color(color)
                .ok_or_else(|| format!("bad color: {color}"))?;
            Ok(BoardEntity::KeyThread(c))
        }
        ["G", color, dir_str, queue_str] => {
            let color = color_serde::str_to_color(color)
                .ok_or_else(|| format!("bad generator color: {color}"))?;
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
            Ok(BoardEntity::Generator(GeneratorData { color, output_dir, queue: queue? }))
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
                vec![BoardEntity::Thread(Color::Red),  BoardEntity::Thread(Color::Blue), BoardEntity::Thread(Color::Red)],
                vec![BoardEntity::Thread(Color::Blue), BoardEntity::Obstacle,             BoardEntity::Thread(Color::Red)],
                vec![BoardEntity::Thread(Color::Red),  BoardEntity::Thread(Color::Blue), BoardEntity::Thread(Color::Red)],
            ],
            height: 3,
            width: 3,
            knit_volume: 1,
        };
        let yarn = Yarn {
            board: vec![
                vec![Patch { color: Color::Red, locked: false }, Patch { color: Color::Blue, locked: false }],
                vec![Patch { color: Color::Red, locked: false }, Patch { color: Color::Red, locked: false }],
            ],
            yarn_lines: 2,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };
        GameEngine {
            board,
            yarn,
            active_threads: vec![],
            cursor_row: 0,
            cursor_col: 0,
            knit_volume: 1,
            active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
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
        assert_eq!(e.active_threads.len(), 1);
    }
    #[test]
    fn pick_up_obstacle_fails() {
        let mut e = default_engine();
        e.cursor_row = 1; e.cursor_col = 1;
        assert_eq!(e.pick_up(), Err(PickError::NotAThread));
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
        e.active_threads_limit = 0;
        assert_eq!(e.pick_up(), Err(PickError::ActiveFull));
    }
    #[test]
    fn process_one_active_removes_when_done() {
        let mut e = default_engine();
        e.active_threads.push(Thread { color: Color::Red, status: 1, has_key: false });
        e.process_one_active();
        // knit_volume=1: after one successful process, status becomes 2 > 1, discarded
        assert_eq!(e.active_threads.len(), 0);
    }
    #[test]
    fn is_won_false_while_board_has_threads() {
        assert!(!default_engine().is_won());
    }
    #[test]
    fn snapshot_roundtrip() {
        let e = default_engine();
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.cursor_row, e.cursor_row);
        assert_eq!(e2.knit_volume, e.knit_volume);
        assert_eq!(e2.board.height, e.board.height);
        assert_eq!(e2.yarn.yarn_lines, e.yarn.yarn_lines);
    }
    #[test]
    fn cell_roundtrip_all_variants() {
        let cells = vec![
            BoardEntity::Thread(Color::Red),
            BoardEntity::KeyThread(Color::Blue),
            BoardEntity::Obstacle,
            BoardEntity::Void,
            BoardEntity::DepletedGenerator,
            BoardEntity::Generator(GeneratorData {
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

    // ── Task 2: missing engine unit tests ──────────────────────────────────

    #[test]
    fn move_cursor_down_succeeds() {
        let mut e = default_engine();
        // Place a Void at (0,0) — surface void in row 0.
        // Thread at (1,0) now has a surface-connected void neighbor → selectable → focusable.
        e.board.board[0][0] = BoardEntity::Void;
        // Cursor starts at (0,0) which is Void (focusable).
        assert!(e.move_cursor(Direction::Down).is_ok());
        assert_eq!(e.cursor_row, 1);
    }
    #[test]
    fn move_cursor_down_at_edge_fails() {
        let mut e = default_engine();
        e.cursor_row = 2; // bottom edge of 3-row board
        assert_eq!(e.move_cursor(Direction::Down), Err(MoveError::OutOfBounds));
    }
    #[test]
    fn move_cursor_right_at_edge_fails() {
        let mut e = default_engine();
        e.cursor_col = 2; // right edge of 3-col board
        assert_eq!(e.move_cursor(Direction::Right), Err(MoveError::OutOfBounds));
    }
    #[test]
    fn move_cursor_skips_non_focusable_knits() {
        // Build a 3×3 board where:
        //   row 0: [Thread, Void,     Void    ]  ← cursor starts at (0,0)
        //   row 1: [Thread, Obstacle, Void    ]  ← (1,0) has no void neighbor → NOT focusable
        //   row 2: [Thread, Void,     Void    ]  ← (2,0) has void neighbor (2,1) connected
        //                                            via (2,2)→(1,2)→(0,2) → surface-connected
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Thread(Color::Red),  BoardEntity::Void,     BoardEntity::Void],
                vec![BoardEntity::Thread(Color::Blue), BoardEntity::Obstacle, BoardEntity::Void],
                vec![BoardEntity::Thread(Color::Red),  BoardEntity::Void,     BoardEntity::Void],
            ],
            height: 3,
            width: 3,
            knit_volume: 1,
        };
        let mut e = GameEngine {
            board,
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 1, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        // (1,0) Thread(Blue): neighbors (0,0)=Thread, (1,1)=Obstacle, (2,0)=Thread → no void → NOT focusable
        // (2,0) Thread(Red): neighbor (2,1)=Void, connected to surface → focusable
        assert!(e.move_cursor(Direction::Down).is_ok());
        assert_eq!(e.cursor_row, 2); // skipped row 1
        assert_eq!(e.cursor_col, 0);
    }
    #[test]
    fn move_cursor_down_into_all_buried_fails() {
        let mut e = default_engine();
        // Default board: rows 1 and 2 are all buried threads → not focusable
        assert_eq!(e.move_cursor(Direction::Down), Err(MoveError::OutOfBounds));
    }

    #[test]
    fn pick_up_makes_cell_void() {
        let mut e = default_engine();
        e.pick_up().unwrap();
        assert!(matches!(e.board.board[0][0], BoardEntity::Void));
    }
    #[test]
    fn pick_up_key_thread_sets_has_key() {
        let mut e = default_engine();
        e.board.board[0][0] = BoardEntity::KeyThread(Color::Red);
        e.pick_up().unwrap();
        assert!(e.active_threads[0].has_key);
        assert_eq!(e.active_threads[0].color, Color::Red);
    }

    #[test]
    fn process_all_active_processes_each_thread() {
        // knit_volume=2 so threads need 2 hits to complete (status starts at 1, done when > 2)
        let mut e = default_engine();
        e.knit_volume = 2;
        // Yarn: col0=[Red, Blue], col1=[Red, Red] — plenty of Red and Blue patches
        e.active_threads = vec![
            Thread { color: Color::Red,  status: 1, has_key: false },
            Thread { color: Color::Blue, status: 1, has_key: false },
            Thread { color: Color::Red,  status: 1, has_key: false },
        ];
        e.process_all_active();
        // Each thread gets one step: Red→2, Blue→2, Red→2 (all still <= knit_volume=2)
        assert_eq!(e.active_threads.len(), 3);
        for t in &e.active_threads {
            assert_eq!(t.status, 2);
        }
    }
    #[test]
    fn process_all_active_removes_completed() {
        let mut e = default_engine(); // knit_volume=1
        e.active_threads = vec![
            Thread { color: Color::Red, status: 1, has_key: false },
            Thread { color: Color::Red, status: 1, has_key: false },
        ];
        // knit_volume=1: after one process, status=2 > 1, thread is discarded
        e.process_all_active();
        assert_eq!(e.active_threads.len(), 0);
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
                    vec![BoardEntity::DepletedGenerator, BoardEntity::Void],
                ],
                height: 2, width: 2, knit_volume: 1,
            },
            yarn: Yarn { board: vec![vec![], vec![]], yarn_lines: 2, visible_patches: 3, balloon_columns: Vec::new() },
            active_threads: vec![],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 1, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert!(e.is_won());
    }
    #[test]
    fn is_won_false_with_active_threads() {
        let mut e = default_engine();
        // Clear board and yarn but leave an active thread
        e.board.board = vec![vec![BoardEntity::Void]];
        e.board.height = 1; e.board.width = 1;
        e.yarn.board = vec![vec![]];
        e.active_threads = vec![Thread { color: Color::Red, status: 1, has_key: false }];
        assert!(!e.is_won());
    }
    #[test]
    fn is_won_false_with_remaining_yarn() {
        let mut e = default_engine();
        e.board.board = vec![vec![BoardEntity::Void]];
        e.board.height = 1; e.board.width = 1;
        e.active_threads = vec![];
        // yarn still has patches
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
            color_mode: "dark".into(), active_threads_limit: 7,
            knit_volume: 2, yarn_lines: 3, obstacle_percentage: 5,
            visible_patches: 4, generator_capacity: 3, generator_percentage: 5,
            layout: "auto".into(),
            scale: 1,
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_threads: 1, balloon_count: 2,
            ad_file: None,
        };
        let e = GameEngine::new(&config);
        assert_eq!(e.board.height, 4);
        assert_eq!(e.board.width, 4);
        assert_eq!(e.knit_volume, 2);
        assert!(e.active_threads.is_empty());
        // yarn should have patches
        let total_patches: usize = e.yarn.board.iter().map(|c| c.len()).sum();
        assert!(total_patches > 0);
    }

    // ── Task 3: snapshot edge case tests ───────────────────────────────────

    #[test]
    fn snapshot_roundtrip_with_generator() {
        let mut e = default_engine();
        e.board.board[1][0] = BoardEntity::Generator(GeneratorData {
            color: Color::Cyan,
            output_dir: Direction::Right,
            queue: vec![Color::Red, Color::Blue, Color::Green],
        });
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        match &e2.board.board[1][0] {
            BoardEntity::Generator(d) => {
                assert_eq!(d.color, Color::Cyan);
                assert_eq!(d.output_dir, Direction::Right);
                assert_eq!(d.queue, vec![Color::Red, Color::Blue, Color::Green]);
            }
            other => panic!("expected Generator, got {:?}", cell_to_str(other)),
        }
    }
    #[test]
    fn snapshot_roundtrip_with_locked_patches() {
        let mut e = default_engine();
        e.yarn.board[0].push(Patch { color: Color::Magenta, locked: true });
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        let last = e2.yarn.board[0].last().unwrap();
        assert!(last.locked);
        assert_eq!(last.color, Color::Magenta);
    }
    #[test]
    fn snapshot_roundtrip_with_key_threads() {
        let mut e = default_engine();
        e.board.board[0][1] = BoardEntity::KeyThread(Color::Yellow);
        e.active_threads.push(Thread { color: Color::Yellow, status: 2, has_key: true });
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        match &e2.board.board[0][1] {
            BoardEntity::KeyThread(c) => assert_eq!(*c, Color::Yellow),
            other => panic!("expected KeyThread, got {:?}", cell_to_str(other)),
        }
        assert!(e2.active_threads[0].has_key);
        assert_eq!(e2.active_threads[0].status, 2);
    }
    #[test]
    fn snapshot_roundtrip_with_active_threads() {
        let mut e = default_engine();
        e.active_threads = vec![
            Thread { color: Color::Red,  status: 1, has_key: false },
            Thread { color: Color::Blue, status: 3, has_key: true },
        ];
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.active_threads.len(), 2);
        assert_eq!(e2.active_threads[0].color, Color::Red);
        assert_eq!(e2.active_threads[0].status, 1);
        assert!(!e2.active_threads[0].has_key);
        assert_eq!(e2.active_threads[1].color, Color::Blue);
        assert_eq!(e2.active_threads[1].status, 3);
        assert!(e2.active_threads[1].has_key);
    }

    #[test]
    fn from_json_rejects_bad_json() {
        assert!(GameEngine::from_json("not json at all").is_err());
    }
    #[test]
    fn from_json_rejects_bad_color() {
        let mut e = default_engine();
        let mut json = e.to_json();
        // Corrupt a color name in the JSON
        json = json.replace("\"red\"", "\"neonpink\"");
        assert!(GameEngine::from_json(&json).is_err());
    }
    #[test]
    fn from_json_rejects_bad_cell() {
        let mut e = default_engine();
        let mut json = e.to_json();
        // Corrupt a cell encoding
        json = json.replace("\"T:red\"", "\"Z:invalid\"");
        assert!(GameEngine::from_json(&json).is_err());
    }

    // ── Task 4: GameStatus tests ─────────────────────────────────────────

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
                height: 1, width: 2, knit_volume: 1,
            },
            yarn: Yarn { board: vec![vec![], vec![]], yarn_lines: 2, visible_patches: 3, balloon_columns: Vec::new() },
            active_threads: vec![],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 1, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Won);
    }

    #[test]
    fn status_stuck_front_thread_blocked() {
        // active_threads[0] is Green, but yarn only has Red patches → deadlock
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![Thread { color: Color::Green, status: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    #[test]
    fn status_stuck_no_selectable_threads_on_board() {
        // No active threads, board has threads but all buried, yarn has patches
        let e = GameEngine {
            board: GameBoard {
                board: vec![
                    vec![BoardEntity::Obstacle, BoardEntity::Obstacle],
                    vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
                ],
                height: 2, width: 2, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 1, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    #[test]
    fn status_playing_when_front_thread_can_match() {
        // active_threads[0] is Red, yarn has Red → can process → still playing
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![Thread { color: Color::Red, status: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn status_stuck_locked_patch_no_key() {
        // active_threads[0] is Red, yarn has locked Red but thread has no key → stuck
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: true }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![Thread { color: Color::Red, status: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    #[test]
    fn status_playing_locked_patch_with_key() {
        // active_threads[0] is Red with key, yarn has locked Red → can unlock → playing
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: true }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![Thread { color: Color::Red, status: 1, has_key: true }],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn status_playing_when_other_thread_can_match() {
        // active_threads[0] is Green (blocked), but [1] is Red which CAN match → not stuck
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![
                Thread { color: Color::Green, status: 1, has_key: false },
                Thread { color: Color::Red, status: 1, has_key: false },
            ],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn status_stuck_when_no_thread_can_match() {
        // Two active threads, neither color matches any yarn top → stuck
        let e = GameEngine {
            board: GameBoard {
                board: vec![vec![BoardEntity::Void]],
                height: 1, width: 1, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![
                Thread { color: Color::Green, status: 1, has_key: false },
                Thread { color: Color::Blue, status: 1, has_key: false },
            ],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    #[test]
    fn status_playing_when_blocked_but_can_pick_up_more() {
        // Active thread can't match yarn, but board has selectable threads
        // and active_threads_limit not reached → player can pick up helpers
        let e = GameEngine {
            board: GameBoard {
                board: vec![
                    vec![BoardEntity::Thread(Color::Red), BoardEntity::Void],
                ],
                height: 1, width: 2, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![Thread { color: Color::Green, status: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 5,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Playing);
    }

    #[test]
    fn status_stuck_when_blocked_and_active_full() {
        // Active thread can't match, AND active_threads is full → truly stuck
        let e = GameEngine {
            board: GameBoard {
                board: vec![
                    vec![BoardEntity::Thread(Color::Red), BoardEntity::Void],
                ],
                height: 1, width: 2, knit_volume: 1,
            },
            yarn: Yarn {
                board: vec![vec![Patch { color: Color::Red, locked: false }]],
                yarn_lines: 1, visible_patches: 3,
                balloon_columns: Vec::new(),
            },
            active_threads: vec![Thread { color: Color::Green, status: 1, has_key: false }],
            cursor_row: 0, cursor_col: 0,
            knit_volume: 3, active_threads_limit: 1,
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
            ad_limit: None,
            ads_used: 0,
        };
        assert_eq!(e.status(), GameStatus::Stuck);
    }

    // ── Task 4: scissors bonus tests ────────────────────────────────────

    #[test]
    fn use_scissors_completes_thread() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        e.bonuses.scissors_threads = 1;
        e.knit_volume = 2;
        // Active thread: Red, status 1 (needs 2 total knits to complete, since done when status > knit_volume)
        e.active_threads = vec![
            Thread { color: Color::Red, status: 1, has_key: false },
        ];
        // default_engine yarn has Red patches in both columns — deep scan will find them
        let result = e.use_scissors();
        assert!(result.is_ok());
        assert_eq!(e.bonuses.scissors, 0);
        // Thread should be fully knitted and removed (status went past knit_volume=2)
        assert_eq!(e.active_threads.len(), 0);
    }

    #[test]
    fn use_scissors_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.scissors = 0;
        e.active_threads = vec![Thread { color: Color::Red, status: 1, has_key: false }];
        assert_eq!(e.use_scissors(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn use_scissors_no_active_threads_fails() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        assert_eq!(e.use_scissors(), Err(BonusError::NoActiveThreads));
    }

    #[test]
    fn use_scissors_picks_least_progress_thread() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        e.bonuses.scissors_threads = 1;
        e.knit_volume = 1;
        e.active_threads = vec![
            Thread { color: Color::Red,  status: 2, has_key: false }, // more progress
            Thread { color: Color::Blue, status: 1, has_key: false }, // least progress
        ];
        // default_engine yarn has Blue patches — deep scan should find one
        let _ = e.use_scissors();
        // The Blue thread (status 1) should have been selected and completed
        // It had status 1, knit_volume=1, so after 1 knit → status 2 > 1 → removed
        assert_eq!(e.active_threads.len(), 1);
        assert_eq!(e.active_threads[0].color, Color::Red);
    }

    // ── Task 5: tweezers bonus tests ───────────────────────────────────

    #[test]
    fn use_tweezers_enters_mode() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.cursor_row = 0;
        e.cursor_col = 0;
        let result = e.use_tweezers();
        assert!(result.is_ok());
        assert_eq!(e.bonus_state, BonusState::TweezersActive { saved_row: 0, saved_col: 0 });
        // Count not decremented until pick completes
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
        // In the default board, row 1 col 0 is a buried Thread (normally not focusable)
        // but tweezers mode should let cursor move there
        let result = e.move_cursor(Direction::Down);
        assert!(result.is_ok());
        // In tweezers mode, cursor moves to immediately adjacent cell (row 1)
        // instead of skipping to the next focusable cell
        assert_eq!(e.cursor_row, 1);
    }

    #[test]
    fn tweezers_pick_up_ignores_selectability() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        // Move cursor directly to buried thread at (2, 0)
        e.cursor_row = 2;
        e.cursor_col = 0;
        // Normally this would fail with NotSelectable, but tweezers overrides
        let result = e.pick_up();
        assert!(result.is_ok());
        assert_eq!(e.active_threads.len(), 1);
        // Cursor restored to saved position
        assert_eq!(e.cursor_row, 0);
        assert_eq!(e.cursor_col, 0);
        // Bonus consumed
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
        // Bonus NOT consumed on cancel
        assert_eq!(e.bonuses.tweezers, 1);
    }

    // ── Task 6: balloons bonus tests ──────────────────────────────────

    #[test]
    fn use_balloons_lifts_patches() {
        let mut e = default_engine();
        e.bonuses.balloons = 1;
        e.bonuses.balloon_count = 1;
        // default_engine yarn: col0=[Red, Blue], col1=[Red, Red]
        // Lifting 1 patch from front (top/last) of each column
        let total_before: usize = e.yarn.board.iter().map(|c| c.len()).sum();
        let result = e.use_balloons();
        assert!(result.is_ok());
        assert_eq!(e.bonuses.balloons, 0);
        // Balloon columns should have been created
        assert!(!e.yarn.balloon_columns.is_empty());
        // Total patches should be conserved (moved, not destroyed)
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
        // Balloon columns are non-empty now
        assert_eq!(e.use_balloons(), Err(BonusError::BalloonColumnsNotEmpty));
    }

    // ── Task 7: snapshot roundtrip with bonuses ─────────────────────────

    #[test]
    fn snapshot_roundtrip_with_bonuses() {
        let mut e = default_engine();
        e.bonuses.scissors = 3;
        e.bonuses.tweezers = 2;
        e.bonuses.balloons = 1;
        e.yarn.balloon_columns = vec![Some(Patch { color: Color::Red, locked: false })];
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.bonuses.scissors, 3);
        assert_eq!(e2.bonuses.tweezers, 2);
        assert_eq!(e2.bonuses.balloons, 1);
        assert_eq!(e2.yarn.balloon_columns.len(), 1);
        assert_eq!(e2.yarn.balloon_columns[0].as_ref().unwrap().color, Color::Red);
    }
}
