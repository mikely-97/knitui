/// knitui_replay — TUI replay viewer.
///
/// Accepts the same board-creation flags as knitui-ni (or --game HASH to load
/// an existing game), runs the DFS solver to find a solution sequence, then
/// renders each step with a 300 ms delay, highlighting the spool about to be
/// picked.  Ends with "Solved in N moves — press any key".

use std::io::{Write, stdout};
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    QueueableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode,
               Clear, ClearType},
    cursor::{MoveTo, Hide, Show},
    event::{poll, read, Event},
    style::{Print, Stylize, SetBackgroundColor, SetForegroundColor, SetAttribute, Attribute,
            ResetColor},
};

use knitui::engine::GameEngine;
use knitui::config::Config;
use knitui::board_entity::BoardEntity;
use knitui::solvability::find_solution;

// ── CLI ───────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "knitui-replay", about = "Replay a knitui puzzle solution step by step")]
struct Args {
    /// Load an existing game by hash (from ~/.local/share/knitui/<hash>.json)
    #[arg(long)]
    game: Option<String>,

    // Board-creation options (same as knitui-ni when --game is absent)
    #[arg(long)] board_height:        Option<u16>,
    #[arg(long)] board_width:         Option<u16>,
    #[arg(long)] color_number:        Option<u16>,
    #[arg(long)] color_mode:          Option<String>,
    #[arg(long)] spool_limit:         Option<usize>,
    #[arg(long)] spool_capacity:      Option<u16>,
    #[arg(long)] yarn_lines:          Option<u16>,
    #[arg(long)] obstacle_percentage: Option<u16>,
    #[arg(long)] visible_stitches:    Option<u16>,
    #[arg(long)] conveyor_capacity:   Option<u16>,
    #[arg(long)] conveyor_percentage: Option<u16>,
    #[arg(long)] scissors:            Option<u16>,
    #[arg(long)] tweezers:            Option<u16>,
    #[arg(long)] balloons:            Option<u16>,
    #[arg(long)] scissors_spools:     Option<u16>,
    #[arg(long)] balloon_count:       Option<u16>,
    #[arg(long)] max_solutions:       Option<u64>,

    /// Delay between steps in milliseconds (default: 300)
    #[arg(long, default_value_t = 300)]
    step_ms: u64,
}

// ── Persistence helpers ───────────────────────────────────────────────────────

fn game_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("knitui")
}

fn load_engine(hash: &str) -> Result<GameEngine, String> {
    let path = game_dir().join(format!("{hash}.json"));
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    GameEngine::from_json(&json)
}

// ── Config builder ────────────────────────────────────────────────────────────

fn base_config() -> Config {
    Config {
        board_height: 6,
        board_width: 6,
        color_number: 6,
        color_mode: "dark".into(),
        spool_limit: 7,
        spool_capacity: 3,
        yarn_lines: 4,
        obstacle_percentage: 5,
        visible_stitches: 6,
        conveyor_capacity: 3,
        conveyor_percentage: 5,
        layout: "auto".into(),
        scale: 1,
        scissors: 0,
        tweezers: 0,
        balloons: 0,
        scissors_spools: 1,
        balloon_count: 2,
        ad_file: None,
        max_solutions: None,
        hard_mode: false,
    }
}

fn config_from_args(args: &Args, mut config: Config) -> Config {
    if let Some(v) = args.board_height        { config.board_height = v; }
    if let Some(v) = args.board_width         { config.board_width = v; }
    if let Some(v) = args.color_number        { config.color_number = v; }
    if let Some(ref v) = args.color_mode      { config.color_mode = v.clone(); }
    if let Some(v) = args.spool_limit         { config.spool_limit = v; }
    if let Some(v) = args.spool_capacity      { config.spool_capacity = v; }
    if let Some(v) = args.yarn_lines          { config.yarn_lines = v; }
    if let Some(v) = args.obstacle_percentage { config.obstacle_percentage = v; }
    if let Some(v) = args.visible_stitches    { config.visible_stitches = v; }
    if let Some(v) = args.conveyor_capacity   { config.conveyor_capacity = v; }
    if let Some(v) = args.conveyor_percentage { config.conveyor_percentage = v; }
    if let Some(v) = args.scissors            { config.scissors = v; }
    if let Some(v) = args.tweezers            { config.tweezers = v; }
    if let Some(v) = args.balloons            { config.balloons = v; }
    if let Some(v) = args.scissors_spools     { config.scissors_spools = v; }
    if let Some(v) = args.balloon_count       { config.balloon_count = v; }
    if let Some(v) = args.max_solutions       { config.max_solutions = Some(v); }
    config
}

// ── Simple board renderer ─────────────────────────────────────────────────────

