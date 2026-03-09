#![allow(warnings)]

use std::io::{Write, stdout, Stdout};
use std::io;

use crossterm::{
    ExecutableCommand, execute, queue, QueueableCommand,
    style::{Print, Stylize, Attribute, SetAttribute},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, SetSize, BeginSynchronizedUpdate, EndSynchronizedUpdate, enable_raw_mode, disable_raw_mode},
    cursor::{MoveTo, Hide, Show, position},
    event::{poll, read, Event, KeyCode},
};
use std::time::Duration;

use clap::Parser;

// ── Spacing constants ────────────────────────────────────────────────────────
const YARN_HGAP: u16 = 2;   // horizontal gap between yarn columns
const YARN_VGAP: u16 = 1;   // vertical gap between yarn rows (< YARN_HGAP)
const THREAD_GAP: u16 = 1;  // gap between active threads
const COMP_GAP: u16 = 3;    // gap between components (> all inner gaps)

use knitui::board_entity::Direction;
use knitui::config::Config;
use knitui::engine::{GameEngine, GameStatus, BonusState};

enum TuiState {
    Playing,
    GameOver(GameStatus),
    Help,
}

#[derive(Clone, Copy)]
enum Layout {
    Vertical,
    Horizontal,
}

fn detect_layout(config_layout: &str, visible_patches: u16, board_height: u16, scale: u16) -> Layout {
    match config_layout {
        "horizontal" => Layout::Horizontal,
        "vertical" => Layout::Vertical,
        _ => {
            let sh = scale;
            let (_, term_height) = terminal::size().unwrap_or((80, 24));
            let yarn_h = visible_patches * sh + visible_patches.saturating_sub(1) * YARN_VGAP;
            let board_h = 1 + board_height * (sh + 1);
            let vertical_height = yarn_h + COMP_GAP + sh + COMP_GAP + board_h;
            if vertical_height + 2 > term_height {
                Layout::Horizontal
            } else {
                Layout::Vertical
            }
        }
    }
}

// ── Scaled rendering helpers ─────────────────────────────────────────────────

