# Endless Mode Rework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace loom-knit's wave-based Endless mode with a single continuous scrolling puzzle: one tall generated board (6 cols × 66 rows), only the top 6 rows visible, exhausted rows shift out and buffered rows shift in, unlimited bonuses, score = rows cleared.

**Architecture:** Row-buffer state (`row_buffer`, `total_rows`, `rows_cleared`) lives directly on `GameEngine` (not a wrapper struct), so it round-trips through `knitui-ni`'s existing `--game` JSON snapshot mechanism for free. The row-shift check runs as the last step inside `GameEngine::process_all_active()`, gated on `total_rows > 0`, so neither the TUI (`tui.rs`) nor the CLI driver (`knitui_ni.rs`) need new call-ordering logic — both already call `process_all_active()` immediately before checking `status()`.

**Tech Stack:** Rust, serde (JSON snapshot persistence), crossterm (TUI rendering), clap (CLI args).

## Global Constraints

- Visible board is always 6 columns × 6 rows (reuse `crate::config::MAX_BOARD_DIM` for the `6`).
- Total board (visible + buffer) is 66 rows: 6 visible + 60 buffered.
- `#[serde(default)]` on every new field so existing save files / non-endless snapshots stay valid.
- `EndlessHighScore` (`crates/loom-engine/src/endless.rs`) is NOT touched — it's shared with loom-match3 and loom-merge2. Its `best_wave: usize` field is reused as-is to store rows-cleared for knit; only the on-screen label changes.
- No campaign-blessing interaction — endless mode has never read `blessing_flags` beyond engine defaults and continues not to.

---

### Task 1: Add row-buffer fields to `GameEngine` and its JSON snapshot

**Files:**
- Modify: `crates/loom-knit/src/engine/mod.rs` (struct def ~line 21-44, `GameStateSnapshot` ~line 572-602, `from_engine`/`into_engine` ~line 610-723, and 9 test-module `GameEngine { .. }` literals)
- Modify: `crates/loom-knit/tests/game_flow_test.rs` (3 `GameEngine { .. }` literals)
- Test: same files (existing tests must keep passing with the new fields defaulted)

**Interfaces:**
- Produces: `GameEngine.row_buffer: Vec<Vec<BoardEntity>>`, `GameEngine.total_rows: u32`, `GameEngine.rows_cleared: u32` — all three read by Task 2 (constructor) and Task 3 (shift logic). `total_rows > 0` is the signal used everywhere else in this plan to mean "this is an endless-mode engine."

- [ ] **Step 1: Add `Debug`/`Clone`/`PartialEq` derives to `BoardEntity` and `ConveyorData`**

