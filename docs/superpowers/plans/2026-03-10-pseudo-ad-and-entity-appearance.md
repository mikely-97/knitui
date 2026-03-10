# Pseudo-Ad & Entity Appearance Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a pseudo-ad mechanic (parody mobile ads that grant scissors) and improve entity rendering at scale 2+, extracting all rendering into a dedicated module.

**Architecture:** Three-phase approach: (1) extract rendering from `main.rs` into `renderer.rs`, (2) add `glyphs.rs` with scaled entity patterns, (3) add pseudo-ad engine logic + TUI overlay + NI command. Each phase produces a compilable, testable commit.

**Tech Stack:** Rust, crossterm 0.27, clap 4, serde/serde_json, dirs 5

**Spec:** `docs/superpowers/specs/2026-03-10-pseudo-ad-and-entity-appearance-design.md`

---

## Chunk 1: Renderer Extraction

### Task 1: Create `renderer.rs` — move rendering functions from `main.rs`

**Files:**
- Create: `src/renderer.rs`
- Modify: `src/main.rs` (lines 18-21, 34-40, 42-553)
- Modify: `src/lib.rs`

This is a pure move refactor. No new logic. The goal is to get `main.rs` down to just the event loop shell.

- [ ] **Step 1: Create `src/renderer.rs` with moved code**

Create `src/renderer.rs`. Move the following from `main.rs`:

1. Spacing constants (lines 18-21):
```rust
pub const YARN_HGAP: u16 = 2;
pub const YARN_VGAP: u16 = 1;
pub const THREAD_GAP: u16 = 1;
pub const COMP_GAP: u16 = 3;
```

2. `Layout` enum (lines 34-38) and `FlankSide` enum (line 40):
```rust
#[derive(Clone, Copy)]
pub enum Layout {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy)]
pub enum FlankSide { Left, Right }
```

3. All rendering functions (keep their exact signatures, just make them `pub`):
- `detect_layout()` (line 42)
- `render_yarn()` (line 67)
- `render_balloon_columns()` (line 100)
- `render_balloon_flank()` (line 138)
- `render_active_h()` (line 196)
- `render_active_v()` (line 212)
- `draw_hline()` (line 227)
- `render_board()` (line 244)
- `render_help()` (line 305)
- `render_keybar()` (line 337)
- `render_bonus_display_h()` (line 374)
- `render_bonus_panel()` (line 392)
- `render()` (line 411) → rename to `render_vertical`
- `render_overlay()` (line 441) → rename to `render_vertical_overlay`
- `render_horizontal()` (line 459)
- `render_horizontal_overlay()` (line 504)
- `do_render()` (line 523)
- `do_render_overlay()` (line 538)

The `renderer.rs` file needs these imports at the top:
```rust
use std::io::{self, Write, Stdout};
use crossterm::{
    QueueableCommand,
    style::{Print, Stylize, Attribute, SetAttribute},
    terminal::{self, Clear, ClearType, BeginSynchronizedUpdate, EndSynchronizedUpdate},
    cursor::MoveTo,
};
use crate::engine::{GameEngine, GameStatus, BonusState};
use crate::board_entity::Direction;
```

- [ ] **Step 2: Update `main.rs` to use `renderer`**

Remove all moved code from `main.rs`. Replace with:
```rust
use knitui::renderer::{self, Layout, COMP_GAP, YARN_HGAP, YARN_VGAP};
```

The `TuiState` enum stays in `main.rs`. The `main()` function stays. All calls like `render_board(...)` become `renderer::render_board(...)` or use the `use` import.

Update `do_render` and `do_render_overlay` calls in the event loop to `renderer::do_render(...)` and `renderer::do_render_overlay(...)`.

- [ ] **Step 3: Register module in `lib.rs`**

Add to `src/lib.rs`:
```rust
pub mod renderer;
```

- [ ] **Step 4: Compile and verify**

Run: `cargo build --bin knitui 2>&1`
Expected: Successful compilation with possibly some warnings.

