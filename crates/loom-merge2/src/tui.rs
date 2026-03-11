#![allow(warnings)]

use std::io::{Write, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode,
               Clear, ClearType},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
};

use loom_engine::ad_content;
use loom_engine::campaign::CampaignEntry;

use crate::blessings::{self, ALL_BLESSINGS};
use crate::campaign::{CampaignSaves, CampaignState};
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT, levels_for_track};
use crate::config::Config;
use crate::endless::{EndlessHighScore, EndlessState};
use crate::engine::{GameEngine, GameStatus};
use crate::order::generate_orders;
use crate::preset::PRESETS;
use crate::renderer::{self, LayoutGeometry};
use crate::settings::{self, UserSettings};

use clap::Parser;

// ── TuiState ──────────────────────────────────────────────────────────────

enum TuiState {
    MainMenu { selected: usize, flash: Option<String> },
    Playing,
    CustomGame {
        preset_idx: usize,
        selected_field: usize,
        config: Config,
    },
    CampaignSelect { selected: usize },
    BlessingSelection { cursor: usize, chosen: Vec<usize> },
    CampaignLevelIntro,
    GameOver(GameStatus),
    Help,
    Options { selected: usize },
    WatchingAd { started_at: Instant, quote: String },
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

    // Install panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
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
    user_settings: &mut UserSettings,
) -> std::io::Result<()> {
    let mut tui_state = TuiState::MainMenu { selected: 0, flash: None };
    let mut engine: Option<GameEngine> = None;
    let mut geo: Option<LayoutGeometry> = None;

    let mut campaign_saves = CampaignSaves::<CampaignState>::load("m2tui");
    let mut campaign_ctx: Option<CampaignState> = None;
    let mut endless_state: Option<EndlessState> = None;
    let mut endless_high = EndlessHighScore::load("m2tui");

    let ad_quotes = ad_content::load_quotes(&None, "m2tui");
    let mut game_config = cli_config.clone();

    let menu_items = ["Custom Game", "Campaign", "Endless", "Options", "Quit"];

    let mut last_tick = Instant::now();
    let tick_interval = Duration::from_millis(200);

    loop {
        // ── Draw ────────────────────────────────────────────────────────
        stdout.execute(Clear(ClearType::All))?;

        match &tui_state {
            TuiState::MainMenu { selected, flash } => {
                renderer::render_main_menu(stdout, &menu_items, *selected, flash.as_deref())?;
            }
            TuiState::Playing => {
                if let (Some(e), Some(g)) = (&engine, &geo) {
                    renderer::render_score(stdout, e)?;
                    renderer::render_board(stdout, e, g)?;
                    renderer::render_orders(stdout, e, g)?;
                    renderer::render_key_bar(stdout, e)?;
                }
            }
            TuiState::GameOver(status) => {
                if let (Some(e), Some(g)) = (&engine, &geo) {
                    renderer::render_score(stdout, e)?;
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
                let help = help_lines();
                renderer::render_help(stdout, &help)?;
            }
            TuiState::Options { selected } => {
                renderer::render_options(stdout, user_settings, *selected)?;
            }
            TuiState::WatchingAd { started_at, quote } => {
                let elapsed = started_at.elapsed().as_secs();
                renderer::render_ad_overlay(stdout, quote, elapsed)?;
            }
        }

        stdout.flush()?;

        // ── Tick (generators) ───────────────────────────────────────────
        if matches!(tui_state, TuiState::Playing) && last_tick.elapsed() >= tick_interval {
            if let Some(e) = &mut engine {
                e.tick();
                // Check for stuck/lost after tick
                let status = e.status();
                if status != GameStatus::Playing {
                    tui_state = TuiState::GameOver(status);
                }
            }
            last_tick = Instant::now();
        }

        // ── Input ───────────────────────────────────────────────────────
        if !poll(Duration::from_millis(50))? { continue; }
        let Event::Key(key) = read()? else { continue; };

        match &mut tui_state {
            // ── Main Menu ───────────────────────────────────────────────
            TuiState::MainMenu { selected, flash } => {
                match key.code {
                    KeyCode::Up => {
                        *flash = None;
                        if *selected > 0 { *selected -= 1; }
                        else { *selected = menu_items.len() - 1; }
                    }
                    KeyCode::Down => {
                        *flash = None;
                        *selected = (*selected + 1) % menu_items.len();
                    }
                    KeyCode::Enter => {
                        *flash = None;
                        match *selected {
                            0 => { // Custom Game
                                tui_state = TuiState::CustomGame {
                                    preset_idx: 0,
                                    selected_field: 0,
                                    config: PRESETS[0].to_config(cli_config),
                                };
                            }
                            1 => { // Campaign
                                tui_state = TuiState::CampaignSelect { selected: 0 };
                            }
                            2 => { // Endless
                                let mut es = EndlessState::new();
                                game_config = es.to_config(cli_config);
                                game_config.scale = user_settings.scale;
                                game_config.color_mode = user_settings.color_mode.clone();
                                let e = GameEngine::new(&game_config);
                                geo = Some(LayoutGeometry::compute(&e));
                                engine = Some(e);
                                endless_state = Some(es);
                                campaign_ctx = None;
                                tui_state = TuiState::Playing;
                                last_tick = Instant::now();
                            }
                            3 => { // Options
                                tui_state = TuiState::Options { selected: 0 };
                            }
                            4 => return Ok(()), // Quit
                            _ => {}
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }

            // ── Custom Game ─────────────────────────────────────────────
            TuiState::CustomGame { preset_idx, selected_field, config } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_field > 0 { *selected_field -= 1; }
                    }
                    KeyCode::Down => {
                        if *selected_field < 10 { *selected_field += 1; }
                    }
                    KeyCode::Left => {
                        if *selected_field == 0 {
                            if *preset_idx > 0 { *preset_idx -= 1; }
                            else { *preset_idx = PRESETS.len() - 1; }
                            *config = PRESETS[*preset_idx].to_config(cli_config);
                        } else {
                            adjust_custom_field(config, *selected_field, -1);
                        }
                    }
                    KeyCode::Right => {
                        if *selected_field == 0 {
                            *preset_idx = (*preset_idx + 1) % PRESETS.len();
                            *config = PRESETS[*preset_idx].to_config(cli_config);
                        } else {
                            adjust_custom_field(config, *selected_field, 1);
                        }
                    }
                    KeyCode::Enter => {
                        game_config = config.clone();
                        game_config.scale = user_settings.scale;
                        game_config.color_mode = user_settings.color_mode.clone();
                        let e = GameEngine::new(&game_config);
                        geo = Some(LayoutGeometry::compute(&e));
                        engine = Some(e);
                        campaign_ctx = None;
                        endless_state = None;
                        tui_state = TuiState::Playing;
                        last_tick = Instant::now();
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Campaign Select ─────────────────────────────────────────
            TuiState::CampaignSelect { selected } => {
                match key.code {
                    KeyCode::Up => { if *selected > 0 { *selected -= 1; } }
                    KeyCode::Down => { if *selected < TRACK_COUNT - 1 { *selected += 1; } }
                    KeyCode::Enter => {
                        let track = *selected;
                        let state = campaign_saves.get(track)
                            .cloned()
                            .unwrap_or_else(|| CampaignState::new(track));
                        if state.is_completed() {
                            // Already completed — offer restart?
                            let state = CampaignState::new(track);
                            campaign_saves.upsert(state.clone());
                            campaign_saves.save("m2tui");
                            campaign_ctx = Some(state);
                        } else {
                            campaign_ctx = Some(state);
                        }
                        tui_state = TuiState::BlessingSelection { cursor: 0, chosen: Vec::new() };
                    }
                    KeyCode::Char('r') => {
                        // Reset track
                        campaign_saves.reset(*selected);
                        campaign_saves.save("m2tui");
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 1, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Blessing Selection ─────────────────────────────────────
            TuiState::BlessingSelection { cursor, chosen } => {
                let completed = campaign_saves.completed_count();
                let available = blessings::available_blessings(completed);
                let total = available.len();
                let cols = 3usize;

                match key.code {
                    KeyCode::Up => {
                        if *cursor >= cols { *cursor -= cols; }
                    }
                    KeyCode::Down => {
                        if *cursor + cols < total { *cursor += cols; }
                    }
                    KeyCode::Left => {
                        if *cursor % cols > 0 { *cursor -= 1; }
                    }
                    KeyCode::Right => {
                        if *cursor % cols < cols - 1 && *cursor + 1 < total {
                            *cursor += 1;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        if let Some(pos) = chosen.iter().position(|&x| x == *cursor) {
                            chosen.remove(pos);
                        } else if chosen.len() < 3 {
                            chosen.push(*cursor);
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        if chosen.len() == 3 {
                            // Commit chosen blessing IDs to campaign state
                            let ids: Vec<String> = chosen.iter()
                                .map(|&i| available[i].id.to_string())
                                .collect();
                            if let Some(ctx) = &mut campaign_ctx {
                                ctx.blessings = ids;
                                campaign_saves.upsert(ctx.clone());
                                campaign_saves.save("m2tui");
                            }
                            tui_state = TuiState::CampaignLevelIntro;
                        }
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::CampaignSelect { selected: 0 };
                    }
                    _ => {}
                }
            }

            // ── Campaign Level Intro ────────────────────────────────────
            TuiState::CampaignLevelIntro => {
                match key.code {
                    KeyCode::Enter => {
                        if let Some(ctx) = &campaign_ctx {
                            game_config = ctx.to_config(cli_config);
                            game_config.scale = user_settings.scale;
                            game_config.color_mode = user_settings.color_mode.clone();
                            let mut e = GameEngine::new_campaign(
                                &game_config, ctx.track_idx, ctx.current_level,
                            );
                            e.set_blessings(&ctx.blessings);
                            geo = Some(LayoutGeometry::compute(&e));
                            engine = Some(e);
                            endless_state = None;
                            tui_state = TuiState::Playing;
                            last_tick = Instant::now();
                        }
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::CampaignSelect { selected: 0 };
                    }
                    _ => {}
                }
            }

            // ── Playing ─────────────────────────────────────────────────
            TuiState::Playing => {
                if let Some(e) = &mut engine {
                    match key.code {
                        KeyCode::Up => { e.move_cursor(-1, 0); }
                        KeyCode::Down => { e.move_cursor(1, 0); }
                        KeyCode::Left => { e.move_cursor(0, -1); }
                        KeyCode::Right => { e.move_cursor(0, 1); }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            e.activate();
                            let status = e.status();
                            if status != GameStatus::Playing {
                                tui_state = TuiState::GameOver(status);
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            e.deliver();
                            let status = e.status();
                            if status != GameStatus::Playing {
                                tui_state = TuiState::GameOver(status);
                            }
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            if e.can_watch_ad() {
                                let quote = ad_content::random_quote(&ad_quotes).to_string();
                                tui_state = TuiState::WatchingAd {
                                    started_at: Instant::now(),
                                    quote,
                                };
                            }
                        }
                        KeyCode::Esc => {
                            e.selected = None;
                        }
                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            tui_state = TuiState::Help;
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            let new_scale = (e.scale + 1).min(4);
                            e.scale = new_scale;
                            user_settings.scale = new_scale;
                            settings::save(user_settings);
                            geo = Some(LayoutGeometry::compute(e));
                        }
                        KeyCode::Char('-') => {
                            let new_scale = e.scale.saturating_sub(1).max(1);
                            e.scale = new_scale;
                            user_settings.scale = new_scale;
                            settings::save(user_settings);
                            geo = Some(LayoutGeometry::compute(e));
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            let next = settings::next_color_mode(&user_settings.color_mode);
                            user_settings.color_mode = next.to_string();
                            settings::save(user_settings);
                        }
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            let prev = settings::prev_color_mode(&user_settings.color_mode);
                            user_settings.color_mode = prev.to_string();
                            settings::save(user_settings);
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            tui_state = TuiState::MainMenu { selected: 0, flash: None };
                        }
                        _ => {}
                    }
                }
            }

            // ── Game Over ───────────────────────────────────────────────
            TuiState::GameOver(status) => {
                match key.code {
                    KeyCode::Enter => {
                        match status {
                            GameStatus::Won => {
                                // Campaign: advance
                                if let Some(ctx) = &mut campaign_ctx {
                                    let done = ctx.complete_level();
                                    campaign_saves.upsert(ctx.clone());
                                    campaign_saves.save("m2tui");
                                    if done {
                                        tui_state = TuiState::MainMenu {
                                            selected: 1,
                                            flash: Some("Campaign Complete!".to_string()),
                                        };
                                    } else {
                                        tui_state = TuiState::CampaignLevelIntro;
                                    }
                                } else if let Some(es) = &mut endless_state {
                                    // Endless: next wave
                                    let score = engine.as_ref().map(|e| e.score).unwrap_or(0);
                                    if endless_high.update(es.wave) {
                                        endless_high.save("m2tui");
                                    }
                                    es.advance();
                                    game_config = es.to_config(cli_config);
                                    game_config.scale = user_settings.scale;
                                    game_config.color_mode = user_settings.color_mode.clone();
                                    let e = GameEngine::new(&game_config);
                                    geo = Some(LayoutGeometry::compute(&e));
                                    engine = Some(e);
                                    tui_state = TuiState::Playing;
                                    last_tick = Instant::now();
                                } else {
                                    // Custom: back to menu
                                    tui_state = TuiState::MainMenu {
                                        selected: 0,
                                        flash: Some(format!("Score: {}", engine.as_ref().map(|e| e.score).unwrap_or(0))),
                                    };
                                }
                            }
                            _ => {
                                // Retry or quit
                                tui_state = TuiState::MainMenu { selected: 0, flash: None };
                            }
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        // Retry
                        if let Some(ctx) = &campaign_ctx {
                            let mut e = GameEngine::new_campaign(
                                &game_config, ctx.track_idx, ctx.current_level,
                            );
                            e.set_blessings(&ctx.blessings);
                            geo = Some(LayoutGeometry::compute(&e));
                            engine = Some(e);
                            tui_state = TuiState::Playing;
                            last_tick = Instant::now();
                        } else {
                            let e = GameEngine::new(&game_config);
                            geo = Some(LayoutGeometry::compute(&e));
                            engine = Some(e);
                            tui_state = TuiState::Playing;
                            last_tick = Instant::now();
                        }
                    }
                    KeyCode::Char('a') | KeyCode::Char('A') => {
                        if matches!(status, GameStatus::Stuck) {
                            if let Some(e) = &engine {
                                if e.can_watch_ad() {
                                    let quote = ad_content::random_quote(&ad_quotes).to_string();
                                    tui_state = TuiState::WatchingAd {
                                        started_at: Instant::now(),
                                        quote,
                                    };
                                }
                            }
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        if let Some(es) = &endless_state {
                            if endless_high.update(es.wave) {
                                endless_high.save("m2tui");
                            }
                        }
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Help ────────────────────────────────────────────────────
            TuiState::Help => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('q') => {
                        tui_state = TuiState::Playing;
                    }
                    _ => {}
                }
            }

            // ── Options ─────────────────────────────────────────────────
            TuiState::Options { selected } => {
                match key.code {
                    KeyCode::Up => { if *selected > 0 { *selected -= 1; } }
                    KeyCode::Down => { if *selected < 1 { *selected += 1; } }
                    KeyCode::Left | KeyCode::Right => {
                        match *selected {
                            0 => { // Scale
                                if key.code == KeyCode::Left {
                                    user_settings.scale = user_settings.scale.saturating_sub(1).max(1);
                                } else {
                                    user_settings.scale = (user_settings.scale + 1).min(4);
                                }
                            }
                            1 => { // Color mode
                                if key.code == KeyCode::Left {
                                    user_settings.color_mode = settings::prev_color_mode(&user_settings.color_mode).to_string();
                                } else {
                                    user_settings.color_mode = settings::next_color_mode(&user_settings.color_mode).to_string();
                                }
                            }
                            _ => {}
                        }
                        settings::save(user_settings);
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 3, flash: None };
                    }
                    _ => {}
                }
            }

            // ── Watching Ad ─────────────────────────────────────────────
            TuiState::WatchingAd { started_at, .. } => {
                if started_at.elapsed().as_secs() >= 15 {
                    // Ad complete — grant reward
                    if let Some(e) = &mut engine {
                        e.watch_ad();
                    }
                    tui_state = TuiState::Playing;
                    last_tick = Instant::now();
                }
                // Esc to skip (no reward)
                if key.code == KeyCode::Esc {
                    tui_state = TuiState::Playing;
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn adjust_custom_field(config: &mut Config, field: usize, delta: i32) {
    match field {
        1 => config.board_height = (config.board_height as i32 + delta).clamp(3, 8) as u16,
        2 => config.board_width = (config.board_width as i32 + delta).clamp(3, 8) as u16,
        3 => config.color_count = (config.color_count as i32 + delta).clamp(1, 6) as u16,
        4 => config.generator_count = (config.generator_count as i32 + delta).clamp(1, 8) as u16,
        5 => config.generator_charges = (config.generator_charges as i32 + delta).clamp(0, 30) as u16,
        6 => config.generator_interval = (config.generator_interval as i32 + delta).clamp(2, 20) as u32,
        7 => config.blocked_cells = (config.blocked_cells as i32 + delta).clamp(0, 8) as u16,
        8 => config.order_count = (config.order_count as i32 + delta).clamp(1, 4) as u16,
        9 => config.max_order_tier = (config.max_order_tier as i32 + delta).clamp(2, 5) as u8,
        10 => config.ad_limit = (config.ad_limit as i32 + delta).clamp(0, 10) as u16,
        _ => {}
    }
}

fn help_lines() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Arrow keys", "Move cursor"),
        ("Enter/Space", "Select / Merge"),
        ("D", "Deliver to order"),
        ("A", "Watch ad for space"),
        ("Esc", "Deselect / Back"),
        ("H", "Toggle help"),
        ("N/P", "Next/prev color mode"),
        ("+/-", "Scale up/down"),
        ("Q", "Quit to menu"),
    ]
}

fn level_intro_lines(ctx: &CampaignState) -> Vec<String> {
    let levels = levels_for_track(ctx.track_idx);
    let l = &levels[ctx.current_level];
    vec![
        format!("{} — Level {}/{}", TRACK_NAMES[ctx.track_idx], ctx.current_level + 1, levels.len()),
        format!("Board: {}×{}, {} colors", l.board_height, l.board_width, l.color_count),
        format!("Generators: {} (charges: {})", l.generator_count,
            if l.generator_charges == 0 { "∞".to_string() } else { l.generator_charges.to_string() }),
        format!("Orders: {}, Ad limit: {}", l.orders.len(), l.ad_limit),
    ]
}