Neither type currently derives anything, which blocks the `assert_eq!`/equality-based tests this plan adds later (Task 1 Step 8, Task 3). In `crates/loom-knit/src/board_entity.rs`, add derives directly above both type definitions:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ConveyorData {
```

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum BoardEntity {
```

(`Color` from `crossterm::style` and `Direction` from `loom_engine::direction` — the two external types `ConveyorData`/`BoardEntity` hold — already derive `Clone`/`Debug`/`PartialEq`/`Copy`, so this compiles without touching either of those.)

Run: `cargo build -p knitui 2>&1 | tail -20`
Expected: builds cleanly (pure derive addition, no behavior change).

- [ ] **Step 2: Add the three fields to the `GameEngine` struct**

In `crates/loom-knit/src/engine/mod.rs`, find the struct (starts at `pub struct GameEngine {`, line 21) and add after `pub anim_cells: loom_engine::anim::AnimOverlay,`:

```rust
    /// Rows not yet shifted into the visible board (endless mode only; empty otherwise).
    pub row_buffer: Vec<Vec<BoardEntity>>,
    /// Total rows in the full generated board (endless mode only; 0 otherwise).
    pub total_rows: u32,
    /// Rows successfully shifted through so far (endless mode only).
    pub rows_cleared: u32,
```

- [ ] **Step 3: Add matching initializers to `GameEngine::new`**

In the same file, in `GameEngine::new` (the `Self { ... }` literal around line 125-149), add after `anim_cells: loom_engine::anim::AnimOverlay::new(),`:

```rust
            row_buffer: Vec::new(),
            total_rows: 0,
            rows_cleared: 0,
```

- [ ] **Step 4: Fix every other `GameEngine { .. }` struct literal so the crate compiles**

Run this to find every site that needs the same three fields added:

```bash
grep -n "anim_cells: loom_engine::anim::AnimOverlay::new()," crates/loom-knit/src/engine/mod.rs crates/loom-knit/tests/game_flow_test.rs
```

For **every** match (9 in `engine/mod.rs`'s test module, 3 in `game_flow_test.rs` — 12 total), add immediately after that line:

```rust
            row_buffer: Vec::new(),
            total_rows: 0,
            rows_cleared: 0,
```

(Match the existing indentation at each site — some are 4-space test helpers, some are inline in test bodies.)

- [ ] **Step 5: Run the full loom-knit test suite to confirm it compiles and passes**

Run: `cargo test -p knitui 2>&1 | tail -30`
Expected: compiles cleanly, all existing tests still pass (no behavior change yet — every new field is zero-valued).

- [ ] **Step 6: Add the same three fields to `GameStateSnapshot`, gated with `#[serde(default)]`**

In `crates/loom-knit/src/engine/mod.rs`, find `pub struct GameStateSnapshot {` (around line 572) and add after `pub generation_attempts: u32,`:

```rust
    #[serde(default)]
    pub row_buffer: Vec<Vec<String>>,
    #[serde(default)]
    pub total_rows: u32,
    #[serde(default)]
    pub rows_cleared: u32,
```

- [ ] **Step 7: Wire the new fields through `from_engine` and `into_engine`**

In `GameStateSnapshot::from_engine` (around line 611), add after `generation_attempts: e.generation_attempts,`:

```rust
            row_buffer: e.row_buffer.iter()
                .map(|row| row.iter().map(cell_to_str).collect())
                .collect(),
            total_rows: e.total_rows,
            rows_cleared: e.rows_cleared,
```

In `GameStateSnapshot::into_engine` (around line 654), first add this alongside the other `let ... = ...;` decodings near the top (after the `let balloon_cols = balloon_cols?;` line):

```rust
        let row_buffer: Result<Vec<Vec<BoardEntity>>, String> = self.row_buffer.iter()
            .map(|row| row.iter().map(|s| str_to_cell(s)).collect())
            .collect();
        let row_buffer = row_buffer?;
```

Then in the `Ok(GameEngine { ... })` literal, add after `generation_attempts: self.generation_attempts,`:

```rust
            row_buffer,
            total_rows: self.total_rows,
            rows_cleared: self.rows_cleared,
```

- [ ] **Step 8: Write a snapshot round-trip test**

Add to the `#[cfg(test)] mod tests` block in `crates/loom-knit/src/engine/mod.rs` (near the existing `snapshot_roundtrip` test):

```rust
    #[test]
    fn snapshot_roundtrip_preserves_row_buffer() {
        let mut e = default_engine();
        e.row_buffer = vec![
            vec![BoardEntity::Spool(Color::Red), BoardEntity::Void, BoardEntity::Obstacle],
            vec![BoardEntity::KeySpool(Color::Blue), BoardEntity::EmptyConveyor, BoardEntity::Void],
        ];
        e.total_rows = 66;
        e.rows_cleared = 12;
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).expect("roundtrip");
        assert_eq!(e2.row_buffer, e.row_buffer);
        assert_eq!(e2.total_rows, 66);
        assert_eq!(e2.rows_cleared, 12);
    }
```

- [ ] **Step 9: Run the test and confirm it passes**

Run: `cargo test -p knitui snapshot_roundtrip_preserves_row_buffer -- --nocapture`
Expected: PASS

- [ ] **Step 10: Commit**

```bash
git add crates/loom-knit/src/board_entity.rs crates/loom-knit/src/engine/mod.rs crates/loom-knit/tests/game_flow_test.rs
git commit -m "feat(knit): add row-buffer fields to GameEngine and its JSON snapshot"
```

---

### Task 2: `GameEngine::new_endless` constructor

**Files:**
- Modify: `crates/loom-knit/src/engine/mod.rs` (new method on `impl GameEngine`, new consts)
- Test: same file

**Interfaces:**
- Consumes: `Task 1`'s new fields; `GameBoard::make_random(height, width, palette, obstacle_pct, spool_capacity, conveyor_pct, conveyor_capacity) -> GameBoard`; `GameBoard::count_spools(&self) -> ColorCounter`; `Yarn::make_from_color_counter(counter, yarn_lines, visible_stitches) -> Yarn`; `crate::solvability::count_balance(board, yarn, spool_capacity) -> bool`; `crate::config::MAX_BOARD_DIM: u16` (already `6`).
- Produces: `pub fn GameEngine::new_endless(base_config: &Config) -> Self` — a fresh continuous-endless engine. Used by Task 5 (tui.rs) and Task 6 (knitui_ni.rs).

- [ ] **Step 1: Add the row-count constants near the top of `engine/mod.rs`**

Right after the `use` block (after `use crate::color_serde;`, around line 17), add:

```rust
/// Total rows generated for a continuous-endless board (visible + buffered).
pub const ENDLESS_TOTAL_ROWS: u16 = 66;
```

(The visible-row count reuses `crate::config::MAX_BOARD_DIM`, which is already `6` — no new constant needed for that.)

- [ ] **Step 2: Write the failing test for `new_endless`**

Add to the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn new_endless_builds_visible_and_buffered_rows() {
        let config = Config {
            board_height: 6, board_width: 6, color_number: 4,
            color_mode: "dark".into(), spool_limit: 7,
            spool_capacity: 2, yarn_lines: 4, obstacle_percentage: 5,
            visible_stitches: 6, conveyor_capacity: 3, conveyor_percentage: 5,
            layout: "auto".into(), scale: 1,
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2, ad_file: None,
            max_solutions: None,
            hard_mode: false,
        };
        let e = GameEngine::new_endless(&config);

        assert_eq!(e.board.height, 6);
        assert_eq!(e.board.width, 6);
        assert_eq!(e.board.board.len(), 6);
        assert_eq!(e.total_rows, ENDLESS_TOTAL_ROWS as u32);
        assert_eq!(e.rows_cleared, 0);
        // 60 buffered rows (66 total - 6 visible), each 6 cells wide
        assert_eq!(e.row_buffer.len(), 60);
        for row in &e.row_buffer {
            assert_eq!(row.len(), 6);
        }
        assert_eq!(e.bonuses.scissors, 999);
        assert_eq!(e.bonuses.tweezers, 999);
        assert_eq!(e.bonuses.balloons, 999);
    }

    #[test]
    fn new_endless_yarn_matches_full_board_color_demand() {
        let config = Config {
            board_height: 6, board_width: 6, color_number: 3,
            color_mode: "dark".into(), spool_limit: 7,
            spool_capacity: 2, yarn_lines: 4, obstacle_percentage: 0,
            visible_stitches: 6, conveyor_capacity: 3, conveyor_percentage: 0,
            layout: "auto".into(), scale: 1,
            scissors: 0, tweezers: 0, balloons: 0,
            scissors_spools: 1, balloon_count: 2, ad_file: None,
            max_solutions: None,
            hard_mode: false,
        };
        let e = GameEngine::new_endless(&config);
        // Yarn was generated from the FULL board (visible + buffer), so the
        // visible board alone won't balance — but every color used anywhere
        // in the yarn must appear somewhere in either the visible board or
        // the buffer (nothing invented, nothing missing).
        use std::collections::HashSet;
        let yarn_colors: HashSet<Color> = e.yarn.board.iter()
            .flat_map(|col| col.iter().map(|s| s.color))
            .collect();
        let mut board_colors: HashSet<Color> = e.board.board.iter().flatten()
            .filter_map(|c| match c {
                BoardEntity::Spool(col) | BoardEntity::KeySpool(col) => Some(*col),
                _ => None,
            }).collect();
        board_colors.extend(e.row_buffer.iter().flatten().filter_map(|c| match c {
            BoardEntity::Spool(col) | BoardEntity::KeySpool(col) => Some(*col),
            _ => None,
        }));
        assert!(yarn_colors.is_subset(&board_colors));
    }
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p knitui new_endless -- --nocapture`
Expected: FAIL with "no function or associated item named `new_endless`"

- [ ] **Step 4: Implement `new_endless`**

In `crates/loom-knit/src/engine/mod.rs`, inside `impl GameEngine { ... }`, add after the existing `new` method (after its closing `}` around line 150):

```rust
    /// Build a fresh continuous-endless engine: one tall board (visible + buffer),
    /// yarn generated to match the whole board's color demand, unlimited bonuses.
    pub fn new_endless(base_config: &Config) -> Self {
        let visible_rows = crate::config::MAX_BOARD_DIM;
        let color_mode = base_config.parsed_color_mode();
        let selected_palette = select_palette(color_mode, base_config.color_number);

        let mut full_board = GameBoard::make_random(
            ENDLESS_TOTAL_ROWS,
            visible_rows, // width stays fixed at the visible column count
            &selected_palette,
            base_config.obstacle_percentage,
            base_config.spool_capacity,
            base_config.conveyor_percentage,
            base_config.conveyor_capacity,
        );

        let yarn = Yarn::make_from_color_counter(
            full_board.count_spools(),
            base_config.yarn_lines,
            base_config.visible_stitches,
        );
        debug_assert!(
            crate::solvability::count_balance(&full_board, &yarn, base_config.spool_capacity),
            "endless yarn must balance the full generated board by construction"
        );

        // Split into visible window (top `visible_rows` rows) + buffer (the rest).
        // `Vec::split_off(n)` returns everything from index `n` onward and leaves
        // the first `n` elements in place, so `full_board.board` is already left
        // holding exactly the visible rows after this call.
        let row_buffer = full_board.board.split_off(visible_rows as usize);
        full_board.height = visible_rows;

        // Find first focusable cell for initial cursor position (same logic as `new`).
        let (mut init_row, mut init_col) = (0u16, 0u16);
        'find_cursor: for r in 0..full_board.height {
            for c in 0..full_board.width {
                if full_board.is_focusable(r as usize, c as usize) {
                    init_row = r;
                    init_col = c;
                    break 'find_cursor;
                }
            }
        }

        Self {
            board: full_board,
            yarn,
            held_spools: Vec::new(),
            cursor_row: init_row,
            cursor_col: init_col,
            spool_capacity: base_config.spool_capacity,
            spool_limit: base_config.spool_limit,
            bonuses: BonusInventory {
                scissors: 999,
                tweezers: 999,
                balloons: 999,
                scissors_spools: base_config.scissors_spools,
                balloon_count: base_config.balloon_count,
            },
            bonus_state: BonusState::None,
            blessing_flags: BlessingFlags::default(),
            last_picked_color: None,
            ad_limit: None,
            ads_used: 0,
            generation_attempts: 0,
            hint_cell: None,
            hint_ticks: 0,
            anim_cells: loom_engine::anim::AnimOverlay::new(),
            row_buffer,
            total_rows: ENDLESS_TOTAL_ROWS as u32,
            rows_cleared: 0,
        }
    }
