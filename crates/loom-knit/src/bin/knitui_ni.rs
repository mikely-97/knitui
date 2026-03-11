use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use knitui::engine::{GameEngine, GameStatus};
use knitui::config::Config;
use knitui::board_entity::Direction;
use knitui::campaign::CampaignState;
use knitui::campaign_levels::{self, TRACK_COUNT};
use knitui::endless::EndlessState;

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
    #[arg(long)] ad_limit:            Option<u16>,

    // Campaign-mode game creation
    #[arg(long, help = "Create a game from a campaign level")]
    campaign: bool,
    #[arg(long, help = "Campaign track index (0-based)")]
    track: Option<usize>,
    #[arg(long, help = "Campaign level index (0-based)")]
    level: Option<usize>,

    // Endless-mode game creation
    #[arg(long, help = "Create a game for an endless-mode wave number")]
    endless_wave: Option<usize>,
}

#[derive(Subcommand)]
enum NiCommand {
    /// Move the cursor
    Move { direction: NiDirection },
    /// Pick up the spool under the cursor
    Pick,
    /// Process all held spools one yarn step each
    Process,
    /// Use scissors bonus
    Scissors,
    /// Use tweezers bonus (enters tweezers mode)
    Tweezers,
    /// Cancel tweezers mode without consuming the bonus
    CancelTweezers,
    /// Use balloons bonus
    Balloons,
    /// Watch a fake ad (grants +1 scissors, no timer)
    Ad,
    /// List all campaign tracks and levels as JSON
    ListCampaign,
    /// Describe the config for a given endless-mode wave
    DescribeWave {
        #[arg(help = "Wave number (1-based)")]
        wave: usize,
    },
    /// Generate multiple boards and output as NDJSON (one JSON object per line)
    BatchGenerate {
        #[arg(long, default_value_t = 100, help = "Number of boards to generate")]
        count: usize,
    },
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

// ── Config builder ───────────────────────────────────────────────────────────

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
    }
}

/// Build a Config from CLI args, optionally layering on top of a provided base.
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

/// Resolve the Config for game creation, handling campaign / endless / raw modes.
fn resolve_config(args: &Args) -> Config {
    if args.campaign {
        let track = args.track.unwrap_or(0);
        let level = args.level.unwrap_or(0);
        let levels = campaign_levels::levels_for_track(track);
        if level >= levels.len() {
            err_response("bad_level", &format!(
                "track {track} has {} levels, requested level {level}", levels.len()
            ));
            std::process::exit(1);
        }
        // Use CampaignState to build the config (with banked bonuses from CLI args)
        let mut state = CampaignState::new(track);
        state.current_level = level;
        if let Some(s) = args.scissors { state.banked_scissors = s; }
        if let Some(t) = args.tweezers { state.banked_tweezers = t; }
        if let Some(b) = args.balloons { state.banked_balloons = b; }
        let config = state.to_config(&base_config());
        // Allow remaining CLI overrides (e.g. max_solutions)
        config_from_args_limited(args, config)
    } else if let Some(wave) = args.endless_wave {
        let mut state = EndlessState::new();
        for _ in 1..wave {
            state.advance();
        }
        // Override wave to target
        state.wave = wave;
        let mut config = state.to_config(&base_config());
        // Force bonuses to 0 for the base endless config (banked comes from advance)
        // Allow CLI overrides
        config = config_from_args(args, config);
        config
    } else {
        config_from_args(args, base_config())
    }
}

