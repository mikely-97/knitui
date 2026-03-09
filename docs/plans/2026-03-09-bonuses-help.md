# Bonuses, Help & Key Bar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add three hotkey-activated bonuses (Scissors Z, Tweezers X, Balloons C), a help overlay (H), and a persistent key bar at the bottom of the screen.

**Architecture:** Bonus state (inventory + active bonus mode) lives on `GameEngine`. Scissors does instant deep-scan knitting. Tweezers sets a mode flag that relaxes cursor/pick-up constraints until one pick completes. Balloons lifts front patches from yarn columns into temporary pseudo-columns on `Yarn`. The TUI maps hotkeys, renders bonus counts, key bar, and help overlay.

**Tech Stack:** Rust, crossterm 0.27, clap 4, serde 1

---

### Task 1: Add bonus config flags

**Files:**
- Modify: `src/config.rs:1-55`

**Step 1: Write the failing test**

No separate test needed — this is a data struct. The compiler is the test.

**Step 2: Add config fields**

In `src/config.rs`, add these fields to the `Config` struct after the `scale` field (line 41):

```rust
    #[arg(long, default_value_t = 0, help = "Starting scissors bonus count")]
    pub scissors: u16,

    #[arg(long, default_value_t = 0, help = "Starting tweezers bonus count")]
    pub tweezers: u16,

    #[arg(long, default_value_t = 0, help = "Starting balloons bonus count")]
    pub balloons: u16,

    #[arg(long, default_value_t = 1, help = "Threads processed per scissors use")]
    pub scissors_threads: u16,

    #[arg(long, default_value_t = 2, help = "Patches lifted per yarn column per balloons use")]
    pub balloon_count: u16,
```

**Step 3: Update knitui_ni default config**

In `src/bin/knitui_ni.rs:166-179`, the `Config { ... }` block needs the new fields with defaults:

```rust
                scissors: 0,
                tweezers: 0,
                balloons: 0,
                scissors_threads: 1,
                balloon_count: 2,
```

**Step 4: Build to verify**

Run: `cargo build 2>&1`
Expected: compiles cleanly

**Step 5: Commit**

```bash
git add src/config.rs src/bin/knitui_ni.rs
git commit -m "feat: add bonus config flags (scissors, tweezers, balloons)"
```

---

### Task 2: Add BonusInventory and BonusState to engine

**Files:**
- Modify: `src/engine.rs:14-45` (error types and GameEngine struct)

**Step 1: Add BonusError enum**

After the `GameStatus` enum (line 33), add:

```rust
#[derive(Debug, PartialEq)]
pub enum BonusError {
    NoneLeft,
    BonusActive,
    NoActiveThreads,
    BalloonColumnsNotEmpty,
}
```

**Step 2: Add BonusState enum**

After `BonusError`:

```rust
#[derive(Debug, PartialEq, Clone)]
pub enum BonusState {
    None,
    TweezersActive { saved_row: u16, saved_col: u16 },
}
```

**Step 3: Add BonusInventory struct**

After `BonusState`:

```rust
pub struct BonusInventory {
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
    pub scissors_threads: u16,
    pub balloon_count: u16,
}
```

**Step 4: Add fields to GameEngine**

In the `GameEngine` struct (line 37-45), add two new fields:

```rust
pub struct GameEngine {
    pub board: GameBoard,
    pub yarn: Yarn,
    pub active_threads: Vec<Thread>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub knit_volume: u16,
    pub active_threads_limit: usize,
    pub bonuses: BonusInventory,
    pub bonus_state: BonusState,
}
```

**Step 5: Initialize in GameEngine::new()**

In `GameEngine::new()` (line 85-93), update the `Self { ... }` block:

```rust
        Self {
            board,
            yarn,
            active_threads: Vec::new(),
            cursor_row: init_row,
            cursor_col: init_col,
            knit_volume: config.knit_volume,
            active_threads_limit: config.active_threads_limit,
            bonuses: BonusInventory {
                scissors: config.scissors,
                tweezers: config.tweezers,
                balloons: config.balloons,
                scissors_threads: config.scissors_threads,
                balloon_count: config.balloon_count,
            },
            bonus_state: BonusState::None,
        }
```

**Step 6: Fix all `default_engine()` calls in tests**

Every test that constructs a `GameEngine` directly needs the two new fields. In the `default_engine()` helper (line 452-480), add:

```rust
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
```

Do the same for EVERY inline `GameEngine { ... }` in the test module — there are many (search for `GameEngine {` in the `#[cfg(test)]` block). Each needs the `bonuses` and `bonus_state` fields.

**Step 7: Fix into_engine() snapshot deserialization**

In `into_engine()` (line 366-384), add the new fields to the constructed `GameEngine`:

```rust
            bonuses: BonusInventory {
                scissors: 0, tweezers: 0, balloons: 0,
                scissors_threads: 1, balloon_count: 2,
            },
            bonus_state: BonusState::None,
```

(Snapshot support for bonuses comes in a later task.)

**Step 8: Build and test**

Run: `cargo test 2>&1`
Expected: all tests pass

**Step 9: Commit**

```bash
git add src/engine.rs
git commit -m "feat: add BonusInventory and BonusState to GameEngine"
```

---

### Task 3: Add balloon_columns to Yarn

**Files:**
- Modify: `src/yarn.rs:26-44` (Yarn struct and constructor)

