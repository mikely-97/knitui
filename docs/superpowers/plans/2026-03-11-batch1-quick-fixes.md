# Batch 1: Quick Fixes Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix board max size to 6x6, make locks/keys actually spawn, and add a held-spool counter.

**Architecture:** Three independent fixes touching config, board generation, renderer, and campaign levels. Each fix is self-contained and can be committed separately.

**Tech Stack:** Rust, crossterm (terminal rendering), clap (CLI args), rand (RNG)

---

## Chunk 1: Board Max 6x6 (Task 1)

### Task 1: Cap board dimensions to 6x6 everywhere

**Files:**
- Modify: `src/config.rs` — add `MAX_BOARD_DIM` constant
- Modify: `src/main.rs:100-116` — cap `adjust_custom_field` sliders
- Modify: `src/endless.rs:35-47` — cap `to_config()` scaling
- Modify: `src/campaign_levels.rs` — clamp all levels exceeding 6 in either dimension
- Test: existing tests in `src/campaign_levels.rs` (the `all_levels_have_valid_board_sizes` test gets a stricter assertion)

- [ ] **Step 1: Add MAX_BOARD_DIM constant to config.rs**

In `src/config.rs`, add before the `Config` struct:

```rust
/// Hard cap on board dimensions (height and width).
pub const MAX_BOARD_DIM: u16 = 6;
```

- [ ] **Step 2: Write a test enforcing the cap on campaign levels**

In `src/campaign_levels.rs`, add to the `tests` module:

```rust
#[test]
fn all_levels_respect_max_board_dim() {
    use crate::config::MAX_BOARD_DIM;
    for levels in [SHORT_CAMPAIGN, MEDIUM_CAMPAIGN, LONG_CAMPAIGN] {
        for (i, level) in levels.iter().enumerate() {
            assert!(level.board_height <= MAX_BOARD_DIM,
                "level {}: board_height {} exceeds max {}", i, level.board_height, MAX_BOARD_DIM);
            assert!(level.board_width <= MAX_BOARD_DIM,
                "level {}: board_width {} exceeds max {}", i, level.board_width, MAX_BOARD_DIM);
        }
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test all_levels_respect_max_board_dim -- --nocapture`
Expected: FAIL — several Short/Medium/Long campaign levels exceed 6.

- [ ] **Step 4: Clamp campaign levels that exceed 6**

In `src/campaign_levels.rs`, find every `CampaignLevel` with `board_height > 6` or `board_width > 6` and clamp both to 6. To compensate for the smaller board, bump `color_number` by +1 (capped at 8) and/or `conveyor_capacity` by +1 on each clamped level, as the spec prescribes.

To find all offending levels, run this grep before editing:
```bash
grep -n 'board_height: [789]' src/campaign_levels.rs
grep -n 'board_width: [789]' src/campaign_levels.rs
```

For each match: set the offending dimension to 6, and add +1 to `color_number` (cap at 8) or +1 to `conveyor_capacity` (cap at 5) to compensate for the reduced cell count.

- [ ] **Step 5: Run the new test to verify it passes**

Run: `cargo test all_levels_respect_max_board_dim -- --nocapture`
Expected: PASS

- [ ] **Step 6: Cap the custom game sliders in main.rs**

In `src/main.rs`, function `adjust_custom_field`, change:

```rust
// Before:
1 => apply(&mut config.board_height, 2, 20),
2 => apply(&mut config.board_width, 2, 20),

// After (import MAX_BOARD_DIM at top of file):
1 => apply(&mut config.board_height, 2, MAX_BOARD_DIM),
2 => apply(&mut config.board_width, 2, MAX_BOARD_DIM),
```

Add `use knitui::config::MAX_BOARD_DIM;` to imports at the top of `main.rs`.

- [ ] **Step 7: Cap endless mode scaling in endless.rs**

In `src/endless.rs`, function `to_config()`, change:

```rust
// Before:
cfg.board_height = (4 + w / 3).min(10);
cfg.board_width  = (4 + w / 3).min(10);

// After (import MAX_BOARD_DIM):
use crate::config::MAX_BOARD_DIM;
// ...
cfg.board_height = (4 + w / 3).min(MAX_BOARD_DIM);
cfg.board_width  = (4 + w / 3).min(MAX_BOARD_DIM);
```

- [ ] **Step 8: Update the endless to_config test**

In `src/endless.rs`, the existing test `to_config_scales_with_wave` asserts that higher waves produce larger boards. After the cap change, the scaling formula `(4 + w/3).min(6)` still produces growth from wave 1 (4x4) to wave 6+ (6x6), so the existing `cfg10.board_height > cfg1.board_height` assertion still holds (6 > 4). However, any assertion that boards reach 10x10 must be updated. Find any `assert_eq!(..., 10)` or `assert!(... >= 10)` on board dimensions and change to `<= 6`. Specifically:

```rust
// If the test checks that wave 20 reaches a certain size, update:
// Before: assert_eq!(cfg20.board_height, 10);
// After:
assert_eq!(cfg20.board_height, 6);
assert_eq!(cfg20.board_width, 6);
```

Read the full test body before editing to identify every assertion that references the old cap of 10.

- [ ] **Step 9: Run all tests**

Run: `cargo test`
Expected: all pass

- [ ] **Step 10: Commit**

```bash
git add src/config.rs src/main.rs src/endless.rs src/campaign_levels.rs
git commit -m "feat: enforce 6x6 max board size everywhere"
```

---

## Chunk 2: Lock/Key Spawning Fix (Tasks 2-3)

### Task 2: Add key-placement pass to make_random

**Files:**
- Modify: `src/game_board.rs:16-100` — add Pass 4 for KeySpool placement
- Test: `src/game_board.rs` (tests module)

- [ ] **Step 1: Write the failing test for key spawning**

In `src/game_board.rs`, add to the `tests` module:

```rust
#[test]
fn test_keys_spawn_on_large_boards() {
    let palette = vec![Color::Red, Color::Blue, Color::Green];
    // 6x6 board with 0 obstacles → ~36 spools → should get ~3 keys
    let board = GameBoard::make_random(6, 6, &palette, 0, 2, 0, 0);
    let key_count = board.board.iter().flatten().filter(|e| {
        matches!(e, BoardEntity::KeySpool(_))
    }).count();
    assert!(key_count > 0, "expected at least 1 KeySpool on a 6x6 board");
}

#[test]
fn test_no_keys_on_tiny_boards() {
    let palette = vec![Color::Red];
    // 2x2 board → 4 spools → below threshold of 12
    let board = GameBoard::make_random(2, 2, &palette, 0, 1, 0, 0);
    let key_count = board.board.iter().flatten().filter(|e| {
        matches!(e, BoardEntity::KeySpool(_))
    }).count();
    assert_eq!(key_count, 0, "tiny boards should not have keys");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test test_keys_spawn -- --nocapture`
Expected: `test_keys_spawn_on_large_boards` FAILS, `test_no_keys_on_tiny_boards` PASSES (vacuously).

- [ ] **Step 3: Implement Pass 4 in make_random**

In `src/game_board.rs`, inside `make_random`, after the conveyor reversion block (after line 97 `}`), add:

```rust
        // Pass 4: convert some spools to KeySpools (auto-scaled)
        let spool_positions: Vec<(usize, usize)> = (0..h)
            .flat_map(|r| (0..w).map(move |c| (r, c)))
            .filter(|&(r, c)| matches!(board[r][c], BoardEntity::Spool(_)))
            .collect();
        let total_spools = spool_positions.len();
        let key_count = total_spools / 12; // ~1 key per 12 spools
        if key_count > 0 {
            let chosen: Vec<(usize, usize)> = spool_positions
                .choose_multiple(&mut rng, key_count)
                .cloned()
                .collect();
            for (r, c) in chosen {
                if let BoardEntity::Spool(color) = board[r][c] {
                    board[r][c] = BoardEntity::KeySpool(color);
                }
            }
        }
```

- [ ] **Step 4: Run the key-spawn tests**

