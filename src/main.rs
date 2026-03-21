use std::io::{Write, stdout, Stdout};

use crossterm::{
    QueueableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::{Hide, Show, MoveTo},
    event::{poll, read, Event, KeyCode},
    style::{Print, Stylize},
};

use loom_engine::stats::AllStats;
use loom_engine::achievements::{ALL_ACHIEVEMENTS, AchievementTracker};

const GAMES: &[(&str, &str)] = &[
    ("Knit",    "Spool-knitting puzzle"),
    ("Match-3", "Classic gem-matching"),
    ("Merge-2", "Merge puzzle"),
    ("Picross", "Nonogram/picross puzzles"),
];

// Menu items: games + extras
const MENU_EXTRA: &[&str] = &["Stats", "Trophies", "Quit"];

fn render_selector(stdout: &mut Stdout, selected: usize) -> std::io::Result<()> {
    let total = GAMES.len() + MENU_EXTRA.len();
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

    // Separator
    let sep_row = 7 + GAMES.len() as u16;
    stdout.queue(MoveTo(4, sep_row))?;
    stdout.queue(Print("──────────────────────────"))?;

    for (j, label) in MENU_EXTRA.iter().enumerate() {
        let i = GAMES.len() + j;
        stdout.queue(MoveTo(4, sep_row + 1 + j as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {}", label).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", label)))?;
        }
    }

    stdout.queue(MoveTo(2, sep_row + 1 + MENU_EXTRA.len() as u16 + 1))?;
    stdout.queue(Print("↑↓ Navigate   Enter Select   Q Quit"))?;
    let _ = total;
    stdout.flush()?;
    Ok(())
}

fn show_stats(stdout: &mut Stdout) -> std::io::Result<()> {
    let stats = AllStats::load();

    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print("╔══════════════════════════════════╗"))?;
    stdout.queue(MoveTo(2, 2))?;
    stdout.queue(Print("║           Game Statistics        ║"))?;
    stdout.queue(MoveTo(2, 3))?;
    stdout.queue(Print("╚══════════════════════════════════╝"))?;

    let rows: &[(&str, &loom_engine::stats::GameStats)] = &[
        ("Knit",    &stats.knit),
        ("Match-3", &stats.match3),
        ("Merge-2", &stats.merge2),
        ("Picross", &stats.picross),
    ];

    let mut row = 5u16;
    for (name, gs) in rows {
        let best = gs.high_scores.first().map(|e| e.score).unwrap_or(0);
        stdout.queue(MoveTo(4, row))?;
        stdout.queue(Print(format!("{:<8}  Played: {:>4}  Win%: {:>5.1}%  Best: {:>8}",
            name, gs.games_played, gs.win_rate(), best)))?;
        row += 2;
    }

    stdout.queue(MoveTo(2, row + 1))?;
    stdout.queue(Print("Press any key to return..."))?;
    stdout.flush()?;

    loop {
        if !poll(std::time::Duration::from_millis(100))? { continue; }
        let Event::Key(_) = read()? else { continue };
        break;
    }
    Ok(())
}

fn show_trophies(stdout: &mut Stdout) -> std::io::Result<()> {
    let tracker = AchievementTracker::load();

    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print("╔══════════════════════════════════╗"))?;
    stdout.queue(MoveTo(2, 2))?;
    stdout.queue(Print("║           Achievements           ║"))?;
    stdout.queue(MoveTo(2, 3))?;
    stdout.queue(Print("╚══════════════════════════════════╝"))?;

    for (i, ach) in ALL_ACHIEVEMENTS.iter().enumerate() {
        let check = if tracker.is_unlocked(ach.id) { "\u{2713}" } else { "\u{25CB}" };
        stdout.queue(MoveTo(4, 5 + i as u16 * 2))?;
        stdout.queue(Print(format!("{} {} {}  - {}", check, ach.icon, ach.name, ach.desc)))?;
    }

    let footer_row = 5 + ALL_ACHIEVEMENTS.len() as u16 * 2 + 1;
    stdout.queue(MoveTo(2, footer_row))?;
    stdout.queue(Print("Press any key to return..."))?;
    stdout.flush()?;

    loop {
        if !poll(std::time::Duration::from_millis(100))? { continue; }
        let Event::Key(_) = read()? else { continue };
        break;
    }
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

    let total_items = GAMES.len() + MENU_EXTRA.len();
    let mut selected = 0usize;
    render_selector(&mut stdout, selected)?;

    loop {
        if !poll(std::time::Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = read()? else { continue };

        match key.code {
            KeyCode::Up => {
                if selected > 0 { selected -= 1; }
            }
            KeyCode::Down => {
                if selected < total_items - 1 { selected += 1; }
            }
            KeyCode::Enter => {
                // Game choices
                if selected < GAMES.len() {
                    disable_raw_mode()?;
                    execute!(stdout, Show, LeaveAlternateScreen)?;
                    let result = match selected {
                        0 => knitui::tui::run_cli(),
                        1 => m3tui::tui::run_from_menu(),
                        2 => m2tui::tui::run_from_menu(),
                        3 => pictui::tui::run_from_menu(),
                        _ => Ok(()),
                    };
                    return result;
                }
                // Extra menu items
                let extra_idx = selected - GAMES.len();
                match extra_idx {
                    0 => {
                        // Stats
                        show_stats(&mut stdout)?;
                        render_selector(&mut stdout, selected)?;
                    }
                    1 => {
                        // Trophies
                        show_trophies(&mut stdout)?;
                        render_selector(&mut stdout, selected)?;
                    }
                    2 => {
                        // Quit
                        break;
                    }
                    _ => {}
                }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                break;
            }
            _ => { continue; }
        }
        render_selector(&mut stdout, selected)?;
    }

    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;
    Ok(())
}