/// Render yarn patches into a region starting at (x0, y0), scaled with spacing.
fn render_yarn(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for offset in 0..(engine.yarn.visible_patches as usize) {
        let true_offset = (engine.yarn.visible_patches as usize) - offset;
        let row_y = y0 + (offset as u16) * (sh + YARN_VGAP);
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, row_y + sy))?;
            for (ci, column) in engine.yarn.board.iter().enumerate() {
                if ci > 0 {
                    for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
                }
                if true_offset <= column.len() {
                    let pos = column.len() - true_offset;
                    for _ in 0..sw { stdout.queue(Print(&column[pos]))?; }
                } else {
                    for _ in 0..sw { stdout.queue(Print(' '))?; }
                }
            }
        }
    }

    // Render balloon columns (if any) to the right with a separator
    if !engine.yarn.balloon_columns.is_empty() {
        let regular_w = engine.yarn.yarn_lines * sw
            + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
        let balloon_x0 = x0 + regular_w + COMP_GAP;

        for offset in 0..(engine.yarn.visible_patches as usize) {
            let true_offset = (engine.yarn.visible_patches as usize) - offset;
            let row_y = y0 + (offset as u16) * (sh + YARN_VGAP);
            for sy in 0..sh {
                stdout.queue(MoveTo(balloon_x0, row_y + sy))?;
                for (ci, column) in engine.yarn.balloon_columns.iter().enumerate() {
                    if ci > 0 {
                        for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
                    }
                    if true_offset <= column.len() {
                        let pos = column.len() - true_offset;
                        for _ in 0..sw { stdout.queue(Print(&column[pos]))?; }
                    } else {
                        for _ in 0..sw { stdout.queue(Print(' '))?; }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Render active threads horizontally (one row, scaled) starting at (x0, y0).
fn render_active_h(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for sy in 0..sh {
        stdout.queue(MoveTo(x0, y0 + sy))?;
        for (i, thread) in engine.active_threads.iter().enumerate() {
            if i > 0 {
                for _ in 0..THREAD_GAP { stdout.queue(Print(' '))?; }
            }
            for _ in 0..sw { stdout.queue(Print(thread))?; }
        }
    }
    Ok(())
}

/// Render active threads vertically (one column, scaled) starting at (x0, y0).
fn render_active_v(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    for (i, thread) in engine.active_threads.iter().enumerate() {
        let ty = y0 + (i as u16) * (sh + THREAD_GAP);
        for sy in 0..sh {
            stdout.queue(MoveTo(x0, ty + sy))?;
            for _ in 0..sw { stdout.queue(Print(thread))?; }
        }
    }
    Ok(())
}

/// Draw a horizontal border line for the board grid.
/// kind: 0=top (┌┬┐), 1=middle (├┼┤), 2=bottom (└┴┘)
fn draw_hline(stdout: &mut Stdout, x0: u16, y: u16, cols: usize, sw: u16, kind: u8) -> io::Result<()> {
    stdout.queue(MoveTo(x0, y))?;
    let (left, fill, cross, right) = match kind {
        0 => ('┌', '─', '┬', '┐'),
        2 => ('└', '─', '┴', '┘'),
        _ => ('├', '─', '┼', '┤'),
    };
    stdout.queue(Print(left))?;
    for c in 0..cols {
        for _ in 0..sw { stdout.queue(Print(fill))?; }
        if c < cols - 1 { stdout.queue(Print(cross))?; }
    }
    stdout.queue(Print(right))?;
    Ok(())
}

/// Render the game board with box borders and bracket cursor markers.
fn render_board(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    let sh = scale;
    let sw = scale * 2;
    let rows = engine.board.height as usize;
    let cols = engine.board.width as usize;
    let cur_r = engine.cursor_row as usize;
    let cur_c = engine.cursor_col as usize;

    let tweezers = matches!(engine.bonus_state, BonusState::TweezersActive { .. });
    let (open_bracket, close_bracket) = if tweezers { ('{', '}') } else { ('[', ']') };

    // Top border
    draw_hline(stdout, x0, y0, cols, sw, 0)?;

    for (row_idx, thread_row) in engine.board.board.iter().enumerate() {
        let content_y = y0 + 1 + (row_idx as u16) * (sh + 1);
        let is_cur_row = row_idx == cur_r;

        for sy in 0..sh {
            stdout.queue(MoveTo(x0, content_y + sy))?;
            for (col_idx, cell) in thread_row.iter().enumerate() {
                let is_cursor = is_cur_row && col_idx == cur_c;
                let is_after_cursor = is_cur_row && col_idx > 0 && col_idx - 1 == cur_c;

                // Left border: bright brackets for cursor edges, normal │ otherwise
                if is_cursor {
                    stdout.queue(Print(open_bracket.bold().white()))?;
                } else if is_after_cursor {
                    stdout.queue(Print(close_bracket.bold().white()))?;
                } else {
                    stdout.queue(Print('│'))?;
                }

                // Cell content: inverted colors for cursor cell
                if is_cursor {
                    stdout.queue(SetAttribute(Attribute::Reverse))?;
                    for _ in 0..sw { stdout.queue(Print(cell))?; }
                    stdout.queue(SetAttribute(Attribute::Reset))?;
                } else {
                    for _ in 0..sw { stdout.queue(Print(cell))?; }
                }
            }
            // Right border
            if is_cur_row && cols - 1 == cur_c {
                stdout.queue(Print(close_bracket.bold().white()))?;
            } else {
                stdout.queue(Print('│'))?;
            }
        }

        let line_y = content_y + sh;
        if row_idx < rows - 1 {
            draw_hline(stdout, x0, line_y, cols, sw, 1)?;
        } else {
            draw_hline(stdout, x0, line_y, cols, sw, 2)?;
        }
    }

    Ok(())
}

fn render_help(stdout: &mut Stdout) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let lines = [
        "",
        "                    ═══ HELP ═══",
        "",
        "  Movement:   ← → ↑ ↓   Move cursor",
        "  Pick up:    Enter       Pick up thread at cursor",
        "  Quit:       Esc / Q     Exit game",
        "  Restart:    R           New game",
        "  Help:       H           Show this screen",
        "",
        "  ─── Bonuses ───",
        "  [Z] ✂ Scissors    Auto-knit thread by deep-scanning yarn",
        "  [X] ⊹ Tweezers    Pick any thread from the board",
        "  [C] ⊛ Balloons    Lift front patches, expose patches behind",
        "",
        "              Press any key to close",
    ];

    for (i, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(0, i as u16))?;
        stdout.queue(Print(line))?;
    }

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

fn render_keybar(stdout: &mut Stdout, engine: &GameEngine, y: u16) -> io::Result<()> {
    stdout.queue(MoveTo(0, y))?;
    let (term_w, _) = terminal::size().unwrap_or((80, 24));
    for _ in 0..term_w { stdout.queue(Print(' '))?; }
    stdout.queue(MoveTo(0, y))?;

    stdout.queue(Print("←→↑↓ ".dark_grey()))?;
    stdout.queue(Print("Move  ".white()))?;
    stdout.queue(Print("Enter ".dark_grey()))?;
    stdout.queue(Print("Pick  ".white()))?;
    stdout.queue(Print("H ".dark_grey()))?;
    stdout.queue(Print("Help  ".white()))?;

    if engine.bonuses.scissors > 0 {
        stdout.queue(Print("Z ".dark_grey()))?;
        stdout.queue(Print(format!("✂x{} ", engine.bonuses.scissors).white()))?;
    } else {
        stdout.queue(Print("Z ✂x0 ".dark_grey()))?;
    }
    if engine.bonuses.tweezers > 0 {
        stdout.queue(Print("X ".dark_grey()))?;
        stdout.queue(Print(format!("⊹x{} ", engine.bonuses.tweezers).white()))?;
    } else {
        stdout.queue(Print("X ⊹x0 ".dark_grey()))?;
    }
    if engine.bonuses.balloons > 0 {
        stdout.queue(Print("C ".dark_grey()))?;
        stdout.queue(Print(format!("⊛x{} ", engine.bonuses.balloons).white()))?;
    } else {
        stdout.queue(Print("C ⊛x0 ".dark_grey()))?;
    }

    stdout.queue(Print("Esc ".dark_grey()))?;
    stdout.queue(Print("Quit".white()))?;
    Ok(())
}

fn render_bonus_display_h(stdout: &mut Stdout, engine: &GameEngine, x: u16, y: u16) -> io::Result<()> {
    stdout.queue(MoveTo(x, y))?;
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        if i > 0 { stdout.queue(Print("  "))?; }
        if *count > 0 {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).white()))?;
        } else {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).dark_grey()))?;
        }
    }
    Ok(())
}