**Step 1: Add field to Yarn struct**

In the `Yarn` struct (line 26-30):

```rust
pub struct Yarn {
    pub board: Vec<Vec<Patch>>,
    pub yarn_lines: u16,
    pub visible_patches: u16,
    pub balloon_columns: Vec<Vec<Patch>>,
}
```

**Step 2: Initialize in make_from_color_counter**

In `make_from_color_counter` (line 43), update the `Self { ... }`:

```rust
        Self { board, yarn_lines, visible_patches, balloon_columns: Vec::new() }
```

**Step 3: Fix all inline Yarn constructors in tests**

Search the entire codebase for `Yarn {` in test code. Every instance needs `balloon_columns: Vec::new()`. There are instances in:
- `src/yarn.rs` tests
- `src/engine.rs` tests
- `src/solvability.rs` (if it constructs Yarn directly)

**Step 4: Build and test**

Run: `cargo test 2>&1`
Expected: all tests pass

**Step 5: Commit**

```bash
git add src/yarn.rs src/engine.rs src/solvability.rs
git commit -m "feat: add balloon_columns field to Yarn"
```

---

### Task 4: Implement Scissors bonus

**Files:**
- Modify: `src/yarn.rs` (add `deep_scan_process` method)
- Modify: `src/engine.rs` (add `use_scissors` method)

**Step 1: Write failing test for deep_scan_process on Yarn**

In `src/yarn.rs`, add at the end of `mod tests`:

```rust
    #[test]
    fn test_deep_scan_process_finds_match_behind_front() {
        // Col 0: [Blue(bottom), Red(top)] — front is Red, but thread is Blue
        // Deep scan should find Blue behind Red and remove it
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Blue, locked: false },  // bottom (index 0)
                Patch { color: Color::Red, locked: false },   // top (index 1, front)
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 2); // knitted once
        assert_eq!(yarn.board[0].len(), 1); // Blue removed, Red remains
        assert_eq!(yarn.board[0][0].color, Color::Red); // Red is still there
    }

    #[test]
    fn test_deep_scan_process_no_match() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: Vec::new(),
        };

        let mut thread = Thread { color: Color::Green, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 1); // no change
        assert_eq!(yarn.board[0].len(), 1);
    }

    #[test]
    fn test_deep_scan_checks_balloon_columns() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: vec![vec![
                Patch { color: Color::Blue, locked: false },
            ]],
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.deep_scan_process(&mut thread);

        assert_eq!(thread.status, 2);
        assert_eq!(yarn.balloon_columns[0].len(), 0);
    }
```

**Step 2: Run tests to see them fail**

Run: `cargo test deep_scan 2>&1`
Expected: FAIL — method does not exist

**Step 3: Implement deep_scan_process on Yarn**

In `src/yarn.rs`, add after the `process_sequence` method (line 82):

```rust
    /// Deep-scan all yarn columns (and balloon columns) for a matching patch.
    /// Unlike process_one, this ignores queue order — it searches ALL patches
    /// in each column, not just the front. Removes the first match found.
    /// Locked patches are skipped entirely.
    pub fn deep_scan_process(&mut self, thread: &mut Thread) {
        // Search regular columns first, then balloon columns
        for column in self.board.iter_mut().chain(self.balloon_columns.iter_mut()) {
            if let Some(pos) = column.iter().position(|p| !p.locked && p.color == thread.color) {
                column.remove(pos);
                thread.knit_on();
                return;
            }
        }
    }
```

**Step 4: Run tests to verify pass**

Run: `cargo test deep_scan 2>&1`
Expected: all 3 pass

**Step 5: Write failing test for use_scissors on GameEngine**

In `src/engine.rs`, add at the end of `mod tests`:

```rust
    #[test]
    fn use_scissors_completes_thread() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        e.bonuses.scissors_threads = 1;
        e.knit_volume = 2;
        // Active thread: Red, status 1 (needs 2 total knits)
        e.active_threads = vec![
            Thread { color: Color::Red, status: 1, has_key: false },
        ];
        // Yarn has Red patches deep inside: col0=[Red, Blue], col1=[Red, Red]
        // Deep scan can find Red anywhere
        let result = e.use_scissors();
        assert!(result.is_ok());
        assert_eq!(e.bonuses.scissors, 0);
        // Thread should be fully knitted and removed (status went past knit_volume)
        assert_eq!(e.active_threads.len(), 0);
    }

    #[test]
    fn use_scissors_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.scissors = 0;
        e.active_threads = vec![Thread { color: Color::Red, status: 1, has_key: false }];
        assert_eq!(e.use_scissors(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn use_scissors_no_active_threads_fails() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        assert_eq!(e.use_scissors(), Err(BonusError::NoActiveThreads));
    }

    #[test]
    fn use_scissors_picks_least_progress_thread() {
        let mut e = default_engine();
        e.bonuses.scissors = 1;
        e.bonuses.scissors_threads = 1;
        e.knit_volume = 1;
        e.active_threads = vec![
            Thread { color: Color::Red,  status: 2, has_key: false }, // more progress
            Thread { color: Color::Blue, status: 1, has_key: false }, // least progress
        ];
        // Yarn default has Blue patches — deep scan should find one
        let _ = e.use_scissors();
        // The Blue thread (status 1) should have been selected and completed
        // It had status 1, knit_volume=1, so after 1 knit → status 2 > 1 → removed
        assert_eq!(e.active_threads.len(), 1);
        assert_eq!(e.active_threads[0].color, Color::Red);
    }
```

