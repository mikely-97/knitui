#![allow(warnings)]

use std::io::{Write, stdout, Stdout};
use std::io;

use crossterm::{
    ExecutableCommand, execute, queue, QueueableCommand,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, style, Attribute, Stylize},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, SetSize, enable_raw_mode, disable_raw_mode},
    cursor::{MoveTo, Hide, Show, position},
    event::{poll, read, Event, KeyCode},
};
use std::time::Duration;
use std::cmp::{min, max};

use clap::Parser;

use knitui::game_board::GameBoard;
use knitui::board_entity::BoardEntity;
use knitui::yarn::Yarn;
use knitui::palette::select_palette;
use knitui::active_threads::Thread;
use knitui::config::Config;
use knitui::solvability::is_solvable;

// ── Animation state ───────────────────────────────────────────────────────────

enum ProcessingState {
    Idle,
    Processing { remaining: Vec<Thread> },
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(
    stdout: &mut Stdout,
    game_board: &GameBoard,
    active_threads: &Vec<Thread>,
    yarn: &Yarn,
    x: u16,
    y: u16,
    minimal_y: u16,
) -> io::Result<()> {
    stdout.queue(Hide);
    stdout.execute(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));

    stdout.queue(MoveTo(0, 0));
    stdout.queue(Print(yarn));

    for thread in active_threads {
        stdout.queue(Print(thread));
    }
    stdout.queue(Print("\n\r"));

    for thread_row in &game_board.board {
        stdout.queue(Print("\n\r"));
        for cell in thread_row {
            stdout.queue(Print(cell));
        }
    }

    let (size_x, size_y) = position()?;
    stdout.queue(SetSize(size_x, size_y));
    stdout.queue(MoveTo(x, max(y, minimal_y)));
    stdout.queue(Show);

    stdout.flush()
}

// ── Generator helpers ─────────────────────────────────────────────────────────

/// Find the (row, col) of a Generator whose output cell is at `(out_row, out_col)`.
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

/// Pop the next thread from the generator queue and place it at the output cell,
/// or convert the generator to DepletedGenerator if the queue is empty.
fn advance_generator(
    board: &mut Vec<Vec<BoardEntity>>,
    gen_row: usize,
    gen_col: usize,
    out_row: usize,
    out_col: usize,
) {
    enum Action { Spawn(Color), Deplete }

    let action = if let BoardEntity::Generator(ref mut data) = board[gen_row][gen_col] {
        if data.queue.is_empty() {
            Action::Deplete
        } else {
            Action::Spawn(data.queue.remove(0))
        }
    } else {
        return;
    };

    match action {
        Action::Spawn(color) => board[out_row][out_col] = BoardEntity::Thread(color),
        Action::Deplete      => board[gen_row][gen_col] = BoardEntity::DepletedGenerator,
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let config = Config::parse();
    let color_mode = config.parsed_color_mode();

    // Terminal setup
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    // Layout constants derived from config
    let yarn_offset: u16 = config.visible_patches;
    let active_offset: u16 = 2;
    let minimal_y: u16 = yarn_offset + active_offset;

    // Generate a solvable board (retry up to 100 times)
    let selected_palette = select_palette(color_mode, config.color_number);
    let mut game_board;
    let mut yarn;
    let mut attempts = 0u32;
    loop {
        game_board = GameBoard::make_random(
            config.board_height,
            config.board_width,
            &selected_palette,
            config.obstacle_percentage,
            config.knit_volume,
        );
        yarn = Yarn::make_from_color_counter(
            game_board.count_knits(),
            config.yarn_lines,
            config.visible_patches,
        );
        if is_solvable(&game_board, &yarn, config.knit_volume, config.active_threads_limit) {
            break;
        }
        attempts += 1;
        if attempts >= 100 {
            break;
        }
    }

    let mut active_threads: Vec<Thread> = Vec::new();
    let mut state = ProcessingState::Idle;

    render(&mut stdout, &game_board, &active_threads, &yarn, 0, 0, minimal_y)?;

    let (mut x, mut y) = position()?;

    loop {
        let timeout = match &state {
            ProcessingState::Idle              => Duration::from_millis(500),
            ProcessingState::Processing { .. } => Duration::from_millis(150),
        };

        if poll(timeout)? {
            if let Event::Key(event) = read()? {
                match state {
                    // Input is blocked while threads animate through the yarn.
                    ProcessingState::Processing { .. } => {}

                    ProcessingState::Idle => {
                        match event.code {
                            KeyCode::Left  => x = x.saturating_sub(1),
                            KeyCode::Right => x = min(x + 1, game_board.width - 1),
                            KeyCode::Up    => y = max(minimal_y, y.saturating_sub(1)),
                            KeyCode::Down  => y = min(y + 1, game_board.height + minimal_y - 1),
                            KeyCode::Esc   => break,

                            KeyCode::Enter => {
                                let board_row = (y - minimal_y) as usize;
                                let board_col = x as usize;

                                if game_board.is_selectable(board_row, board_col)
                                    && active_threads.len() < config.active_threads_limit
                                {
                                    let thread_opt = match &game_board.board[board_row][board_col] {
                                        BoardEntity::Thread(c) =>
                                            Some(Thread { color: *c, status: 1, has_key: false }),
                                        BoardEntity::KeyThread(c) =>
                                            Some(Thread { color: *c, status: 1, has_key: true }),
                                        _ => None,
                                    };

                                    if let Some(t) = thread_opt {
                                        active_threads.push(t);
                                        game_board.board[board_row][board_col] = BoardEntity::Void;

                                        // If this cell was a generator output, produce the next thread.
                                        if let Some((gr, gc)) = find_generator_for_output(
                                            &game_board.board, board_row, board_col,
                                        ) {
                                            advance_generator(
                                                &mut game_board.board,
                                                gr, gc, board_row, board_col,
                                            );
                                        }

                                        render(&mut stdout, &game_board, &active_threads, &yarn, x, y, minimal_y)?;
                                    }
                                }
                            }

                            KeyCode::Backspace => {
                                if !active_threads.is_empty() {
                                    let remaining = active_threads.drain(..).collect();
                                    state = ProcessingState::Processing { remaining };
                                }
                            }

                            _ => {}
                        }

                        stdout.execute(MoveTo(x, y));
                    }
                }
            }
        } else {
            // Timeout: advance the animation by one thread.
            if let ProcessingState::Processing { ref mut remaining } = state {
                if let Some(mut thread) = remaining.pop() {
                    yarn.process_one(&mut thread);
                    if thread.status <= config.knit_volume {
                        active_threads.push(thread);
                    }
                }
                if remaining.is_empty() {
                    state = ProcessingState::Idle;
                }
                render(&mut stdout, &game_board, &active_threads, &yarn, x, y, minimal_y)?;
            }
        }
    }

    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