fn render_bonus_panel(stdout: &mut Stdout, engine: &GameEngine, x: u16, y: u16) -> io::Result<()> {
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        stdout.queue(MoveTo(x, y + i as u16))?;
        if *count > 0 {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).white()))?;
        } else {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).dark_grey()))?;
        }
    }
    Ok(())
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
) -> io::Result<()> {
    let sh = scale;
    let yarn_h = engine.yarn.visible_patches * sh
        + engine.yarn.visible_patches.saturating_sub(1) * YARN_VGAP;
    let active_y = yarn_h + COMP_GAP;

    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    render_yarn(stdout, engine, 0, 0, scale)?;
    render_active_h(stdout, engine, 0, active_y, scale)?;
    render_board(stdout, engine, 0, board_y, scale)?;

    let board_h = 1 + engine.board.height * (sh + 1);
    let bonus_y = board_y + board_h + 1;
    render_bonus_display_h(stdout, engine, 0, bonus_y)?;

    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    render_keybar(stdout, engine, term_h.saturating_sub(1))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

fn render_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_y: u16,
    scale: u16,
    status: &GameStatus,
) -> io::Result<()> {
    render(stdout, engine, board_y, scale)?;
    let message = match status {
        GameStatus::Stuck => "You're lost! Press R to restart, Q to quit",
        GameStatus::Won   => "You won! Press R to play again, Q to quit",
        _ => return Ok(()),
    };
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Print(message))?;
    stdout.flush()
}

fn render_horizontal(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_x: u16,
    scale: u16,
) -> io::Result<()> {
    let sw = scale * 2;
    let yarn_w = engine.yarn.yarn_lines * sw
        + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let active_x = yarn_w + COMP_GAP;

    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    render_yarn(stdout, engine, 0, 0, scale)?;
    render_active_v(stdout, engine, active_x, 0, scale)?;
    render_board(stdout, engine, board_x, 0, scale)?;

    let board_w = 1 + engine.board.width * (sw + 1);
    let panel_x = board_x + board_w + 2;
    render_bonus_panel(stdout, engine, panel_x, 0)?;

    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    render_keybar(stdout, engine, term_h.saturating_sub(1))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

fn render_horizontal_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    board_x: u16,
    scale: u16,
    status: &GameStatus,
) -> io::Result<()> {
    render_horizontal(stdout, engine, board_x, scale)?;
    let message = match status {
        GameStatus::Stuck => "You're lost! Press R to restart, Q to quit",
        GameStatus::Won   => "You won! Press R to play again, Q to quit",
        _ => return Ok(()),
    };
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Print(message))?;
    stdout.flush()
}