**Step 6: Run tests to see them fail**

Run: `cargo test use_scissors 2>&1`
Expected: FAIL — method does not exist

**Step 7: Implement use_scissors on GameEngine**

In `src/engine.rs`, add after `can_any_thread_progress()` method (before the serialisation section):

```rust
    // ── Bonuses ─────────────────────────────────────────────────────────

    /// Check if any bonus is currently active.
    pub fn is_bonus_active(&self) -> bool {
        self.bonus_state != BonusState::None || !self.yarn.balloon_columns.is_empty()
    }

    /// Scissors: deep-scan auto-knit the least-progressed thread(s).
    pub fn use_scissors(&mut self) -> Result<(), BonusError> {
        if self.bonuses.scissors == 0 {
            return Err(BonusError::NoneLeft);
        }
        if self.active_threads.is_empty() {
            return Err(BonusError::NoActiveThreads);
        }
        if self.is_bonus_active() {
            return Err(BonusError::BonusActive);
        }

        self.bonuses.scissors -= 1;

        // Process up to scissors_threads threads, picking lowest status each time
        for _ in 0..self.bonuses.scissors_threads {
            if self.active_threads.is_empty() { break; }

            // Find the thread with the lowest status
            let min_idx = self.active_threads.iter()
                .enumerate()
                .min_by_key(|(_, t)| t.status)
                .map(|(i, _)| i)
                .unwrap();

            // Deep-scan knit until complete or no more matches
            loop {
                if self.active_threads[min_idx].status > self.knit_volume {
                    break;
                }
                let prev_status = self.active_threads[min_idx].status;
                self.yarn.deep_scan_process(&mut self.active_threads[min_idx]);
                if self.active_threads[min_idx].status == prev_status {
                    break; // no match found anywhere
                }
            }

            // Remove if completed
            if self.active_threads[min_idx].status > self.knit_volume {
                self.active_threads.remove(min_idx);
            }
        }

        Ok(())
    }
```

**Step 8: Run tests**

Run: `cargo test use_scissors 2>&1`
Expected: all 4 pass

**Step 9: Run full test suite**

Run: `cargo test 2>&1`
Expected: all tests pass

**Step 10: Commit**

```bash
git add src/yarn.rs src/engine.rs
git commit -m "feat: implement scissors bonus (deep-scan auto-knit)"
```

---

### Task 5: Implement Tweezers bonus

**Files:**
- Modify: `src/engine.rs` (modify `move_cursor`, `pick_up`, add `use_tweezers`, `cancel_tweezers`)

**Step 1: Write failing tests**

In `src/engine.rs` tests, add:

```rust
    #[test]
    fn use_tweezers_enters_mode() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.cursor_row = 0;
        e.cursor_col = 0;
        let result = e.use_tweezers();
        assert!(result.is_ok());
        assert_eq!(e.bonus_state, BonusState::TweezersActive { saved_row: 0, saved_col: 0 });
        // Count not decremented until pick completes
        assert_eq!(e.bonuses.tweezers, 1);
    }

    #[test]
    fn use_tweezers_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.tweezers = 0;
        assert_eq!(e.use_tweezers(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn tweezers_mode_cursor_moves_anywhere() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        // Row 2 col 0 is buried (not focusable normally), but tweezers allows it
        let result = e.move_cursor(Direction::Down);
        assert!(result.is_ok());
        assert_eq!(e.cursor_row, 1); // moved to row 1 (normally skipped)
    }

    #[test]
    fn tweezers_pick_up_ignores_selectability() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        // Move cursor to buried thread at (2, 0)
        e.cursor_row = 2;
        e.cursor_col = 0;
        // Normally this would fail with NotSelectable, but tweezers overrides
        let result = e.pick_up();
        assert!(result.is_ok());
        assert_eq!(e.active_threads.len(), 1);
        // Cursor restored to saved position
        assert_eq!(e.cursor_row, 0);
        assert_eq!(e.cursor_col, 0);
        // Bonus consumed
        assert_eq!(e.bonuses.tweezers, 0);
        assert_eq!(e.bonus_state, BonusState::None);
    }

    #[test]
    fn cancel_tweezers_restores_cursor() {
        let mut e = default_engine();
        e.bonuses.tweezers = 1;
        e.use_tweezers().unwrap();
        e.cursor_row = 2;
        e.cursor_col = 1;
        e.cancel_tweezers();
        assert_eq!(e.cursor_row, 0);
        assert_eq!(e.cursor_col, 0);
        assert_eq!(e.bonus_state, BonusState::None);
        // Bonus NOT consumed on cancel
        assert_eq!(e.bonuses.tweezers, 1);
    }
```

**Step 2: Run tests to see them fail**

Run: `cargo test tweezers 2>&1`
Expected: FAIL

**Step 3: Implement use_tweezers and cancel_tweezers**

In `src/engine.rs`, add after `use_scissors`:

```rust
    /// Tweezers: enter free-cursor mode. Cursor can move to any cell
    /// and pick up any thread regardless of selectability.
    pub fn use_tweezers(&mut self) -> Result<(), BonusError> {
        if self.bonuses.tweezers == 0 {
            return Err(BonusError::NoneLeft);
        }
        if self.is_bonus_active() {
            return Err(BonusError::BonusActive);
        }

        self.bonus_state = BonusState::TweezersActive {
            saved_row: self.cursor_row,
            saved_col: self.cursor_col,
        };
        // Don't decrement yet — only on successful pick
        Ok(())
    }

    /// Cancel tweezers mode without consuming the bonus.
    pub fn cancel_tweezers(&mut self) {
        if let BonusState::TweezersActive { saved_row, saved_col } = self.bonus_state {
            self.cursor_row = saved_row;
            self.cursor_col = saved_col;
            self.bonus_state = BonusState::None;
        }
    }
```

**Step 4: Modify move_cursor to respect tweezers mode**

Replace `move_cursor` (lines 98-116) with:

```rust
    pub fn move_cursor(&mut self, dir: Direction) -> Result<(), MoveError> {
        let (dr, dc) = dir.offset();
        let mut new_row = self.cursor_row as i32 + dr;
        let mut new_col = self.cursor_col as i32 + dc;
        let tweezers = matches!(self.bonus_state, BonusState::TweezersActive { .. });
        loop {
            if new_row < 0 || new_row >= self.board.height as i32
                || new_col < 0 || new_col >= self.board.width as i32
            {
                return Err(MoveError::OutOfBounds);
            }
            if tweezers || self.board.is_focusable(new_row as usize, new_col as usize) {
                self.cursor_row = new_row as u16;
                self.cursor_col = new_col as u16;
                return Ok(());
            }
            new_row += dr;
            new_col += dc;
        }
    }
```

**Step 5: Modify pick_up to respect tweezers mode**

Replace `pick_up` (lines 118-143) with:

```rust
    pub fn pick_up(&mut self) -> Result<(), PickError> {
        let row = self.cursor_row as usize;
        let col = self.cursor_col as usize;
        let tweezers = matches!(self.bonus_state, BonusState::TweezersActive { .. });

        let thread = match &self.board.board[row][col] {
            BoardEntity::Thread(c)    => Thread { color: *c, status: 1, has_key: false },
            BoardEntity::KeyThread(c) => Thread { color: *c, status: 1, has_key: true },
            _ => return Err(PickError::NotAThread),
        };

        if !tweezers && !self.board.is_selectable(row, col) {
            return Err(PickError::NotSelectable);
        }
        if self.active_threads.len() >= self.active_threads_limit {
            return Err(PickError::ActiveFull);
        }

        self.active_threads.push(thread);
        self.board.board[row][col] = BoardEntity::Void;

        if let Some((gr, gc)) = find_generator_for_output(&self.board.board, row, col) {
            advance_generator(&mut self.board.board, gr, gc, row, col);
        }

        // Exit tweezers mode after successful pick
        if let BonusState::TweezersActive { saved_row, saved_col } = self.bonus_state {
            self.cursor_row = saved_row;
            self.cursor_col = saved_col;
            self.bonus_state = BonusState::None;
            self.bonuses.tweezers -= 1;
        }

        Ok(())
    }
```

**Step 6: Run tests**

Run: `cargo test tweezers 2>&1`
Expected: all 5 pass

**Step 7: Run full test suite**

Run: `cargo test 2>&1`
Expected: all tests pass

**Step 8: Commit**

```bash
git add src/engine.rs
git commit -m "feat: implement tweezers bonus (free-cursor pick)"
```

---

### Task 6: Implement Balloons bonus

**Files:**
- Modify: `src/yarn.rs` (modify `process_one` to check balloon_columns, add cleanup method)
- Modify: `src/engine.rs` (add `use_balloons`)

**Step 1: Write failing tests for Yarn balloon processing**

In `src/yarn.rs` tests, add:

```rust
    #[test]
    fn test_process_one_checks_balloon_columns() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: vec![vec![
                Patch { color: Color::Blue, locked: false },
            ]],
        };

        let mut thread = Thread { color: Color::Blue, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        // Should match against balloon column, not regular column
        assert_eq!(thread.status, 2);
        assert_eq!(yarn.balloon_columns[0].len(), 0);
        assert_eq!(yarn.board[0].len(), 1); // regular column unchanged
    }

    #[test]
    fn test_process_one_prefers_regular_over_balloon() {
        let mut yarn = Yarn {
            board: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
            yarn_lines: 1,
            visible_patches: 3,
            balloon_columns: vec![vec![
                Patch { color: Color::Red, locked: false },
            ]],
        };

        let mut thread = Thread { color: Color::Red, status: 1, has_key: false };
        yarn.process_one(&mut thread);

        // Regular columns checked first
        assert_eq!(thread.status, 2);
        assert_eq!(yarn.board[0].len(), 0); // regular consumed
        assert_eq!(yarn.balloon_columns[0].len(), 1); // balloon untouched
    }
```

**Step 2: Run tests to see them fail**

