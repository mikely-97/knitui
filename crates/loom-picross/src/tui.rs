//! Native terminal entry point. Thin: owns disk I/O (settings/campaign
//! loading), the crossterm event loop, and terminal init/restore. All
//! actual game/menu/campaign state and logic lives in the portable
//! `loom_engine::shell::Shell<PicrossGame>` (Phase 4) -- this file's job is
//! purely to drive it.

use std::io::{stdout, Stdout};
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode};

use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::input::{Key, KeyEvent};
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;
use loom_engine_term::TermSurface;

use crate::campaign::PicrossCampaignEntry;
use crate::config::Config;
use crate::game::PicrossGame;

/// Poll timeout / tick cadence, matching the original's poll rate.
const TICK_MS: u64 = 100;

/// Run picross from the game selector (default config, always shows menu).
/// picross's original had no CLI-args-triggers-skip-menu entry point --
/// only this one.
pub fn run_from_menu() -> std::io::Result<()> {
    let user_settings = UserSettings::load("picross");
    let mut config = Config::default();
    config.scale = user_settings.scale;
    config.color_mode = user_settings.color_mode.clone();

    let campaign_saves = CampaignSaves::<PicrossCampaignEntry>::load("picross");
    let endless_hs = EndlessHighScore::load("picross");

    let mut shell = Shell::new(
        PicrossGame,
        config,
        user_settings,
        campaign_saves,
        endless_hs,
        Vec::new(),
        String::new(),
        false,
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

fn render(stdout: &mut Stdout, shell: &Shell<PicrossGame>) -> std::io::Result<()> {
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
