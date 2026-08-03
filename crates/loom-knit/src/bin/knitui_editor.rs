//! knitui_editor — interactive board editor for building custom knitui presets.
//!
//! Controls:
//!   Arrow keys     move cursor
//!   1-6            place Spool of that color (1-indexed into palette)
//!   k              place KeySpool at cursor (same color as selected)
//!   c              place Conveyor at cursor
//!   x              place Obstacle at cursor
//!   Space / 0      clear cell (Void)
//!   +/-            grow/shrink board (2-6 rows/cols)
//!   v              validate board (solvability quick-check)
//!   s              save board as preset JSON
//!   q              quit

use std::io::{self, Write as IoWrite};
use std::path::PathBuf;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use knitui::board_entity::{BoardEntity, ConveyorData, Direction};
use knitui::game_board::GameBoard;
use knitui::solvability::all_spools_reachable;

// ── Palette ──────────────────────────────────────────────────────────────────

const EDITOR_PALETTE: [Color; 6] = [
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Magenta,
    Color::Red,
    Color::Blue,
];

const COLOR_NAMES: [&str; 6] = ["Cyan", "Green", "Yellow", "Magenta", "Red", "Blue"];

// ── Cell type ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum EditorCell {
    Empty,
    Spool(usize),    // palette index
    KeySpool(usize), // palette index
    Conveyor,
    Obstacle,
}

impl EditorCell {
    fn glyph(&self) -> char {
        match self {
            EditorCell::Empty => '.',
            EditorCell::Spool(_) => 'T',
            EditorCell::KeySpool(_) => 'K',
            EditorCell::Conveyor => '>',
            EditorCell::Obstacle => 'X',
        }
    }

    fn color(&self) -> Color {
        match self {
            EditorCell::Spool(i) | EditorCell::KeySpool(i) => EDITOR_PALETTE[*i],
            EditorCell::Conveyor => Color::White,
            EditorCell::Obstacle => Color::DarkGrey,
            EditorCell::Empty => Color::DarkGrey,
        }
    }
}

// ── Editor state ─────────────────────────────────────────────────────────────

struct Editor {
    rows: usize,
    cols: usize,
    cells: Vec<Vec<EditorCell>>,
    cursor: (usize, usize), // (row, col)
    status_msg: String,
    selected_color: usize, // 0-5
}

