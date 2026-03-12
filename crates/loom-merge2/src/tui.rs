use std::io::{Write, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode,
               Clear, ClearType},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode},
};
use rand::prelude::*;

use loom_engine::ad_content;

use crate::ad::{self, AdReward};
use crate::blessings::{self, ALL_BLESSINGS};
use crate::campaign::{CampaignSaves, CampaignState};
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT};
use crate::config::Config;
use crate::endless::{load_high_score, save_high_score, new_endless_engine, apply_scaling};
use crate::engine::{GameEngine, GameStatus};
use crate::preset::PRESETS;
use crate::renderer::{self, LayoutGeometry};
use crate::settings;

use clap::Parser;

// ── TuiState ──────────────────────────────────────────────────────────────

enum TuiState {
    MainMenu { selected: usize, flash: Option<String> },
    Playing { label: String },
    InventoryMode { label: String, selected_slot: usize },
    PlacingItem { label: String, piece: crate::item::Piece },
    CustomGame { preset_idx: usize, selected_field: usize, config: Config },
    CampaignSelect { selected: usize },
    BlessingSelection { cursor: usize, chosen: Vec<usize> },
    CampaignLevelIntro,
    GameOver(GameStatus),
    Help,
    Options { selected: usize },
    WatchingAd { started_at: Instant, quote: String, reward: AdReward },
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn level_intro_lines(ctx: &CampaignState) -> Vec<String> {
    let track_name = TRACK_NAMES.get(ctx.track_idx).copied().unwrap_or("Unknown");
    let mission = ctx.current_mission + 1;
    let total   = ctx.total_missions();
    let thawed  = ctx.cells_thawed;
    vec![
        format!("{} — Mission {}/{}", track_name, mission, total),
        format!("Merges: {}  Thawed: {}  Stars: ★{}",
                ctx.total_merges, thawed, ctx.stars),
    ]
}

fn help_lines() -> Vec<(&'static str, &'static str)> {
    vec![
        ("↑↓←→",    "Move cursor"),
        ("Enter",   "Select / Merge / Activate gen"),
        ("D",       "Deliver selected to order"),
        ("S",       "Store selected in inventory"),
        ("I",       "Open inventory"),
        ("A",       "Watch ad (if available)"),
        ("+/-",     "Scale up / down"),
        ("N/P",     "Next/prev color mode"),
        ("H",       "Toggle help"),
        ("Q / Esc", "Back / Quit"),
    ]
}

// ── Main entry ────────────────────────────────────────────────────────────

pub fn run_from_menu() -> std::io::Result<()> {
    let mut user_settings = settings::load();
    let cli_config = {
        let mut c = Config::parse_from::<[&str; 0], &str>([]);
        c.scale = user_settings.scale;
        c.color_mode = user_settings.color_mode.clone();
        c
    };

    let mut stdout = stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
        original_hook(info);
    }));

    let result = run_loop(&mut stdout, &cli_config, &mut user_settings);

    execute!(stdout, LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;
    result
}