fn do_render(
    stdout: &mut Stdout,
    engine: &GameEngine,
    layout: Layout,
    board_x: u16,
    board_y: u16,
    scale: u16,
) -> io::Result<()> {
    match layout {
        Layout::Vertical => render(stdout, engine, board_y, scale),
        Layout::Horizontal => render_horizontal(stdout, engine, board_x, scale),
    }
}

fn do_render_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    layout: Layout,
    board_x: u16,
    board_y: u16,
    scale: u16,
    status: &GameStatus,
) -> io::Result<()> {
    match layout {
        Layout::Vertical => render_overlay(stdout, engine, board_y, scale, status),
        Layout::Horizontal => render_horizontal_overlay(stdout, engine, board_x, scale, status),
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let config = Config::parse();
    let scale = config.scale;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let sh = scale;
    let sw = scale * 2;

    let layout = detect_layout(&config.layout, config.visible_patches, config.board_height, scale);

    // Vertical layout offsets
    let yarn_h = config.visible_patches * sh + config.visible_patches.saturating_sub(1) * YARN_VGAP;
    let board_y: u16 = yarn_h + COMP_GAP + sh + COMP_GAP; // yarn + gap + active row + gap

    // Horizontal layout offsets
    let yarn_w = config.yarn_lines * sw + config.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let board_x: u16 = yarn_w + COMP_GAP + sw + COMP_GAP; // yarn + gap + active col + gap

    let mut engine = GameEngine::new(&config);
    let mut tui_state = TuiState::Playing;

    do_render(&mut stdout, &engine, layout, board_x, board_y, scale)?;

    loop {
        if poll(Duration::from_millis(150))? {
            if let Event::Key(event) = read()? {
                match tui_state {
                    TuiState::GameOver(_) => {
                        match event.code {
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                engine = GameEngine::new(&config);
                                tui_state = TuiState::Playing;
                                do_render(&mut stdout, &engine, layout, board_x, board_y, scale)?;
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    TuiState::Help => {
                        tui_state = TuiState::Playing;
                        do_render(&mut stdout, &engine, layout, board_x, board_y, scale)?;
                    }
                    TuiState::Playing => {
                        match event.code {
                            KeyCode::Left  => { let _ = engine.move_cursor(Direction::Left);  }
                            KeyCode::Right => { let _ = engine.move_cursor(Direction::Right); }
                            KeyCode::Up    => { let _ = engine.move_cursor(Direction::Up);    }
                            KeyCode::Down  => { let _ = engine.move_cursor(Direction::Down);  }
                            KeyCode::Esc => {
                                if engine.bonus_state != BonusState::None {
                                    engine.cancel_tweezers();
                                } else {
                                    break;
                                }
                            }

                            KeyCode::Enter => {
                                if engine.pick_up().is_ok() {
                                    match engine.status() {
                                        GameStatus::Playing => {}
                                        s => {
                                            do_render_overlay(&mut stdout, &engine, layout, board_x, board_y, scale, &s)?;
                                            tui_state = TuiState::GameOver(s);
                                            continue;
                                        }
                                    };
                                }
                            }

                            KeyCode::Char('h') | KeyCode::Char('H') => {
                                render_help(&mut stdout)?;
                                tui_state = TuiState::Help;
                                continue;
                            }
                            KeyCode::Char('z') | KeyCode::Char('Z') => {
                                let _ = engine.use_scissors();
                            }
                            KeyCode::Char('x') | KeyCode::Char('X') => {
                                let _ = engine.use_tweezers();
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                let _ = engine.use_balloons();
                            }

                            _ => { continue; }
                        }

                        // Re-render to update bracket cursor markers
                        do_render(&mut stdout, &engine, layout, board_x, board_y, scale)?;
                    }
                }
            }
        } else if matches!(tui_state, TuiState::Playing) && !engine.active_threads.is_empty() {
            engine.process_all_active();
            match engine.status() {
                GameStatus::Playing => do_render(&mut stdout, &engine, layout, board_x, board_y, scale)?,
                s => {
                    do_render_overlay(&mut stdout, &engine, layout, board_x, board_y, scale, &s)?;
                    tui_state = TuiState::GameOver(s);
                }
            };
        }
    }

    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