impl Editor {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            cells: vec![vec![EditorCell::Empty; cols]; rows],
            cursor: (0, 0),
            status_msg: "Welcome! Arrow keys=move  1-6=spool  k=key  c=conv  x=obs  Space=clear  +=grow  -=shrink  v=validate  s=save  q=quit".into(),
            selected_color: 0,
        }
    }

    fn resize(&mut self, new_rows: usize, new_cols: usize) {
        let new_rows = new_rows.clamp(2, 6);
        let new_cols = new_cols.clamp(2, 6);
        // Trim or extend rows
        self.cells.resize_with(new_rows, || vec![EditorCell::Empty; new_cols]);
        // Trim or extend cols in each row
        for row in &mut self.cells {
            row.resize_with(new_cols, || EditorCell::Empty);
            row.truncate(new_cols);
        }
        self.rows = new_rows;
        self.cols = new_cols;
        self.cursor.0 = self.cursor.0.min(new_rows - 1);
        self.cursor.1 = self.cursor.1.min(new_cols - 1);
    }

    fn place(&mut self, cell: EditorCell) {
        let (r, c) = self.cursor;
        self.cells[r][c] = cell;
    }

    fn validate(&mut self) {
        // Build a GameBoard from the editor cells for a quick reachability check.
        let board_cells: Vec<Vec<BoardEntity>> = self.cells.iter().map(|row| {
            row.iter().map(|cell| match cell {
                EditorCell::Empty => BoardEntity::Void,
                EditorCell::Spool(i) => BoardEntity::Spool(EDITOR_PALETTE[*i]),
                EditorCell::KeySpool(i) => BoardEntity::KeySpool(EDITOR_PALETTE[*i]),
                EditorCell::Obstacle => BoardEntity::Obstacle,
                EditorCell::Conveyor => BoardEntity::Conveyor(ConveyorData {
                    color: Color::White,
                    output_dir: Direction::Right,
                    queue: vec![Color::White],
                }),
            }).collect()
        }).collect();

        let gb = GameBoard {
            board: board_cells,
            height: self.rows as u16,
            width: self.cols as u16,
            spool_capacity: 3,
        };

        // Count spools
        let spool_count: usize = self.cells.iter().flatten().filter(|c| {
            matches!(c, EditorCell::Spool(_) | EditorCell::KeySpool(_) | EditorCell::Conveyor)
        }).count();

        if spool_count == 0 {
            self.status_msg = "Validation: no spools placed — board is empty.".into();
            return;
        }

        if all_spools_reachable(&gb) {
            self.status_msg = format!(
                "Validation OK: {} spools, all reachable.",
                spool_count
            );
        } else {
            self.status_msg = "Validation FAILED: some spools are unreachable (buried with no void neighbour).".into();
        }
    }

    fn to_preset_json(&self, name: &str) -> String {
        // Count colors actually used
        let mut colors_used = std::collections::HashSet::new();
        for cell in self.cells.iter().flatten() {
            match cell {
                EditorCell::Spool(i) | EditorCell::KeySpool(i) => { colors_used.insert(*i); }
                _ => {}
            }
        }
        let color_number = colors_used.len().max(2);

        // Has conveyors?
        let has_conv = self.cells.iter().flatten().any(|c| matches!(c, EditorCell::Conveyor));
        let conveyor_percentage = if has_conv { 15u16 } else { 0u16 };

        // Has obstacles?
        let has_obs = self.cells.iter().flatten().any(|c| matches!(c, EditorCell::Obstacle));
        let obstacle_percentage = if has_obs { 15u16 } else { 0u16 };

        // Build board layout as array-of-rows of entity strings
        let board_rows: Vec<serde_json::Value> = self.cells.iter().map(|row| {
            let cells: Vec<serde_json::Value> = row.iter().map(|cell| {
                let s = match cell {
                    EditorCell::Empty => "void".to_string(),
                    EditorCell::Spool(i) => format!("spool:{}", i),
                    EditorCell::KeySpool(i) => format!("keyspool:{}", i),
                    EditorCell::Conveyor => "conveyor".to_string(),
                    EditorCell::Obstacle => "obstacle".to_string(),
                };
                serde_json::Value::String(s)
            }).collect();
            serde_json::Value::Array(cells)
        }).collect();

        let preset = serde_json::json!({
            "name": name,
            "board_height": self.rows,
            "board_width": self.cols,
            "color_number": color_number,
            "obstacle_percentage": obstacle_percentage,
            "conveyor_percentage": conveyor_percentage,
            "scissors": 0,
            "tweezers": 0,
            "balloons": 0,
            "board_layout": board_rows
        });

        serde_json::to_string_pretty(&preset).unwrap_or_default()
    }

    fn save_preset(&mut self, name: &str) -> io::Result<()> {
        let json = self.to_preset_json(name);
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("knitui")
            .join("presets");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", name));
        std::fs::write(&path, json)?;
        self.status_msg = format!("Saved to {}", path.display());
        Ok(())
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(editor: &Editor, stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0))?;

    // Title
    queue!(
        stdout,
        SetForegroundColor(Color::White),
        Print("knitui Board Editor"),
        ResetColor,
        Print("\r\n\r\n")
    )?;

    // Color palette selector
    queue!(stdout, Print("Colors: "))?;
    for (i, &col) in EDITOR_PALETTE.iter().enumerate() {
        let label = format!("[{}]{}", i + 1, COLOR_NAMES[i]);
        if i == editor.selected_color {
            queue!(
                stdout,
                SetBackgroundColor(col),
                SetForegroundColor(Color::Black),
                Print(&label),
                ResetColor,
                Print(" ")
            )?;
        } else {
            queue!(
                stdout,
                SetForegroundColor(col),
                Print(&label),
                ResetColor,
                Print(" ")
            )?;
        }
    }
    queue!(stdout, Print("\r\n\r\n"))?;

    // Grid header
    queue!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("Board {}x{}  (cursor {},{}):\r\n", editor.rows, editor.cols, editor.cursor.0, editor.cursor.1)),
        ResetColor
    )?;

    // Top border
    let border_top = format!("  +{}+\r\n", "---+".repeat(editor.cols));
    queue!(stdout, SetForegroundColor(Color::DarkGrey), Print(&border_top), ResetColor)?;

    for r in 0..editor.rows {
        queue!(stdout, SetForegroundColor(Color::DarkGrey), Print(format!("{} |", r)), ResetColor)?;
        for c in 0..editor.cols {
            let cell = &editor.cells[r][c];
            let glyph = cell.glyph();
            let fg = cell.color();
            let is_cursor = editor.cursor == (r, c);

            if is_cursor {
                queue!(
                    stdout,
                    SetBackgroundColor(Color::White),
                    SetForegroundColor(Color::Black),
                    Print(format!(" {} ", glyph)),
                    ResetColor
                )?;
            } else {
                queue!(
                    stdout,
                    SetForegroundColor(fg),
                    Print(format!(" {} ", glyph)),
                    ResetColor
                )?;
            }
            queue!(stdout, SetForegroundColor(Color::DarkGrey), Print("|"), ResetColor)?;
        }
        queue!(stdout, Print("\r\n"))?;

        // Row separator
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  +{}+\r\n", "---+".repeat(editor.cols))),
            ResetColor
        )?;
    }

    // Column index footer
    queue!(stdout, Print("   "))?;
    for c in 0..editor.cols {
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" {}  ", c)),
            ResetColor
        )?;
    }
    queue!(stdout, Print("\r\n\r\n"))?;

    // Status bar
    queue!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print(&editor.status_msg),
        ResetColor,
        Print("\r\n")
    )?;

    stdout.flush()
}

// ── Save prompt ───────────────────────────────────────────────────────────────

