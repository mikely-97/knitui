#![allow(warnings)]

use std::io::{Write, stdout};
use std::time::Duration;

use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode, KeyModifiers},
};
use clap::Parser;

use crate::bonuses::BonusState;
use crate::campaign::{CampaignSaves, CampaignState, objective_met};
use crate::campaign_levels::{TRACK_NAMES, TRACK_COUNT};
use crate::config::Config;
use crate::endless::{EndlessHighScore, EndlessState};
use crate::engine::{GameEngine, GamePhase, GameStatus};
use crate::preset::PRESETS;
use crate::renderer::{self, LayoutGeometry};
use crate::settings::{self, UserSettings};

// ── TuiState ──────────────────────────────────────────────────────────────

enum TuiState {
    MainMenu {
        selected: usize,
        flash: Option<String>,
    },
    Playing,
    CustomGame {
        preset_idx: usize,
        selected_field: usize,
        config: Config,
    },
    CampaignSelect {
        selected: usize,
    },
    CampaignLevelIntro,
    GameOver {
        status: GameStatus,
        can_retry: bool,
    },
    Help,
    Options {
        selected: usize,
    },
}

// ── Layout helper ─────────────────────────────────────────────────────────

fn make_geo(config: &Config) -> LayoutGeometry {
    LayoutGeometry::compute(
        config.board_height as usize,
        config.board_width as usize,
        config.scale,
    )
}

// ── Objective label ───────────────────────────────────────────────────────

fn objective_label_for(engine: &GameEngine, campaign_ctx: &Option<CampaignState>) -> String {
    if let Some(ctx) = campaign_ctx {
        let def = ctx.current_level_def();
        let mut parts = Vec::new();
        if let Some(target) = def.objective.score_target {
            parts.push(format!("Score: {}/{}", engine.score, target));
        }
        if !def.objective.gem_quota.is_empty() {
            parts.push("Quota: see HUD".to_string());
        }
        if def.objective.clear_all_specials {
            let remaining = engine.board.count_modifier(|_| true);
            parts.push(format!("Tiles left: {}", remaining));
        }
        parts.join("  ")
    } else {
        String::new()
    }
}

// ── Entry points ──────────────────────────────────────────────────────────

/// Run the m3 game from the standalone binary (parses CLI args).
pub fn run_cli() -> std::io::Result<()> {
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

    let cli_config = Config::parse();
    let user_settings = UserSettings::load();
    let campaign_saves = CampaignSaves::<CampaignState>::load("m3tui");
    let endless_hs = EndlessHighScore::load("m3tui");

    let mut game_config = cli_config.clone();
    game_config.scale = user_settings.scale;
    game_config.color_mode = user_settings.color_mode.clone();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    enable_raw_mode()?;

    let result = run_loop(
        &mut stdout,
        cli_config,
        game_config,
        user_settings,
        campaign_saves,
        endless_hs,
    );

    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;

    result
}

/// Run the m3 game from the game selector (default config, always shows menu).
pub fn run_from_menu() -> std::io::Result<()> {
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

    let user_settings = UserSettings::load();
    let campaign_saves = CampaignSaves::<CampaignState>::load("m3tui");
    let endless_hs = EndlessHighScore::load("m3tui");

    let mut config = Config::parse_from::<[&str; 0], &str>([]);
    config.scale = user_settings.scale;
    config.color_mode = user_settings.color_mode.clone();

    let game_config = config.clone();

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;
    enable_raw_mode()?;

    let result = run_loop(
        &mut stdout,
        config,
        game_config,
        user_settings,
        campaign_saves,
        endless_hs,
    );

    disable_raw_mode()?;
    execute!(stdout, Show, LeaveAlternateScreen)?;

    result
}

// ── Event loop ────────────────────────────────────────────────────────────