Run: `cargo test balloon 2>&1`
Expected: first test fails (process_one doesn't check balloon_columns)

**Step 3: Modify process_one to check balloon_columns**

Replace `process_one` in `src/yarn.rs` (lines 55-76) with:

```rust
    pub fn process_one(&mut self, thread: &mut Thread) {
        // Check regular columns first
        for column in &mut self.board {
            let Some(last) = column.last() else { continue };

            if last.locked {
                if last.color == thread.color && thread.has_key {
                    column.pop();
                    thread.knit_on();
                    thread.has_key = false;
                    return;
                }
                continue;
            }

            if last.color == thread.color {
                column.pop();
                thread.knit_on();
                return;
            }
        }

        // Then check balloon columns (same logic, but no locked patches expected)
        for column in &mut self.balloon_columns {
            let Some(last) = column.last() else { continue };
            if last.color == thread.color {
                column.pop();
                thread.knit_on();
                return;
            }
        }
    }
```

Also add a cleanup helper after `process_one`:

```rust
    /// Remove empty balloon columns.
    pub fn cleanup_balloon_columns(&mut self) {
        self.balloon_columns.retain(|col| !col.is_empty());
    }
```

**Step 4: Run tests to verify pass**

Run: `cargo test balloon 2>&1`
Expected: both pass

**Step 5: Write failing test for use_balloons on GameEngine**

In `src/engine.rs` tests, add:

```rust
    #[test]
    fn use_balloons_lifts_patches() {
        let mut e = default_engine();
        e.bonuses.balloons = 1;
        e.bonuses.balloon_count = 1;
        // Yarn col0 has 2 patches: [Red(bottom), Blue(top)]
        // Lifting 1 patch from front (top) of each column
        let yarn_cols_before: Vec<usize> = e.yarn.board.iter().map(|c| c.len()).collect();
        let result = e.use_balloons();
        assert!(result.is_ok());
        assert_eq!(e.bonuses.balloons, 0);
        // Balloon columns should have been created
        assert!(!e.yarn.balloon_columns.is_empty());
        // Regular columns should have fewer patches
        let yarn_cols_after: Vec<usize> = e.yarn.board.iter().map(|c| c.len()).collect();
        let total_before: usize = yarn_cols_before.iter().sum();
        let total_after: usize = yarn_cols_after.iter().sum();
        let balloon_total: usize = e.yarn.balloon_columns.iter().map(|c| c.len()).sum();
        assert_eq!(total_before, total_after + balloon_total);
    }

    #[test]
    fn use_balloons_none_left_fails() {
        let mut e = default_engine();
        e.bonuses.balloons = 0;
        assert_eq!(e.use_balloons(), Err(BonusError::NoneLeft));
    }

    #[test]
    fn use_balloons_while_active_fails() {
        let mut e = default_engine();
        e.bonuses.balloons = 2;
        e.bonuses.balloon_count = 1;
        e.use_balloons().unwrap();
        // Balloon columns are non-empty now
        assert_eq!(e.use_balloons(), Err(BonusError::BalloonColumnsNotEmpty));
    }
```

**Step 6: Run tests to see them fail**

Run: `cargo test use_balloons 2>&1`
Expected: FAIL

**Step 7: Implement use_balloons**

In `src/engine.rs`, add after `cancel_tweezers`:

```rust
    /// Balloons: lift the front N patches from each yarn column into
    /// separate pseudo-columns, exposing the patches behind them.
    pub fn use_balloons(&mut self) -> Result<(), BonusError> {
        if self.bonuses.balloons == 0 {
            return Err(BonusError::NoneLeft);
        }
        if !self.yarn.balloon_columns.is_empty() {
            return Err(BonusError::BalloonColumnsNotEmpty);
        }
        if self.bonus_state != BonusState::None {
            return Err(BonusError::BonusActive);
        }

        self.bonuses.balloons -= 1;

        for column in &mut self.yarn.board {
            let mut lifted = Vec::new();
            for _ in 0..self.bonuses.balloon_count {
                if let Some(patch) = column.pop() {
                    lifted.push(patch);
                }
            }
            if !lifted.is_empty() {
                self.yarn.balloon_columns.push(lifted);
            }
        }

        Ok(())
    }
```

**Step 8: Run tests**

Run: `cargo test use_balloons 2>&1`
Expected: all 3 pass

**Step 9: Add balloon cleanup call to process_one_active and process_all_active**

In `process_one_active`, after the thread removal check, add:

```rust
        self.yarn.cleanup_balloon_columns();
```

Similarly in `process_all_active`, at the end of the method add:

```rust
        self.yarn.cleanup_balloon_columns();
```

**Step 10: Run full test suite**

Run: `cargo test 2>&1`
Expected: all tests pass

**Step 11: Commit**

```bash
git add src/yarn.rs src/engine.rs
git commit -m "feat: implement balloons bonus (lift front patches to pseudo-columns)"
```

---

### Task 7: Snapshot support for bonuses

**Files:**
- Modify: `src/engine.rs` (GameStateSnapshot, from_engine, into_engine)

**Step 1: Add bonus fields to GameStateSnapshot**

In `GameStateSnapshot` struct (lines 292-304), add:

```rust
    #[serde(default)]
    pub scissors: u16,
    #[serde(default)]
    pub tweezers: u16,
    #[serde(default)]
    pub balloons: u16,
    #[serde(default)]
    pub scissors_threads: u16,
    #[serde(default)]
    pub balloon_count: u16,
    #[serde(default)]
    pub balloon_columns: Vec<Vec<YarnPatchSnap>>,
```

**Step 2: Update from_engine**

In `GameStateSnapshot::from_engine` (lines 313-340), add the new fields:

```rust
            scissors: e.bonuses.scissors,
            tweezers: e.bonuses.tweezers,
            balloons: e.bonuses.balloons,
            scissors_threads: e.bonuses.scissors_threads,
            balloon_count: e.bonuses.balloon_count,
            balloon_columns: e.yarn.balloon_columns.iter()
                .map(|col| col.iter().map(|p| YarnPatchSnap {
                    color: color_serde::color_to_str(&p.color),
                    locked: p.locked,
                }).collect())
                .collect(),
```

**Step 3: Update into_engine**

In `into_engine` (lines 342-384), parse balloon_columns and populate bonuses:

After the `yarn_cols` parsing, add:

```rust
        let balloon_cols: Result<Vec<Vec<Patch>>, String> = self.balloon_columns.iter()
            .map(|col| col.iter().map(|p| {
                let color = color_serde::str_to_color(&p.color)
                    .ok_or_else(|| format!("bad color: {}", p.color))?;
                Ok(Patch { color, locked: p.locked })
            }).collect())
            .collect();
        let balloon_cols = balloon_cols?;
```

Then update the constructed `GameEngine`:

```rust
            yarn: Yarn {
                board: yarn_cols,
                yarn_lines: self.yarn_lines,
                visible_patches: self.visible_patches,
                balloon_columns: balloon_cols,
            },
            ...
            bonuses: BonusInventory {
                scissors: self.scissors,
                tweezers: self.tweezers,
                balloons: self.balloons,
                scissors_threads: if self.scissors_threads == 0 { 1 } else { self.scissors_threads },
                balloon_count: if self.balloon_count == 0 { 2 } else { self.balloon_count },
            },
            bonus_state: BonusState::None,
```

**Step 4: Write test**

```rust
    #[test]
    fn snapshot_roundtrip_with_bonuses() {
        let mut e = default_engine();
        e.bonuses.scissors = 3;
        e.bonuses.tweezers = 2;
        e.bonuses.balloons = 1;
        e.yarn.balloon_columns = vec![vec![Patch { color: Color::Red, locked: false }]];
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.bonuses.scissors, 3);
        assert_eq!(e2.bonuses.tweezers, 2);
        assert_eq!(e2.bonuses.balloons, 1);
        assert_eq!(e2.yarn.balloon_columns.len(), 1);
        assert_eq!(e2.yarn.balloon_columns[0][0].color, Color::Red);
    }
```

**Step 5: Run tests**

Run: `cargo test 2>&1`
Expected: all pass

**Step 6: Commit**

```bash
git add src/engine.rs
git commit -m "feat: snapshot serialization for bonus state"
```

---

### Task 8: NI binary bonus commands

**Files:**
- Modify: `src/bin/knitui_ni.rs`

**Step 1: Add NI bonus subcommands**

In the `NiCommand` enum (lines 33-40), add:

```rust
    /// Use scissors bonus
    Scissors,
    /// Use tweezers bonus (enters tweezers mode)
    Tweezers,
    /// Use balloons bonus
    Balloons,
```

**Step 2: Add NI args for bonus config**

In the `Args` struct (lines 11-30), add:

```rust
    #[arg(long)] scissors:          Option<u16>,
    #[arg(long)] tweezers:          Option<u16>,
    #[arg(long)] balloons:          Option<u16>,
    #[arg(long)] scissors_threads:  Option<u16>,
    #[arg(long)] balloon_count:     Option<u16>,
```

**Step 3: Handle bonus commands in match**

In the command match block (lines 127-155), add before `None`:

```rust
                Some(NiCommand::Scissors) => {
                    if let Err(e) = engine.use_scissors() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
                Some(NiCommand::Tweezers) => {
                    if let Err(e) = engine.use_tweezers() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
                Some(NiCommand::Balloons) => {
                    if let Err(e) = engine.use_balloons() {
                        let msg = format!("{:?}", e);
                        err_response("bonus_failed", &msg);
                        return;
                    }
                }
```

**Step 4: Update default Config construction with bonus fields**

In the `Config { ... }` block (lines 166-179), add:

```rust
                scissors:         args.scissors.unwrap_or(0),
                tweezers:         args.tweezers.unwrap_or(0),
                balloons:         args.balloons.unwrap_or(0),
                scissors_threads: args.scissors_threads.unwrap_or(1),
                balloon_count:    args.balloon_count.unwrap_or(2),
```

**Step 5: Build**

Run: `cargo build 2>&1`
Expected: compiles

**Step 6: Commit**

```bash
git add src/bin/knitui_ni.rs
git commit -m "feat: NI binary bonus commands (scissors, tweezers, balloons)"
```

---

### Task 9: TUI help overlay

**Files:**
- Modify: `src/main.rs`

**Step 1: Add TuiState::Help variant**

In the `TuiState` enum (lines 27-30), add:

```rust
enum TuiState {
    Playing,
    GameOver(GameStatus),
    Help,
}
```

**Step 2: Add render_help function**

Add a new function after the overlay rendering functions:

```rust
fn render_help(stdout: &mut Stdout) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let lines = [
        "",
        "                    ═══ HELP ═══",
        "",
        "  Movement:   ← → ↑ ↓   Move cursor",
        "  Pick up:    Enter       Pick up thread at cursor",
        "  Quit:       Esc / Q     Exit game",
        "  Restart:    R           New game",
        "  Help:       H           Show this screen",
        "",
        "  ─── Bonuses ───",
        "  [Z] ✂ Scissors    Auto-knit thread by deep-scanning yarn",
        "  [X] ⊹ Tweezers    Pick any thread from the board",
        "  [C] ⊛ Balloons    Lift front patches, expose patches behind",
        "",
        "              Press any key to close",
    ];

    for (i, line) in lines.iter().enumerate() {
        stdout.queue(MoveTo(0, i as u16))?;
        stdout.queue(Print(line))?;
    }

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}
```

**Step 3: Add H key handler in the Playing state**

In the key handling match for `TuiState::Playing` (lines 348-368), add before the `_ => { continue; }` arm:

```rust
                            KeyCode::Char('h') | KeyCode::Char('H') => {
                                render_help(&mut stdout)?;
                                tui_state = TuiState::Help;
                                continue;
                            }
```

**Step 4: Add Help state handler in the main event loop**

In the outer `match tui_state` (line 335), add a new arm before `TuiState::Playing`:

```rust
                    TuiState::Help => {
                        // Any key dismisses help
                        tui_state = TuiState::Playing;
                        do_render(&mut stdout, &engine, layout, board_x, board_y, scale)?;
                    }
```

**Step 5: Build**

Run: `cargo build 2>&1`
Expected: compiles

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: help overlay (H key)"
```

---

### Task 10: Key bar and bonus display

**Files:**
- Modify: `src/main.rs`

**Step 1: Add render_keybar function**

```rust
fn render_keybar(stdout: &mut Stdout, engine: &GameEngine, y: u16) -> io::Result<()> {
    stdout.queue(MoveTo(0, y))?;
    // Clear the line
    let (term_w, _) = terminal::size().unwrap_or((80, 24));
    for _ in 0..term_w { stdout.queue(Print(' '))?; }
    stdout.queue(MoveTo(0, y))?;

    stdout.queue(Print("←→↑↓ ".dark_grey()))?;
    stdout.queue(Print("Move  ".white()))?;
    stdout.queue(Print("Enter ".dark_grey()))?;
    stdout.queue(Print("Pick  ".white()))?;
    stdout.queue(Print("H ".dark_grey()))?;
    stdout.queue(Print("Help  ".white()))?;

    // Scissors
    if engine.bonuses.scissors > 0 {
        stdout.queue(Print("Z ".dark_grey()))?;
        stdout.queue(Print(format!("✂x{} ", engine.bonuses.scissors).white()))?;
    } else {
        stdout.queue(Print(format!("Z ✂x0 ").dark_grey()))?;
    }

    // Tweezers
    if engine.bonuses.tweezers > 0 {
        stdout.queue(Print("X ".dark_grey()))?;
        stdout.queue(Print(format!("⊹x{} ", engine.bonuses.tweezers).white()))?;
    } else {
        stdout.queue(Print(format!("X ⊹x0 ").dark_grey()))?;
    }

    // Balloons
    if engine.bonuses.balloons > 0 {
        stdout.queue(Print("C ".dark_grey()))?;
        stdout.queue(Print(format!("⊛x{} ", engine.bonuses.balloons).white()))?;
    } else {
        stdout.queue(Print(format!("C ⊛x0 ").dark_grey()))?;
    }

    stdout.queue(Print("Esc ".dark_grey()))?;
    stdout.queue(Print("Quit".white()))?;
    Ok(())
}
```

**Step 2: Add render_bonus_panel function (for horizontal layout right side)**

```rust
fn render_bonus_panel(stdout: &mut Stdout, engine: &GameEngine, x: u16, y: u16) -> io::Result<()> {
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];

    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        let row = y + i as u16;
        stdout.queue(MoveTo(x, row))?;
        if *count > 0 {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).white()))?;
        } else {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).dark_grey()))?;
        }
    }
    Ok(())
}
```

**Step 3: Integrate into vertical render**

In the `render` function (lines 192-214), before `stdout.queue(EndSynchronizedUpdate)?`:

Add bonus display below the board. Compute the board bottom y:

```rust
    let sh = scale;
    let board_h = 1 + engine.board.height * (sh + 1);
    let bonus_y = board_y + board_h + 1;
    render_bonus_display_h(stdout, engine, 0, bonus_y)?;

    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    render_keybar(stdout, engine, term_h.saturating_sub(1))?;
