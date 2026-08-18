//! Native terminal entry point. Thin: owns CLI parsing, disk I/O (settings/
//! campaign/high-score/ad-quotes loading), the crossterm event loop, and
//! terminal init/restore. All actual game/menu/campaign state and logic
//! lives in the portable `loom_engine::shell::Shell<KnitGame>` (Phase 3) --
//! this file's job is purely to drive it.

use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode};

use clap::{CommandFactory, Parser, parser::ValueSource};

use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::input::{Key, KeyEvent};
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;
use loom_engine_term::TermSurface;

use crate::ad_content;
use crate::campaign::CampaignState;
use crate::config::Config;
use crate::game::KnitGame;

const GAME_ARGS: &[&str] = &[
    "board_height", "board_width", "color_number",
    "obstacle_percentage", "conveyor_percentage",
    "scissors", "tweezers", "balloons",
];

/// Poll timeout / tick cadence. 80ms matches the original hand-rolled
/// loop's celebration-animation frame pacing (16 frames * 80ms) -- using it
/// as the general poll timeout too (rather than a slower idle timeout with
/// a separate fast path for celebration) is a small, harmless unification.
const TICK_MS: u64 = 80;

/// Run knitui from the standalone binary (parses CLI args).
pub fn run_cli() -> std::io::Result<()> {
    let matches = Config::command().get_matches_from(std::env::args_os());
    let skip_menu = GAME_ARGS.iter().any(|name| {
        matches.value_source(name) == Some(ValueSource::CommandLine)
    });
    let mut cli_config = Config::parse();

    let user_settings = UserSettings::load("knitui");
    if matches.value_source("scale") != Some(ValueSource::CommandLine) {
        cli_config.scale = user_settings.scale;
    }
    if matches.value_source("color_mode") != Some(ValueSource::CommandLine) {
        cli_config.color_mode = user_settings.color_mode.clone();
    }

    run_event_loop(cli_config, user_settings, skip_menu)
}

/// Run knitui from the game selector (default config, always shows menu).
pub fn run_from_menu() -> std::io::Result<()> {
    let user_settings = UserSettings::load("knitui");
    let mut config = Config::parse_from::<[&str; 0], &str>([]);
    config.scale = user_settings.scale;
    config.color_mode = user_settings.color_mode.clone();
    run_event_loop(config, user_settings, false)
}

fn run_event_loop(
    cli_config: Config,
    user_settings: UserSettings,
    skip_menu: bool,
) -> std::io::Result<()> {
    let ad_quotes = ad_content::load_quotes(&cli_config.ad_file, "knitui");
    let campaign_saves = CampaignSaves::<CampaignState>::load("knitui");
    let endless_hs = EndlessHighScore::load("knitui");

    let mut shell = Shell::new(
        KnitGame,
        cli_config,
        user_settings,
        campaign_saves,
        endless_hs,
        ad_quotes,
        "FREE SCISSORS".to_string(),
        skip_menu,
    );

    let mut stdout = stdout();
    loom_engine_term::init()?;

    render(&mut stdout, &shell)?;

    loop {
        if poll(Duration::from_millis(TICK_MS))? {
            if let Event::Key(event) = read()? {
                if let Some(key) = convert_key(event.code) {
                    shell.handle_key(key);
                }
            }
        }
        // Unconditional every iteration (not just on poll timeout) --
        // matches the original loop's process_all_active() call, which ran
        // every pass regardless of whether a key arrived that pass.
        shell.tick();

        if shell.should_quit() {
            break;
        }
        render(&mut stdout, &shell)?;
    }

    shell.save_on_exit();
    loom_engine_term::restore()?;
    Ok(())
}

fn render(stdout: &mut Stdout, shell: &Shell<KnitGame>) -> std::io::Result<()> {
    let mut surface = TermSurface::begin(stdout)?;
    shell.render(&mut surface);
    surface.finish()
}

fn convert_key(code: KeyCode) -> Option<KeyEvent> {
    let key = match code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Enter => Key::Enter,
        KeyCode::Esc => Key::Esc,
        KeyCode::Char(c) => Key::Char(c),
        _ => return None,
    };
    Some(KeyEvent::new(key))
}