Run: `cargo build --bin knitui-ni 2>&1`
Expected: Successful compilation (NI binary doesn't use renderer, should be unaffected).

- [ ] **Step 5: Run existing tests**

Run: `cargo test 2>&1`
Expected: All existing tests pass (this was a pure move refactor).

- [ ] **Step 6: Commit**

```bash
git add src/renderer.rs src/main.rs src/lib.rs
git commit -m "refactor: extract rendering functions into renderer.rs"
```

---

## Chunk 2: Entity Glyph System

### Task 2: Create `glyphs.rs` with scaled entity patterns

**Files:**
- Create: `src/glyphs.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write tests for `entity_glyph()`**

Add to the bottom of the new `src/glyphs.rs` file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board_entity::{BoardEntity, Direction, GeneratorData};
    use crossterm::style::Color;

    #[test]
    fn scale1_thread_returns_single_row() {
        let rows = entity_glyph_thread(1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2); // scale * 2
    }

    #[test]
    fn scale2_thread_returns_cross_stitch() {
        let rows = entity_glyph_thread(2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "╲╱╲╱");
        assert_eq!(rows[1], "╱╲╱╲");
    }

    #[test]
    fn scale3_thread_returns_tiled_cross_stitch() {
        let rows = entity_glyph_thread(3);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], "╲╱╲╱╲╱");
        assert_eq!(rows[1], "╱╲╱╲╱╲");
        assert_eq!(rows[2], "╲╱╲╱╲╱");
    }

    #[test]
    fn scale2_obstacle_returns_shade() {
        let rows = entity_glyph_obstacle(2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "░░░░");
        assert_eq!(rows[1], "░░░░");
    }

    #[test]
    fn scale2_generator_right() {
        let rows = entity_glyph_generator(Direction::Right, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "⊞──▸");
        assert_eq!(rows[1], "⊞──▸");
    }

    #[test]
    fn scale2_generator_left() {
        let rows = entity_glyph_generator(Direction::Left, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "◂──⊞");
        assert_eq!(rows[1], "◂──⊞");
    }

    #[test]
    fn scale2_generator_down() {
        let rows = entity_glyph_generator(Direction::Down, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "·⊞⊞·");
        assert_eq!(rows[1], "·▾▾·");
    }

    #[test]
    fn scale2_generator_up() {
        let rows = entity_glyph_generator(Direction::Up, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "·▴▴·");
        assert_eq!(rows[1], "·⊞⊞·");
    }

    #[test]
    fn scale2_depleted_generator() {
        let rows = entity_glyph_depleted(2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "⊞──·");
        assert_eq!(rows[1], "⊞──·");
    }

    #[test]
    fn scale2_key_thread() {
        let rows = entity_glyph_key_thread(2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "╲╱⚷╱");
        assert_eq!(rows[1], "╱╲╲╱");
    }

    #[test]
    fn scale2_void_returns_spaces() {
        let rows = entity_glyph_void(2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "    ");
        assert_eq!(rows[1], "    ");
    }

    #[test]
    fn all_scale2_patterns_have_correct_width() {
        // All scale 2 patterns should be 4 chars wide (scale * 2)
        let sw = 4; // scale 2 * 2
        for rows in [
            entity_glyph_thread(2),
            entity_glyph_key_thread(2),
            entity_glyph_obstacle(2),
            entity_glyph_generator(Direction::Right, 2),
            entity_glyph_generator(Direction::Left, 2),
            entity_glyph_generator(Direction::Down, 2),
            entity_glyph_generator(Direction::Up, 2),
            entity_glyph_depleted(2),
            entity_glyph_void(2),
        ] {
            for row in &rows {
                // Count chars (not bytes — these are unicode)
                assert_eq!(row.chars().count(), sw,
                    "Pattern {:?} has wrong width", rows);
            }
        }
    }

    #[test]
    fn all_scale3_patterns_have_correct_dimensions() {
        let sw = 6; // scale 3 * 2
        let sh = 3;
        for rows in [
            entity_glyph_thread(3),
            entity_glyph_key_thread(3),
            entity_glyph_obstacle(3),
            entity_glyph_generator(Direction::Right, 3),
            entity_glyph_generator(Direction::Left, 3),
            entity_glyph_generator(Direction::Down, 3),
            entity_glyph_generator(Direction::Up, 3),
            entity_glyph_depleted(3),
            entity_glyph_void(3),
        ] {
            assert_eq!(rows.len(), sh);
            for row in &rows {
                assert_eq!(row.chars().count(), sw,
                    "Pattern {:?} has wrong width", rows);
            }
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test glyphs 2>&1`
Expected: Compilation error — module `glyphs` doesn't exist yet / functions not defined.

- [ ] **Step 3: Implement `glyphs.rs`**

Create `src/glyphs.rs`:

```rust
use crate::board_entity::Direction;

/// Returns the glyph rows for a Thread entity at the given scale.
/// Each row is `scale * 2` characters wide. Returns `scale` rows.
pub fn entity_glyph_thread(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["╲╱╲╱", "╱╲╱╲"],
        3 => vec!["╲╱╲╱╲╱", "╱╲╱╲╱╲", "╲╱╲╱╲╱"],
        _ => vec!["╲╱"], // scale 1 fallback (2 chars)
    }
}

/// Returns the glyph rows for a KeyThread entity at the given scale.
pub fn entity_glyph_key_thread(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["╲╱⚷╱", "╱╲╲╱"],
        3 => vec!["╲╱╲╱╲╱", "╱╲⚷╲╱╲", "╲╱╲╱╲╱"],
        _ => vec!["⚷╱"], // scale 1 fallback
    }
}

/// Returns the glyph rows for an Obstacle entity at the given scale.
pub fn entity_glyph_obstacle(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["░░░░", "░░░░"],
        3 => vec!["░░░░░░", "░░░░░░", "░░░░░░"],
        _ => vec!["░░"], // scale 1 fallback
    }
}

/// Returns the glyph rows for a Generator entity at the given scale and direction.
pub fn entity_glyph_generator(dir: Direction, scale: u16) -> Vec<&'static str> {
    match (dir, scale) {
        (Direction::Right, 2) => vec!["⊞──▸", "⊞──▸"],
        (Direction::Left,  2) => vec!["◂──⊞", "◂──⊞"],
        (Direction::Down,  2) => vec!["·⊞⊞·", "·▾▾·"],
        (Direction::Up,    2) => vec!["·▴▴·", "·⊞⊞·"],
        (Direction::Right, 3) => vec!["⊞────▸", "⊞────▸", "⊞────▸"],
        (Direction::Left,  3) => vec!["◂────⊞", "◂────⊞", "◂────⊞"],
        (Direction::Down,  3) => vec!["··⊞⊞··", "··╏╏··", "··▾▾··"],
        (Direction::Up,    3) => vec!["··▴▴··", "··╏╏··", "··⊞⊞··"],
        // scale 1 fallback — single arrow char repeated
        (Direction::Right, _) => vec!["▸·"],
        (Direction::Left,  _) => vec!["◂·"],
        (Direction::Down,  _) => vec!["▾·"],
        (Direction::Up,    _) => vec!["▴·"],
    }
}

/// Returns the glyph rows for a DepletedGenerator entity at the given scale.
pub fn entity_glyph_depleted(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["⊞──·", "⊞──·"],
        3 => vec!["⊞───··", "⊞───··", "⊞───··"],
        _ => vec!["⊞·"], // scale 1 fallback
    }
}

/// Returns the glyph rows for a Void entity at the given scale.
pub fn entity_glyph_void(scale: u16) -> Vec<&'static str> {
    match scale {
        2 => vec!["    ", "    "],
        3 => vec!["      ", "      ", "      "],
        _ => vec!["  "], // scale 1 fallback
    }
}
```

- [ ] **Step 4: Register module in `lib.rs`**

Add to `src/lib.rs`:
```rust
pub mod glyphs;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test glyphs 2>&1`
Expected: All glyph tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/glyphs.rs src/lib.rs
git commit -m "feat: add entity glyph lookup table for scale 2+ rendering"
```

### Task 3: Integrate glyphs into `renderer.rs` board rendering

**Files:**
- Modify: `src/renderer.rs` (`render_board()` function)

- [ ] **Step 1: Modify `render_board()` to use glyphs at scale > 1**

In `renderer.rs`, in the `render_board()` function, find the section that renders each cell's content. Currently it repeats the `Display` output of each `BoardEntity` to fill the scaled cell.

Replace the cell-content rendering (the inner loop that does `for _ in 0..sw { stdout.queue(Print(&cell))?; }` for each row `sy`) with:

```rust
use crate::glyphs;
use crate::board_entity::BoardEntity;

// Inside render_board, for each cell at (r, c):
if scale > 1 {
    let rows = match &engine.board.board[r][c] {
        BoardEntity::Thread(_) => glyphs::entity_glyph_thread(scale),
        BoardEntity::KeyThread(_) => glyphs::entity_glyph_key_thread(scale),
        BoardEntity::Obstacle => glyphs::entity_glyph_obstacle(scale),
        BoardEntity::Generator(data) => glyphs::entity_glyph_generator(data.output_dir, scale),
        BoardEntity::DepletedGenerator => glyphs::entity_glyph_depleted(scale),
        BoardEntity::Void => glyphs::entity_glyph_void(scale),
    };
    // Get the color for styled output
    let color = match &engine.board.board[r][c] {
        BoardEntity::Thread(c) | BoardEntity::KeyThread(c) => Some(*c),
        BoardEntity::Generator(data) => Some(data.color),
        _ => None,
    };
    for (sy, glyph_row) in rows.iter().enumerate() {
        // Position cursor at cell interior
        let cell_x = x0 + 1 + (c as u16) * (sw + 1);
        let cell_y = y0 + 1 + (r as u16) * (sh + 1) + sy as u16;
        stdout.queue(MoveTo(cell_x, cell_y))?;
        // Apply cursor brackets if this is the selected cell
        let is_cursor = r as u16 == engine.cursor_row && c as u16 == engine.cursor_col;
        if is_cursor {
            stdout.queue(Print("["))?;
            let inner = &glyph_row[1..glyph_row.len()-1]; // trim first/last char for brackets
            match color {
                Some(col) => { stdout.queue(Print(inner.with(col)))?; }
                None => { stdout.queue(Print(inner))?; }
            }
            stdout.queue(Print("]"))?;
        } else {
            match color {
                Some(col) => { stdout.queue(Print(glyph_row.with(col)))?; }
                None => { stdout.queue(Print(glyph_row))?; }
            }
        }
    }
} else {
    // existing scale 1 rendering path (unchanged)
}
```

Note: The cursor bracket logic for multi-row cells puts `[` and `]` on every row, replacing the first and last character of the glyph pattern. This is consistent with how the spec describes it.

- [ ] **Step 2: Compile and verify**

Run: `cargo build --bin knitui 2>&1`
Expected: Successful compilation.

- [ ] **Step 3: Manual visual test**

Run: `cargo run --bin knitui -- --scale 2 2>&1`
Expected: Threads show cross-stitch pattern, generators show source+arrow, obstacles show light shade.

Run: `cargo run --bin knitui -- --scale 3 2>&1`
Expected: Same patterns tiled to fill 3×6 cells.

- [ ] **Step 4: Run all tests**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/renderer.rs
git commit -m "feat: integrate entity glyphs into board rendering at scale 2+"
```

---

## Chunk 3: Pseudo-Ad Engine Logic

### Task 4: Add ad fields and methods to `GameEngine`

**Files:**
- Modify: `src/engine.rs` (struct at line 59, impl at line 71, snapshot at line 457, tests at line 654)

- [ ] **Step 1: Write tests for `watch_ad()` and `can_watch_ad()`**

Add to the existing `#[cfg(test)] mod tests` in `engine.rs` (after line 654):

```rust
    // ── Pseudo-ad tests ─────────────────────────────────────────────

    #[test]
    fn watch_ad_grants_scissors() {
        let mut e = default_engine();
        assert_eq!(e.bonuses.scissors, 0);
        assert_eq!(e.ads_used, 0);
        e.watch_ad();
        assert_eq!(e.bonuses.scissors, 1);
        assert_eq!(e.ads_used, 1);
    }

    #[test]
    fn can_watch_ad_unlimited() {
        let mut e = default_engine();
        // ad_limit is None (unlimited)
        assert!(e.can_watch_ad());
        e.watch_ad();
        e.watch_ad();
        e.watch_ad();
        assert!(e.can_watch_ad()); // still unlimited
    }

    #[test]
    fn can_watch_ad_with_limit() {
        let mut e = default_engine();
        e.ad_limit = Some(2);
        assert!(e.can_watch_ad());
        e.watch_ad();
        assert!(e.can_watch_ad());
        e.watch_ad();
        assert!(!e.can_watch_ad()); // limit reached
    }

    #[test]
    fn watch_ad_respects_limit() {
        let mut e = default_engine();
        e.ad_limit = Some(1);
        e.watch_ad();
        assert_eq!(e.bonuses.scissors, 1);
        // can_watch_ad is false now, but watch_ad doesn't check — caller should check first
        assert!(!e.can_watch_ad());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test pseudo 2>&1`
Expected: Compilation error — `ads_used`, `ad_limit`, `watch_ad`, `can_watch_ad` don't exist.

- [ ] **Step 3: Add fields to `GameEngine` struct**

In `src/engine.rs`, add to the `GameEngine` struct (after `bonus_state` field, around line 68):

```rust
    pub ad_limit: Option<u16>,
    pub ads_used: u16,
```

- [ ] **Step 4: Update `GameEngine::new()` to initialize new fields**

In the `Self { ... }` block inside `new()` (around line 107), add:

```rust
            ad_limit: None,
            ads_used: 0,
```

- [ ] **Step 5: Update `default_engine()` in tests**

In the test helper `default_engine()` (around line 660), add the new fields to the `GameEngine` struct literal:

```rust
            ad_limit: None,
            ads_used: 0,
```

- [ ] **Step 6: Update all other test `GameEngine` struct literals**

Search for all `GameEngine {` in the test module and add `ad_limit: None, ads_used: 0,` to each. These are the manual struct constructions in tests like `status_won_*`, `status_stuck_*`, etc.

- [ ] **Step 7: Implement `watch_ad()` and `can_watch_ad()`**

Add to the `impl GameEngine` block:

```rust
    /// Grant +1 scissors from watching a pseudo-ad. Increments ads_used counter.
    pub fn watch_ad(&mut self) {
        self.bonuses.scissors += 1;
        self.ads_used += 1;
    }

    /// Returns true if the player can watch another ad (unlimited or under limit).
    pub fn can_watch_ad(&self) -> bool {
        match self.ad_limit {
            None => true,
            Some(limit) => self.ads_used < limit,
        }
    }
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test 2>&1`
Expected: All tests pass, including the new pseudo-ad tests.

- [ ] **Step 9: Commit**

```bash
git add src/engine.rs
git commit -m "feat: add watch_ad() and can_watch_ad() to GameEngine"
```

### Task 5: Update `GameStateSnapshot` for ad fields

**Files:**
- Modify: `src/engine.rs` (snapshot struct at line 457, `from_engine` at line 489, `into_engine` around line 530)

- [ ] **Step 1: Write snapshot round-trip test**

Add to the test module in `engine.rs`:

```rust
    #[test]
    fn snapshot_round_trips_ad_fields() {
        let mut e = default_engine();
        e.ad_limit = Some(5);
        e.ads_used = 3;
        let json = e.to_json();
        let e2 = GameEngine::from_json(&json).unwrap();
        assert_eq!(e2.ad_limit, Some(5));
        assert_eq!(e2.ads_used, 3);
    }

    #[test]
    fn snapshot_defaults_ad_fields_when_missing() {
        // Simulate loading a save from before ad fields existed
        let mut e = default_engine();
        let json = e.to_json();
        // The JSON should deserialize fine even if ad fields are missing
        // (serde(default) handles this)
        let e2 = GameEngine::from_json(&json).unwrap();
        assert_eq!(e2.ad_limit, None);
        assert_eq!(e2.ads_used, 0);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test snapshot_round_trips_ad 2>&1`
Expected: Fail — snapshot doesn't include ad fields yet.

- [ ] **Step 3: Add ad fields to `GameStateSnapshot`**

In the `GameStateSnapshot` struct (around line 457), add:

```rust
    #[serde(default)]
    pub ad_limit: Option<u16>,
    #[serde(default)]
    pub ads_used: u16,
```

- [ ] **Step 4: Update `from_engine()` to serialize ad fields**

In `GameStateSnapshot::from_engine()`, add to the `Self { ... }` block:

```rust
            ad_limit: e.ad_limit,
            ads_used: e.ads_used,
```

- [ ] **Step 5: Update `into_engine()` to restore ad fields**

In `GameStateSnapshot::into_engine()`, add to the `GameEngine { ... }` block being constructed:

```rust
            ad_limit: self.ad_limit,
            ads_used: self.ads_used,
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/engine.rs
git commit -m "feat: add ad_limit and ads_used to GameStateSnapshot"
```

---

## Chunk 4: Ad Content Loading & Config

### Task 6: Add `--ad-file` config flag

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Add `ad_file` field to `Config` struct**

In `src/config.rs`, add the import and the field:

```rust
use std::path::PathBuf;
```

Add to the `Config` struct:

```rust
    #[arg(long, help = "Path to ad quotes file (one per line, default: ~/.config/knitui/ads.txt)")]
    pub ad_file: Option<PathBuf>,
```

- [ ] **Step 2: Compile and verify**

Run: `cargo build 2>&1`
Expected: Successful compilation.

- [ ] **Step 3: Commit**

```bash
git add src/config.rs
git commit -m "feat: add --ad-file CLI flag to Config"
```

### Task 7: Add ad content loader module

**Files:**
- Create: `src/ad_content.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write tests for ad content loading**

Create `src/ad_content.rs` with tests:

```rust
use std::path::PathBuf;

const FALLBACK_QUOTE: &str = "You are watching a fake ad. Touch grass.";

/// Load ad quotes from the configured file path.
/// Returns a Vec of quote strings. If the file is missing or empty,
/// returns a single fallback quote.
pub fn load_quotes(ad_file: &Option<PathBuf>) -> Vec<String> {
    todo!()
}

/// Pick a random quote from the list.
pub fn random_quote(quotes: &[String]) -> &str {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_quotes_missing_file_returns_fallback() {
        let path = Some(PathBuf::from("/nonexistent/path/ads.txt"));
        let quotes = load_quotes(&path);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], FALLBACK_QUOTE);
    }

    #[test]
    fn load_quotes_none_path_uses_default() {
        // When path is None, tries ~/.config/knitui/ads.txt
        // which likely doesn't exist in test → fallback
        let quotes = load_quotes(&None);
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], FALLBACK_QUOTE);
    }

    #[test]
    fn load_quotes_parses_file_skipping_comments_and_blanks() {
        let dir = std::env::temp_dir().join("knitui_test_ads");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test_ads.txt");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(f, "# This is a comment").unwrap();
        writeln!(f, "").unwrap();
        writeln!(f, "First quote").unwrap();
        writeln!(f, "  # Indented comment").unwrap();
        writeln!(f, "Second quote").unwrap();
        drop(f);

        let quotes = load_quotes(&Some(file_path.clone()));
        assert_eq!(quotes, vec!["First quote", "Second quote"]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_quotes_empty_file_returns_fallback() {
        let dir = std::env::temp_dir().join("knitui_test_empty_ads");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("empty.txt");
        std::fs::File::create(&file_path).unwrap();

        let quotes = load_quotes(&Some(file_path.clone()));
        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0], FALLBACK_QUOTE);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn random_quote_returns_valid_entry() {
        let quotes = vec!["Alpha".to_string(), "Beta".to_string()];
        let q = random_quote(&quotes);
        assert!(q == "Alpha" || q == "Beta");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Register in `src/lib.rs`:
```rust
pub mod ad_content;
```

Run: `cargo test ad_content 2>&1`
Expected: Panics from `todo!()`.

- [ ] **Step 3: Implement `load_quotes()` and `random_quote()`**

Replace the `todo!()` bodies:

```rust
use std::path::PathBuf;
use std::fs;
use rand::Rng;

const FALLBACK_QUOTE: &str = "You are watching a fake ad. Touch grass.";

pub fn load_quotes(ad_file: &Option<PathBuf>) -> Vec<String> {
    let path = match ad_file {
        Some(p) => p.clone(),
        None => {
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."));
            config_dir.join("knitui").join("ads.txt")
        }
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![FALLBACK_QUOTE.to_string()],
    };

    let quotes: Vec<String> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_string())
        .collect();

    if quotes.is_empty() {
        vec![FALLBACK_QUOTE.to_string()]
    } else {
        quotes
    }
}

pub fn random_quote(quotes: &[String]) -> &str {
    let mut rng = rand::rng();
    let idx = rng.random_range(0..quotes.len());
    &quotes[idx]
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test ad_content 2>&1`
Expected: All ad_content tests pass.

- [ ] **Step 5: Run all tests**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/ad_content.rs src/lib.rs
git commit -m "feat: add ad content loader with file parsing and fallback"
```

---

## Chunk 5: Pseudo-Ad TUI Integration

### Task 8: Add `WatchingAd` state and ad overlay to TUI

**Files:**
- Modify: `src/main.rs` (TuiState enum, event loop)
- Modify: `src/renderer.rs` (new `render_ad_overlay()` function)

- [ ] **Step 1: Add `WatchingAd` variant to `TuiState`**

In `src/main.rs`, update the `TuiState` enum:

```rust
use std::time::Instant;

enum TuiState {
    Playing,
    GameOver(GameStatus),
    Help,
    WatchingAd { started_at: Instant, quote: String },
}
```

- [ ] **Step 2: Implement `render_ad_overlay()` in `renderer.rs`**

Add to `src/renderer.rs`:

```rust
use std::time::Instant;

/// Render the pseudo-ad full-screen overlay.
/// Shows a centered box with the quote, progress bar, and countdown.
pub fn render_ad_overlay(
    stdout: &mut Stdout,
    quote: &str,
    started_at: &Instant,
    ad_duration_secs: u64,
) -> io::Result<()> {
    let elapsed = started_at.elapsed().as_secs();
    let remaining = ad_duration_secs.saturating_sub(elapsed);
    let progress = if ad_duration_secs > 0 {
        ((elapsed as f64 / ad_duration_secs as f64) * 100.0).min(100.0) as u16
    } else {
        100
    };
    let done = remaining == 0;

    let (term_w, term_h) = terminal::size().unwrap_or((80, 24));

    stdout.queue(BeginSynchronizedUpdate)?;
    stdout.queue(Clear(ClearType::All))?;

    // Box dimensions
    let box_w = 50u16.min(term_w.saturating_sub(4));
    let box_inner = (box_w - 2) as usize; // space inside border

    // Word-wrap the quote
    let wrapped = word_wrap(quote, box_inner);

    // Box height: top border + blank + header + blank + quote lines + blank + progress + countdown/close + blank + bottom border
    let box_h = 6 + wrapped.len() as u16 + if done { 1 } else { 1 };
    let x0 = (term_w.saturating_sub(box_w)) / 2;
    let y0 = (term_h.saturating_sub(box_h)) / 2;

    let mut y = y0;

    // Top border
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("╔"))?;
    for _ in 0..box_inner { stdout.queue(Print("═"))?; }
    stdout.queue(Print("╗"))?;
    y += 1;

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Header
    let header = "✂ FREE SCISSORS ✂";
    print_boxed_line(stdout, x0, y, box_inner, &center_text(header, box_inner))?;
    y += 1;

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Quote lines
    for line in &wrapped {
        print_boxed_line(stdout, x0, y, box_inner, &center_text(line, box_inner))?;
        y += 1;
    }

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Progress bar
    let bar_width = box_inner.saturating_sub(8); // leave room for " NNN%"
    let filled = (bar_width as u16 * progress / 100) as usize;
    let empty = bar_width - filled;
    let bar = format!(
        "{}{}  {:>3}%",
        "█".repeat(filled),
        "░".repeat(empty),
        progress
    );
    print_boxed_line(stdout, x0, y, box_inner, &center_text(&bar, box_inner))?;
    y += 1;

    // Countdown or close prompt
    if done {
        let msg = "[ Press ESC to collect your reward ]";
        print_boxed_line(stdout, x0, y, box_inner, &center_text(msg, box_inner))?;
    } else {
        let msg = format!("[{}s remaining]", remaining);
        print_boxed_line(stdout, x0, y, box_inner, &center_text(&msg, box_inner))?;
    }
    y += 1;

    // Empty line
    print_boxed_line(stdout, x0, y, box_inner, "")?;
    y += 1;

    // Bottom border
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("╚"))?;
    for _ in 0..box_inner { stdout.queue(Print("═"))?; }
    stdout.queue(Print("╝"))?;

    stdout.queue(EndSynchronizedUpdate)?;
    stdout.flush()
}

