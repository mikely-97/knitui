use std::io::{Write, stdout, Stdout};

use crossterm::{
    QueueableCommand,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode},
    style::{Print, Stylize},
};

use crate::campaign::{TRACK_COUNT, TRACK_NAMES, levels_for_track};
use crate::engine::{GameEngine, GameStatus};
use crate::puzzle::Puzzle;
use crate::renderer;

// ── TUI state machine ─────────────────────────────────────────────────────

enum TuiState {
    MainMenu { selected: usize },
    TrackSelect { selected: usize },
    LevelSelect { track: usize, selected: usize },
    Playing { puzzle_name: String },
    GameOver { won: bool },
}

pub fn run_from_menu() -> std::io::Result<()> {
    loom_engine_term::init()?;
    let result = run_loop();
    loom_engine_term::restore()?;
    result
}

fn run_loop() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut state = TuiState::MainMenu { selected: 0 };
    let mut engine: Option<GameEngine> = None;

    render_state(&mut stdout, &state, engine.as_ref())?;

    loop {
        if !poll(std::time::Duration::from_millis(100))? {
            continue;
        }
        let Event::Key(key) = read()? else { continue };

        match &mut state {
            // ── Main Menu ─────────────────────────────────────────────────
            TuiState::MainMenu { selected } => {
                const ITEMS: &[&str] = &["Play Campaign", "Quit"];
                match key.code {
                    KeyCode::Up => {
                        if *selected > 0 { *selected -= 1; }
                    }
                    KeyCode::Down => {
                        if *selected < ITEMS.len() - 1 { *selected += 1; }
                    }
                    KeyCode::Enter => {
                        match *selected {
                            0 => { state = TuiState::TrackSelect { selected: 0 }; }
                            _ => return Ok(()),
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }

            // ── Track Select ──────────────────────────────────────────────
            TuiState::TrackSelect { selected } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected > 0 { *selected -= 1; }
                    }
                    KeyCode::Down => {
                        if *selected < TRACK_COUNT - 1 { *selected += 1; }
                    }
                    KeyCode::Enter => {
                        let track = *selected;
                        state = TuiState::LevelSelect { track, selected: 0 };
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        state = TuiState::MainMenu { selected: 0 };
                    }
                    _ => {}
                }
            }

            // ── Level Select ──────────────────────────────────────────────
            TuiState::LevelSelect { track, selected } => {
                let track = *track;
                let levels = levels_for_track(track);
                match key.code {
                    KeyCode::Up => {
                        if *selected > 0 { *selected -= 1; }
                    }
                    KeyCode::Down => {
                        if *selected < levels.len().saturating_sub(1) { *selected += 1; }
                    }
                    KeyCode::Enter => {
                        let idx = *selected;
                        let puzzle: Puzzle = levels.into_iter().nth(idx).unwrap();
                        let name = puzzle.name.to_string();
                        engine = Some(GameEngine::new(puzzle));
                        state = TuiState::Playing { puzzle_name: name };
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        state = TuiState::TrackSelect { selected: track };
                    }
                    _ => {}
                }
            }

            // ── Playing ───────────────────────────────────────────────────
            TuiState::Playing { .. } => {
                let mut quit_to_menu = false;
                if let Some(e) = engine.as_mut() {
                    match key.code {
                        KeyCode::Up    => e.move_cursor(-1, 0),
                        KeyCode::Down  => e.move_cursor(1, 0),
                        KeyCode::Left  => e.move_cursor(0, -1),
                        KeyCode::Right => e.move_cursor(0, 1),
                        KeyCode::Char(' ') | KeyCode::Enter => e.toggle_fill(),
                        KeyCode::Char('x') | KeyCode::Char('X') => e.toggle_cross(),
                        KeyCode::Char('q') | KeyCode::Esc => { quit_to_menu = true; }
                        _ => {}
                    }
                }
                if quit_to_menu {
                    engine = None;
                    state = TuiState::MainMenu { selected: 0 };
                } else {
                    // Check for terminal game states — reborrow after key handling
                    let status = engine.as_ref().map(|e| e.status);
                    match status {
                        Some(GameStatus::Won) => {
                            state = TuiState::GameOver { won: true };
                        }
                        Some(GameStatus::TooManyMistakes) => {
                            state = TuiState::GameOver { won: false };
                        }
                        _ => {}
                    }
                }
            }

            // ── Game Over ─────────────────────────────────────────────────
            TuiState::GameOver { .. } => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
                        engine = None;
                        state = TuiState::MainMenu { selected: 0 };
                    }
                    _ => {}
                }
            }
        }

        render_state(&mut stdout, &state, engine.as_ref())?;
    }
}

