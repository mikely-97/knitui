//! Native terminal entry point. Thin: owns CLI parsing, disk I/O (settings/
//! campaign/high-score loading), the crossterm event loop, and terminal
//! init/restore. All actual game/menu/campaign state and logic lives in the
//! portable `loom_engine::shell::Shell<M3Game>` (Phase 4) -- this file's job
//! is purely to drive it.

use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode};

use clap::Parser;

use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::input::{Key, KeyEvent};
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;
use loom_engine_term::TermSurface;

use crate::campaign::CampaignState;
use crate::config::Config;
use crate::game::M3Game;

/// Poll timeout / tick cadence. 50ms matches the original hand-rolled
/// loop's input-poll timeout, which also doubled as the phase-pipeline
/// (bounce/fall/refill) animation rate. The original's win-celebration
/// sweep specifically slowed to 80ms/frame; `Shell`'s celebration duration
/// is a fixed tick count, so unifying on 50ms here makes that one animation
/// finish faster (~0.8s vs ~1.3s) -- a minor, cosmetic-only timing change.
const TICK_MS: u64 = 50;

/// Run the m3 game from the standalone binary (parses CLI args).
pub fn run_cli() -> std::io::Result<()> {
    let cli_config = Config::parse();
    let user_settings = UserSettings::load("m3tui");

    let mut game_config = cli_config.clone();
    game_config.scale = user_settings.scale;
    game_config.color_mode = user_settings.color_mode.clone();

    run_event_loop(cli_config, user_settings)
}

/// Run the m3 game from the game selector (default config, always shows menu).
pub fn run_from_menu() -> std::io::Result<()> {
    let user_settings = UserSettings::load("m3tui");
    let mut config = Config::parse_from::<[&str; 0], &str>([]);
    config.scale = user_settings.scale;
    config.color_mode = user_settings.color_mode.clone();
    run_event_loop(config, user_settings)
}

fn run_event_loop(
    cli_config: Config,
    user_settings: UserSettings,
) -> std::io::Result<()> {
    let campaign_saves = CampaignSaves::<CampaignState>::load("m3tui");
    let endless_hs = EndlessHighScore::load("m3tui");

    let mut shell = Shell::new(
        M3Game,
        cli_config,
        user_settings,
        campaign_saves,
        endless_hs,
        Vec::new(), // match3 has no watch-an-ad feature (no ad_reward_label/quotes)
        String::new(),
        false, // match3's original tui.rs always starts at the main menu
        Box::new(loom_engine::storage::FsStorage),
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
        // matches the original loop's phase-pipeline tick, which ran every
        // pass regardless of whether a key arrived that pass.
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

fn render(stdout: &mut Stdout, shell: &Shell<M3Game>) -> std::io::Result<()> {
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