fn print_boxed_line(stdout: &mut Stdout, x0: u16, y: u16, inner_w: usize, content: &str) -> io::Result<()> {
    stdout.queue(MoveTo(x0, y))?;
    stdout.queue(Print("║"))?;
    let content_chars: usize = content.chars().count();
    stdout.queue(Print(content))?;
    for _ in content_chars..inner_w {
        stdout.queue(Print(' '))?;
    }
    stdout.queue(Print("║"))?;
    Ok(())
}

fn center_text(text: &str, width: usize) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }
    let padding = (width - text_len) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
```

- [ ] **Step 3: Add ad hotkey handling in `main.rs` event loop**

In `main.rs`, load ad quotes at startup (after `Config::parse()`):

```rust
use knitui::ad_content;

// After config parsing, before event loop:
let ad_quotes = ad_content::load_quotes(&config.ad_file);
const AD_DURATION_SECS: u64 = 15;
```

In the event loop, add `KeyCode::Char('a') | KeyCode::Char('A')` handling:

In `TuiState::Playing` match arm, add:
```rust
KeyCode::Char('a') | KeyCode::Char('A') => {
    if engine.can_watch_ad() {
        let quote = ad_content::random_quote(&ad_quotes).to_string();
        tui_state = TuiState::WatchingAd {
            started_at: Instant::now(),
            quote,
        };
    }
}
```

In `TuiState::GameOver(_)` match arm, add the same (before the existing `'r'`/`'q'` handling):
```rust
KeyCode::Char('a') | KeyCode::Char('A') => {
    if engine.can_watch_ad() {
        let quote = ad_content::random_quote(&ad_quotes).to_string();
        tui_state = TuiState::WatchingAd {
            started_at: Instant::now(),
            quote,
        };
    }
}
```

Add a new match arm for `TuiState::WatchingAd`:
```rust
TuiState::WatchingAd { ref started_at, ref quote } => {
    match event.code {
        KeyCode::Esc => {
            if started_at.elapsed().as_secs() >= AD_DURATION_SECS {
                engine.watch_ad();
                // Return to playing — re-check status in case we un-stuck
                let status = engine.status();
                tui_state = match status {
                    GameStatus::Playing => TuiState::Playing,
                    _ => TuiState::GameOver(status),
                };
                // Re-render the game
                renderer::do_render(&mut stdout, &engine, layout, yarn_x, board_x, board_y, scale)?;
            }
            // If timer not done, ignore ESC (straight timer — no tricks)
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Add rendering dispatch for `WatchingAd` state**

In the `do_render`/rendering section of the event loop (or wherever the screen is redrawn), add handling for the `WatchingAd` state. Since `WatchingAd` needs continuous re-rendering (progress bar ticks), update the main loop's poll timeout section:

After the `if poll(...)` block, add a re-render for the ad state on every tick:
```rust
// After the event-handling block, always re-render if watching an ad
if let TuiState::WatchingAd { ref started_at, ref quote } = tui_state {
    renderer::render_ad_overlay(&mut stdout, quote, started_at, AD_DURATION_SECS)?;
}
```

- [ ] **Step 5: Compile and verify**

Run: `cargo build --bin knitui 2>&1`
Expected: Successful compilation.

- [ ] **Step 6: Manual test**

Run: `cargo run --bin knitui 2>&1`
Then press `A` during gameplay. Expected:
- Full-screen ad overlay appears with a quote
- Progress bar fills over 15 seconds
- ESC does nothing until timer completes
- After 15s, ESC collects reward and returns to game
- Scissors bonus count increases by 1

- [ ] **Step 7: Run all tests**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
git add src/main.rs src/renderer.rs
git commit -m "feat: add pseudo-ad TUI overlay with 15-second timer"
```

### Task 9: Update key bar and help overlay

**Files:**
- Modify: `src/renderer.rs` (`render_keybar()` and `render_help()` functions)

- [ ] **Step 1: Add `A:Ad` to key bar**

In `renderer.rs`, in `render_keybar()`, add before the `Esc` entry:

```rust
stdout.queue(Print("A ".dark_grey()))?;
stdout.queue(Print("Ad ".white()))?;
```

- [ ] **Step 2: Add ad explanation to help overlay**

In `renderer.rs`, in `render_help()`, add a line to the help text:

```
A — Watch a fake ad for +1 scissors
```

- [ ] **Step 3: Compile and verify**

Run: `cargo build --bin knitui 2>&1`
Expected: Successful compilation.

- [ ] **Step 4: Commit**

```bash
git add src/renderer.rs
git commit -m "feat: add ad hotkey to key bar and help overlay"
```

---

## Chunk 6: NI Binary Ad Command

### Task 10: Add `ad` command to `knitui-ni`

**Files:**
- Modify: `src/bin/knitui_ni.rs`

- [ ] **Step 1: Add `Ad` variant to `NiCommand`**

In the `NiCommand` enum, add:

```rust
    /// Watch a fake ad (grants +1 scissors, no timer)
    Ad,
```

- [ ] **Step 2: Add ad command handling**

In the command match block (where `Scissors`, `Tweezers`, `Balloons` are handled), add:

```rust
Some(NiCommand::Ad) => {
    if !engine.can_watch_ad() {
        err_response("ad_limit_reached", "ad limit reached for this game");
        return;
    }
    engine.watch_ad();
    // Include the quote in response for fun
    let quotes = knitui::ad_content::load_quotes(&None);
    let quote = knitui::ad_content::random_quote(&quotes);
    // Custom response with quote
    if let Err(e) = save_engine(&hash, &engine) {
        err_response("save_failed", &e);
        return;
    }
    let state_snap = serde_json::to_value(
        engine.to_json_value()
    ).unwrap_or(serde_json::Value::Null);
    let response = serde_json::json!({
        "status": "ok",
        "game": hash,
        "quote": quote,
        "bonus_granted": "scissors",
        "won": engine.status() == knitui::engine::GameStatus::Won,
        "game_status": format!("{:?}", engine.status()).to_lowercase(),
    });
    println!("{}", serde_json::to_string(&response).unwrap());
    return;
}
```

Note: Check how the existing `ok_response()` function works — the ad response may be slightly different (includes `quote` and `bonus_granted` fields). If `ok_response` can be extended, use that; otherwise, build the JSON directly as shown.

- [ ] **Step 3: Compile and verify**

Run: `cargo build --bin knitui-ni 2>&1`
Expected: Successful compilation.

- [ ] **Step 4: Integration test**

Run:
```bash
cargo run --bin knitui-ni 2>&1
# note the game hash from output
cargo run --bin knitui-ni -- --game <HASH> ad 2>&1
```
Expected: JSON response with `"status":"ok"`, `"bonus_granted":"scissors"`, and a quote.

- [ ] **Step 5: Run all tests**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/bin/knitui_ni.rs
git commit -m "feat: add ad command to NI binary"
```

### Task 11: Final integration verification

- [ ] **Step 1: Full test suite**

Run: `cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 2: Both binaries build cleanly**

Run: `cargo build --release 2>&1`
Expected: Clean build, no warnings (or only pre-existing ones from `#![allow(warnings)]`).

- [ ] **Step 3: Manual end-to-end test checklist**

1. `cargo run --bin knitui -- --scale 2` → entities show cross-stitch/arrow/shade patterns
2. `cargo run --bin knitui -- --scale 3` → same patterns tiled to 3×6
3. `cargo run --bin knitui -- --scale 1` → unchanged from before
4. Press `A` during play → ad overlay appears, timer counts down, ESC works after 15s, scissors +1
5. Press `H` → help overlay shows ad hotkey
6. Key bar shows `A Ad`
7. Get stuck → game-over offers ad → press A → ad plays → ESC → scissors granted → game un-stucks if possible
8. NI: `cargo run --bin knitui-ni` then `cargo run --bin knitui-ni -- --game <HASH> ad` → returns quote + scissors

- [ ] **Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: address integration issues from end-to-end testing"
```
