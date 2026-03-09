#![allow(warnings)]

use std::io::{Write, stdout, Stdout};
use std::io;

use crossterm::{
    ExecutableCommand, execute, queue, QueueableCommand,
    style::{Print, Stylize},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType, SetSize, enable_raw_mode, disable_raw_mode},
    cursor::{MoveTo, Hide, Show, position},
    event::{poll, read, Event, KeyCode},
};
use std::time::Duration;
use std::cmp::max;

use clap::Parser;

use knitui::board_entity::Direction;
use knitui::config::Config;
use knitui::engine::GameEngine;

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(
    stdout: &mut Stdout,
    engine: &GameEngine,
    minimal_y: u16,
) -> io::Result<()> {
    let x = engine.cursor_col;
    let y = engine.cursor_row + minimal_y;

    stdout.queue(Hide);
    stdout.execute(Clear(ClearType::All))?.execute(Clear(ClearType::Purge));

    stdout.queue(MoveTo(0, 0));
    stdout.queue(Print(&engine.yarn));

    for thread in &engine.active_threads {
        stdout.queue(Print(thread));
    }
    stdout.queue(Print("\n\r"));

    for thread_row in &engine.board.board {
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

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let config = Config::parse();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let yarn_offset: u16 = config.visible_patches;
    let active_offset: u16 = 2;
    let minimal_y: u16 = yarn_offset + active_offset;

    let mut engine = GameEngine::new(&config);

    render(&mut stdout, &engine, minimal_y)?;

    loop {
        if poll(Duration::from_millis(150))? {
            if let Event::Key(event) = read()? {
                match event.code {
                    KeyCode::Left  => { let _ = engine.move_cursor(Direction::Left);  }
                    KeyCode::Right => { let _ = engine.move_cursor(Direction::Right); }
                    KeyCode::Up    => { let _ = engine.move_cursor(Direction::Up);    }
                    KeyCode::Down  => { let _ = engine.move_cursor(Direction::Down);  }
                    KeyCode::Esc   => break,

                    KeyCode::Enter => {
                        if engine.pick_up().is_ok() {
                            render(&mut stdout, &engine, minimal_y)?;
                        }
                    }

                    _ => {}
                }

                let x = engine.cursor_col;
                let y = max(engine.cursor_row + minimal_y, minimal_y);
                stdout.execute(MoveTo(x, y));
            }
        } else if !engine.active_threads.is_empty() {
            engine.process_one_active();
            render(&mut stdout, &engine, minimal_y)?;
        }
    }

    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