/// Render the board to stdout, highlighting `highlight` cell with a bright '*'.
fn render_board(
    stdout: &mut std::io::Stdout,
    engine: &GameEngine,
    x0: u16,
    y0: u16,
    highlight: Option<(usize, usize)>,
) -> std::io::Result<()> {
    let rows = engine.board.height as usize;
    let cols = engine.board.width as usize;

    // Top border
    stdout.queue(MoveTo(x0, y0))?;
    stdout.queue(Print('┌'))?;
    for c in 0..cols {
        stdout.queue(Print("──"))?;
        if c < cols - 1 { stdout.queue(Print('┬'))?; }
    }
    stdout.queue(Print('┐'))?;

    for r in 0..rows {
        // Content row
        stdout.queue(MoveTo(x0, y0 + 1 + (r as u16) * 2))?;
        stdout.queue(Print('│'))?;
        for c in 0..cols {
            let cell = &engine.board.board[r][c];
            let is_hl = highlight == Some((r, c));
            let is_cursor = r == engine.cursor_row as usize && c == engine.cursor_col as usize;
            if is_hl {
                stdout.queue(SetBackgroundColor(crossterm::style::Color::White))?;
                stdout.queue(SetForegroundColor(crossterm::style::Color::Black))?;
                stdout.queue(Print("**"))?;
                stdout.queue(SetAttribute(Attribute::Reset))?;
            } else if is_cursor {
                stdout.queue(SetAttribute(Attribute::Reverse))?;
                stdout.queue(Print(format!("{cell}{cell}")))?;
                stdout.queue(SetAttribute(Attribute::Reset))?;
            } else {
                match cell {
                    BoardEntity::Spool(c) | BoardEntity::KeySpool(c) => {
                        stdout.queue(Print("██".with(*c)))?;
                    }
                    BoardEntity::Obstacle => { stdout.queue(Print("▓▓".dark_grey()))?; }
                    BoardEntity::Void | BoardEntity::EmptyConveyor => {
                        stdout.queue(Print("  "))?;
                    }
                    BoardEntity::Conveyor(d) => {
                        stdout.queue(Print("◈◈".with(d.color)))?;
                    }
                }
            }
            stdout.queue(Print('│'))?;
        }

        // Horizontal separator / bottom border
        let line_y = y0 + 2 + (r as u16) * 2;
        stdout.queue(MoveTo(x0, line_y))?;
        if r < rows - 1 {
            stdout.queue(Print('├'))?;
            for c in 0..cols {
                stdout.queue(Print("──"))?;
                if c < cols - 1 { stdout.queue(Print('┼'))?; }
            }
            stdout.queue(Print('┤'))?;
        } else {
            stdout.queue(Print('└'))?;
            for c in 0..cols {
                stdout.queue(Print("──"))?;
                if c < cols - 1 { stdout.queue(Print('┴'))?; }
            }
            stdout.queue(Print('┘'))?;
        }
    }

    stdout.queue(ResetColor)?;
    Ok(())
}

/// Render a status line showing move count.
fn render_status(stdout: &mut std::io::Stdout, y: u16, step: usize, total: usize, highlight: Option<(usize, usize)>) -> std::io::Result<()> {
    stdout.queue(MoveTo(0, y))?;
    stdout.queue(Clear(ClearType::CurrentLine))?;
    let msg = if let Some((r, c)) = highlight {
        format!("Step {}/{}: picking ({},{})", step, total, r, c)
    } else {
        format!("Step {}/{}", step, total)
    };
    stdout.queue(Print(msg))?;
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    // Build or load engine
    let engine = if let Some(ref hash) = args.game {
        match load_engine(hash) {
            Ok(e) => e,
            Err(e) => { eprintln!("Error loading game: {e}"); std::process::exit(1); }
        }
    } else {
        let config = config_from_args(&args, base_config());
        GameEngine::new(&config)
    };

    // Find solution
    let solution = find_solution(
        &engine.board,
        &engine.yarn,
        engine.spool_capacity,
        engine.spool_limit,
    );

    let solution = match solution {
        Some(s) => s,
        None => {
            eprintln!("No solution found for this board.");
            std::process::exit(1);
        }
    };

    // Set up TUI
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide).unwrap();
    enable_raw_mode().unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            Show,
            LeaveAlternateScreen,
        );
        default_hook(info);
    }));

    let delay = Duration::from_millis(args.step_ms);
    let total = solution.len();

    // Replay: for each move, show the board with the upcoming pick highlighted, wait, then apply.
    let mut current = engine;
    for (step_idx, &(row, col)) in solution.iter().enumerate() {
        // Clear and render board with highlight
        stdout.queue(Clear(ClearType::All)).unwrap();
        render_board(&mut stdout, &current, 2, 1, Some((row, col))).unwrap();
        render_status(&mut stdout, 0, step_idx + 1, total, Some((row, col))).unwrap();
        stdout.flush().unwrap();

        // Wait step_ms (or keypress to skip)
        if poll(delay).unwrap_or(false) {
            if let Ok(Event::Key(_)) = read() {} // consume key, continue
        }

        // Apply the move: move cursor to cell and pick
        current.cursor_row = row as u16;
        current.cursor_col = col as u16;
        let _ = current.pick_up();
        current.process_all_active();
    }

    // Final state
    stdout.queue(Clear(ClearType::All)).unwrap();
    render_board(&mut stdout, &current, 2, 1, None).unwrap();
    let (_, term_h) = crossterm::terminal::size().unwrap_or((80, 24));
    stdout.queue(MoveTo(0, term_h / 2 + (current.board.height as u16) + 3)).unwrap();
    stdout.queue(Print(format!("Solved in {} moves — press any key", total)
        .bold().white())).unwrap();
    stdout.flush().unwrap();

    // Wait for any key
    enable_raw_mode().unwrap();
    loop {
        if let Ok(Event::Key(_)) = read() { break; }
    }

    execute!(stdout, Show, LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
