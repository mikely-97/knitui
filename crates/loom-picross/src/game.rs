use loom_engine::render::{Color, Style, Surface};
use loom_engine::game::{
    Action, Game, GameId, GameEngine as GameEngineTrait, GameStatus as EngineGameStatus,
    MenuItem, RenderArea,
};
use loom_engine::input::{Key, KeyEvent};

use crate::campaign::{TRACK_NAMES, TRACK_COUNT, levels_for_track, level_count, PicrossCampaignEntry};
use crate::config::Config;
use crate::engine::{GameEngine as PxEngine, GameStatus as PxStatus};
use crate::puzzle::Puzzle;
use crate::renderer;

pub struct PicrossGame;

impl Game for PicrossGame {
    type Config = Config;
    type CampaignEntry = PicrossCampaignEntry;

    fn id(&self) -> GameId { GameId::Picross }
    fn name(&self) -> &'static str { "Picross" }
    fn config_dir(&self) -> &'static str { "picross" }

    fn create_engine(&self, _config: &Config, _palette: &[Color]) -> Box<dyn GameEngineTrait> {
        // Unreachable in practice: main_menu_items() offers only Campaign
        // and Quit, so Shell never routes through Quick/Custom/Endless for
        // picross. Kept as a safe, harmless fallback (first track-0 puzzle)
        // rather than panicking, in case that ever changes.
        let puzzle = levels_for_track(0).into_iter().next()
            .expect("track 0 has at least one puzzle");
        Box::new(PicrossEngineAdapter::new(puzzle))
    }

    fn main_menu_items(&self) -> &'static [MenuItem] {
        // Picross has no Quick/Custom/Endless concept -- puzzles are
        // curated, not generated from parameters, and there's no
        // continuous "endless" mode.
        &[MenuItem::Campaign, MenuItem::Quit]
    }

    fn default_config(&self) -> Config {
        Config::default()
    }

    fn track_names(&self) -> &'static [&'static str] { TRACK_NAMES }
    fn track_count(&self) -> usize { TRACK_COUNT }

    fn level_count(&self, track: usize) -> usize {
        level_count(track)
    }

    fn level_config(&self, _track: usize, _level: usize, base: &Config) -> Config {
        base.clone()
    }

    fn level_intro_lines(&self, track: usize, level: usize) -> Vec<String> {
        let levels = levels_for_track(track);
        if let Some(p) = levels.into_iter().nth(level) {
            vec![
                format!("{} — Puzzle {}/{}", TRACK_NAMES[track], level + 1, level_count(track)),
                format!("\"{}\" ({}×{})", p.name, p.rows, p.cols),
            ]
        } else {
            vec!["Unknown puzzle".to_string()]
        }
    }

    fn new_campaign_entry(&self, track: usize) -> PicrossCampaignEntry {
        PicrossCampaignEntry::new(track)
    }

    fn campaign_config(&self, entry: &PicrossCampaignEntry, base: &Config) -> Config {
        let puzzle = current_puzzle(entry);
        Config { board_rows: puzzle.rows, board_cols: puzzle.cols, ..base.clone() }
    }

    fn create_campaign_engine(
        &self, entry: &PicrossCampaignEntry, _config: &Config, _palette: &[Color],
    ) -> Box<dyn GameEngineTrait> {
        Box::new(PicrossEngineAdapter::new(current_puzzle(entry)))
    }

    fn complete_campaign_level(&self, entry: &mut PicrossCampaignEntry) -> bool {
        entry.complete_level()
    }

    fn endless_wave_config(&self, _wave: u32, base: &Config) -> Config {
        base.clone()
    }

    fn help_lines(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("Arrow keys", "Move cursor"),
            ("Space / Enter", "Fill cell"),
            ("X", "Cross out cell"),
            ("Q / Esc", "Quit to menu"),
        ]
    }

    fn presets(&self) -> Vec<(&'static str, Config)> {
        vec![]
    }
}

/// Look up the specific `Puzzle` a campaign entry currently points at.
/// Picross's campaign is sequential-only by design decision (2026-08-21):
/// the original `tui.rs` let players freely jump to any puzzle within a
/// track via a dedicated `LevelSelect` screen, which `Shell`'s generic
/// campaign flow has no equivalent for (it only ever plays
/// `entry.current_level()`, advancing on win). That free-jump ability is a
/// real, permanent loss from this migration -- not a bug, not preserved.
fn current_puzzle(entry: &PicrossCampaignEntry) -> Puzzle {
    levels_for_track(entry.track_idx).into_iter().nth(entry.current_level)
        .expect("PicrossCampaignEntry::current_level always in range (complete_level() clamps)")
}

