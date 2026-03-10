#![allow(warnings)]

use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode},
};

use clap::{CommandFactory, Parser, parser::ValueSource};

use knitui::ad_content;
use knitui::board_entity::Direction;
use knitui::config::Config;
use knitui::engine::{GameEngine, GameStatus, BonusState};
use knitui::preset::PRESETS;
use knitui::renderer::{self, Layout, COMP_GAP, YARN_HGAP, YARN_VGAP};

enum TuiState {
    MainMenu { selected: usize, flash: Option<String> },
    CustomGame {
        preset_idx: usize,
        selected_field: usize, // 0 = preset row, 1-9 = fields
        config: Config,
    },
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
            &config.layout, config.visible_patches, config.board_height, scale,
        );

        // Vertical layout offsets
        let yarn_h = config.visible_patches * sh
            + config.visible_patches.saturating_sub(1) * YARN_VGAP;
        let board_y: u16 = yarn_h + COMP_GAP + sh + COMP_GAP;

        // Horizontal layout offsets — reserve flanking columns for balloon patches
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
        ("Generator %", config.generator_percentage),
        ("Scale", config.scale),
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
        1 => apply(&mut config.board_height, 2, 20),
        2 => apply(&mut config.board_width, 2, 20),
        3 => apply(&mut config.color_number, 2, 8),
        4 => apply(&mut config.obstacle_percentage, 0, 50),
        5 => apply(&mut config.generator_percentage, 0, 50),
        6 => apply(&mut config.scale, 1, 5),
        7 => apply(&mut config.scissors, 0, 99),
        8 => apply(&mut config.tweezers, 0, 99),
        9 => apply(&mut config.balloons, 0, 99),
        _ => {}
    }
}

/// Game-specific args: if any were explicitly passed, skip the menu.
const GAME_ARGS: &[&str] = &[
    "board_height", "board_width", "color_number",
    "obstacle_percentage", "generator_percentage",
    "scissors", "tweezers", "balloons",
];

fn has_game_args() -> bool {
    let matches = Config::command().get_matches_from(std::env::args_os());
    GAME_ARGS.iter().any(|name| {
        matches.value_source(name) == Some(ValueSource::CommandLine)
    })
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let skip_menu = has_game_args();
    let cli_config = Config::parse();
    let ad_quotes = ad_content::load_quotes(&cli_config.ad_file);
    const AD_DURATION_SECS: u64 = 15;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

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
                        *flash = None; // Clear flash on any keypress
                        match event.code {
                            KeyCode::Up => {
                                if *selected > 0 { *selected -= 1; }
                            }
                            KeyCode::Down => {
                                if *selected < 4 { *selected += 1; }
                            }
                            KeyCode::Enter => {
                                match *selected {
                                    0 => {
                                        // Quick Game — use CLI defaults
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
                                        // Custom Game — start with Medium preset
                                        let preset_cfg = PRESETS[1].to_config(&cli_config);
                                        tui_state = TuiState::CustomGame {
                                            preset_idx: 1,
                                            selected_field: 0,
                                            config: preset_cfg,
                                        };
                                    }
                                    2 | 3 => {
                                        // Campaign / Endless — coming soon
                                        *flash = Some("Coming soon!".to_string());
                                    }
                                    4 => break, // Quit
                                    _ => {}
                                }
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            _ => {}
                        }
                        // Re-render menu (or transition to custom game screen)
                        if let TuiState::MainMenu { selected, ref flash } = tui_state {
                            renderer::render_main_menu(
                                &mut stdout, selected, flash.as_deref(),
                            )?;
                        } else if let TuiState::CustomGame { preset_idx, selected_field, ref config } = tui_state {
                            let fields = custom_game_fields(config);
                            renderer::render_custom_game(
                                &mut stdout, PRESETS[preset_idx].name, &fields, selected_field,
                            )?;
                        }
                    }
                    TuiState::CustomGame { ref mut preset_idx, ref mut selected_field, ref mut config } => {
                        match event.code {
                            KeyCode::Up => {
                                if *selected_field > 0 { *selected_field -= 1; }
                            }
                            KeyCode::Down => {
                                if *selected_field < 9 { *selected_field += 1; }
                            }
                            KeyCode::Left => {
                                if *selected_field == 0 {
                                    // Cycle preset backward
                                    if *preset_idx > 0 { *preset_idx -= 1; }
                                    else { *preset_idx = PRESETS.len() - 1; }
                                    *config = PRESETS[*preset_idx].to_config(&cli_config);
                                } else {
                                    adjust_custom_field(config, *selected_field, -1);
                                }
                            }
                            KeyCode::Right => {
                                if *selected_field == 0 {
                                    // Cycle preset forward
                                    *preset_idx = (*preset_idx + 1) % PRESETS.len();
                                    *config = PRESETS[*preset_idx].to_config(&cli_config);
                                } else {
                                    adjust_custom_field(config, *selected_field, 1);
                                }
                            }
                            KeyCode::Enter => {
                                // Start game with custom config
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
                        // Re-render custom game screen
                        if let TuiState::CustomGame { preset_idx, selected_field, ref config } = tui_state {
                            let fields = custom_game_fields(config);
                            renderer::render_custom_game(
                                &mut stdout, PRESETS[preset_idx].name, &fields, selected_field,
                            )?;
                        }
                    }
                    TuiState::GameOver(_) => {
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
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                engine = Some(GameEngine::new(&game_config));
                                tui_state = TuiState::Playing;
                                renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') => {
                                tui_state = TuiState::MainMenu { selected: 0, flash: None };
                                renderer::render_main_menu(&mut stdout, 0, None)?;
                                continue;
                            }
                            KeyCode::Esc => {
                                tui_state = TuiState::MainMenu { selected: 0, flash: None };
                                renderer::render_main_menu(&mut stdout, 0, None)?;
                                continue;
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') => break,
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
                                // If timer not done, ignore ESC
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
                                    tui_state = TuiState::MainMenu { selected: 0, flash: None };
                                    renderer::render_main_menu(&mut stdout, 0, None)?;
                                    continue;
                                }
                            }

                            KeyCode::Enter => {
                                if engine.as_mut().unwrap().pick_up().is_ok() {
                                    match engine.as_ref().unwrap().status() {
                                        GameStatus::Playing => {}
                                        s => {
                                            renderer::do_render_overlay(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale, &s)?;
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

                        // Re-render to update bracket cursor markers
                        renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?;
                    }
                }
            }
        } else if matches!(tui_state, TuiState::Playing) && !engine.as_ref().unwrap().active_threads.is_empty() {
            engine.as_mut().unwrap().process_all_active();
            match engine.as_ref().unwrap().status() {
                GameStatus::Playing => renderer::do_render(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale)?,
                s => {
                    renderer::do_render_overlay(&mut stdout, engine.as_ref().unwrap(), geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale, &s)?;
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