```

Where `render_bonus_display_h` is a horizontal bonus row:

```rust
fn render_bonus_display_h(stdout: &mut Stdout, engine: &GameEngine, x: u16, y: u16) -> io::Result<()> {
    stdout.queue(MoveTo(x, y))?;
    let bonuses = [
        ("Z", "✂", engine.bonuses.scissors),
        ("X", "⊹", engine.bonuses.tweezers),
        ("C", "⊛", engine.bonuses.balloons),
    ];
    for (i, (key, icon, count)) in bonuses.iter().enumerate() {
        if i > 0 { stdout.queue(Print("  "))?; }
        if *count > 0 {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).white()))?;
        } else {
            stdout.queue(Print(format!("[{}] {} x{}", key, icon, count).dark_grey()))?;
        }
    }
    Ok(())
}
```

**Step 4: Integrate into horizontal render**

In the `render_horizontal` function (lines 234-255), add the bonus panel and key bar. Compute the right side position:

```rust
    let board_w = 1 + engine.board.width * (sw + 1);
    let panel_x = board_x + board_w + 2;
    render_bonus_panel(stdout, engine, panel_x, 0)?;

    let (_, term_h) = terminal::size().unwrap_or((80, 24));
    render_keybar(stdout, engine, term_h.saturating_sub(1))?;
