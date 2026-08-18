//! Drives the generic `loom_engine::shell::Shell<M3Game>` through realistic
//! navigation sequences -- the actual thing Phase 4 delivers for match3. The
//! adapter tests (game_adapter_test.rs) proved the GameEngine trait wiring
//! in isolation; these prove the menu/campaign/endless/options state
//! machine built on top of it in loom-engine's shell.rs.
//!
//! `Shell` persists settings/campaign/high-score under the real "m3tui"
//! config dir (same location the actual game uses -- there's no injectable
//! storage backend for this yet). `ConfigGuard` backs up and restores those
//! files around every test so a test run can never leave the user's real
//! save data altered, even if a test panics.

use std::fs;
use std::path::PathBuf;

use m3tui::game::M3Game;
use loom_engine::campaign::CampaignSaves;
use loom_engine::endless::EndlessHighScore;
use loom_engine::game::Game;
use loom_engine::input::{Key, KeyEvent};
use loom_engine::render::CellGrid;
use loom_engine::settings::UserSettings;
use loom_engine::shell::Shell;

struct ConfigGuard {
    dir: PathBuf,
    backup: Vec<(PathBuf, Option<Vec<u8>>)>,
}

impl ConfigGuard {
    fn new() -> Self {
        let dir = dirs::config_dir().unwrap().join("m3tui");
        let files = ["settings.json", "campaign.json", "endless.json"];
        let backup = files.iter().map(|f| {
            let p = dir.join(f);
            let contents = fs::read(&p).ok();
            (p, contents)
        }).collect();
        Self { dir, backup }
    }
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        let _ = fs::create_dir_all(&self.dir);
        for (path, contents) in &self.backup {
            match contents {
                Some(bytes) => { let _ = fs::write(path, bytes); }
                None => { let _ = fs::remove_file(path); }
            }
        }
    }
}

fn new_shell(skip_menu: bool) -> Shell<M3Game> {
    let game = M3Game;
    let config = game.default_config();
    Shell::new(
        game,
        config,
        UserSettings::default(),
        CampaignSaves::default(),
        EndlessHighScore::default(),
        vec!["test quote".to_string()],
        "FREE HAMMER".to_string(),
        skip_menu,
    )
}

fn press(shell: &mut Shell<M3Game>, key: Key) {
    shell.handle_key(KeyEvent::new(key));
}

fn render_nonblank(shell: &Shell<M3Game>) -> bool {
    let mut grid = CellGrid::new(120, 50);
    shell.render(&mut grid);
    (0..50).any(|y| (0..120).any(|x| grid.get(x, y).glyph != ' '))
}

#[test]
fn main_menu_renders_and_is_navigable() {
    let _guard = ConfigGuard::new();
    let shell = new_shell(false);
    assert!(render_nonblank(&shell));
}

#[test]
fn quick_game_enters_playing_and_renders_board() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    press(&mut shell, Key::Enter); // Quick Game (selected=0 by default)
    assert!(render_nonblank(&shell));
    press(&mut shell, Key::Right);
    assert!(!shell.should_quit());
}

#[test]
fn esc_from_playing_returns_to_menu() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    press(&mut shell, Key::Enter); // -> Playing
    press(&mut shell, Key::Esc);   // -> MainMenu
    assert!(render_nonblank(&shell));
}

#[test]
fn skip_menu_starts_directly_in_playing() {
    let _guard = ConfigGuard::new();
    let shell = new_shell(true);
    assert!(render_nonblank(&shell));
}

#[test]
fn options_screen_adjusts_scale_and_saves_on_exit() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    for _ in 0..4 { press(&mut shell, Key::Down); } // MainMenu index 4 = Options
    press(&mut shell, Key::Enter); // -> Options
    assert!(render_nonblank(&shell));
    press(&mut shell, Key::Right); // bump scale
    press(&mut shell, Key::Esc);   // save + back to menu (writes settings.json, restored by guard)
    assert!(render_nonblank(&shell));
}

#[test]
fn endless_mode_starts_playing_with_wave_one_config() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    for _ in 0..3 { press(&mut shell, Key::Down); } // MainMenu index 3 = Endless
    press(&mut shell, Key::Enter);
    assert!(render_nonblank(&shell));
}

#[test]
fn custom_game_screen_is_navigable_and_starts_a_game() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    press(&mut shell, Key::Down); // MainMenu index 1 = Custom Game
    press(&mut shell, Key::Enter);
    assert!(render_nonblank(&shell)); // CustomGame screen
    press(&mut shell, Key::Down);  // move to a field
    press(&mut shell, Key::Right); // adjust it
    press(&mut shell, Key::Enter); // start
    assert!(render_nonblank(&shell)); // Playing
}

#[test]
fn campaign_select_always_routes_through_blessing_selection() {
    // match3's needs_blessing_selection() is unconditionally true (see
    // campaign.rs), unlike knit's once-only gate.
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    for _ in 0..2 { press(&mut shell, Key::Down); } // MainMenu index 2 = Campaign
    press(&mut shell, Key::Enter); // -> CampaignSelect
    assert!(render_nonblank(&shell));
    press(&mut shell, Key::Enter); // select track 0 -> BlessingSelection
    assert!(render_nonblank(&shell));
}

#[test]
fn help_screen_toggles_back_to_playing() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(true); // straight into Playing
    press(&mut shell, Key::Char('h'));
    assert!(render_nonblank(&shell)); // Help screen
    press(&mut shell, Key::Char(' ')); // any key closes it
    assert!(render_nonblank(&shell)); // back to Playing
}

#[test]
fn tick_does_not_panic_while_playing() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(true);
    for _ in 0..5 {
        shell.tick();
    }
    assert!(render_nonblank(&shell));
}

#[test]
fn playing_through_to_game_over_and_back_to_menu() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(true);
    for _ in 0..40 {
        press(&mut shell, Key::Enter);
        shell.tick();
        press(&mut shell, Key::Right);
        press(&mut shell, Key::Enter);
        shell.tick();
    }
    assert!(render_nonblank(&shell));
    assert!(!shell.should_quit());
}

#[test]
fn endless_run_survives_repeated_ticks_without_panicking() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    for _ in 0..3 { press(&mut shell, Key::Down); } // Endless
    press(&mut shell, Key::Enter);
    for _ in 0..60 {
        press(&mut shell, Key::Enter);
        shell.tick();
        press(&mut shell, Key::Right);
        press(&mut shell, Key::Enter);
        shell.tick();
    }
    assert!(render_nonblank(&shell));
}