```


- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p knitui new_endless -- --nocapture`
Expected: PASS (both tests)

- [ ] **Step 6: Run the full loom-knit suite**

Run: `cargo test -p knitui 2>&1 | tail -20`
Expected: all pass

- [ ] **Step 7: Commit**

```bash
git add crates/loom-knit/src/engine/mod.rs
git commit -m "feat(knit): add GameEngine::new_endless for continuous endless mode"
```

---

### Task 3: Row-shift logic + `is_won()` fix

**Files:**
- Modify: `crates/loom-knit/src/engine/mod.rs` (`process_all_active`, `is_won`)
- Test: same file

**Interfaces:**
- Consumes: `Task 1`'s fields, `Task 2`'s constructor (for test fixtures).
- Produces: `process_all_active()` now shifts exhausted rows as its last step when `total_rows > 0`. `is_won()` now also requires `row_buffer.is_empty()`.

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block:

```rust
    fn endless_test_engine(visible: Vec<Vec<BoardEntity>>, buffer: Vec<Vec<BoardEntity>>) -> GameEngine {
        let height = visible.len() as u16;
        let width = visible[0].len() as u16;
        let total_rows = height as u32 + buffer.len() as u32;
        GameEngine {
            board: GameBoard { board: visible, height, width, spool_capacity: 1 },
            yarn: Yarn { board: vec![vec![]], yarn_lines: 1, visible_stitches: 3, balloon_columns: Vec::new() },
            held_spools: vec![],
            cursor_row: 2, cursor_col: 0,
            spool_capacity: 1, spool_limit: 5,
            bonuses: BonusInventory { scissors: 999, tweezers: 999, balloons: 999, scissors_spools: 1, balloon_count: 2 },
            bonus_state: BonusState::None,
            blessing_flags: BlessingFlags::default(),
            last_picked_color: None,
            ad_limit: None, ads_used: 0,
            generation_attempts: 0,
            hint_cell: None, hint_ticks: 0,
            anim_cells: loom_engine::anim::AnimOverlay::new(),
            row_buffer: buffer,
            total_rows,
            rows_cleared: 0,
        }
    }

    #[test]
    fn process_all_active_shifts_exhausted_top_row_from_buffer() {
        let visible = vec![
            vec![BoardEntity::Void, BoardEntity::Obstacle],       // row 0: exhausted
            vec![BoardEntity::Spool(Color::Red), BoardEntity::Void], // row 1: has content
        ];
        let buffer = vec![
            vec![BoardEntity::Spool(Color::Blue), BoardEntity::Void], // next buffered row
        ];
        let mut e = endless_test_engine(visible, buffer);
        e.process_all_active(); // no held spools, but the shift check still runs

        assert_eq!(e.board.board.len(), 2, "still 6-tall.. here 2-tall since test board is 2 rows");
        assert_eq!(e.board.board[0], vec![BoardEntity::Spool(Color::Red), BoardEntity::Void]);
        assert_eq!(e.board.board[1], vec![BoardEntity::Spool(Color::Blue), BoardEntity::Void]);
        assert!(e.row_buffer.is_empty());
        assert_eq!(e.rows_cleared, 1);
        assert_eq!(e.cursor_row, 1, "cursor at row 2 saturating_sub(1) = 1");
    }

    #[test]
    fn process_all_active_shrinks_board_when_buffer_empty() {
        let visible = vec![
            vec![BoardEntity::Void, BoardEntity::Obstacle],
            vec![BoardEntity::Spool(Color::Red), BoardEntity::Void],
        ];
        let mut e = endless_test_engine(visible, Vec::new());
        e.process_all_active();

        assert_eq!(e.board.board.len(), 1);
        assert_eq!(e.board.height, 1);
        assert_eq!(e.rows_cleared, 1);
    }

    #[test]
    fn process_all_active_does_not_shift_for_non_endless_engine() {
        let mut e = default_engine(); // total_rows == 0
        let before = e.board.board.clone();
        e.process_all_active();
        assert_eq!(e.board.board, before);
    }

    #[test]
    fn is_won_false_while_row_buffer_has_rows_even_if_visible_board_clear() {
        let visible = vec![
            vec![BoardEntity::Void, BoardEntity::Void],
        ];
        let buffer = vec![
            vec![BoardEntity::Spool(Color::Red), BoardEntity::Void],
        ];
        let e = endless_test_engine(visible, buffer);
        assert!(!e.is_won(), "row_buffer still has content, must not be won");
    }

    #[test]
    fn is_won_true_when_board_and_buffer_both_clear() {
        let visible = vec![
            vec![BoardEntity::Void, BoardEntity::Void],
        ];
        let e = endless_test_engine(visible, Vec::new());
        assert!(e.is_won());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p knitui process_all_active_shifts -- --nocapture`