fn prompt_save(editor: &mut Editor, stdout: &mut io::Stdout) -> io::Result<()> {
    // Draw a simple prompt at the bottom
    let (_term_cols, term_rows) = terminal::size()?;
    let row = term_rows.saturating_sub(3);
    queue!(
        stdout,
        cursor::MoveTo(0, row),
        terminal::Clear(terminal::ClearType::FromCursorDown),
        SetForegroundColor(Color::Yellow),
        Print("Save preset name (Enter to confirm, Esc to cancel): "),
        ResetColor,
        cursor::Show
    )?;
    stdout.flush()?;

    let mut name = String::new();
    loop {
        match event::read()? {
            Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                execute!(stdout, cursor::Hide)?;
                let trimmed = name.trim().to_string();
                if trimmed.is_empty() {
                    editor.status_msg = "Save cancelled (empty name).".into();
                } else if let Err(e) = editor.save_preset(&trimmed) {
                    editor.status_msg = format!("Save error: {}", e);
                }
                return Ok(());
            }
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                execute!(stdout, cursor::Hide)?;
                editor.status_msg = "Save cancelled.".into();
                return Ok(());
            }
            Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                name.pop();
                queue!(
                    stdout,
                    cursor::MoveTo(51, row),
                    terminal::Clear(terminal::ClearType::UntilNewLine),
                    Print(&name)
                )?;
                stdout.flush()?;
            }
            Event::Key(KeyEvent { code: KeyCode::Char(ch), modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT, .. }) => {
                if name.len() < 40 {
                    name.push(ch);
                    queue!(stdout, Print(ch))?;
                    stdout.flush()?;
                }
            }
            _ => {}
        }
    }
}

// ── Main event loop ───────────────────────────────────────────────────────────

fn run_editor() -> io::Result<()> {
    let mut stdout = io::stdout();
    let mut editor = Editor::new(4, 4);

    render(&editor, &mut stdout)?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent { code, modifiers: _, .. }) => {
                match code {
                    // Quit
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,

                    // Navigation
                    KeyCode::Up => {
                        if editor.cursor.0 > 0 { editor.cursor.0 -= 1; }
                    }
                    KeyCode::Down => {
                        if editor.cursor.0 + 1 < editor.rows { editor.cursor.0 += 1; }
                    }
                    KeyCode::Left => {
                        if editor.cursor.1 > 0 { editor.cursor.1 -= 1; }
                    }
                    KeyCode::Right => {
                        if editor.cursor.1 + 1 < editor.cols { editor.cursor.1 += 1; }
                    }

                    // Place spool 1-6
                    KeyCode::Char(ch @ '1'..='6') => {
                        let idx = (ch as u8 - b'1') as usize;
                        editor.selected_color = idx;
                        editor.place(EditorCell::Spool(idx));
                        editor.status_msg = format!("Placed Spool ({})", COLOR_NAMES[idx]);
                    }

                    // Place key spool
                    KeyCode::Char('k') | KeyCode::Char('K') => {
                        let idx = editor.selected_color;
                        editor.place(EditorCell::KeySpool(idx));
                        editor.status_msg = format!("Placed KeySpool ({})", COLOR_NAMES[idx]);
                    }

                    // Place conveyor
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        editor.place(EditorCell::Conveyor);
                        editor.status_msg = "Placed Conveyor".into();
                    }

                    // Place obstacle
                    KeyCode::Char('x') | KeyCode::Char('X') => {
                        editor.place(EditorCell::Obstacle);
                        editor.status_msg = "Placed Obstacle".into();
                    }

                    // Clear cell
                    KeyCode::Char(' ') | KeyCode::Char('0') => {
                        editor.place(EditorCell::Empty);
                        editor.status_msg = "Cleared cell".into();
                    }

                    // Grow board
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let (r, c) = (editor.rows, editor.cols);
                        let nr = (r + 1).min(6);
                        let nc = (c + 1).min(6);
                        if nr != r || nc != c {
                            editor.resize(nr, nc);
                            editor.status_msg = format!("Board grown to {}x{}", nr, nc);
                        } else {
                            editor.status_msg = "Board is already at max size (6x6).".into();
                        }
                    }

                    // Shrink board
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let (r, c) = (editor.rows, editor.cols);
                        let nr = (r.saturating_sub(1)).max(2);
                        let nc = (c.saturating_sub(1)).max(2);
                        if nr != r || nc != c {
                            editor.resize(nr, nc);
                            editor.status_msg = format!("Board shrunk to {}x{}", nr, nc);
                        } else {
                            editor.status_msg = "Board is already at minimum size (2x2).".into();
                        }
                    }

                    // Validate
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        editor.validate();
                    }

                    // Save
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        prompt_save(&mut editor, &mut stdout)?;
                    }

                    _ => {}
                }

                render(&editor, &mut stdout)?;
            }
            Event::Resize(_, _) => {
                render(&editor, &mut stdout)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    loom_engine::terminal::init()?;
    let result = run_editor();
    loom_engine::terminal::restore()?;
    result
}
