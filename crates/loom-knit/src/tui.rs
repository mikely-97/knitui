#![allow(warnings)]

use std::io::{Write, stdout, Stdout};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode},
};

use clap::{CommandFactory, Parser, parser::ValueSource};

use crate::ad_content;
use crate::board_entity::Direction;
use crate::campaign::{CampaignSaves, CampaignState};
use crate::campaign_levels::{self, TRACK_NAMES, TRACK_COUNT};
use crate::endless::{EndlessState, EndlessHighScore};
use crate::config::{Config, MAX_BOARD_DIM};
use crate::engine::{GameEngine, GameStatus, BonusState};
use crate::preset::PRESETS;
use crate::renderer::{self, Layout, COMP_GAP, YARN_HGAP, YARN_VGAP};
use crate::settings::{self, UserSettings};

enum TuiState {
    MainMenu { selected: usize, flash: Option<String> },
    CustomGame {
        preset_idx: usize,
        selected_field: usize,
        config: Config,
    },
    CampaignSelect { selected: usize },
    CampaignLevelIntro,
    Options { selected: usize },
    Playing,
    GameOver(GameStatus),
    Help,
    WatchingAd { started_at: Instant, quote: String },
}

struct LayoutGeometry {
    layout: Layout,
    yarn_x: u16,
    board_x: u16,
    board_y: u16,
    scale: u16,
}

impl LayoutGeometry {
    fn compute(config: &Config) -> Self {
        let scale = config.scale;
        let sh = scale;
        let sw = scale * 2;

        let layout = renderer::detect_layout(
            &config.layout, config.visible_stitches, config.board_height, scale,
        );

        let yarn_h = config.visible_stitches * sh
            + config.visible_stitches.saturating_sub(1) * YARN_VGAP;
        let board_y: u16 = yarn_h + COMP_GAP + sh + COMP_GAP;

        let yarn_w = config.yarn_lines * sw
            + config.yarn_lines.saturating_sub(1) * YARN_HGAP;
        let has_flanks = config.balloons > 0 && config.balloon_count > 0;
        let (yarn_x, board_x) = if has_flanks {
            let has_left  = config.balloon_count / 2 > 0;
            let has_right = (config.balloon_count + 1) / 2 > 0;
            let left_w  = if has_left  { sw } else { 0 };
            let right_w = if has_right { sw } else { 0 };
            let left_gap  = if has_left  { YARN_HGAP } else { 0 };
            let right_gap = if has_right { YARN_HGAP } else { 0 };
            let yx = left_w + left_gap;
            let bx = yx + yarn_w + right_gap + right_w + COMP_GAP + sw + COMP_GAP;
            (yx, bx)
        } else {
            (0u16, yarn_w + COMP_GAP + sw + COMP_GAP)
        };

        Self { layout, yarn_x, board_x, board_y, scale }
    }
}

fn custom_game_fields(config: &Config) -> Vec<(&'static str, u16)> {
    vec![
        ("Board Height", config.board_height),
        ("Board Width", config.board_width),
        ("Color Count", config.color_number),
        ("Obstacle %", config.obstacle_percentage),
        ("Conveyor %", config.conveyor_percentage),
        ("Scissors", config.scissors),
        ("Tweezers", config.tweezers),
        ("Balloons", config.balloons),
    ]
}

fn adjust_custom_field(config: &mut Config, field: usize, delta: i16) {
    let apply = |val: &mut u16, min: u16, max: u16| {
        let new = (*val as i16 + delta).clamp(min as i16, max as i16) as u16;
        *val = new;
    };
    match field {
        1 => apply(&mut config.board_height, 2, MAX_BOARD_DIM),
        2 => apply(&mut config.board_width, 2, MAX_BOARD_DIM),
        3 => apply(&mut config.color_number, 2, 8),
        4 => apply(&mut config.obstacle_percentage, 0, 50),
        5 => apply(&mut config.conveyor_percentage, 0, 50),
        6 => apply(&mut config.scissors, 0, 99),
        7 => apply(&mut config.tweezers, 0, 99),
        8 => apply(&mut config.balloons, 0, 99),
        _ => {}
    }
}

const GAME_ARGS: &[&str] = &[
    "board_height", "board_width", "color_number",
    "obstacle_percentage", "conveyor_percentage",
    "scissors", "tweezers", "balloons",
];

