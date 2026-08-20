//! Native terminal entry point. Thin: owns disk I/O (settings/campaign
//! loading), the crossterm event loop, and terminal init/restore. All
//! actual game/menu/campaign state and logic lives in the portable
//! `loom_engine::shell::Shell<M2Game>` (Phase 4) -- this file's job is
//! purely to drive it.

use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::event::{poll, read, Event, KeyCode};

use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::input::{Key, KeyEvent};
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;
use loom_engine_term::TermSurface;

use crate::campaign::CampaignState;
use crate::config::Config;
use crate::game::M2Game;

/// Input-poll timeout, matching the original's poll rate.
const POLL_MS: u64 = 50;
/// Tick cadence, matching the original's separately-throttled tick_interval
/// (decoupled from the input poll rate, unlike knit/match3 which tick every
/// loop pass).
const TICK_INTERVAL: Duration = Duration::from_millis(200);

/// Run the m2 game from the game selector (default config, always shows menu).
/// merge2's original had no CLI-args-triggers-skip-menu entry point --
/// only this one.
pub fn run_from_menu() -> std::io::Result<()> {
    let user_settings = UserSettings::load("m2tui");
    let mut config = Config::default();
    config.scale = user_settings.scale;
    config.color_mode = user_settings.color_mode.clone();

    run_event_loop(config, user_settings)
}

fn run_event_loop(cli_config: Config, user_settings: UserSettings) -> std::io::Result<()> {
    let campaign_saves = CampaignSaves::<CampaignState>::load("m2tui");
    let endless_hs = EndlessHighScore::load("m2tui");

    let mut shell = Shell::new(
        M2Game,
        cli_config,
        user_settings,
        campaign_saves,
        endless_hs,
        Vec::new(), // no ad quotes wired yet (matches an already-disclosed gap)
        String::new(),
        false,
    );

    let mut stdout = stdout();
    loom_engine_term::init()?;

    render(&mut stdout, &shell)?;

    let mut last_tick = Instant::now();

    loop {
        if poll(Duration::from_millis(POLL_MS))? {
            if let Event::Key(event) = read()? {
                if let Some(key) = convert_key(event.code) {
                    shell.handle_key(key);
                }
            }
        }
        if last_tick.elapsed() >= TICK_INTERVAL {
            last_tick = Instant::now();
            shell.tick();
        }

        if shell.should_quit() {
            break;
        }
        render(&mut stdout, &shell)?;
    }

    shell.save_on_exit();
    loom_engine_term::restore()?;
    Ok(())
}

fn render(stdout: &mut Stdout, shell: &Shell<M2Game>) -> std::io::Result<()> {
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