fn render_state(stdout: &mut Stdout, state: &TuiState, engine: Option<&GameEngine>) -> std::io::Result<()> {
    stdout.queue(Clear(ClearType::All))?;

    match state {
        TuiState::MainMenu { selected } => {
            render_main_menu(stdout, *selected)?;
        }
        TuiState::TrackSelect { selected } => {
            render_track_select(stdout, *selected)?;
        }
        TuiState::LevelSelect { track, selected } => {
            render_level_select(stdout, *track, *selected)?;
        }
        TuiState::Playing { puzzle_name } => {
            stdout.queue(MoveTo(2, 0))?;
            stdout.queue(Print(format!("Picross — {}", puzzle_name)))?;
            if let Some(e) = engine {
                renderer::render(stdout, e, 2, 2)?;
            }
        }
        TuiState::GameOver { won } => {
            render_game_over(stdout, *won)?;
        }
    }

    stdout.flush()
}

fn render_main_menu(stdout: &mut Stdout, selected: usize) -> std::io::Result<()> {
    const ITEMS: &[&str] = &["Play Campaign", "Quit"];

    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print("╔══════════════════════════╗"))?;
    stdout.queue(MoveTo(2, 2))?;
    stdout.queue(Print("║    Picross / Nonogram    ║"))?;
    stdout.queue(MoveTo(2, 3))?;
    stdout.queue(Print("╚══════════════════════════╝"))?;

    for (i, item) in ITEMS.iter().enumerate() {
        stdout.queue(MoveTo(4, 5 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {}", item).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", item)))?;
        }
    }
    Ok(())
}

fn render_track_select(stdout: &mut Stdout, selected: usize) -> std::io::Result<()> {
    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print("Select Difficulty:"))?;

    for (i, name) in TRACK_NAMES.iter().enumerate() {
        stdout.queue(MoveTo(4, 3 + i as u16))?;
        if i == selected {
            stdout.queue(Print(format!("► {}", name).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", name)))?;
        }
    }

    stdout.queue(MoveTo(2, 3 + TRACK_COUNT as u16 + 1))?;
    stdout.queue(Print("Esc: back"))?;
    Ok(())
}

fn render_level_select(stdout: &mut Stdout, track: usize, selected: usize) -> std::io::Result<()> {
    let levels = levels_for_track(track);
    let track_name = TRACK_NAMES.get(track).copied().unwrap_or("?");

    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print(format!("{} Puzzles:", track_name)))?;

    for (i, puzzle) in levels.iter().enumerate() {
        stdout.queue(MoveTo(4, 3 + i as u16))?;
        let label = format!("{:>2}. {} ({}×{})", i + 1, puzzle.name, puzzle.rows, puzzle.cols);
        if i == selected {
            stdout.queue(Print(format!("► {}", label).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", label)))?;
        }
    }

    stdout.queue(MoveTo(2, 3 + levels.len() as u16 + 1))?;
    stdout.queue(Print("Esc: back"))?;
    Ok(())
}

fn render_game_over(stdout: &mut Stdout, won: bool) -> std::io::Result<()> {
    stdout.queue(MoveTo(4, 4))?;
    if won {
        stdout.queue(Print("  *** Puzzle Solved! Congratulations! ***  "))?;
    } else {
        stdout.queue(Print("  Too many mistakes. Better luck next time!  "))?;
    }
    stdout.queue(MoveTo(4, 6))?;
    stdout.queue(Print("Press Enter or Q to return to the menu."))?;
    Ok(())
}