fn advance_endless_wave(
    endless_ctx: &mut Option<EndlessState>,
    game_config: &mut Config,
    cli_config: &Config,
    geo: &mut LayoutGeometry,
    engine: &mut Option<GameEngine>,
) {
    let ctx = endless_ctx.as_mut().unwrap();
    ctx.advance();
    *game_config = ctx.to_config(cli_config);
    *geo = LayoutGeometry::compute(game_config);
    *engine = Some(GameEngine::new(game_config));
}

fn campaign_overlay_msg(ctx: &Option<CampaignState>, status: &GameStatus) -> Option<String> {
    let ctx = ctx.as_ref()?;
    let level_label = format!("[{}/{}]", ctx.current_level + 1, ctx.total_levels());
    Some(match status {
        GameStatus::Won => format!("{} You won! N:Next Level  M:Menu  Q:Quit", level_label),
        GameStatus::Stuck => format!("{} You're lost! R:Retry  A:Ad  M:Menu  Q:Quit", level_label),
        _ => return None,
    })
}

/// Run the knitui game from the standalone binary (parses CLI args).
pub fn run_cli() -> std::io::Result<()> {
    let matches = Config::command().get_matches_from(std::env::args_os());
    let skip_menu = GAME_ARGS.iter().any(|name| {
        matches.value_source(name) == Some(ValueSource::CommandLine)
    });
    let mut cli_config = Config::parse();

    let mut user_settings = UserSettings::load("knitui");
    if matches.value_source("scale") != Some(ValueSource::CommandLine) {
        cli_config.scale = user_settings.scale;
    }
    if matches.value_source("color_mode") != Some(ValueSource::CommandLine) {
        cli_config.color_mode = user_settings.color_mode.clone();
    }

    run_event_loop(cli_config, user_settings, skip_menu)
}

/// Run the knitui game from the game selector (default config, always shows menu).
pub fn run_from_menu() -> std::io::Result<()> {
    let mut user_settings = UserSettings::load("knitui");
    let mut config = Config::parse_from::<[&str; 0], &str>([]);
    config.scale = user_settings.scale;
    config.color_mode = user_settings.color_mode.clone();
    run_event_loop(config, user_settings, false)
}

