use std::io::{Write, stdout, Stdout};

use crossterm::{
    QueueableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::{Hide, Show, MoveTo},
    event::{poll, read, Event, KeyCode},
    style::{Print, Stylize},
};

const GAMES: &[(&str, &str)] = &[
    ("Knit",    "Spool-knitting puzzle"),
    ("Match-3", "Classic gem-matching"),
    ("Merge-2", "Merge puzzle"),
];

fn render_selector(stdout: &mut Stdout, selected: usize) -> std::io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print("╔══════════════════════════╗"))?;
    stdout.queue(MoveTo(2, 2))?;
    stdout.queue(Print("║     Welcome to Loom      ║"))?;
    stdout.queue(MoveTo(2, 3))?;
    stdout.queue(Print("╚══════════════════════════╝"))?;

    stdout.queue(MoveTo(2, 5))?;
    stdout.queue(Print("Select a game:"))?;

    for (i, (name, desc)) in GAMES.iter().enumerate() {
        let line = format!("{:<10} {}", name, desc);
        stdout.queue(MoveTo(4, 7 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {}", line).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", line)))?;
        }
    }

    stdout.queue(MoveTo(2, 7 + GAMES.len() as u16 + 1))?;
    stdout.queue(Print("↑↓ Navigate   Enter Select   Q Quit"))?;
    stdout.flush()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Panic hook for terminal cleanup
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        default_hook(info);
    }));

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    enable_raw_mode()?;

    let mut selected = 0usize;
    render_selector(&mut stdout, selected)?;

    let choice = loop {
        if !poll(std::time::Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = read()? else { continue };

        match key.code {
            KeyCode::Up => {
                if selected > 0 { selected -= 1; }
            }
            KeyCode::Down => {
                if selected < GAMES.len() - 1 { selected += 1; }
            }
            KeyCode::Enter => {
                break Some(selected);
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                break None;
            }
            _ => { continue; }
        }
        render_selector(&mut stdout, selected)?;
    };

    // Leave alternate screen before launching game (game will re-enter)
    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;

    match choice {
        Some(0) => knitui::tui::run_cli(),
        Some(1) => m3tui::tui::run_from_menu(),
        Some(2) => m2tui::tui::run_from_menu(),
        _ => Ok(()),
    }
}