fn run_loop(
    stdout: &mut std::io::Stdout,
    cli_config: Config,
    mut game_config: Config,
    mut user_settings: UserSettings,
    mut campaign_saves: CampaignSaves<CampaignState>,
    mut endless_hs: EndlessHighScore,
) -> std::io::Result<()> {
    let mut tui_state = TuiState::MainMenu { selected: 0, flash: None };
    let mut engine: Option<GameEngine> = None;
    let mut geo = make_geo(&game_config);
    let mut campaign_ctx: Option<CampaignState> = None;
    let mut endless_ctx: Option<EndlessState> = None;

    // Initial render
    renderer::render_main_menu(stdout, 0, None)?;
    stdout.flush()?;

    loop {
        // ── Advance non-input phases ──────────────────────────────────────
        if matches!(tui_state, TuiState::Playing) {
            if let Some(ref mut eng) = engine {
                let changed = eng.tick();
                if changed {
                    let label = objective_label_for(eng, &campaign_ctx);
                    renderer::do_render(stdout, eng, &geo, &label)?;
                }

                // Check end-of-turn status after tick
                if matches!(eng.phase, GamePhase::PlayerInput) {
                    // Campaign win check
                    if let Some(ref mut ctx) = campaign_ctx {
                        let def = ctx.current_level_def();
                        let special_remaining = eng.board.count_modifier(|_| true);
                        if objective_met(&def.objective, eng.score, &[], special_remaining) {
                            let done = ctx.complete_level();
                            campaign_saves.upsert(ctx.clone());
                            campaign_saves.save("m3tui");
                            if done {
                                tui_state = TuiState::GameOver { status: GameStatus::Won, can_retry: false };
                            } else {
                                tui_state = TuiState::CampaignLevelIntro;
                            }
                        }
                    } else if let Some(ref mut ctx) = endless_ctx {
                        // Endless wave completion
                        if eng.moves_used >= eng.move_limit && eng.score > 0 {
                            ctx.advance();
                            endless_hs.update(ctx.wave);
                            endless_hs.save("m3tui");
                            game_config = ctx.to_config(&cli_config);
                            geo = make_geo(&game_config);
                            engine = Some(GameEngine::new(&game_config));
                        } else {
                            let status = eng.game_status();
                            match status {
                                GameStatus::Stuck => {
                                    if eng.bonuses.warp > 0 {
                                        eng.activate_warp();
                                    } else {
                                        tui_state = TuiState::GameOver { status: status.clone(), can_retry: true };
                                        if let Some(ref eng2) = engine {
                                            renderer::render_game_over(stdout, &status, eng2.score)?;
                                            stdout.flush()?;
                                        }
                                    }
                                }
                                GameStatus::OutOfMoves => {
                                    tui_state = TuiState::GameOver { status: status.clone(), can_retry: true };
                                    if let Some(ref eng2) = engine {
                                        renderer::render_game_over(stdout, &status, eng2.score)?;
                                        stdout.flush()?;
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else {
                        // Quick / Custom game
                        let status = eng.game_status();
                        match status {
                            GameStatus::OutOfMoves | GameStatus::Stuck => {
                                tui_state = TuiState::GameOver { status: status.clone(), can_retry: true };
                                renderer::render_game_over(stdout, &status, eng.score)?;
                                stdout.flush()?;
                            }
                            GameStatus::Playing | GameStatus::Won => {}
                        }
                    }
                }
            }
        }

        // ── Event handling ────────────────────────────────────────────────
        if !poll(Duration::from_millis(50))? {
            continue;
        }

        let Event::Key(key) = read()? else { continue };

        // Global: Ctrl+C or Ctrl+Q always exits
        if (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('q'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            break;
        }

        match tui_state {
            // ── Main menu ─────────────────────────────────────────────────
            TuiState::MainMenu { ref mut selected, ref mut flash } => {
                *flash = None;
                match key.code {
                    KeyCode::Up   => { if *selected > 0 { *selected -= 1; } }
                    KeyCode::Down => { if *selected < 5  { *selected += 1; } }
                    KeyCode::Enter => {
                        match *selected {
                            0 => { // Quick Game
                                game_config = cli_config.clone();
                                game_config.scale = user_settings.scale;
                                game_config.color_mode = user_settings.color_mode.clone();
                                geo = make_geo(&game_config);
                                engine = Some(GameEngine::new(&game_config));
                                campaign_ctx = None;
                                endless_ctx = None;
                                tui_state = TuiState::Playing;
                            }
                            1 => { // Custom Game
                                let preset_cfg = PRESETS[1].to_config(&cli_config);
                                tui_state = TuiState::CustomGame {
                                    preset_idx: 1,
                                    selected_field: 0,
                                    config: preset_cfg,
                                };
                            }
                            2 => { // Campaign
                                tui_state = TuiState::CampaignSelect { selected: 0 };
                            }
                            3 => { // Endless
                                let state = EndlessState::new();
                                game_config = state.to_config(&cli_config);
                                game_config.scale = user_settings.scale;
                                game_config.color_mode = user_settings.color_mode.clone();
                                geo = make_geo(&game_config);
                                engine = Some(GameEngine::new(&game_config));
                                endless_ctx = Some(state);
                                campaign_ctx = None;
                                tui_state = TuiState::Playing;
                            }
                            4 => { // Options
                                tui_state = TuiState::Options { selected: 0 };
                            }
                            5 => break, // Quit
                            _ => {}
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    _ => {}
                }
                render_current_state(stdout, &tui_state, engine.as_ref(), &geo, &campaign_ctx, &campaign_saves, &user_settings)?;
            }

            // ── Playing ───────────────────────────────────────────────────
            TuiState::Playing => {
                if let Some(ref mut eng) = engine {
                    let bonus_active = !matches!(eng.bonus_state, BonusState::None);

                    match key.code {
                        KeyCode::Up    => { eng.move_cursor(-1, 0); }
                        KeyCode::Down  => { eng.move_cursor(1, 0); }
                        KeyCode::Left  => { eng.move_cursor(0, -1); }
                        KeyCode::Right => { eng.move_cursor(0, 1); }

                        KeyCode::Enter => {
                            if matches!(eng.bonus_state, BonusState::HammerActive { .. }) {
                                eng.confirm_hammer();
                            } else if matches!(eng.phase, GamePhase::PlayerInput) {
                                eng.confirm_selection();
                            }
                        }

                        KeyCode::Esc => {
                            if bonus_active {
                                eng.cancel_bonus();
                            } else if eng.selected.is_some() {
                                eng.selected = None;
                            } else {
                                tui_state = TuiState::MainMenu { selected: 0, flash: None };
                                renderer::render_main_menu(stdout, 0, None)?;
                                stdout.flush()?;
                                continue;
                            }
                        }

                        KeyCode::Char('h') | KeyCode::Char('H') => {
                            if !bonus_active { tui_state = TuiState::Help; }
                        }
                        KeyCode::Char('z') | KeyCode::Char('Z') => {
                            // Hammer
                            if !bonus_active && eng.bonuses.hammer > 0 {
                                eng.activate_hammer();
                            }
                        }
                        KeyCode::Char('x') | KeyCode::Char('X') => {
                            // Laser
                            if !bonus_active && eng.bonuses.laser > 0 {
                                eng.activate_laser();
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            // Blaster
                            if !bonus_active && eng.bonuses.blaster > 0 {
                                eng.activate_blaster();
                            }
                        }
                        KeyCode::Char('v') | KeyCode::Char('V') => {
                            // Warp
                            if !bonus_active && eng.bonuses.warp > 0 {
                                eng.activate_warp();
                            }
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') => {
                            tui_state = TuiState::MainMenu { selected: 0, flash: None };
                            renderer::render_main_menu(stdout, 0, None)?;
                            stdout.flush()?;
                            continue;
                        }
                        _ => {}
                    }

                    // Re-render after input (only if still Playing)
                    if matches!(tui_state, TuiState::Playing) {
                        let label = objective_label_for(eng, &campaign_ctx);
                        renderer::do_render(stdout, eng, &geo, &label)?;
                    } else if matches!(tui_state, TuiState::Help) {
                        renderer::render_help(stdout)?;
                    }
                    stdout.flush()?;
                }
            }

            // ── Game over ─────────────────────────────────────────────────
            TuiState::GameOver { ref status, can_retry } => {
                match key.code {
                    KeyCode::Char('r') | KeyCode::Char('R') if can_retry => {
                        // Retry: restart same config
                        engine = Some(GameEngine::new(&game_config));
                        tui_state = TuiState::Playing;
                        if let Some(ref eng) = engine {
                            let label = objective_label_for(eng, &campaign_ctx);
                            renderer::do_render(stdout, eng, &geo, &label)?;
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 0, flash: None };
                        renderer::render_main_menu(stdout, 0, None)?;
                        stdout.flush()?;
                    }
                    _ => {}
                }
            }

            // ── Help ──────────────────────────────────────────────────────
            TuiState::Help => {
                tui_state = TuiState::Playing;
                if let Some(ref eng) = engine {
                    let label = objective_label_for(eng, &campaign_ctx);
                    renderer::do_render(stdout, eng, &geo, &label)?;
                }
            }

            // ── Campaign select ───────────────────────────────────────────
            TuiState::CampaignSelect { ref mut selected } => {
                match key.code {
                    KeyCode::Up   => { if *selected > 0 { *selected -= 1; } }
                    KeyCode::Down => { if *selected < TRACK_COUNT - 1 { *selected += 1; } }
                    KeyCode::Enter => {
                        let track_idx = *selected;
                        let ctx = campaign_saves
                            .get(track_idx)
                            .cloned()
                            .unwrap_or_else(|| CampaignState::new(track_idx));
                        campaign_ctx = Some(ctx);
                        tui_state = TuiState::CampaignLevelIntro;
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 2, flash: None };
                    }
                    _ => {}
                }
                render_current_state(stdout, &tui_state, engine.as_ref(), &geo, &campaign_ctx, &campaign_saves, &user_settings)?;
            }

            // ── Campaign level intro ──────────────────────────────────────
            TuiState::CampaignLevelIntro => {
                match key.code {
                    KeyCode::Enter => {
                        if let Some(ref ctx) = campaign_ctx {
                            game_config = ctx.to_config(&cli_config);
                            game_config.scale = user_settings.scale;
                            game_config.color_mode = user_settings.color_mode.clone();
                            geo = make_geo(&game_config);
                            engine = Some(GameEngine::new(&game_config));
                            tui_state = TuiState::Playing;
                            let eng = engine.as_ref().unwrap();
                            let label = objective_label_for(eng, &campaign_ctx);
                            renderer::do_render(stdout, eng, &geo, &label)?;
                            continue;
                        }
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::CampaignSelect { selected: 0 };
                    }
                    _ => {}
                }
                render_current_state(stdout, &tui_state, engine.as_ref(), &geo, &campaign_ctx, &campaign_saves, &user_settings)?;
            }

            // ── Custom game config ────────────────────────────────────────
            TuiState::CustomGame { ref mut preset_idx, ref mut selected_field, ref mut config } => {
                match key.code {
                    KeyCode::Up => {
                        if *selected_field > 0 { *selected_field -= 1; }
                    }
                    KeyCode::Down => {
                        if *selected_field < custom_field_count() - 1 { *selected_field += 1; }
                    }
                    KeyCode::Left => {
                        adjust_custom_field(config, *selected_field, -1);
                    }
                    KeyCode::Right => {
                        adjust_custom_field(config, *selected_field, 1);
                    }
                    KeyCode::Enter => {
                        game_config = config.clone();
                        game_config.scale = user_settings.scale;
                        game_config.color_mode = user_settings.color_mode.clone();
                        geo = make_geo(&game_config);
                        engine = Some(GameEngine::new(&game_config));
                        campaign_ctx = None;
                        endless_ctx = None;
                        tui_state = TuiState::Playing;
                        let eng = engine.as_ref().unwrap();
                        renderer::do_render(stdout, eng, &geo, "")?;
                        continue;
                    }
                    KeyCode::Esc => {
                        tui_state = TuiState::MainMenu { selected: 1, flash: None };
                    }
                    _ => {}
                }
                render_current_state(stdout, &tui_state, engine.as_ref(), &geo, &campaign_ctx, &campaign_saves, &user_settings)?;
            }

            // ── Options ───────────────────────────────────────────────────
            TuiState::Options { ref mut selected } => {
                match key.code {
                    KeyCode::Up   => { if *selected > 0 { *selected -= 1; } }
                    KeyCode::Down => { if *selected < 2 { *selected += 1; } }
                    KeyCode::Left => {
                        match *selected {
                            0 => { if user_settings.scale > 1 { user_settings.scale -= 1; } }
                            1 => { user_settings.color_mode = settings::prev_color_mode(&user_settings.color_mode).to_string(); }
                            _ => {}
                        }
                    }
                    KeyCode::Right => {
                        match *selected {
                            0 => { if user_settings.scale < 5 { user_settings.scale += 1; } }
                            1 => { user_settings.color_mode = settings::next_color_mode(&user_settings.color_mode).to_string(); }
                            _ => {}
                        }
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        user_settings.save();
                        tui_state = TuiState::MainMenu { selected: 4, flash: None };
                    }
                    _ => {}
                }
                match tui_state {
                    TuiState::Options { selected } => {
                        renderer::render_options(stdout, selected, user_settings.scale, &user_settings.color_mode)?;
                        stdout.flush()?;
                    }
                    _ => {
                        render_current_state(stdout, &tui_state, engine.as_ref(), &geo, &campaign_ctx, &campaign_saves, &user_settings)?;
                    }
                }
            }
        }
    }

    Ok(())
}

// ── Helper: custom field count and adjustment ─────────────────────────────

fn custom_field_count() -> usize { 5 }

fn custom_fields(config: &Config) -> Vec<(&'static str, u16)> {
    vec![
        ("Board Height",    config.board_height),
        ("Board Width",     config.board_width),
        ("Color Count",     config.color_number as u16),
        ("Move Limit",      config.move_limit as u16),
        ("Special Tile %",  config.special_tile_pct),
    ]
}

fn adjust_custom_field(config: &mut Config, field: usize, delta: i32) {
    match field {
        0 => config.board_height     = (config.board_height     as i32 + delta).clamp(4, 16) as u16,
        1 => config.board_width      = (config.board_width      as i32 + delta).clamp(4, 16) as u16,
        2 => config.color_number     = (config.color_number     as i32 + delta).clamp(3, 7)  as u8,
        3 => config.move_limit       = (config.move_limit       as i32 + delta).clamp(10, 99) as u32,
        4 => config.special_tile_pct = (config.special_tile_pct as i32 + delta).clamp(0, 50) as u16,
        _ => {}
    }
}

// ── State-specific renders ────────────────────────────────────────────────

fn render_current_state(
    stdout: &mut std::io::Stdout,
    state: &TuiState,
    engine: Option<&GameEngine>,
    geo: &LayoutGeometry,
    campaign_ctx: &Option<CampaignState>,
    campaign_saves: &CampaignSaves<CampaignState>,
    user_settings: &UserSettings,
) -> std::io::Result<()> {
    match state {
        TuiState::MainMenu { selected, flash } => {
            renderer::render_main_menu(stdout, *selected, flash.as_deref())?;
        }
        TuiState::Playing => {
            if let Some(eng) = engine {
                let label = objective_label_for(eng, campaign_ctx);
                renderer::do_render(stdout, eng, geo, &label)?;
            }
        }
        TuiState::Help => {
            renderer::render_help(stdout)?;
        }
        TuiState::GameOver { status, .. } => {
            if let Some(eng) = engine {
                renderer::render_game_over(stdout, status, eng.score)?;
            }
        }
        TuiState::CampaignSelect { selected } => {
            render_campaign_select(stdout, *selected, campaign_saves)?;
        }
        TuiState::CampaignLevelIntro => {
            render_level_intro(stdout, campaign_ctx)?;
        }
        TuiState::CustomGame { preset_idx, selected_field, config } => {
            render_custom_game(stdout, *preset_idx, *selected_field, config)?;
        }
        TuiState::Options { selected } => {
            renderer::render_options(stdout, *selected, user_settings.scale, &user_settings.color_mode)?;
        }
    }
    stdout.flush()?;
    Ok(())
}

fn render_campaign_select(
    stdout: &mut std::io::Stdout,
    selected: usize,
    saves: &CampaignSaves<CampaignState>,
) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, style::Print, cursor::MoveTo, terminal::{Clear, ClearType}};
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(2, 0))?;
    stdout.queue(Print("SELECT CAMPAIGN TRACK"))?;
    stdout.queue(MoveTo(2, 1))?;
    stdout.queue(Print("──────────────────────────────"))?;

    for (i, name) in TRACK_NAMES.iter().enumerate() {
        let progress = saves.progress_label(i);
        let line = format!("{:12}  {}", name, progress);
        stdout.queue(MoveTo(2, 3 + i as u16))?;
        if i == selected {
            use crossterm::style::Stylize;
            stdout.queue(Print(format!("► {}", line).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", line)))?;
        }
    }

    stdout.queue(MoveTo(2, 3 + TRACK_COUNT as u16 + 1))?;
    stdout.queue(Print("Enter select  Esc back"))?;
    Ok(())
}

fn render_level_intro(
    stdout: &mut std::io::Stdout,
    campaign_ctx: &Option<CampaignState>,
) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, style::Print, cursor::MoveTo, terminal::{Clear, ClearType}};
    stdout.queue(Clear(ClearType::All))?;

    let Some(ctx) = campaign_ctx else { return Ok(()); };
    let def = ctx.current_level_def();

    stdout.queue(MoveTo(2, 2))?;
    stdout.queue(Print(format!(
        "Track: {}  Level {}/{}",
        TRACK_NAMES[ctx.track_idx],
        ctx.current_level + 1,
        ctx.total_levels(),
    )))?;

    stdout.queue(MoveTo(2, 4))?;
    stdout.queue(Print(format!("Board: {}x{}  Colors: {}  Moves: {}",
        def.board_height, def.board_width,
        def.color_number, def.move_limit,
    )))?;

    stdout.queue(MoveTo(2, 6))?;
    let obj_str = if let Some(target) = def.objective.score_target {
        format!("Goal: reach {} points", target)
    } else if def.objective.clear_all_specials {
        "Goal: clear all special tiles".to_string()
    } else {
        "Goal: collect gem quota".to_string()
    };
    stdout.queue(Print(obj_str))?;

    stdout.queue(MoveTo(2, 8))?;
    stdout.queue(Print("Press Enter to start, Esc to go back"))?;

    Ok(())
}

fn render_custom_game(
    stdout: &mut std::io::Stdout,
    preset_idx: usize,
    selected_field: usize,
    config: &Config,
) -> std::io::Result<()> {
    use crossterm::{QueueableCommand, style::{Print, Stylize}, cursor::MoveTo, terminal::{Clear, ClearType}};
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(2, 0))?;
    stdout.queue(Print("CUSTOM GAME"))?;

    let fields = custom_fields(config);
    for (i, (name, value)) in fields.iter().enumerate() {
        let line = format!("{:<16} {}", name, value);
        stdout.queue(MoveTo(2, 2 + i as u16))?;
        if i == selected_field {
            stdout.queue(Print(format!("► {}", line).negative()))?;
        } else {
            stdout.queue(Print(format!("  {}", line)))?;
        }
    }

    stdout.queue(MoveTo(2, 2 + fields.len() as u16 + 1))?;
    stdout.queue(Print("← → change value  Enter start  Esc back"))?;
    Ok(())
}