/// Adapts loom-picross's concrete `GameEngine` to the portable
/// `loom_engine::game::GameEngine` trait. Picross's `CampaignEntry` is pure
/// bookkeeping (track/level index, no live world state), so unlike merge2
/// this needs no `sync_campaign_entry` override.
struct PicrossEngineAdapter {
    engine: PxEngine,
}

impl PicrossEngineAdapter {
    fn new(puzzle: Puzzle) -> Self {
        Self { engine: PxEngine::new(puzzle) }
    }
}

impl GameEngineTrait for PicrossEngineAdapter {
    fn handle_key(&mut self, key: KeyEvent) -> Action {
        match key.key {
            Key::Up => { self.engine.move_cursor(-1, 0); Action::Redraw }
            Key::Down => { self.engine.move_cursor(1, 0); Action::Redraw }
            Key::Left => { self.engine.move_cursor(0, -1); Action::Redraw }
            Key::Right => { self.engine.move_cursor(0, 1); Action::Redraw }
            Key::Enter | Key::Char(' ') => { self.engine.toggle_fill(); Action::Redraw }
            Key::Char('x') | Key::Char('X') => { self.engine.toggle_cross(); Action::Redraw }
            Key::Char('h') | Key::Char('H') => Action::ShowHelp,
            Key::Char('q') | Key::Char('Q') | Key::Esc => Action::QuitToMenu,
            _ => Action::None,
        }
    }

    fn tick(&mut self) -> bool { false }

    fn status(&self) -> EngineGameStatus {
        match self.engine.status {
            PxStatus::Playing => EngineGameStatus::Playing,
            PxStatus::Won => EngineGameStatus::Won { score: None },
            PxStatus::TooManyMistakes => EngineGameStatus::Lost { reason: "Too many mistakes".to_string() },
        }
    }

    fn render(&self, surface: &mut dyn Surface, _area: RenderArea) {
        renderer::render_inner(surface, &self.engine, 2, 2);
    }

    fn render_keybar(&self, _surface: &mut dyn Surface, _y: u16) {
        // No-op: render() above already draws the key bar as part of the
        // same pass (render_inner's status+keybar lines).
    }

    fn render_game_over_overlay(&self, surface: &mut dyn Surface, _status: &EngineGameStatus, overlay_msg: Option<&str>) {
        // render_inner's own status line already shows a win/loss message;
        // only draw something extra when Shell wants to override it with
        // campaign-progress text (e.g. "[3/10] You won! ...").
        if let Some(msg) = overlay_msg {
            surface.print(0, 0, msg, Style::default());
        }
    }

    fn render_help(&self, surface: &mut dyn Surface) {
        let lines: &[(&str, &str)] = &[
            ("Arrow keys", "Move cursor"),
            ("Space / Enter", "Fill cell"),
            ("X", "Cross out cell"),
            ("Q / Esc", "Quit to menu"),
        ];
        let (tw, th) = surface.size();
        let box_w = 40u16;
        let bx = (tw / 2).saturating_sub(box_w / 2);
        let by = (th / 2).saturating_sub(lines.len() as u16 / 2 + 2);
        let inner = box_w as usize - 2;

        surface.print(bx, by, &format!("╔{}╗", "═".repeat(inner)), Style::default());
        surface.print(bx, by + 1, &format!("║{:^w$}║", "PICROSS HELP", w = inner), Style::default());
        surface.print(bx, by + 2, &format!("╠{}╣", "═".repeat(inner)), Style::default());
        for (i, (key, desc)) in lines.iter().enumerate() {
            let y = by + 3 + i as u16;
            let line = format!(" {:<16}{}", key, desc);
            surface.print(bx, y, &format!("║{:<w$}║", line, w = inner), Style::default());
        }
        surface.print(bx, by + 3 + lines.len() as u16, &format!("╚{}╝", "═".repeat(inner)), Style::default());
    }

    fn score(&self) -> Option<u32> { None }

    fn scale(&self) -> u16 { 1 }
    fn set_scale(&mut self, _scale: u16) {
        // No-op: picross's renderer uses a fixed cell size, not a
        // config-driven scale factor.
    }

    fn board_dims(&self) -> (u16, u16) { (self.engine.puzzle.rows as u16, self.engine.puzzle.cols as u16) }

    fn as_any(&self) -> &dyn std::any::Any { self }
}
