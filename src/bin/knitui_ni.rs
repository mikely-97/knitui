use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use knitui::engine::{GameEngine, GameStatus};
use knitui::config::Config;
use knitui::board_entity::Direction;

// ── CLI types ─────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "knitui-ni", about = "Non-interactive knitui driver")]
struct Args {
    /// Operate on an existing game (load it, run COMMAND, save it)
    #[arg(long)]
    game: Option<String>,

    #[command(subcommand)]
    command: Option<NiCommand>,

    // Game-creation options (used only when --game is absent)
    #[arg(long)] board_height:         Option<u16>,
    #[arg(long)] board_width:          Option<u16>,
    #[arg(long)] color_number:         Option<u16>,
    #[arg(long)] color_mode:           Option<String>,
    #[arg(long)] active_threads_limit: Option<usize>,
    #[arg(long)] knit_volume:          Option<u16>,
    #[arg(long)] yarn_lines:           Option<u16>,
    #[arg(long)] obstacle_percentage:  Option<u16>,
    #[arg(long)] visible_patches:      Option<u16>,
    #[arg(long)] generator_capacity:   Option<u16>,
    #[arg(long)] scissors:          Option<u16>,
    #[arg(long)] tweezers:          Option<u16>,
    #[arg(long)] balloons:          Option<u16>,
    #[arg(long)] scissors_threads:  Option<u16>,
    #[arg(long)] balloon_count:     Option<u16>,
}

#[derive(Subcommand)]
enum NiCommand {
    /// Move the cursor
    Move { direction: NiDirection },
    /// Pick up the thread under the cursor
    Pick,
    /// Process all active threads one yarn step each
    Process,
    /// Use scissors bonus
    Scissors,
    /// Use tweezers bonus (enters tweezers mode)
    Tweezers,
    /// Use balloons bonus
    Balloons,
}

#[derive(Clone, ValueEnum)]
enum NiDirection { Up, Down, Left, Right }

impl From<NiDirection> for Direction {
    fn from(d: NiDirection) -> Self {
        match d {
            NiDirection::Up    => Direction::Up,
            NiDirection::Down  => Direction::Down,
            NiDirection::Left  => Direction::Left,
            NiDirection::Right => Direction::Right,
        }
    }
}

// ── Persistence ───────────────────────────────────────────────────────────────

fn game_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("knitui")
}

fn game_path(hash: &str) -> PathBuf {
    game_dir().join(format!("{hash}.json"))
}

fn load_engine(hash: &str) -> Result<GameEngine, String> {
    let path = game_path(hash);
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    GameEngine::from_json(&json)
}

fn save_engine(hash: &str, engine: &GameEngine) -> Result<(), String> {
    let dir = game_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let path = game_path(hash);
    std::fs::write(&path, engine.to_json())
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

// ── Output helpers ────────────────────────────────────────────────────────────

fn ok_response(hash: &str, engine: &GameEngine) {
    let state_json = engine.to_json();
    let state_val: serde_json::Value = serde_json::from_str(&state_json).unwrap();
    let game_status = match engine.status() {
        GameStatus::Playing => "playing",
        GameStatus::Won     => "won",
        GameStatus::Stuck   => "stuck",
    };
    let response = serde_json::json!({
        "status": "ok",
        "game": hash,
        "won": engine.is_won(),
        "game_status": game_status,
        "state": state_val,
    });
    println!("{}", serde_json::to_string(&response).unwrap());
}

fn err_response(code: &str, message: &str) {
    let response = serde_json::json!({
        "status": "error",
        "code": code,
        "message": message,
    });
    eprintln!("{}", serde_json::to_string(&response).unwrap());
    std::process::exit(1);
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    match args.game {
        // ── Execute a command on an existing game ──────────────────────────
        Some(hash) => {
            let mut engine = match load_engine(&hash) {
                Ok(e) => e,
                Err(e) => { err_response("load_failed", &e); return; }
            };

            match args.command {
                Some(NiCommand::Move { direction }) => {
                    if let Err(_) = engine.move_cursor(direction.into()) {
                        err_response("out_of_bounds", "cursor is already at the board edge");
                        return;
                    }
                }
                Some(NiCommand::Pick) => {
                    if let Err(e) = engine.pick_up() {
                        let (code, msg) = match e {
                            knitui::engine::PickError::NotSelectable =>
                                ("not_selectable", "thread is not exposed (not in top row and no Void neighbour)"),
                            knitui::engine::PickError::NotAThread =>
                                ("not_a_thread", "cell under cursor is not a thread"),
                            knitui::engine::PickError::ActiveFull =>
                                ("active_full", "active thread limit reached"),
                        };
                        err_response(code, msg);
                        return;
                    }
                }
                Some(NiCommand::Process) => {
                    engine.process_all_active();
                }
                Some(NiCommand::Scissors) => {
                    if let Err(e) = engine.use_scissors() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
                Some(NiCommand::Tweezers) => {
                    if let Err(e) = engine.use_tweezers() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
                Some(NiCommand::Balloons) => {
                    if let Err(e) = engine.use_balloons() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
                None => {
                    err_response("no_command", "provide a subcommand: move, pick, or process");
                    return;
                }
            }

            if let Err(e) = save_engine(&hash, &engine) {
                err_response("save_failed", &e);
                return;
            }
            ok_response(&hash, &engine);
        }

        // ── Create a new game ──────────────────────────────────────────────
        None => {
            let config = Config {
                board_height:         args.board_height.unwrap_or(6),
                board_width:          args.board_width.unwrap_or(6),
                color_number:         args.color_number.unwrap_or(6),
                color_mode:           args.color_mode.unwrap_or_else(|| "dark".into()),
                active_threads_limit: args.active_threads_limit.unwrap_or(7),
                knit_volume:          args.knit_volume.unwrap_or(3),
                yarn_lines:           args.yarn_lines.unwrap_or(4),
                obstacle_percentage:  args.obstacle_percentage.unwrap_or(5),
                visible_patches:      args.visible_patches.unwrap_or(6),
                generator_capacity:   args.generator_capacity.unwrap_or(3),
                layout:               "auto".into(),
                scale:                1,
                scissors:             args.scissors.unwrap_or(0),
                tweezers:             args.tweezers.unwrap_or(0),
                balloons:             args.balloons.unwrap_or(0),
                scissors_threads:     args.scissors_threads.unwrap_or(1),
                balloon_count:        args.balloon_count.unwrap_or(2),
            };

            let engine = GameEngine::new(&config);
            let hash = GameEngine::generate_hash();

            if let Err(e) = save_engine(&hash, &engine) {
                err_response("save_failed", &e);
                return;
            }
            ok_response(&hash, &engine);
        }
    }
}