Expected: FAIL (`process_all_active` doesn't shift rows yet; `is_won` doesn't check `row_buffer` yet — the two `is_won_*` tests should currently fail or pass vacuously wrong)

- [ ] **Step 3: Fix `is_won()`**

In `crates/loom-knit/src/engine/mod.rs`, find `pub fn is_won(&self) -> bool {` (around line 319) and change:

```rust
    pub fn is_won(&self) -> bool {
        self.held_spools.is_empty()
            && self.yarn.board.iter().all(|col| col.is_empty())
            && self.board.board.iter().all(|row| {
                row.iter().all(|cell| !matches!(
                    cell,
                    BoardEntity::Spool(_) | BoardEntity::KeySpool(_)
                        | BoardEntity::Conveyor(_)
                ))
            })
    }
```

to:

```rust
    pub fn is_won(&self) -> bool {
        self.row_buffer.is_empty()
            && self.held_spools.is_empty()
            && self.yarn.board.iter().all(|col| col.is_empty())
            && self.board.board.iter().all(|row| {
                row.iter().all(|cell| !matches!(
                    cell,
                    BoardEntity::Spool(_) | BoardEntity::KeySpool(_)
                        | BoardEntity::Conveyor(_)
                ))
            })
    }
```

- [ ] **Step 4: Add the row-shift method and call it from `process_all_active`**

In the same file, add this new method right after `process_all_active` (after its closing `}`, around line 317):

```rust
    /// Shift exhausted top rows out of the visible board and pull buffered
    /// rows in from the bottom. No-op for non-endless engines (`total_rows == 0`).
    fn shift_exhausted_rows(&mut self) {
        if self.total_rows == 0 {
            return;
        }
        loop {
            let Some(top_row) = self.board.board.first() else { break };
            let exhausted = top_row.iter().all(|cell| matches!(
                cell,
                BoardEntity::Void | BoardEntity::Obstacle | BoardEntity::EmptyConveyor
            ));
            if !exhausted {
                break;
            }
            self.board.board.remove(0);
            self.rows_cleared += 1;
            self.cursor_row = self.cursor_row.saturating_sub(1);
            if !self.row_buffer.is_empty() {
                self.board.board.push(self.row_buffer.remove(0));
            }
            self.board.height = self.board.board.len() as u16;
        }
    }
```

