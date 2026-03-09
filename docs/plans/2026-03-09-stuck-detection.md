# Stuck Detection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Detect when the player is in an unwinnable deadlock and show "You're lost!" with restart/quit options.

**Architecture:** Add `GameStatus` enum to the engine with a `status()` method that checks two deadlock conditions. The TUI main loop uses a `TuiState` to switch between playing and game-over overlay modes. The NI binary includes the status in its JSON response.

**Tech Stack:** Rust, crossterm (TUI), serde_json (NI), clap (CLI)

---

### Task 1: Add `has_selectable_thread()` to GameBoard

**Files:**
- Modify: `src/game_board.rs:8-13` (impl block)
- Test: `src/game_board.rs` (inline tests module)

**Step 1: Write the failing test**

In `src/game_board.rs`, add to the `mod tests` block:

```rust
#[test]
fn has_selectable_thread_true_when_exposed() {
    let board = GameBoard {
        board: vec![
            vec![BoardEntity::Thread(Color::Red), BoardEntity::Void],
            vec![BoardEntity::Thread(Color::Blue), BoardEntity::Obstacle],
        ],
        height: 2, width: 2, knit_volume: 1,
    };
    assert!(board.has_selectable_thread());
}

#[test]
fn has_selectable_thread_false_when_all_buried() {
    let board = GameBoard {
        board: vec![
            vec![BoardEntity::Obstacle, BoardEntity::Obstacle],
            vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
            vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
        ],
        height: 3, width: 2, knit_volume: 1,
    };
    assert!(!board.has_selectable_thread());
}

#[test]
fn has_selectable_thread_false_when_no_threads() {
    let board = GameBoard {
        board: vec![
            vec![BoardEntity::Void, BoardEntity::Obstacle],
        ],
        height: 1, width: 2, knit_volume: 1,
    };
    assert!(!board.has_selectable_thread());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib game_board::tests::has_selectable_thread -- --nocapture`
Expected: compilation error — `has_selectable_thread` not found

**Step 3: Implement `has_selectable_thread()`**

In `src/game_board.rs`, add this method to the `impl GameBoard` block (after `is_focusable`):

```rust
/// Returns true if at least one Thread or KeyThread on the board is selectable.
pub fn has_selectable_thread(&self) -> bool {
    for row in 0..self.height as usize {
        for col in 0..self.width as usize {
            if self.is_selectable(row, col) {
                return true;
            }
        }
    }
    false
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib game_board::tests::has_selectable_thread -- --nocapture`
Expected: all 3 tests PASS

**Step 5: Commit**

```bash
git add src/game_board.rs
git commit -m "feat: add has_selectable_thread() to GameBoard"
```

---

### Task 2: Add `GameStatus` enum and `status()` to GameEngine

**Files:**
- Modify: `src/engine.rs:1-27` (add enum after existing enums)
- Modify: `src/engine.rs:156-166` (add `status()` near `is_won()`)
- Test: `src/engine.rs` (inline tests module)

**Step 1: Write the failing tests**

In `src/engine.rs`, add to `mod tests`:

```rust
#[test]
fn status_playing_at_start() {
    let e = default_engine();
    assert_eq!(e.status(), GameStatus::Playing);
}

#[test]
fn status_won_when_cleared() {
    let e = GameEngine {
        board: GameBoard {
            board: vec![vec![BoardEntity::Void, BoardEntity::Obstacle]],
            height: 1, width: 2, knit_volume: 1,
        },
        yarn: Yarn { board: vec![vec![], vec![]], yarn_lines: 2, visible_patches: 3 },
        active_threads: vec![],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 1, active_threads_limit: 5,
    };
    assert_eq!(e.status(), GameStatus::Won);
}

#[test]
fn status_stuck_front_thread_blocked() {
    // active_threads[0] is Green, but yarn only has Red patches → deadlock
    let e = GameEngine {
        board: GameBoard {
            board: vec![vec![BoardEntity::Void]],
            height: 1, width: 1, knit_volume: 1,
        },
        yarn: Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: false }]],
            yarn_lines: 1, visible_patches: 3,
        },
        active_threads: vec![Thread { color: Color::Green, status: 1, has_key: false }],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 3, active_threads_limit: 5,
    };
    assert_eq!(e.status(), GameStatus::Stuck);
}

#[test]
fn status_stuck_no_selectable_threads_on_board() {
    // No active threads, board has threads but all buried, yarn has patches
    let e = GameEngine {
        board: GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle, BoardEntity::Obstacle],
                vec![BoardEntity::Thread(Color::Red), BoardEntity::Thread(Color::Blue)],
            ],
            height: 2, width: 2, knit_volume: 1,
        },
        yarn: Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: false }]],
            yarn_lines: 1, visible_patches: 3,
        },
        active_threads: vec![],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 1, active_threads_limit: 5,
    };
    assert_eq!(e.status(), GameStatus::Stuck);
}

#[test]
fn status_playing_when_front_thread_can_match() {
    // active_threads[0] is Red, yarn has Red → can process → still playing
    let e = GameEngine {
        board: GameBoard {
            board: vec![vec![BoardEntity::Void]],
            height: 1, width: 1, knit_volume: 1,
        },
        yarn: Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: false }]],
            yarn_lines: 1, visible_patches: 3,
        },
        active_threads: vec![Thread { color: Color::Red, status: 1, has_key: false }],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 3, active_threads_limit: 5,
    };
    assert_eq!(e.status(), GameStatus::Playing);
}

#[test]
fn status_stuck_locked_patch_no_key() {
    // active_threads[0] is Red, yarn has locked Red but thread has no key → stuck
    let e = GameEngine {
        board: GameBoard {
            board: vec![vec![BoardEntity::Void]],
            height: 1, width: 1, knit_volume: 1,
        },
        yarn: Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: true }]],
            yarn_lines: 1, visible_patches: 3,
        },
        active_threads: vec![Thread { color: Color::Red, status: 1, has_key: false }],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 3, active_threads_limit: 5,
    };
    assert_eq!(e.status(), GameStatus::Stuck);
}

#[test]
fn status_playing_locked_patch_with_key() {
    // active_threads[0] is Red with key, yarn has locked Red → can unlock → playing
    let e = GameEngine {
        board: GameBoard {
            board: vec![vec![BoardEntity::Void]],
            height: 1, width: 1, knit_volume: 1,
        },
        yarn: Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: true }]],
            yarn_lines: 1, visible_patches: 3,
        },
        active_threads: vec![Thread { color: Color::Red, status: 1, has_key: true }],
        cursor_row: 0, cursor_col: 0,
        knit_volume: 3, active_threads_limit: 5,
    };
    assert_eq!(e.status(), GameStatus::Playing);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib engine::tests::status_ -- --nocapture`
Expected: compilation error — `GameStatus` not found

**Step 3: Add `GameStatus` enum and `status()` method**

In `src/engine.rs`, add the enum after `PickError` (around line 26):

```rust
#[derive(Debug, PartialEq)]
pub enum GameStatus {
    Playing,
    Won,
    Stuck,
}
```

Add `status()` method to `impl GameEngine`, right after `is_won()`:

```rust
pub fn status(&self) -> GameStatus {
    if self.is_won() {
        return GameStatus::Won;
    }
    if !self.active_threads.is_empty() {
        if !self.can_front_thread_progress() {
            return GameStatus::Stuck;
        }
    } else if !self.board.has_selectable_thread() {
        return GameStatus::Stuck;
    }
    GameStatus::Playing
}

/// Check if active_threads[0] can match any yarn column's last patch.
fn can_front_thread_progress(&self) -> bool {
    let front = &self.active_threads[0];
    for column in &self.yarn.board {
        let Some(last) = column.last() else { continue };
        if last.locked {
            if last.color == front.color && front.has_key {
                return true;
            }
            continue;
        }
        if last.color == front.color {
            return true;
        }
    }
    false
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib engine::tests::status_ -- --nocapture`
Expected: all 7 tests PASS

**Step 5: Commit**

```bash
git add src/engine.rs
git commit -m "feat: add GameStatus enum and status() deadlock detection"
```

---

### Task 3: Add TUI state machine with stuck/won overlay

**Files:**
- Modify: `src/main.rs` (add TuiState enum, modify main loop)

**Step 1: Add `TuiState` enum at module level**

After the imports in `src/main.rs`, add:

```rust
use knitui::engine::GameStatus;

enum TuiState {
    Playing,
    GameOver(GameStatus),
}
```

**Step 2: Add overlay render function**

After the existing `render()` function, add:

```rust
fn render_overlay(
    stdout: &mut Stdout,
    engine: &GameEngine,
    minimal_y: u16,
    status: &GameStatus,
) -> io::Result<()> {
    render(stdout, engine, minimal_y)?;
    let message = match status {
        GameStatus::Stuck => "You're lost! Press R to restart, Q to quit",
        GameStatus::Won   => "You won! Press R to play again, Q to quit",
        _ => return Ok(()),
    };
    stdout.queue(MoveTo(0, 0))?;
    stdout.queue(Print(message.with(crossterm::style::Color::White)))?;
    stdout.flush()
}
```

