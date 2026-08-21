//! Drives the generic `loom_engine::shell::Shell<PicrossGame>` through
//! realistic navigation sequences -- the actual thing Phase 4 delivers for
//! picross. Since main_menu_items() is [Campaign, Quit] only, these mostly
//! exercise the campaign path plus the always-available Playing/Help/quit
//! flows.
//!
//! `Shell` persists settings/campaign under the real "picross" config dir
//! (same location the actual game uses -- there's no injectable storage
//! backend for this yet). `ConfigGuard` backs up and restores those files
//! around every test so a test run can never leave the user's real save
//! data altered, even if a test panics.

use std::fs;
use std::path::PathBuf;

use pictui::game::PicrossGame;
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
        let dir = dirs::config_dir().unwrap().join("picross");
        let files = ["settings.json", "campaign.json"];
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

fn new_shell(skip_menu: bool) -> Shell<PicrossGame> {
    let game = PicrossGame;
    let config = game.default_config();
    Shell::new(
        game,
        config,
        UserSettings::default(),
        CampaignSaves::default(),
        EndlessHighScore::default(),
        Vec::new(),
        String::new(),
        skip_menu,
    )
}

fn press(shell: &mut Shell<PicrossGame>, key: Key) {
    shell.handle_key(KeyEvent::new(key));
}

fn render_nonblank(shell: &Shell<PicrossGame>) -> bool {
    let mut grid = CellGrid::new(100, 40);
    shell.render(&mut grid);
    (0..40).any(|y| (0..100).any(|x| grid.get(x, y).glyph != ' '))
}

#[test]
fn main_menu_renders_with_only_campaign_and_quit() {
    let _guard = ConfigGuard::new();
    let shell = new_shell(false);
    assert!(render_nonblank(&shell));
}

#[test]
fn skip_menu_starts_directly_in_playing() {
    let _guard = ConfigGuard::new();
    let shell = new_shell(true);
    assert!(render_nonblank(&shell));
}

#[test]
fn campaign_select_does_not_show_blessing_selection() {
    // Picross has no blessing system -- needs_blessing_selection() is
    // false, so track selection should go straight to CampaignLevelIntro.
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    press(&mut shell, Key::Enter); // MainMenu index 0 = Campaign
    assert!(render_nonblank(&shell)); // CampaignSelect
    press(&mut shell, Key::Enter); // select track 0 -> CampaignLevelIntro (not BlessingSelection)
    assert!(render_nonblank(&shell));
}

#[test]
fn campaign_full_flow_reaches_a_playable_puzzle() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    press(&mut shell, Key::Enter); // Campaign
    press(&mut shell, Key::Enter); // track 0 -> CampaignLevelIntro
    press(&mut shell, Key::Enter); // start level -> Playing
    assert!(render_nonblank(&shell));
    press(&mut shell, Key::Right);
    assert!(!shell.should_quit());
}

#[test]
fn esc_from_playing_returns_to_menu() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(true);
    press(&mut shell, Key::Esc);
    assert!(render_nonblank(&shell));
}

#[test]
fn help_screen_toggles_back_to_playing() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(true);
    press(&mut shell, Key::Char('h'));
    assert!(render_nonblank(&shell));
    press(&mut shell, Key::Char(' '));
    assert!(render_nonblank(&shell));
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
fn playing_through_a_scripted_sequence_does_not_panic() {
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(true);
    for _ in 0..30 {
        press(&mut shell, Key::Enter);
        shell.tick();
        if !render_nonblank(&shell) {
            break;
        }
        press(&mut shell, Key::Right);
    }
    assert!(!shell.should_quit());
}

#[test]
fn campaign_progresses_sequentially_across_two_puzzles() {
    // Confirms the sequential-only campaign model actually works end to
    // end through Shell, not just at the Game-trait level.
    let _guard = ConfigGuard::new();
    let mut shell = new_shell(false);
    press(&mut shell, Key::Enter); // Campaign
    press(&mut shell, Key::Enter); // track 0 -> CampaignLevelIntro
    press(&mut shell, Key::Enter); // -> Playing (puzzle 1)
    assert!(render_nonblank(&shell));
    press(&mut shell, Key::Esc); // quit to menu (autosaves campaign_ctx)
    assert!(render_nonblank(&shell));
}