Then change `process_all_active` (around line 304-317) from:

```rust
    pub fn process_all_active(&mut self) {
        let mut i = 0;
        let count = self.held_spools.len();
        for _ in 0..count {
            if i >= self.held_spools.len() { break; }
            self.yarn.process_one(&mut self.held_spools[i]);
            if self.held_spools[i].fill > self.spool_capacity {
                self.held_spools.remove(i);
            } else {
                i += 1;
            }
        }
        self.yarn.cleanup_balloon_columns();
    }
```

to:

```rust
    pub fn process_all_active(&mut self) {
        let mut i = 0;
        let count = self.held_spools.len();
        for _ in 0..count {
            if i >= self.held_spools.len() { break; }
            self.yarn.process_one(&mut self.held_spools[i]);
            if self.held_spools[i].fill > self.spool_capacity {
                self.held_spools.remove(i);
            } else {
                i += 1;
            }
        }
        self.yarn.cleanup_balloon_columns();
        self.shift_exhausted_rows();
    }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p knitui process_all_active -- --nocapture` and `cargo test -p knitui is_won -- --nocapture`
Expected: all PASS

- [ ] **Step 6: Run the full loom-knit suite**

Run: `cargo test -p knitui 2>&1 | tail -20`
Expected: all pass (this includes the pre-existing `is_won_true_when_board_cleared` and `is_won_false_while_board_has_spools` tests — both construct engines via the pre-Task-1 literal style with `row_buffer: Vec::new()` from Step 3 of Task 1, so `is_won()`'s new `row_buffer.is_empty()` check is trivially true for them and doesn't change their outcome)

- [ ] **Step 7: Commit**

```bash
git add crates/loom-knit/src/engine/mod.rs
git commit -m "feat(knit): shift exhausted rows in process_all_active, fix is_won for endless"
```

---

### Task 4: Renderer — rows-remaining counter and rewritten game-over screen

