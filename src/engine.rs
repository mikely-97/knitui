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

// ── GameEngine ─────────────────────────────────────────────────────────────

pub struct GameEngine {
    pub board: GameBoard,
    pub yarn: Yarn,
    pub active_threads: Vec<Thread>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub knit_volume: u16,
    pub active_threads_limit: usize,
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
        Self {
            board,
            yarn,
            active_threads: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            knit_volume: config.knit_volume,
            active_threads_limit: config.active_threads_limit,
        }
    }

    // ── Actions ────────────────────────────────────────────────────────────

    pub fn move_cursor(&mut self, dir: Direction) -> Result<(), MoveError> {
        let (dr, dc) = dir.offset();
        let new_row = self.cursor_row as i32 + dr;
        let new_col = self.cursor_col as i32 + dc;
        if new_row < 0 || new_row >= self.board.height as i32
            || new_col < 0 || new_col >= self.board.width as i32
        {
            return Err(MoveError::OutOfBounds);
        }
        self.cursor_row = new_row as u16;
        self.cursor_col = new_col as u16;
        Ok(())
    }

    pub fn pick_up(&mut self) -> Result<(), PickError> {
        let row = self.cursor_row as usize;
        let col = self.cursor_col as usize;

        let thread = match &self.board.board[row][col] {
            BoardEntity::Thread(c)    => Thread { color: *c, status: 1, has_key: false },
            BoardEntity::KeyThread(c) => Thread { color: *c, status: 1, has_key: true },
            _ => return Err(PickError::NotAThread),
        };

        if !self.board.is_selectable(row, col) {
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

        Ok(())
    }

    /// Process the first active thread one yarn step.
    /// Removes the thread if it has completed `knit_volume` steps.
    /// Returns true if a thread was processed, false if active list was empty.
    pub fn process_one_active(&mut self) -> bool {
        if self.active_threads.is_empty() {
            return false;
        }
        let mut thread = self.active_threads.remove(0);
        self.yarn.process_one(&mut thread);
        if thread.status <= self.knit_volume {
            self.active_threads.push(thread);
        }
        true
    }

    /// Process all active threads one yarn step each (for NI binary).
    pub fn process_all_active(&mut self) {
        let count = self.active_threads.len();
        for _ in 0..count {
            self.process_one_active();
        }
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
            },
            active_threads: threads,
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            knit_volume: self.knit_volume,
            active_threads_limit: self.active_threads_limit,
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
        };
        GameEngine {
            board,
            yarn,
            active_threads: vec![],
            cursor_row: 0,
            cursor_col: 0,
            knit_volume: 1,
            active_threads_limit: 5,
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
}