Note: add `use crossterm::style::Stylize;` to imports if not already present (it is — via `Print, Stylize` on line 8).

**Step 3: Modify main loop to use TuiState**

Replace the main loop body. After `let mut engine = GameEngine::new(&config);` and the initial `render()`, change the loop to:

```rust
let mut tui_state = TuiState::Playing;

loop {
    if poll(Duration::from_millis(150))? {
        if let Event::Key(event) = read()? {
            match tui_state {
                TuiState::GameOver(_) => {
                    match event.code {
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            engine = GameEngine::new(&config);
                            tui_state = TuiState::Playing;
                            render(&mut stdout, &engine, minimal_y)?;
                        }
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                        _ => {}
                    }
                }
                TuiState::Playing => {
                    match event.code {
                        KeyCode::Left  => { let _ = engine.move_cursor(Direction::Left);  }
                        KeyCode::Right => { let _ = engine.move_cursor(Direction::Right); }
                        KeyCode::Up    => { let _ = engine.move_cursor(Direction::Up);    }
                        KeyCode::Down  => { let _ = engine.move_cursor(Direction::Down);  }
                        KeyCode::Esc   => break,

                        KeyCode::Enter => {
                            if engine.pick_up().is_ok() {
                                match engine.status() {
                                    GameStatus::Playing => render(&mut stdout, &engine, minimal_y)?,
                                    s => {
                                        tui_state = TuiState::GameOver(s);
                                        render_overlay(&mut stdout, &engine, minimal_y, &s)?;
                                        continue;
                                    }
                                };
                            }
                        }

                        _ => {}
                    }

                    let x = engine.cursor_col;
                    let y = max(engine.cursor_row + minimal_y, minimal_y);
                    stdout.execute(MoveTo(x, y));
                }
            }
        }
    } else if matches!(tui_state, TuiState::Playing) && !engine.active_threads.is_empty() {
        engine.process_one_active();
        match engine.status() {
            GameStatus::Playing => render(&mut stdout, &engine, minimal_y)?,
            s => {
                tui_state = TuiState::GameOver(s);
                render_overlay(&mut stdout, &engine, minimal_y, &s)?;
            }
        };
    }
}
```

**Step 4: Build and smoke test**

Run: `cargo build`
Expected: compiles cleanly

Run the game manually: `cargo run -- --board-height 3 --board-width 3 --color-number 2`
Verify: game runs, Esc still quits. (Stuck detection will be exercised once you get yourself stuck.)

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: TUI state machine with stuck/won overlay"
```

---

### Task 4: Add game_status to NI binary JSON output

**Files:**
- Modify: `src/bin/knitui_ni.rs:86-96` (`ok_response` function)

**Step 1: Modify `ok_response` to include game_status**

In `src/bin/knitui_ni.rs`, update the `ok_response` function. Add the import at the top of the file:

```rust
use knitui::engine::GameStatus;
```

Change the `ok_response` function:

```rust
fn ok_response(hash: &str, engine: &GameEngine) {
    let state_json = engine.to_json();
    let state_val: serde_json::Value = serde_json::from_str(&state_json).unwrap();
    let game_status = match engine.status() {
        GameStatus::Playing => "playing",
        GameStatus::Won     => "won",
        GameStatus::Stuck   => "stuck",
    };
    let response = serde_json::json!({
        "status": "ok",
        "game": hash,
        "won": engine.is_won(),
        "game_status": game_status,
        "state": state_val,
    });
    println!("{}", serde_json::to_string(&response).unwrap());
}
```

Note: `"won"` field is kept for backwards compatibility. `"game_status"` is the new canonical field.

**Step 2: Build both binaries**

Run: `cargo build`
Expected: compiles cleanly for both `knitui` and `knitui-ni`

**Step 3: Commit**

```bash
git add src/bin/knitui_ni.rs
git commit -m "feat: add game_status field to NI binary JSON output"
```

---

### Task 5: Run full test suite and final verification

**Step 1: Run all tests**

Run: `cargo test`
Expected: all tests pass (existing + new)

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings` (if clippy is available)
Expected: no warnings

**Step 3: Final commit if any fixups needed**

Only if clippy or tests required changes.
