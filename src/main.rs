#![allow(warnings)]

use std::io::{Write, stdout};
use std::time::{Duration, Instant};

use crossterm::{
    ExecutableCommand, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode},
};

use clap::Parser;

use knitui::ad_content;
use knitui::board_entity::Direction;
use knitui::config::Config;
use knitui::engine::{GameEngine, GameStatus, BonusState};
use knitui::renderer::{self, Layout, COMP_GAP, YARN_HGAP, YARN_VGAP};

enum TuiState {
    Playing,
    GameOver(GameStatus),
    Help,
    WatchingAd { started_at: Instant, quote: String },
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let config = Config::parse();
    let ad_quotes = ad_content::load_quotes(&config.ad_file);
    const AD_DURATION_SECS: u64 = 15;
    let scale = config.scale;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let sh = scale;
    let sw = scale * 2;

    let layout = renderer::detect_layout(&config.layout, config.visible_patches, config.board_height, scale);

    // Vertical layout offsets
    let yarn_h = config.visible_patches * sh + config.visible_patches.saturating_sub(1) * YARN_VGAP;
    let board_y: u16 = yarn_h + COMP_GAP + sh + COMP_GAP; // yarn + gap + active row + gap

    // Horizontal layout offsets — reserve flanking columns for balloon patches
    let yarn_w = config.yarn_lines * sw + config.yarn_lines.saturating_sub(1) * YARN_HGAP;
    let has_flanks = config.balloons > 0 && config.balloon_count > 0;
    let (yarn_x, board_x) = if has_flanks {
        let has_left  = config.balloon_count / 2 > 0;
        let has_right = (config.balloon_count + 1) / 2 > 0;
        let left_w  = if has_left  { sw } else { 0 };  // single column width
        let right_w = if has_right { sw } else { 0 };
        let left_gap  = if has_left  { YARN_HGAP } else { 0 };
        let right_gap = if has_right { YARN_HGAP } else { 0 };
        let yx = left_w + left_gap;
        let bx = yx + yarn_w + right_gap + right_w + COMP_GAP + sw + COMP_GAP;
        (yx, bx)
    } else {
        (0u16, yarn_w + COMP_GAP + sw + COMP_GAP)
    };

    let mut engine = GameEngine::new(&config);
    let mut tui_state = TuiState::Playing;

    renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?;

    loop {
        if poll(Duration::from_millis(150))? {
            if let Event::Key(event) = read()? {
                match tui_state {
                    TuiState::GameOver(_) => {
                        match event.code {
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if engine.can_watch_ad() {
                                    let quote = ad_content::random_quote(&ad_quotes).to_string();
                                    tui_state = TuiState::WatchingAd {
                                        started_at: Instant::now(),
                                        quote,
                                    };
                                }
                            }
                            KeyCode::Char('r') | KeyCode::Char('R') => {
                                engine = GameEngine::new(&config);
                                tui_state = TuiState::Playing;
                                renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?;
                            }
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    TuiState::Help => {
                        tui_state = TuiState::Playing;
                        renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?;
                    }
                    TuiState::WatchingAd { ref started_at, .. } => {
                        match event.code {
                            KeyCode::Esc => {
                                if started_at.elapsed().as_secs() >= AD_DURATION_SECS {
                                    engine.watch_ad();
                                    let status = engine.status();
                                    tui_state = match status {
                                        GameStatus::Playing => TuiState::Playing,
                                        _ => TuiState::GameOver(status),
                                    };
                                    renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?;
                                }
                                // If timer not done, ignore ESC
                            }
                            _ => {}
                        }
                    }
                    TuiState::Playing => {
                        match event.code {
                            KeyCode::Left  => { let _ = engine.move_cursor(Direction::Left);  }
                            KeyCode::Right => { let _ = engine.move_cursor(Direction::Right); }
                            KeyCode::Up    => { let _ = engine.move_cursor(Direction::Up);    }
                            KeyCode::Down  => { let _ = engine.move_cursor(Direction::Down);  }
                            KeyCode::Esc => {
                                if engine.bonus_state != BonusState::None {
                                    engine.cancel_tweezers();
                                } else {
                                    break;
                                }
                            }

                            KeyCode::Enter => {
                                if engine.pick_up().is_ok() {
                                    match engine.status() {
                                        GameStatus::Playing => {}
                                        s => {
                                            renderer::do_render_overlay(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale, &s)?;
                                            tui_state = TuiState::GameOver(s);
                                            continue;
                                        }
                                    };
                                }
                            }

                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if engine.can_watch_ad() {
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
                                let _ = engine.use_scissors();
                            }
                            KeyCode::Char('x') | KeyCode::Char('X') => {
                                let _ = engine.use_tweezers();
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                let _ = engine.use_balloons();
                            }

                            _ => { continue; }
                        }

                        // Re-render to update bracket cursor markers
                        renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?;
                    }
                }
            }
        } else if matches!(tui_state, TuiState::Playing) && !engine.active_threads.is_empty() {
            engine.process_all_active();
            match engine.status() {
                GameStatus::Playing => renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?,
                s => {
                    renderer::do_render_overlay(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale, &s)?;
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