**Files:**
- Modify: `crates/loom-knit/src/renderer/panels.rs` (`render_endless_gameover`, `render_bonus_display_h`, `render_bonus_panel`)
- Test: manual verification only (renderer functions have no unit tests in this codebase, consistent with existing renderer code — see Batch 1's held-spool counter, which also shipped without renderer unit tests)

**Interfaces:**
- Consumes: `GameEngine.total_rows`, `GameEngine.rows_cleared` (Task 1).
- Produces: no new public signatures other than changed parameter names on `render_endless_gameover`; nothing downstream depends on the old ones since Task 5 is the only caller and is updated in this same plan.

- [ ] **Step 1: Rewrite `render_endless_gameover` to show rows cleared instead of wave**

In `crates/loom-knit/src/renderer/panels.rs`, replace the whole function (currently at line 489-530):

```rust
/// Render the endless mode game-over screen (shown when stuck).
pub fn render_endless_gameover(
    stdout: &mut Stdout,
    rows_cleared: u32,
    best_rows_cleared: usize,
) -> io::Result<()> {
    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Hide)?;
    stdout.queue(Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
    let start_y = term_h / 2 - 3;

    let title = "═══ ENDLESS MODE ═══";
    let tx = (term_w.saturating_sub(title.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(tx, start_y))?;
    stdout.queue(Print(title))?;

    let rows_str = format!("You cleared {} rows", rows_cleared);
    let rx2 = (term_w.saturating_sub(rows_str.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(rx2, start_y + 2))?;
    stdout.queue(Print(&rows_str))?;

    if rows_cleared as usize >= best_rows_cleared {
        let record_str = "New record!";
        let rx = (term_w.saturating_sub(record_str.chars().count() as u16)) / 2;
        stdout.queue(MoveTo(rx, start_y + 3))?;
        stdout.queue(Print(record_str))?;
    } else {
        let best_str = format!("Best: {} rows", best_rows_cleared);
        let bx = (term_w.saturating_sub(best_str.chars().count() as u16)) / 2;
        stdout.queue(MoveTo(bx, start_y + 3))?;
        stdout.queue(Print(best_str.dark_grey()))?;
    }

    let hint = "R:Play Again  M:Menu  Q:Quit";
    let hx = (term_w.saturating_sub(hint.chars().count() as u16)) / 2;
    stdout.queue(MoveTo(hx, start_y + 5))?;
    stdout.queue(Print(hint.dark_grey()))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}
```

- [ ] **Step 2: Add a "Rows: N remaining" indicator to both bonus panels**

In `crates/loom-knit/src/renderer/panels.rs`, in `render_bonus_display_h` (starts line 532), find the held-spool counter block (ends with the `color_count` blessing check) and add right before the final `Ok(())`:

```rust
    // Endless mode: rows remaining in the buffer
    if engine.total_rows > 0 {
        let remaining = engine.total_rows.saturating_sub(engine.rows_cleared);
        stdout.queue(Print("  "))?;
        stdout.queue(Print(format!("Rows: {} remaining", remaining).dark_grey()))?;
    }
```

Do the same in `render_bonus_panel` (starts line 567) — find its equivalent held-spool-counter block and add the same snippet before its final `Ok(())`, adjusting the `MoveTo` row offset the same way the existing held-spool counter does in that function (match the pattern already used there for consistency — read the function body first to place it on its own row like the other panel does, since `render_bonus_panel` is the vertical/multi-row layout, unlike `render_bonus_display_h`'s single inline row).

- [ ] **Step 3: Build to confirm it compiles**

Run: `cargo build -p knitui 2>&1 | tail -20`
Expected: builds cleanly (renderer changes only, no test to run yet — this crate's renderer has no unit tests, matching existing convention)

- [ ] **Step 4: Commit**

```bash
git add crates/loom-knit/src/renderer/panels.rs
git commit -m "feat(knit): show rows-cleared and remaining-rows in endless mode UI"
```

---

### Task 5: Strip `EndlessState`, rewire `tui.rs`

**Files:**
- Modify: `crates/loom-knit/src/endless.rs` (remove `EndlessState` struct/impl/tests entirely, keep only the `EndlessHighScore` re-export)
- Modify: `crates/loom-knit/src/tui.rs` (endless flow: menu entry, restart, tick-loop status handling, remove `advance_endless_wave`)
- Test: `cargo test -p knitui` (no new unit tests here — this is UI wiring; behavior is already covered by Task 2/3's engine tests. Manual verification via `cargo run --bin knitui` is the real check, per this plan's final task)

**Interfaces:**
- Consumes: `GameEngine::new_endless` (Task 2), `GameEngine.total_rows`/`.rows_cleared` (Task 1), `render_endless_gameover(rows_cleared: u32, best_rows_cleared: usize)` (Task 4).
- Produces: nothing new — this task only removes/rewires existing call sites so the crate compiles and behaves per the spec.

- [ ] **Step 1: Strip `EndlessState` out of `endless.rs`**

Replace the entire contents of `crates/loom-knit/src/endless.rs` with just:

```rust
pub use loom_engine::endless::EndlessHighScore;
```

(This deletes the `EndlessState` struct, its `impl` block, and its 3 tests — `new_state_starts_at_wave_one`, `advance_increments_wave_and_awards_bonus`, `to_config_scales_with_wave` — along with `MAX_BOARD_DIM`/`Config` imports that were only used by the removed code. The 3 `EndlessHighScore`-related tests, `high_score_update_initial_record`, `high_score_update_returns_true_for_new_record`, `high_score_serialization_roundtrip`, also get removed since they were testing shared `loom-engine` infra by re-implementing its own tests here — that infra is already covered by `loom-engine`'s own test suite, so nothing goes untested.)

- [ ] **Step 2: Update the `tui.rs` import**

In `crates/loom-knit/src/tui.rs`, change:

```rust
use crate::endless::{EndlessState, EndlessHighScore};
```

to:

```rust
use crate::endless::EndlessHighScore;
```

- [ ] **Step 3: Remove `advance_endless_wave` and update the `endless_ctx` variable**

Delete the `advance_endless_wave` function entirely (lines 127-139):

```rust
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
```

In `run_event_loop`, change the state declaration (around line 192):

```rust
    let mut endless_ctx: Option<EndlessState> = None;
```

to:

```rust
    let mut is_endless = false;
```

(`is_endless` tracks the same "are we in endless mode" signal `endless_ctx.is_some()` used to, but without a separate struct — the actual row-buffer state now lives inside `engine` itself via `total_rows`.)

- [ ] **Step 4: Update the MainMenu "Endless" entry (menu item 3)**

Around line 244-256, change:

```rust
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
```

to:

```rust
                                    3 => {
                                        engine = Some(GameEngine::new_endless(&cli_config));
                                        geo = LayoutGeometry::compute(&cli_config);
                                        is_endless = true;
                                        tui_state = TuiState::Playing;
                                        renderer::do_render(
                                            &mut stdout, engine.as_ref().unwrap(),
                                            geo.layout, geo.yarn_x, geo.board_x, geo.board_y, geo.scale,
                                        )?;
                                        continue;
                                    }
```

(`LayoutGeometry::compute` takes a `&Config` and only reads `board_height`/`board_width` from it to size the render area — `cli_config`'s default 6×6 is correct here since the *visible* board is always 6×6, matching what `GameEngine::new_endless` actually built. The previous code passed `game_config` because the old wave system varied board size 4×4 up to 6×6; that variability is gone.)

- [ ] **Step 5: Update the GameOver "R/N: retry" endless restart (around line 575-581)**

Change:

```rust
                                if endless_ctx.is_some() {
                                    endless_ctx = None;
                                    let state = EndlessState::new();
                                    game_config = state.to_config(&cli_config);
                                    geo = LayoutGeometry::compute(&game_config);
                                    engine = Some(GameEngine::new(&game_config));
                                    endless_ctx = Some(state);
```

to:

```rust
                                if is_endless {
                                    engine = Some(GameEngine::new_endless(&cli_config));
                                    geo = LayoutGeometry::compute(&cli_config);
```

(Read the surrounding lines first — this `if` block continues after the shown snippet with `tui_state = TuiState::Playing;` and a `renderer::do_render(...)` call; leave those lines as they are, only replace the 6 lines shown above with the 3-line replacement.)

- [ ] **Step 6: Reset `is_endless` wherever `endless_ctx = None` currently appears**

Find the remaining `endless_ctx = None;` line (around line 633, in the menu/quit handling) and change it to `is_endless = false;`.

- [ ] **Step 7: Update both `GameStatus::Won if endless_ctx.is_some()` blocks**

There are two near-identical blocks (around line 727-728 and line 821-822), both currently:

```rust
                                        GameStatus::Won if endless_ctx.is_some() => {
                                            advance_endless_wave(&mut endless_ctx, &mut game_config, &cli_config, &mut geo, &mut engine);
```

Continuous endless has no "advance to next wave" step — a win only happens when the whole 66-row board is cleared (`is_won()` per Task 3's fix), which is a genuine game-over, not a mid-game transition. So both blocks should be **deleted** — remove the `GameStatus::Won if endless_ctx.is_some() => { ... }` arm entirely from both match statements, letting `GameStatus::Won` fall through to the existing plain `s @ GameStatus::Won => { tui_state = TuiState::Celebration { ticks_remaining: 16, next_status: s }; }` arm that already exists right after it in both places. (Read each match statement fully before editing to confirm the fallthrough arm is there and unchanged — it already handles the "won" celebration+overlay flow generically for every mode.)

- [ ] **Step 8: Update both Stuck-with-endless branches to show rows instead of wave**

Both blocks also contain (around line 737-741 and 829-833):

```rust
                                            if endless_ctx.is_some() && s == GameStatus::Stuck {
                                                let wave = endless_ctx.as_ref().unwrap().wave;
                                                endless_hs.update(wave);
                                                endless_hs.save("knitui");
                                                renderer::render_endless_gameover(&mut stdout, wave, endless_hs.best_wave)?;
```

Change each to:

```rust
                                            if is_endless && s == GameStatus::Stuck {
                                                let rows_cleared = engine.as_ref().unwrap().rows_cleared;
                                                endless_hs.update(rows_cleared as usize);
                                                endless_hs.save("knitui");
                                                renderer::render_endless_gameover(&mut stdout, rows_cleared, endless_hs.best_wave)?;
```

(`endless_hs.best_wave` keeps its field name — per this plan's Global Constraints, `EndlessHighScore` is shared infra and isn't renamed; it's just reinterpreted as "best rows cleared" for knit.)

- [ ] **Step 9: Remove now-unused `Config`/`game_config` mutation tied to endless, if any remain**

Search for any remaining reference to `game_config` that was only ever set from `EndlessState::to_config` (the menu-entry and restart sites already handled in Steps 4-5 no longer touch `game_config` at all for endless). Confirm with:

```bash
grep -n "endless_ctx\|EndlessState" crates/loom-knit/src/tui.rs
```

Expected: no matches remain.

- [ ] **Step 10: Build and run the full test suite**

Run: `cargo build -p knitui 2>&1 | tail -30`
Expected: builds cleanly with no leftover references to `EndlessState`/`endless_ctx`.

Run: `cargo test -p knitui 2>&1 | tail -20`
Expected: all pass.

- [ ] **Step 11: Commit**

```bash
git add crates/loom-knit/src/endless.rs crates/loom-knit/src/tui.rs
git commit -m "feat(knit): rewire TUI endless flow onto continuous GameEngine::new_endless"
```

---

### Task 6: `knitui-ni` — `--endless` flag, drop `DescribeWave`

**Files:**
- Modify: `crates/loom-knit/src/bin/knitui_ni.rs`
- Test: `crates/loom-knit/tests/knitui_ni_test.rs` (check existing endless-wave tests first and update/replace them)

**Interfaces:**
- Consumes: `GameEngine::new_endless` (Task 2).
- Produces: `--endless` CLI flag on the `knitui-ni` binary; `DescribeWave` subcommand removed.

- [ ] **Step 1: Confirm there are no existing `--endless-wave`/`DescribeWave` tests to migrate**

Run:
```bash
grep -n "endless" crates/loom-knit/tests/knitui_ni_test.rs
```

Expected: no matches (`crates/loom-knit/tests/knitui_ni_test.rs` currently has no endless-specific tests at all — the only existing coverage of the old `--endless-wave`/`describe-wave` behavior was manual). Step 6 below adds fresh coverage for `--endless` from scratch, not a replacement.

- [ ] **Step 2: Replace the `--endless-wave` flag with `--endless`**

In `crates/loom-knit/src/bin/knitui_ni.rs`, change the `Args` struct (around line 50-52):

```rust
    // Endless-mode game creation
    #[arg(long, help = "Create a game for an endless-mode wave number")]
    endless_wave: Option<usize>,
```

to:

```rust
    // Endless-mode game creation
    #[arg(long, help = "Create a fresh continuous-endless game")]
    endless: bool,
```

- [ ] **Step 3: Remove the `DescribeWave` subcommand and its handler**

Remove this variant from the `NiCommand` enum (around line 76-79):

```rust
    /// Describe the config for a given endless-mode wave
    DescribeWave {
        #[arg(help = "Wave number (1-based)")]
        wave: usize,
    },
```

Remove its dispatch arm in `main()` (around line 352-354):

```rust
        Some(NiCommand::DescribeWave { wave }) => {
            handle_describe_wave(*wave);
            return;
        }
```

Remove the whole `handle_describe_wave` function (around line 288-310 — read its full body first since the exact end line may differ slightly from this plan's earlier read; delete from `fn handle_describe_wave(wave: usize) {` through its matching closing `}`).

Remove the now-dead `Some(NiCommand::DescribeWave { .. })` arm from the `unreachable!("handled above")` match (around line 427-431) — leave `ListCampaign` and `BatchGenerate` in that arm, just drop the `DescribeWave` line.

- [ ] **Step 4: Update `resolve_config` to stop handling `endless_wave`**

Change (around line 200-211):

```rust
    } else if let Some(wave) = args.endless_wave {
        let mut state = EndlessState::new();
        for _ in 1..wave {
            state.advance();
        }
        // Override wave to target
        state.wave = wave;
        let mut config = state.to_config(&base_config());
        // Force bonuses to 0 for the base endless config (banked comes from advance)
        // Allow CLI overrides
        config = config_from_args(args, config);
        config
    } else {
```

to:

```rust
    } else if args.endless {
        // Endless games are constructed directly via GameEngine::new_endless in
        // main() (not through this Config-only resolver), since they need the
        // tall-board split logic, not just a Config. This branch only needs to
        // exist so `campaign` and `endless` remain mutually exclusive paths;
        // it's unreachable in practice because main() checks args.endless first.
        base_config()
    } else {
```

- [ ] **Step 5: Wire `--endless` into `main()`'s "Create a new game" branch**

Change (around line 446-448):

```rust
        None => {
            let config = resolve_config(&args);
            let mut engine = GameEngine::new(&config);
```

to:

```rust
        None => {
            let mut engine = if args.endless {
                GameEngine::new_endless(&config_from_args(&args, base_config()))
            } else {
                let config = resolve_config(&args);
                GameEngine::new(&config)
            };
```

(`config_from_args` still applies CLI overrides like `--color-mode`/`--color-number`/`--obstacle-percentage` on top of `base_config()` for endless games, consistent with every other mode — only `board_height`/`board_width` overrides are meaningless here since `new_endless` ignores them and always builds 66×6, which is fine: the flag is simply a no-op for endless, not an error.)

- [ ] **Step 6: Add `--endless` test coverage**

In `crates/loom-knit/tests/knitui_ni_test.rs`, add these two tests near `test_create_game_custom_options` (using the file's existing `create_game`/`run` helpers, defined at the top of the file):

```rust
#[test]
fn test_create_endless_game() {
    let (_, v) = create_game(&["--endless"]);
    let state = &v["state"];
    assert_eq!(state["board_height"], 6);
    assert_eq!(state["board_width"], 6);
    assert_eq!(state["total_rows"], 66);
    assert_eq!(state["rows_cleared"], 0);
    assert_eq!(state["row_buffer"].as_array().unwrap().len(), 60);
    assert_eq!(state["scissors"], 999);
    assert_eq!(state["tweezers"], 999);
    assert_eq!(state["balloons"], 999);
}

#[test]
fn test_endless_game_persists_row_buffer_across_commands() {
    let (hash, v) = create_game(&["--endless"]);
    let buffer_len_before = v["state"]["row_buffer"].as_array().unwrap().len();

    let (stdout, _, code) = run(&["--game", &hash, "move", "right"]);
    assert_eq!(code, 0, "move failed: {stdout}");
    let v2 = parse_ok(&stdout);
    // Row buffer must round-trip through the --game save/load cycle unchanged
    // by an action that doesn't clear a row.
    assert_eq!(
        v2["state"]["row_buffer"].as_array().unwrap().len(),
        buffer_len_before
    );
    assert_eq!(v2["state"]["total_rows"], 66);
}
```

(`test_endless_game_persists_row_buffer_across_commands` exercises exactly the `--game <hash>` save/load round-trip this plan's architecture depends on — Task 1's `GameStateSnapshot` changes are what make `row_buffer`/`total_rows`/`rows_cleared` survive the trip from the `--endless` creation call to the following `move` call as two separate process invocations.)

- [ ] **Step 7: Run the knitui-ni tests**

Run: `cargo test -p knitui --test knitui_ni_test 2>&1 | tail -30`
Expected: all pass.

- [ ] **Step 8: Build and run the full workspace test suite**

Run: `cargo build --workspace --all-targets 2>&1 | tail -20`
Expected: no errors, no leftover references to `EndlessState`, `endless_wave`, or `DescribeWave` anywhere in the workspace.

Run: `cargo test -p knitui 2>&1 | tail -20`
Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add crates/loom-knit/src/bin/knitui_ni.rs crates/loom-knit/tests/knitui_ni_test.rs
git commit -m "feat(knit): replace knitui-ni --endless-wave with continuous --endless flag"
```

---

### Task 7: Full verification and manual playtest

**Files:** none (verification only)

- [ ] **Step 1: Full workspace build**

Run: `cargo build --workspace --all-targets 2>&1 | tail -30`
Expected: clean build, no warnings about unused `EndlessState`/`advance_endless_wave`/etc.

- [ ] **Step 2: Full workspace test suite**

Run: `cargo test --workspace 2>&1 | grep -E "test result:|FAILED|error\[|panicked"`
Expected: every `test result:` line shows `0 failed`.

- [ ] **Step 3: Clippy check on the touched crate**

Run: `cargo clippy -p knitui --all-targets 2>&1 | tail -40`
Expected: no new warnings beyond whatever pre-existing ones were already there before this plan (compare against a `git stash` + clippy run if unsure which warnings are pre-existing).

- [ ] **Step 4: Manual playtest via the TUI**

Run: `cargo run --bin knitui`, select "Endless" from the main menu, and verify:
- Board starts at 6×6.
- Picking/processing spools until a row empties visibly shifts a new row in from below.
- The "Rows: N remaining" counter decrements as rows shift.
- Bonuses show 999/999/999 and never run out.
- Losing (getting stuck) shows "You cleared N rows" and a best-score comparison, not a wave number.

- [ ] **Step 5: Manual CLI check**

Run:
```bash
cargo run --bin knitui-ni -- --endless
```
Expected: JSON response with `"status": "ok"` and a `state.board_height` of 6, `state.board_width` of 6.

- [ ] **Step 6: Commit any final fixups discovered during manual verification**

If Steps 3-5 turn up issues, fix them and commit with a message describing what was found (e.g. `fix(knit): <specific issue found during endless mode playtest>`). If nothing turns up, no commit needed for this task.