fn run_event_loop(
    mut cli_config: Config,
    mut user_settings: UserSettings,
    skip_menu: bool,
) -> std::io::Result<()> {
    let ad_quotes = ad_content::load_quotes(&cli_config.ad_file, "knitui");
    const AD_DURATION_SECS: u64 = 15;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    // Ensure terminal cleanup on panic
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        default_hook(info);
    }));

    let mut campaign_saves = CampaignSaves::<CampaignState>::load("knitui");
    let mut campaign_ctx: Option<CampaignState> = None;
    let mut endless_ctx: Option<EndlessState> = None;
    let mut endless_hs = EndlessHighScore::load("knitui");

    let mut game_config = cli_config.clone();
    let mut geo = LayoutGeometry::compute(&game_config);

    let (mut engine, mut tui_state): (Option<GameEngine>, TuiState) = if skip_menu {
        let e = GameEngine::new(&game_config);
        renderer::do_render(&mut stdout, &e, geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
        (Some(e), TuiState::Playing)
    } else {
        renderer::render_main_menu(&mut stdout, 0, None)?;
        (None, TuiState::MainMenu { selected: 0, flash: None })
    };

    loop {
        if poll(Duration::from_millis(150))? {
            if let Event::Key(event) = read()? {
                match tui_state {
                    TuiState::MainMenu { ref mut selected, ref mut flash } => {
                        *flash = None;
                        match event.code {
                            KeyCode::Up => {
                                if *selected > 0 { *selected -= 1; }
                            }
                            KeyCode::Down => {
                                if *selected < 5 { *selected += 1; }
                            }
                            KeyCode::Enter => {
                                match *selected {
                                    0 => {
                                        game_config = cli_config.clone();
                                        geo = LayoutGeometry::compute(&game_config);
                                        engine = Some(GameEngine::new(&game_config));
                                        tui_state = TuiState::Playing;
                                        renderer::do_render(
                                            &mut stdout, engine.as_ref().unwrap(),
                                            geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale,
                                        )?;
                                        continue;
                                    }
                                    1 => {
                                        let preset_cfg = PRESETS[1].to_config(&cli_config);
                                        tui_state = TuiState::CustomGame {
                                            preset_idx: 1,
                                            selected_field: 0,
                                            config: preset_cfg,
                                        };
                                    }
                                    2 => {
                                        tui_state = TuiState::CampaignSelect { selected: 0 };
                                    }
                                    3 => {
                                        let state = EndlessState::new();
                                        game_config = state.to_config(&cli_config);
                                        geo = LayoutGeometry::compute(&game_config);
                                        engine = Some(GameEngine::new(&game_config));
                                        endless_ctx = Some(state);
                                        tui_state = TuiState::Playing;
                                        renderer::do_render(
                                            &mut stdout, engine.as_ref().unwrap(),
                                            geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale,
                                        )?;
                                        continue;
                                    }
                                    4 => {
                                        tui_state = TuiState::Options { selected: 0 };
                                    }
                                    5 => break,
                                    _ => {}
                                }
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            _ => {}
                        }
                        if let TuiState::MainMenu { selected, ref flash } = tui_state {
                            renderer::render_main_menu(
                                &mut stdout, selected, flash.as_deref(),
                            )?;
                        } else if let TuiState::CustomGame { preset_idx, selected_field, ref config } = tui_state {
                            let fields = custom_game_fields(config);
                            renderer::render_custom_game(
                                &mut stdout, PRESETS[preset_idx].name, &fields, selected_field,
                            )?;
                        } else if let TuiState::CampaignSelect { selected } = tui_state {
                            let sizes: Vec<usize> = (0..TRACK_COUNT).map(|i| campaign_levels::levels_for_track(i).len()).collect();
                            let labels: Vec<String> = (0..TRACK_COUNT).map(|i| campaign_saves.progress_label(i)).collect();
                            renderer::render_campaign_select(
                                &mut stdout, selected, TRACK_NAMES, &sizes, &labels,
                            )?;
                        } else if let TuiState::Options { selected } = tui_state {
                            renderer::render_options(
                                &mut stdout, selected,
                                user_settings.scale, &user_settings.color_mode,
                            )?;
                        }
                    }
                    TuiState::Options { ref mut selected } => {
                        match event.code {
                            KeyCode::Up => {
                                if *selected > 0 { *selected -= 1; }
                            }
                            KeyCode::Down => {
                                if *selected < 1 { *selected += 1; }
                            }
                            KeyCode::Left => {
                                match *selected {
                                    0 => {
                                        if user_settings.scale > 1 {
                                            user_settings.scale -= 1;
                                        }
                                    }
                                    1 => {
                                        user_settings.color_mode = settings::prev_color_mode(&user_settings.color_mode).to_string();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Right => {
                                match *selected {
                                    0 => {
                                        if user_settings.scale < 5 {
                                            user_settings.scale += 1;
                                        }
                                    }
                                    1 => {
                                        user_settings.color_mode = settings::next_color_mode(&user_settings.color_mode).to_string();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Esc => {
                                user_settings.save("knitui");
                                cli_config.scale = user_settings.scale;
                                cli_config.color_mode = user_settings.color_mode.clone();
                                tui_state = TuiState::MainMenu { selected: 4, flash: None };
                                renderer::render_main_menu(&mut stdout, 4, None)?;
                                continue;
                            }
                            _ => {}
                        }
                        renderer::render_options(
                            &mut stdout, *selected,
                            user_settings.scale, &user_settings.color_mode,
                        )?;
                    }
                    TuiState::CampaignSelect { ref mut selected } => {
                        match event.code {
                            KeyCode::Up => {
                                if *selected > 0 { *selected -= 1; }
                            }
                            KeyCode::Down => {
                                if *selected < TRACK_COUNT - 1 { *selected += 1; }
                            }
                            KeyCode::Enter => {
                                let track_idx = *selected;
                                let state = campaign_saves.get(track_idx).cloned().unwrap_or_else(|| {
                                    CampaignState::new(track_idx)
                                });
                                if state.completed {
                                    campaign_saves.reset(track_idx);
                                    let s = CampaignState::new(track_idx);
                                    campaign_ctx = Some(s);
                                } else {
                                    campaign_ctx = Some(state);
                                }
                                tui_state = TuiState::CampaignLevelIntro;
                                let ctx = campaign_ctx.as_ref().unwrap();
                                let levels = campaign_levels::levels_for_track(ctx.track_idx);
                                let level = &levels[ctx.current_level];
                                renderer::render_level_intro(
                                    &mut stdout,
                                    TRACK_NAMES[ctx.track_idx],
                                    ctx.current_level + 1,
                                    ctx.total_levels(),
                                    level.board_height,
                                    level.board_width,
                                    level.color_number,
                                )?;
                                continue;
                            }
                            KeyCode::Esc => {
                                tui_state = TuiState::MainMenu { selected: 2, flash: None };
                                renderer::render_main_menu(&mut stdout, 2, None)?;
                                continue;
                            }
                            _ => {}
                        }
                        let sizes: Vec<usize> = (0..TRACK_COUNT).map(|i| campaign_levels::levels_for_track(i).len()).collect();
                        let labels: Vec<String> = (0..TRACK_COUNT).map(|i| campaign_saves.progress_label(i)).collect();
                        renderer::render_campaign_select(
                            &mut stdout, *selected, TRACK_NAMES, &sizes, &labels,
                        )?;
                    }
                    TuiState::CampaignLevelIntro => {
                        match event.code {
                            KeyCode::Enter => {
                                let ctx = campaign_ctx.as_ref().unwrap();
                                game_config = ctx.to_config(&cli_config);
                                geo = LayoutGeometry::compute(&game_config);
                                let mut e = GameEngine::new(&game_config);
                                e.set_ad_limit(ctx.ad_limit());
                                engine = Some(e);
                                tui_state = TuiState::Playing;
                                renderer::do_render(
                                    &mut stdout, engine.as_ref().unwrap(),
                                    geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale,
                                )?;
                                continue;
                            }
                            KeyCode::Esc => {
                                campaign_ctx = None;
                                tui_state = TuiState::CampaignSelect { selected: 0 };
                                let sizes: Vec<usize> = (0..TRACK_COUNT).map(|i| campaign_levels::levels_for_track(i).len()).collect();
                                let labels: Vec<String> = (0..TRACK_COUNT).map(|i| campaign_saves.progress_label(i)).collect();
                                renderer::render_campaign_select(
                                    &mut stdout, 0, TRACK_NAMES, &sizes, &labels,
                                )?;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    TuiState::CustomGame { ref mut preset_idx, ref mut selected_field, ref mut config } => {
                        match event.code {
                            KeyCode::Up => {
                                if *selected_field > 0 { *selected_field -= 1; }
                            }
                            KeyCode::Down => {
                                if *selected_field < 8 { *selected_field += 1; }
                            }
                            KeyCode::Left => {
                                if *selected_field == 0 {
                                    if *preset_idx > 0 { *preset_idx -= 1; }
                                    else { *preset_idx = PRESETS.len() - 1; }
                                    *config = PRESETS[*preset_idx].to_config(&cli_config);
                                } else {
                                    adjust_custom_field(config, *selected_field, -1);
                                }
                            }
                            KeyCode::Right => {
                                if *selected_field == 0 {
                                    *preset_idx = (*preset_idx + 1) % PRESETS.len();
                                    *config = PRESETS[*preset_idx].to_config(&cli_config);
                                } else {
                                    adjust_custom_field(config, *selected_field, 1);
                                }
                            }
                            KeyCode::Enter => {
                                game_config = config.clone();
                                geo = LayoutGeometry::compute(&game_config);
                                engine = Some(GameEngine::new(&game_config));
                                tui_state = TuiState::Playing;
                                renderer::do_render(
                                    &mut stdout, engine.as_ref().unwrap(),
                                    geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale,
                                )?;
                                continue;
                            }
                            KeyCode::Esc => {
                                tui_state = TuiState::MainMenu { selected: 1, flash: None };
                                renderer::render_main_menu(&mut stdout, 1, None)?;
                                continue;
                            }
                            _ => {}
                        }
                        if let TuiState::CustomGame { preset_idx, selected_field, ref config } = tui_state {
                            let fields = custom_game_fields(config);
                            renderer::render_custom_game(
                                &mut stdout, PRESETS[preset_idx].name, &fields, selected_field,
                            )?;
                        }
                    }
                    TuiState::GameOver(ref status) => {
                        match event.code {
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if engine.as_ref().unwrap().can_watch_ad() {
                                    let quote = ad_content::random_quote(&ad_quotes).to_string();
                                    tui_state = TuiState::WatchingAd {
                                        started_at: Instant::now(),
                                        quote,
                                    };
                                }
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') | KeyCode::Char('n') | KeyCode::Char('N') => {
                                if endless_ctx.is_some() {
                                    endless_ctx = None;
                                    let state = EndlessState::new();
                                    game_config = state.to_config(&cli_config);
                                    geo = LayoutGeometry::compute(&game_config);
                                    engine = Some(GameEngine::new(&game_config));
                                    endless_ctx = Some(state);
                                    tui_state = TuiState::Playing;
                                    renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                } else if let Some(ref mut ctx) = campaign_ctx {
                                    if *status == GameStatus::Won {
                                        let done = ctx.complete_level();
                                        campaign_saves.upsert(ctx.clone());
                                        campaign_saves.save("knitui");
                                        if done {
                                            campaign_ctx = None;
                                            tui_state = TuiState::MainMenu {
                                                selected: 2,
                                                flash: Some("Campaign complete!".to_string()),
                                            };
                                            renderer::render_main_menu(&mut stdout, 2, Some("Campaign complete!"))?;
                                            continue;
                                        }
                                        tui_state = TuiState::CampaignLevelIntro;
                                        let levels = campaign_levels::levels_for_track(ctx.track_idx);
                                        let level = &levels[ctx.current_level];
                                        renderer::render_level_intro(
                                            &mut stdout,
                                            TRACK_NAMES[ctx.track_idx],
                                            ctx.current_level + 1,
                                            ctx.total_levels(),
                                            level.board_height,
                                            level.board_width,
                                            level.color_number,
                                        )?;
                                        continue;
                                    } else {
                                        game_config = ctx.to_config(&cli_config);
                                        geo = LayoutGeometry::compute(&game_config);
                                        let mut e = GameEngine::new(&game_config);
                                        e.set_ad_limit(ctx.ad_limit());
                                        engine = Some(e);
                                        tui_state = TuiState::Playing;
                                        renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                    }
                                } else {
                                    engine = Some(GameEngine::new(&game_config));
                                    tui_state = TuiState::Playing;
                                    renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                }
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') | KeyCode::Esc => {
                                if campaign_ctx.is_some() {
                                    campaign_saves.upsert(campaign_ctx.as_ref().unwrap().clone());
                                    campaign_saves.save("knitui");
                                    campaign_ctx = None;
                                }
                                endless_ctx = None;
                                tui_state = TuiState::MainMenu { selected: 0, flash: None };
                                renderer::render_main_menu(&mut stdout, 0, None)?;
                                continue;
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') => {
                                if campaign_ctx.is_some() {
                                    campaign_saves.upsert(campaign_ctx.as_ref().unwrap().clone());
                                    campaign_saves.save("knitui");
                                }
                                break;
                            }
                            KeyCode::Char('z') | KeyCode::Char('Z') if *status == GameStatus::Stuck => {
                                let _ = engine.as_mut().unwrap().use_scissors();
                                let e = engine.as_ref().unwrap();
                                match e.status() {
                                    GameStatus::Playing => {
                                        tui_state = TuiState::Playing;
                                        renderer::do_render(&mut stdout, e, geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                    }
                                    s => {
                                        renderer::do_render_overlay(&mut stdout, e, geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale, &s, None)?;
                                        tui_state = TuiState::GameOver(s);
                                    }
                                }
                            }
                            KeyCode::Char('x') | KeyCode::Char('X') if *status == GameStatus::Stuck => {
                                if engine.as_mut().unwrap().use_tweezers().is_ok() {
                                    tui_state = TuiState::Playing;
                                    renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                }
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') if *status == GameStatus::Stuck => {
                                let _ = engine.as_mut().unwrap().use_balloons();
                                let e = engine.as_ref().unwrap();
                                match e.status() {
                                    GameStatus::Playing => {
                                        tui_state = TuiState::Playing;
                                        renderer::do_render(&mut stdout, e, geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                    }
                                    s => {
                                        renderer::do_render_overlay(&mut stdout, e, geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale, &s, None)?;
                                        tui_state = TuiState::GameOver(s);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    TuiState::Help => {
                        tui_state = TuiState::Playing;
                        renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                    }
                    TuiState::WatchingAd { ref started_at, .. } => {
                        match event.code {
                            KeyCode::Esc => {
                                if started_at.elapsed().as_secs() >= AD_DURATION_SECS {
                                    engine.as_mut().unwrap().watch_ad();
                                    let status = engine.as_ref().unwrap().status();
                                    tui_state = match status {
                                        GameStatus::Playing => TuiState::Playing,
                                        _ => TuiState::GameOver(status),
                                    };
                                    renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                }
                            }
                            _ => {}
                        }
                    }
                    TuiState::Playing => {
                        match event.code {
                            KeyCode::Left  => { let _ = engine.as_mut().unwrap().move_cursor(Direction::Left);  }
                            KeyCode::Right => { let _ = engine.as_mut().unwrap().move_cursor(Direction::Right); }
                            KeyCode::Up    => { let _ = engine.as_mut().unwrap().move_cursor(Direction::Up);    }
                            KeyCode::Down  => { let _ = engine.as_mut().unwrap().move_cursor(Direction::Down);  }
                            KeyCode::Esc => {
                                if engine.as_ref().unwrap().bonus_state != BonusState::None {
                                    engine.as_mut().unwrap().cancel_tweezers();
                                } else {
                                    if campaign_ctx.is_some() {
                                        campaign_saves.upsert(campaign_ctx.as_ref().unwrap().clone());
                                        campaign_saves.save("knitui");
                                        campaign_ctx = None;
                                    }
                                    tui_state = TuiState::MainMenu { selected: 0, flash: None };
                                    renderer::render_main_menu(&mut stdout, 0, None)?;
                                    continue;
                                }
                            }

                            KeyCode::Enter => {
                                if engine.as_mut().unwrap().pick_up().is_ok() {
                                    match engine.as_ref().unwrap().status() {
                                        GameStatus::Playing => {}
                                        GameStatus::Won if endless_ctx.is_some() => {
                                            advance_endless_wave(&mut endless_ctx, &mut game_config, &cli_config, &mut geo, &mut engine);
                                            renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                                            continue;
                                        }
                                        s => {
                                            if endless_ctx.is_some() && s == GameStatus::Stuck {
                                                let wave = endless_ctx.as_ref().unwrap().wave;
                                                endless_hs.update(wave);
                                                endless_hs.save("knitui");
                                                renderer::render_endless_gameover(&mut stdout, wave, endless_hs.best_wave)?;
                                            } else {
                                                let overlay = campaign_overlay_msg(&campaign_ctx, &s);
                                                renderer::do_render_overlay(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale, &s, overlay.as_deref())?;
                                            }
                                            tui_state = TuiState::GameOver(s);
                                            continue;
                                        }
                                    };
                                }
                            }

                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if engine.as_ref().unwrap().can_watch_ad() {
                                    let quote = ad_content::random_quote(&ad_quotes).to_string();
                                    tui_state = TuiState::WatchingAd {
                                        started_at: Instant::now(),
                                        quote,
                                    };
                                }
                                continue;
                            }
                            KeyCode::Char('h') | KeyCode::Char('H') => {
                                renderer::render_help(&mut stdout)?;
                                tui_state = TuiState::Help;
                                continue;
                            }
                            KeyCode::Char('z') | KeyCode::Char('Z') => {
                                let _ = engine.as_mut().unwrap().use_scissors();
                            }
                            KeyCode::Char('x') | KeyCode::Char('X') => {
                                let _ = engine.as_mut().unwrap().use_tweezers();
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                let _ = engine.as_mut().unwrap().use_balloons();
                            }

                            _ => { continue; }
                        }

                        renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                    }
                }
            }
        } else if matches!(tui_state, TuiState::Playing) && !engine.as_ref().unwrap().held_spools.is_empty() {
            engine.as_mut().unwrap().process_all_active();
            match engine.as_ref().unwrap().status() {
                GameStatus::Playing => renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?,
                GameStatus::Won if endless_ctx.is_some() => {
                    advance_endless_wave(&mut endless_ctx, &mut game_config, &cli_config, &mut geo, &mut engine);
                    renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                }
                s => {
                    if endless_ctx.is_some() && s == GameStatus::Stuck {
                        let wave = endless_ctx.as_ref().unwrap().wave;
                        endless_hs.update(wave);
                        endless_hs.save("knitui");
                        renderer::render_endless_gameover(&mut stdout, wave, endless_hs.best_wave)?;
                    } else {
                        let overlay = campaign_overlay_msg(&campaign_ctx, &s);
                        renderer::do_render_overlay(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale, &s, overlay.as_deref())?;
                    }
                    tui_state = TuiState::GameOver(s);
                }
            };
        }

        if let TuiState::WatchingAd { ref started_at, ref quote } = tui_state {
            renderer::render_ad_overlay(&mut stdout, quote, started_at, AD_DURATION_SECS)?;
        }
    }

    execute!(stdout, LeaveAlternateScreen);
    disable_raw_mode()?;
    Ok(())
}