/// Limited config overlay: only applies non-gameplay overrides (max_solutions, color_mode)
/// so campaign/endless core params are preserved.
fn config_from_args_limited(args: &Args, mut config: Config) -> Config {
    if let Some(ref v) = args.color_mode  { config.color_mode = v.clone(); }
    if let Some(v) = args.max_solutions   { config.max_solutions = Some(v); }
    config
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

// ── Standalone subcommand handlers ───────────────────────────────────────────

fn handle_list_campaign() {
    let mut tracks = Vec::new();
    for track_idx in 0..TRACK_COUNT {
        let levels = campaign_levels::levels_for_track(track_idx);
        let name = campaign_levels::TRACK_NAMES[track_idx];
        let level_data: Vec<serde_json::Value> = levels.iter().map(|l| {
            serde_json::json!({
                "board_height": l.board_height,
                "board_width": l.board_width,
                "color_number": l.color_number,
                "obstacle_percentage": l.obstacle_percentage,
                "conveyor_percentage": l.conveyor_percentage,
                "scissors": l.scissors,
                "tweezers": l.tweezers,
                "balloons": l.balloons,
                "ad_limit": l.ad_limit,
                "reward_scissors": l.reward_scissors,
                "reward_tweezers": l.reward_tweezers,
                "reward_balloons": l.reward_balloons,
            })
        }).collect();
        tracks.push(serde_json::json!({
            "track_index": track_idx,
            "name": name,
            "level_count": levels.len(),
            "levels": level_data,
        }));
    }
    println!("{}", serde_json::to_string_pretty(&tracks).unwrap());
}

fn handle_describe_wave(wave: usize) {
    let mut state = EndlessState::new();
    for _ in 1..wave {
        state.advance();
    }
    state.wave = wave;
    let config = state.to_config(&base_config());
    let desc = serde_json::json!({
        "wave": wave,
        "board_height": config.board_height,
        "board_width": config.board_width,
        "color_number": config.color_number,
        "obstacle_percentage": config.obstacle_percentage,
        "conveyor_percentage": config.conveyor_percentage,
        "scissors": config.scissors,
        "tweezers": config.tweezers,
        "balloons": config.balloons,
    });
    println!("{}", serde_json::to_string_pretty(&desc).unwrap());
}

fn handle_batch_generate(args: &Args, count: usize) {
    let config = resolve_config(args);
    for _ in 0..count {
        let engine = GameEngine::new(&config);
        let state_json = engine.to_json();
        let mut state_val: serde_json::Value = serde_json::from_str(&state_json).unwrap();
        // Inject generation_attempts at top level for easy access
        if let serde_json::Value::Object(ref mut map) = state_val {
            map.insert("generation_attempts".into(),
                       serde_json::Value::Number(engine.generation_attempts.into()));
        }
        let game_status = match engine.status() {
            GameStatus::Playing => "playing",
            GameStatus::Won     => "won",
            GameStatus::Stuck   => "stuck",
        };
        let line = serde_json::json!({
            "game_status": game_status,
            "config": {
                "board_height": config.board_height,
                "board_width": config.board_width,
                "color_number": config.color_number,
                "obstacle_percentage": config.obstacle_percentage,
                "conveyor_percentage": config.conveyor_percentage,
                "spool_capacity": config.spool_capacity,
                "spool_limit": config.spool_limit,
            },
            "state": state_val,
        });
        println!("{}", serde_json::to_string(&line).unwrap());
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    // Handle standalone subcommands that don't need a game
    match &args.command {
        Some(NiCommand::ListCampaign) => {
            handle_list_campaign();
            return;
        }
        Some(NiCommand::DescribeWave { wave }) => {
            handle_describe_wave(*wave);
            return;
        }
        Some(NiCommand::BatchGenerate { count }) => {
            handle_batch_generate(&args, *count);
            return;
        }
        _ => {}
    }

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
                                ("not_selectable", "spool is not exposed (not in top row and no Void neighbour)"),
                            knitui::engine::PickError::NotASpool =>
                                ("not_a_spool", "cell under cursor is not a spool"),
                            knitui::engine::PickError::ActiveFull =>
                                ("active_full", "held spool limit reached"),
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
                Some(NiCommand::CancelTweezers) => {
                    engine.cancel_tweezers();
                }
                Some(NiCommand::Balloons) => {
                    if let Err(e) = engine.use_balloons() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
                Some(NiCommand::Ad) => {
                    if !engine.can_watch_ad() {
                        err_response("ad_limit_reached", "ad limit reached for this game");
                        return;
                    }
                    engine.watch_ad();
                }
                Some(NiCommand::ListCampaign)
                | Some(NiCommand::DescribeWave { .. })
                | Some(NiCommand::BatchGenerate { .. }) => {
                    unreachable!("handled above");
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
            let config = resolve_config(&args);
            let mut engine = GameEngine::new(&config);

            // Apply ad_limit if specified (campaign levels set this)
            if let Some(limit) = args.ad_limit {
                engine.set_ad_limit(limit);
            } else if args.campaign {
                // Auto-set ad_limit from campaign level definition
                let track = args.track.unwrap_or(0);
                let level = args.level.unwrap_or(0);
                let levels = campaign_levels::levels_for_track(track);
                if level < levels.len() {
                    engine.set_ad_limit(levels[level].ad_limit);
                }
            }

            let hash = GameEngine::generate_hash();

            if let Err(e) = save_engine(&hash, &engine) {
                err_response("save_failed", &e);
                return;
            }
            ok_response(&hash, &engine);
        }
    }
}