Run: `cargo test test_keys_spawn -- --nocapture`
Expected: both PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add src/game_board.rs
git commit -m "feat: auto-spawn KeySpools on boards with ≥12 spools"
```

### Task 3: Mark yarn stitches as locked to match KeySpools

**Files:**
- Modify: `src/engine.rs:75-140` — after yarn generation, lock stitches matching KeySpool colors
- Test: `src/engine.rs` (tests module)

- [ ] **Step 1: Write the failing test**

In `src/engine.rs`, add to the `tests` module:

```rust
#[test]
fn new_engine_has_locked_stitches_when_keys_present() {
    // Use a config that produces a large enough board for keys
    let config = Config {
        board_height: 6, board_width: 6, color_number: 4,
        color_mode: "dark".into(), spool_limit: 7,
        spool_capacity: 2, yarn_lines: 4, obstacle_percentage: 0,
        visible_stitches: 6, conveyor_capacity: 0, conveyor_percentage: 0,
        layout: "auto".into(), scale: 1,
        scissors: 0, tweezers: 0, balloons: 0,
        scissors_spools: 1, balloon_count: 2, ad_file: None,
        max_solutions: None,
    };
    let e = GameEngine::new(&config);

    // Count keys on the board
    let key_count: usize = e.board.board.iter().flatten().filter(|c|
        matches!(c, BoardEntity::KeySpool(_))
    ).count();

    // Count locked stitches in the yarn
    let lock_count: usize = e.yarn.board.iter()
        .flat_map(|col| col.iter())
        .filter(|s| s.locked)
        .count();

    // They should match: one locked stitch per KeySpool
    assert_eq!(key_count, lock_count,
        "keys={} but locked_stitches={}", key_count, lock_count);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test new_engine_has_locked_stitches -- --nocapture`
Expected: FAIL — lock_count is 0 because yarn never gets locked stitches.

- [ ] **Step 3: Add lock-placement logic in GameEngine::new**

In `src/engine.rs`, inside `GameEngine::new`, after the generation loop ends (after `if attempts >= 100 { break; }`) and before the cursor-finding code, add:

```rust
        // Lock yarn stitches to match KeySpools on the board
        let mut key_colors: Vec<Color> = Vec::new();
        for row in &board.board {
            for cell in row {
                if let BoardEntity::KeySpool(c) = cell {
                    key_colors.push(*c);
                }
            }
        }
        for key_color in &key_colors {
            // Find the deepest (last) unlocked stitch of this color in any yarn column
            let mut best: Option<(usize, usize)> = None; // (col_idx, stitch_idx)
            for (ci, col) in yarn.board.iter().enumerate() {
                for (si, stitch) in col.iter().enumerate() {
                    if stitch.color == *key_color && !stitch.locked {
                        match best {
                            None => best = Some((ci, si)),
                            Some((_, prev_si)) if si > prev_si => best = Some((ci, si)),
                            _ => {}
                        }
                    }
                }
            }
            if let Some((ci, si)) = best {
                yarn.board[ci][si].locked = true;
            }
        }
```

- [ ] **Step 4: Run the test**

Run: `cargo test new_engine_has_locked_stitches -- --nocapture`
Expected: PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test`
Expected: all pass

- [ ] **Step 6: Commit**

```bash
git add src/engine.rs
git commit -m "feat: lock yarn stitches to match KeySpools on the board"
```

---

## Chunk 3: Held Spool Counter (Task 4)

### Task 4: Display held/limit counter in bonus panels

**Files:**
- Modify: `src/renderer.rs:733-770` — add counter to `render_bonus_display_h` and `render_bonus_panel`

- [ ] **Step 1: Add held counter to render_bonus_display_h**

In `src/renderer.rs`, function `render_bonus_display_h`, after the bonus loop, before `Ok(())`, add:

```rust
    // Held spool counter
    let held = engine.held_spools.len() as u16;
    let limit = engine.spool_limit as u16;
    stdout.queue(Print("  "))?;
    let counter_str = format!("⊞ {}/{}", held, limit);
    if held >= limit.saturating_sub(1) {
        stdout.queue(Print(counter_str.red()))?;
    } else if held >= limit.saturating_sub(2) {
        stdout.queue(Print(counter_str.yellow()))?;
    } else {
        stdout.queue(Print(counter_str.white()))?;
    }
```

- [ ] **Step 2: Add held counter to render_bonus_panel**

In `src/renderer.rs`, function `render_bonus_panel`, after the bonus loop, before `Ok(())`, add:

```rust
    // Held spool counter
    let held = engine.held_spools.len() as u16;
    let limit = engine.spool_limit as u16;
    let row = bonuses.len() as u16;
    stdout.queue(MoveTo(x, y + row))?;
    let counter_str = format!("⊞ {}/{}", held, limit);
    if held >= limit.saturating_sub(1) {
        stdout.queue(Print(counter_str.red()))?;
    } else if held >= limit.saturating_sub(2) {
        stdout.queue(Print(counter_str.yellow()))?;
    } else {
        stdout.queue(Print(counter_str.white()))?;
    }
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: all pass (renderer functions have no unit tests, but compilation confirms correctness)

- [ ] **Step 4: Manual verification**

Run: `cargo run` and start a custom game. Verify:
- Counter shows `⊞ 0/7` (or current spool_limit) in the bonus area
- Picking up spools increments the counter
- Counter turns yellow when 2 away from limit, red when 1 away

- [ ] **Step 5: Commit**

```bash
git add src/renderer.rs
git commit -m "feat: add held spool counter to bonus display panels"
```

---

## Final Step

- [ ] **Run full test suite one last time**

Run: `cargo test`
Expected: all pass

All three Batch 1 fixes are now implemented and committed separately.