fn run_loop(
    stdout: &mut Stdout,
    cli_config: &Config,
    user_settings: &mut loom_engine::settings::UserSettings,
) -> std::io::Result<()> {
    let mut tui_state = TuiState::MainMenu { selected: 0, flash: None };
    let mut engine: Option<GameEngine> = None;
    let mut geo: Option<LayoutGeometry> = None;

    let mut campaign_saves = CampaignSaves::<CampaignState>::load("m2tui");
    let mut campaign_ctx: Option<CampaignState> = None;
    let mut endless_high = load_high_score();
    let mut endless_merges_at_last_scale: u64 = 0;
    let mut game_config = cli_config.clone();
    let mut is_endless = false;

    let ad_quotes = ad_content::load_quotes(&None, "m2tui");

    let menu_items = ["Custom Game", "Campaign", "Endless", "Options", "Quit"];

    let mut last_tick = Instant::now();
    let tick_interval = Duration::from_millis(200);

    // Vim-style count prefix (e.g. 3j = move down 3 cells).
    // Accumulated from digit keypresses; consumed by the next motion; reset otherwise.
    let mut move_count: u32 = 0;

    loop {
        // ── Draw ────────────────────────────────────────────────────────
        stdout.execute(Clear(ClearType::All))?;

        match &tui_state {
            TuiState::MainMenu { selected, flash } => {
                renderer::render_main_menu(stdout, &menu_items, *selected, flash.as_deref())?;
            }
            TuiState::Playing { label } => {
                if let (Some(e), Some(g)) = (&engine, &geo) {
                    renderer::render_hud(stdout, e, label)?;
                    renderer::render_board(stdout, e, g)?;
                    renderer::render_orders(stdout, e, g)?;
                    renderer::render_inventory(stdout, e, g, None)?;
                    renderer::render_key_bar(stdout, e, g)?;
                }
            }
            TuiState::InventoryMode { label, selected_slot } => {
                if let (Some(e), Some(g)) = (&engine, &geo) {
                    renderer::render_hud(stdout, e, label)?;
                    renderer::render_board(stdout, e, g)?;
                    renderer::render_orders(stdout, e, g)?;
                    renderer::render_inventory(stdout, e, g, Some(*selected_slot))?;
                    renderer::render_key_bar(stdout, e, g)?;
                }
            }
            TuiState::PlacingItem { label, .. } => {
                if let (Some(e), Some(g)) = (&engine, &geo) {
                    renderer::render_hud(stdout, e, label)?;
                    renderer::render_board(stdout, e, g)?;
                    renderer::render_key_bar(stdout, e, g)?;
                }
            }
            TuiState::GameOver(status) => {
                if let (Some(e), Some(g)) = (&engine, &geo) {
                    renderer::render_hud(stdout, e, "Game Over")?;
                    renderer::render_board(stdout, e, g)?;
                    renderer::render_orders(stdout, e, g)?;
                    renderer::render_game_over(stdout, status, e.score)?;
                }
            }
            TuiState::CustomGame { preset_idx, selected_field, config } => {
                renderer::render_custom_game(stdout, config, PRESETS[*preset_idx].name, *selected_field)?;
            }
            TuiState::CampaignSelect { selected } => {
                let progress: Vec<String> = (0..TRACK_COUNT)
                    .map(|t| campaign_saves.progress_label(t))
                    .collect();
                renderer::render_campaign_select(stdout, TRACK_NAMES, &progress, *selected)?;
            }
            TuiState::BlessingSelection { cursor, chosen } => {
                let completed = campaign_saves.completed_count();
                renderer::render_blessing_selection(stdout, *cursor, chosen, completed)?;
            }
            TuiState::CampaignLevelIntro => {
                if let Some(ctx) = &campaign_ctx {
                    let lines = level_intro_lines(ctx);
                    renderer::render_level_intro(stdout, &lines)?;
                }
            }
            TuiState::Help => {
                renderer::render_help(stdout, &help_lines())?;
            }
            TuiState::Options { selected } => {
                renderer::render_options(stdout, user_settings, *selected)?;
            }
            TuiState::WatchingAd { started_at, quote, .. } => {
                let elapsed = started_at.elapsed().as_secs();
                renderer::render_ad_overlay(stdout, quote, elapsed)?;
            }
        }

        stdout.flush()?;

        // ── Tick ────────────────────────────────────────────────────────
        if matches!(tui_state, TuiState::Playing { .. }) && last_tick.elapsed() >= tick_interval {
            last_tick = Instant::now();
            if let Some(e) = &mut engine {
                e.tick();
                if is_endless {
                    let merges = e.total_merges;
                    if merges != endless_merges_at_last_scale {
                        apply_scaling(e);
                        endless_merges_at_last_scale = merges;
                    }
                }
            }
        }

        // ── Ad timeout ──────────────────────────────────────────────────
        if let TuiState::WatchingAd { started_at, reward, .. } = &tui_state {
            if started_at.elapsed().as_secs() >= 10 {
                let reward = reward.clone();
                if let Some(e) = &mut engine {
                    e.watch_ad_reward(reward);
                }
                let label = engine.as_ref().map(|_| make_label(&campaign_ctx, is_endless))
                    .unwrap_or_default();
                tui_state = TuiState::Playing { label };
            }
        }

        // ── Input ───────────────────────────────────────────────────────
        if !poll(Duration::from_millis(50))? { continue; }
        let Event::Key(key) = read()? else { continue; };

        match &mut tui_state {
            // ── Main Menu ───────────────────────────────────────────────
            TuiState::MainMenu { selected, flash } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        *flash = None;
                        if *selected > 0 { *selected -= 1; }
                        else { *selected = menu_items.len() - 1; }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        *flash = None;
                        *selected = (*selected + 1) % menu_items.len();
                    }
                    KeyCode::Enter => match *selected {
                        0 => { // Custom Game
                            is_endless = false;
                            tui_state = TuiState::CustomGame {
                                preset_idx: 0,
                                selected_field: 0,
                                config: game_config.clone(),
                            };
                        }
                        1 => { // Campaign
                            tui_state = TuiState::CampaignSelect { selected: 0 };
                        }
                        2 => { // Endless
                            is_endless = true;
                            tui_state = TuiState::BlessingSelection {
                                cursor: 0,
                                chosen: Vec::new(),
                            };
                        }
                        3 => { // Options
                            tui_state = TuiState::Options { selected: 0 };
                        }
                        _ => return Ok(()), // Quit
                    },
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    _ => {}
                }
            }

            // ── Playing ─────────────────────────────────────────────────
            TuiState::Playing { label } => {
                let label = label.clone();
                match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() && (c != '0' || move_count > 0) => {
                        let d = c as u32 - '0' as u32;
                        move_count = move_count.saturating_mul(10).saturating_add(d).min(99);
                    }

                    KeyCode::Up    | KeyCode::Char('k') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(-1, 0); } }
                    }
                    KeyCode::Down  | KeyCode::Char('j') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(1, 0); } }
                    }
                    KeyCode::Left  | KeyCode::Char('h') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(0, -1); } }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(0, 1); } }
                    }

                    KeyCode::Enter | KeyCode::Char(' ') => { move_count = 0;
                        if let Some(e) = &mut engine {
                            e.activate();
                            // Check win/loss/stuck
                            let status = check_status(e, &campaign_ctx);
                            if status != GameStatus::Playing {
                                // Campaign: advance mission on win
                                if status == GameStatus::Won {
                                    if let Some(ctx) = &mut campaign_ctx {
                                        ctx.sync_from_engine(e);
                                        if ctx.advance_mission() {
                                            // More missions remain
                                            ctx.load_mission_orders();
                                            let new_e = ctx.build_engine();
                                            geo = Some(LayoutGeometry::compute(&new_e));
                                            *e = new_e;
                                            tui_state = TuiState::CampaignLevelIntro;
                                            continue;
                                        } else {
                                            // Track complete
                                            campaign_saves.upsert(ctx.clone());
                                            campaign_saves.save("m2tui");
                                        }
                                    } else if is_endless {
                                        endless_high.update(e.total_merges as usize);
                                        save_high_score(&endless_high);
                                    }
                                }
                                tui_state = TuiState::GameOver(status);
                            }
                        }
                    }

                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if let Some(e) = &mut engine { e.deliver_from_board(); }
                    }

                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        if let Some(e) = &mut engine { e.store_selected_to_inventory(); }
                    }

                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        tui_state = TuiState::InventoryMode { label, selected_slot: 0 };
                    }

                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        if let Some(e) = &engine {
                            if e.can_watch_ad() {
                                let reward = ad::reward_for_use(e.ads_used, &e.available_families);
                                let quote = ad_quotes.choose(&mut rand::rng())
                                .cloned().unwrap_or_default();
                                tui_state = TuiState::WatchingAd {
                                    started_at: Instant::now(),
                                    quote,
                                    reward,
                                };
                            }
                        }
                    }

                    KeyCode::Char('H') => {
                        tui_state = TuiState::Help;
                    }

                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        if let Some(e) = &mut engine { e.activate_enhanced(); }
                    }

                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        if let Some(e) = &mut engine {
                            e.scale = (e.scale + 1).min(3);
                            user_settings.scale = e.scale;
                            settings::save(user_settings);
                            geo = Some(LayoutGeometry::compute(e));
                        }
                    }
                    KeyCode::Char('-') => {
                        if let Some(e) = &mut engine {
                            e.scale = (e.scale - 1).max(1);
                            user_settings.scale = e.scale;
                            settings::save(user_settings);
                            geo = Some(LayoutGeometry::compute(e));
                        }
                    }

                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        user_settings.color_mode = settings::next_color_mode(&user_settings.color_mode).to_string();
                        settings::save(user_settings);
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        user_settings.color_mode = settings::prev_color_mode(&user_settings.color_mode).to_string();
                        settings::save(user_settings);
                    }

                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        move_count = 0;
                        // Autosave campaign
                        if let (Some(e), Some(ctx)) = (&engine, &mut campaign_ctx) {
                            ctx.sync_from_engine(e);
                            campaign_saves.upsert(ctx.clone());
                            campaign_saves.save("m2tui");
                        }
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }

                    _ => { move_count = 0; }
                }
            }

            // ── Inventory Mode ──────────────────────────────────────────
            TuiState::InventoryMode { label, selected_slot } => {
                let label = label.clone();
                let slot = *selected_slot;
                match key.code {
                    KeyCode::Left => {
                        if engine.is_some() {
                            if slot > 0 { *selected_slot = slot - 1; }
                        }
                    }
                    KeyCode::Right => {
                        if let Some(e) = &engine {
                            let max = e.inventory.slot_count().saturating_sub(1);
                            *selected_slot = (slot + 1).min(max);
                        }
                    }
                    KeyCode::Enter => {
                        // Pick up item from slot
                        if let Some(e) = &mut engine {
                            if let Some(piece) = e.inventory.take(slot) {
                                tui_state = TuiState::PlacingItem { label, piece };
                            }
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        if let Some(e) = &mut engine {
                            e.deliver_from_inventory(slot);
                        }
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::Playing { label };
                    }
                    _ => {}
                }
            }

            // ── Placing Item ────────────────────────────────────────────
            TuiState::PlacingItem { label, piece } => {
                let label = label.clone();
                let piece = piece.clone();
                match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() && (c != '0' || move_count > 0) => {
                        let d = c as u32 - '0' as u32;
                        move_count = move_count.saturating_mul(10).saturating_add(d).min(99);
                    }

                    KeyCode::Up    | KeyCode::Char('k') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(-1, 0); } }
                    }
                    KeyCode::Down  | KeyCode::Char('j') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(1, 0); } }
                    }
                    KeyCode::Left  | KeyCode::Char('h') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(0, -1); } }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let n = move_count.max(1); move_count = 0;
                        if let Some(e) = &mut engine { for _ in 0..n { e.move_cursor(0, 1); } }
                    }

                    KeyCode::Enter => { move_count = 0;
                        if let Some(e) = &mut engine {
                            let r = e.cursor_row;
                            let c = e.cursor_col;
                            if e.board.cells[r][c].is_empty() {
                                e.board.cells[r][c] = crate::board::Cell::Piece(piece.clone());
                                tui_state = TuiState::Playing { label };
                            }
                            // else: try merge from inventory
                            else if let Some(idx) = find_piece_in_inv(e, &piece) {
                                e.merge_from_inventory(idx);
                                tui_state = TuiState::Playing { label };
                            }
                        }
                    }
                    KeyCode::Esc => {
                        move_count = 0;
                        // Put piece back in first free inventory slot
                        if let Some(e) = &mut engine {
                            e.inventory.store(piece);
                        }
                        tui_state = TuiState::Playing { label };
                    }
                    _ => { move_count = 0; }
                }
            }

            // ── Game Over ────────────────────────────────────────────────
            TuiState::GameOver(_) => {
                match key.code {
                    KeyCode::Enter => {
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    _ => {}
                }
            }

            // ── Custom Game ──────────────────────────────────────────────
            TuiState::CustomGame { preset_idx, selected_field, config } => {
                let num_fields = 11usize;
                match key.code {
                    KeyCode::Up => {
                        if *selected_field > 0 { *selected_field -= 1; }
                    }
                    KeyCode::Down => {
                        *selected_field = (*selected_field + 1) % num_fields;
                    }
                    KeyCode::Left => {
                        adjust_config_field(config, *selected_field, preset_idx, -1);
                    }
                    KeyCode::Right => {
                        adjust_config_field(config, *selected_field, preset_idx, 1);
                    }
                    KeyCode::Enter => {
                        game_config = config.clone();
                        game_config.scale = user_settings.scale;
                        let blessings: Vec<String> = Vec::new();
                        let e = GameEngine::new_endless(&game_config, &blessings);
                        geo = Some(LayoutGeometry::compute(&e));
                        campaign_ctx = None;
                        is_endless = false;
                        engine = Some(e);
                        tui_state = TuiState::Playing { label: "Custom Game".to_string() };
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Campaign Select ──────────────────────────────────────────
            TuiState::CampaignSelect { selected } => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *selected > 0 { *selected -= 1; }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        *selected = (*selected + 1).min(TRACK_COUNT - 1);
                    }
                    KeyCode::Enter => {
                        let track = *selected;
                        // Load or create campaign state
                        let ctx = campaign_saves.get(track)
                            .cloned()
                            .unwrap_or_else(|| CampaignState::new(track));
                        campaign_ctx = Some(ctx);
                        tui_state = TuiState::BlessingSelection {
                            cursor: 0,
                            chosen: campaign_ctx.as_ref()
                                .map(|c| blessings_to_indices(&c.blessings))
                                .unwrap_or_default(),
                        };
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Blessing Selection ────────────────────────────────────────
            TuiState::BlessingSelection { cursor, chosen } => {
                let completed = campaign_saves.completed_count();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if *cursor > 0 { *cursor -= 1; }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        *cursor = (*cursor + 1).min(ALL_BLESSINGS.len() - 1);
                    }
                    KeyCode::Enter => {
                        // Toggle blessing
                        let b = &ALL_BLESSINGS[*cursor];
                        if blessings::is_unlocked(b, completed) {
                            if let Some(pos) = chosen.iter().position(|&i| i == *cursor) {
                                chosen.remove(pos);
                            } else {
                                chosen.push(*cursor);
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        // Start game with selected blessings
                        let ids: Vec<String> = chosen.iter()
                            .map(|&i| ALL_BLESSINGS[i].id.to_string())
                            .collect();
                        let _ = ids.clone(); // track chosen blessings for future use

                        if is_endless {
                            let mut e = new_endless_engine(&ids);
                            e.scale = user_settings.scale;
                            geo = Some(LayoutGeometry::compute(&e));
                            campaign_ctx = None;
                            endless_merges_at_last_scale = 0;
                            engine = Some(e);
                            tui_state = TuiState::Playing { label: "Endless".to_string() };
                        } else if let Some(ctx) = &mut campaign_ctx {
                            ctx.blessings = ids;
                            ctx.load_mission_orders();
                            let e = ctx.build_engine();
                            geo = Some(LayoutGeometry::compute(&e));
                            engine = Some(e);
                            tui_state = TuiState::CampaignLevelIntro;
                        }
                    }
                    KeyCode::Esc => {
                        if campaign_ctx.is_some() {
                            tui_state = TuiState::CampaignSelect { selected: 0 };
                        } else {
                            tui_state = TuiState::MainMenu { selected: 0, flash: None };
                        }
                    }
                    _ => {}
                }
            }

            // ── Campaign Level Intro ──────────────────────────────────────
            TuiState::CampaignLevelIntro => {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                    if let Some(ctx) = &campaign_ctx {
                        let label = format!("{} M{}/{}",
                            TRACK_NAMES.get(ctx.track_idx).copied().unwrap_or(""),
                            ctx.current_mission + 1, ctx.total_missions());
                        tui_state = TuiState::Playing { label };
                    }
                } else if key.code == KeyCode::Esc {
                    tui_state = TuiState::MainMenu { selected: 0, flash: None };
                }
            }

            // ── Help ─────────────────────────────────────────────────────
            TuiState::Help => {
                let label = make_label(&campaign_ctx, is_endless);
                tui_state = TuiState::Playing { label };
            }

            // ── Options ──────────────────────────────────────────────────
            TuiState::Options { selected } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected > 0 { *selected -= 1; }
                    }
                    KeyCode::Down => {
                        *selected = (*selected + 1).min(1);
                    }
                    KeyCode::Left => match *selected {
                        0 => {
                            user_settings.color_mode = settings::prev_color_mode(&user_settings.color_mode).to_string();
                            settings::save(user_settings);
                        }
                        1 => {
                            if user_settings.scale > 1 {
                                user_settings.scale -= 1;
                                settings::save(user_settings);
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Right => match *selected {
                        0 => {
                            user_settings.color_mode = settings::next_color_mode(&user_settings.color_mode).to_string();
                            settings::save(user_settings);
                        }
                        1 => {
                            user_settings.scale = (user_settings.scale + 1).min(3);
                            settings::save(user_settings);
                        }
                        _ => {}
                    },
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Watching Ad ───────────────────────────────────────────────
            TuiState::WatchingAd { started_at, .. } => {
                // Any key skips after 3s, cancels before
                let elapsed = started_at.elapsed().as_secs();
                if key.code == KeyCode::Esc && elapsed < 3 {
                    let label = make_label(&campaign_ctx, is_endless);
                    tui_state = TuiState::Playing { label };
                } else if elapsed >= 3 {
                    let label = make_label(&campaign_ctx, is_endless);
                    if let TuiState::WatchingAd { reward, .. } =
                        std::mem::replace(&mut tui_state, TuiState::Playing { label: label.clone() })
                    {
                        if let Some(e) = &mut engine {
                            e.watch_ad_reward(reward);
                        }
                    }
                    tui_state = TuiState::Playing { label };
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn check_status(engine: &GameEngine, ctx: &Option<CampaignState>) -> GameStatus {
    if let Some(ctx) = ctx {
        if ctx.current_mission_complete() {
            // Recheck via engine state
        }
    }
    if engine.is_stuck() {
        GameStatus::Stuck
    } else {
        GameStatus::Playing
    }
}

fn make_label(ctx: &Option<CampaignState>, is_endless: bool) -> String {
    if is_endless {
        return "Endless".to_string();
    }
    match ctx {
        Some(c) => format!("{} M{}/{}",
            TRACK_NAMES.get(c.track_idx).copied().unwrap_or(""),
            c.current_mission + 1, c.total_missions()),
        None => "Custom Game".to_string(),
    }
}

fn blessings_to_indices(ids: &[String]) -> Vec<usize> {
    ids.iter().filter_map(|id| {
        ALL_BLESSINGS.iter().position(|b| b.id == id.as_str())
    }).collect()
}

fn find_piece_in_inv(engine: &GameEngine, piece: &crate::item::Piece) -> Option<usize> {
    engine.inventory.slots.iter().position(|s| s.as_ref() == Some(piece))
}

fn adjust_config_field(config: &mut Config, field: usize, preset_idx: &mut usize, delta: i32) {
    match field {
        0 => { // preset cycle
            let n = PRESETS.len();
            if delta > 0 { *preset_idx = (*preset_idx + 1) % n; }
            else if *preset_idx > 0 { *preset_idx -= 1; } else { *preset_idx = n - 1; }
            let new_cfg = PRESETS[*preset_idx].to_config(config);
            *config = new_cfg;
        }
        1 => config.board_rows = (config.board_rows as i32 + delta).max(4).min(20) as u16,
        2 => config.board_cols = (config.board_cols as i32 + delta).max(4).min(16) as u16,
        3 => config.scale      = (config.scale as i32 + delta).max(1).min(3) as u16,
        4 => config.energy_max = (config.energy_max as i32 + delta * 10).max(10).min(500) as u16,
        5 => config.energy_regen_secs = (config.energy_regen_secs as i32 + delta * 5).max(5).min(300) as u32,
        6 => config.generator_cost = (config.generator_cost as i32 + delta).max(0).min(20) as u16,
        7 => config.family_count = (config.family_count as i32 + delta).max(1).min(6) as u16,
        8 => config.random_order_count = (config.random_order_count as i32 + delta).max(0).min(5) as u16,
        9 => config.max_order_tier = (config.max_order_tier as i32 + delta).max(1).min(8) as u8,
        10 => config.inventory_slots = (config.inventory_slots as i32 + delta).max(0).min(16) as u16,
        _ => {}
    }
}