```

**Step 5: Build and manually test**

Run: `cargo build && cargo run --bin knitui -- --scissors 2 --tweezers 1 --balloons 3`
Expected: bonuses visible, key bar at bottom, H opens help

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: bonus display area, key bar, and status line"
```

---

### Task 11: TUI hotkey handling for bonuses

**Files:**
- Modify: `src/main.rs`

**Step 1: Add bonus hotkey handlers in Playing state**

In the key match for `TuiState::Playing` (lines 348-368), add before the `_ => { continue; }` arm:

```rust
                            KeyCode::Char('z') | KeyCode::Char('Z') => {
                                let _ = engine.use_scissors();
                            }
                            KeyCode::Char('x') | KeyCode::Char('X') => {
                                let _ = engine.use_tweezers();
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                let _ = engine.use_balloons();
                            }
```

**Step 2: Handle Esc during tweezers mode**

The current `KeyCode::Esc => break` in the Playing state should first check if tweezers is active:

```rust
                            KeyCode::Esc => {
                                if engine.bonus_state != BonusState::None {
                                    engine.cancel_tweezers();
                                } else {
                                    break;
                                }
                            }
```

**Step 3: Handle tweezers cursor rendering**

In `render_board`, modify the cursor brackets to show `{ }` during tweezers mode. This requires passing the `bonus_state` to `render_board`. Update the function signature:

```rust
fn render_board(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
```

The engine is already passed, so check `engine.bonus_state` inside the function. Replace the cursor bracket characters:

```rust
                let tweezers = matches!(engine.bonus_state, BonusState::TweezersActive { .. });
                let (open, close) = if tweezers { ('{', '}') } else { ('[', ']') };
```

Then use `open` and `close` instead of hardcoded `[` and `]` throughout `render_board`.

**Step 4: Add the BonusState import to main.rs**

At the top of `main.rs`, update the import:

```rust
use knitui::engine::{GameEngine, GameStatus, BonusState};
```

**Step 5: Build and test**

Run: `cargo build && cargo run --bin knitui -- --scissors 2 --tweezers 1 --balloons 3`
Expected: Z/X/C keys activate bonuses, Esc cancels tweezers, bracket style changes in tweezers

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat: TUI hotkey handling for bonuses (Z/X/C) with tweezers cursor"
```

---

### Task 12: Render balloon columns in yarn area

**Files:**
- Modify: `src/main.rs`

**Step 1: Show balloon columns visually**

In `render_yarn`, after drawing regular columns, add balloon column rendering. Balloon columns appear to the right of regular yarn with a visual separator:

```rust
fn render_yarn(stdout: &mut Stdout, engine: &GameEngine, x0: u16, y0: u16, scale: u16) -> io::Result<()> {
    // ... existing regular yarn rendering ...

    // Render balloon columns (if any) to the right with a separator
    if !engine.yarn.balloon_columns.is_empty() {
        let sw = scale * 2;
        let sh = scale;
        let regular_w = engine.yarn.yarn_lines * sw
            + engine.yarn.yarn_lines.saturating_sub(1) * YARN_HGAP;
        let balloon_x0 = x0 + regular_w + COMP_GAP;

        for offset in 0..(engine.yarn.visible_patches as usize) {
            let true_offset = (engine.yarn.visible_patches as usize) - offset;
            let row_y = y0 + (offset as u16) * (sh + YARN_VGAP);
            for sy in 0..sh {
                stdout.queue(MoveTo(balloon_x0, row_y + sy))?;
                for (ci, column) in engine.yarn.balloon_columns.iter().enumerate() {
                    if ci > 0 {
                        for _ in 0..YARN_HGAP { stdout.queue(Print(' '))?; }
                    }
                    if true_offset <= column.len() {
                        let pos = column.len() - true_offset;
                        stdout.queue(Print("^".dark_grey()))?;
                        for _ in 1..sw { stdout.queue(Print(&column[pos]))?; }
                    } else {
                        for _ in 0..sw { stdout.queue(Print(' '))?; }
                    }
                }
            }
        }
    }

    Ok(())
}
```

**Step 2: Build and test**

Run: `cargo build && cargo run --bin knitui -- --balloons 3 --balloon-count 1`
Expected: pressing C shows lifted patches with `^` marker

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: render balloon pseudo-columns in yarn area"
```

---

### Task 13: Update README

**Files:**
- Modify: `README.md`

**Step 1: Add bonus documentation to README**

In the "How to Play" section, add a "Bonuses" subsection. In the Configuration table, add the new flags. In the key bindings section, add Z/X/C/H.

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add bonus mechanics and help to README"
```

---

### Task 14: Final integration test

**Step 1: Run the full test suite**

Run: `cargo test 2>&1`
Expected: all tests pass

**Step 2: Build release**

Run: `cargo build --release 2>&1`
Expected: compiles

**Step 3: Manual smoke test**

Run: `cargo run --bin knitui -- --scissors 3 --tweezers 2 --balloons 2 --balloon-count 1 --scissors-threads 2`

Verify:
- Key bar visible at bottom
- Bonus counts visible below board / right panel
- H opens help overlay, any key dismisses
- Z activates scissors (thread auto-knitted)
- X activates tweezers (cursor changes to `{}`, can select any cell, Esc cancels)
- C activates balloons (patches lifted, shown in separate area)
- Bonuses with 0 count are greyed out
- Game still works normally (move, pick up, win/stuck detection)
