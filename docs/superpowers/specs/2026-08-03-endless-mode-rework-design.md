# Endless Mode Rework — Design

**Date:** 2026-08-03
**Status:** Approved
**Supersedes:** "Batch 2: Endless Mode Rework" section of `docs/superpowers/specs/2026-03-11-proposals-batch-design.md` (that doc predates the `crates/` workspace migration and used flat `src/` paths; this spec restates the same approved mechanic against the current `crates/loom-knit/` layout, with additional design decisions made where the original left gaps).

## Context

Endless mode is currently wave-based: each wave generates a fresh 4×4-to-6×6 board with scaling difficulty, and the player earns bonuses every 3 waves. It gets reworked into a single continuous scrolling puzzle — a chill, low-pressure mode with no earning mechanic and no per-wave restarts.

## Core Mechanic

- On start, generate one tall board: 6 columns × 66 rows (6 visible + 60 buffer). Row count may be tuned later; 66 is the initial target from the original design.
- Only the top 6 rows are the visible/playable window.
- Yarn is generated up front to match the color counts of the **entire** 66-row board, not just the visible window.
- Player plays normally on the visible 6×6 grid.

## Where State Lives

`GameEngine` (in `crates/loom-knit/src/engine/mod.rs`) gains three new fields, following the existing pattern where per-mode state (`ad_limit`, `ads_used`) lives directly on the engine rather than in an external wrapper:

```rust
pub struct GameEngine {
    // ...existing fields...
    #[serde(default)]
    pub row_buffer: Vec<Vec<BoardEntity>>,   // rows not yet shifted in; empty for non-endless games
    #[serde(default)]
    pub total_rows: u32,                     // 0 for non-endless games
    #[serde(default)]
    pub rows_cleared: u32,
}
```

`#[serde(default)]` keeps existing save files and non-endless snapshots valid without changes.

`EndlessState` (`crates/loom-knit/src/endless.rs`) is reduced to what genuinely doesn't fit inside a single engine snapshot — nothing, in fact, since the row buffer now lives on the engine. `EndlessState` as a distinct in-session struct is removed; the TUI's endless flow just constructs a `GameEngine` directly (mirroring how Custom Game already works), flagged as endless via `total_rows > 0`. `EndlessHighScore` (persisted separately, unaffected by this) stays as the cross-session record.

**Why not a wrapper struct**: `CampaignState` wraps a `GameEngine` because campaign play is a *sequence* of discrete engines — one per level, rebuilt on `advance_mission`. Continuous endless is one engine that lives from game-start to game-over; the row buffer is core mid-game state, not session-progression metadata. Putting it on `GameEngine` also means `knitui-ni`'s existing `--game <hash>` load/save (which round-trips `GameEngine` via `GameStateSnapshot`) gets continuous-endless support for free, with no new persistence path to build.

`GameStateSnapshot` (same file) gains matching fields, same `#[serde(default)]` treatment as the other optional fields already there (`ad_limit`, `ads_used`, etc.):

```rust
pub struct GameStateSnapshot {
    // ...existing fields...
    #[serde(default)]
    pub row_buffer: Vec<Vec<String>>,   // same string-encoded cell format as `board`
    #[serde(default)]
    pub total_rows: u32,
    #[serde(default)]
    pub rows_cleared: u32,
}
```

## Row Shifting

After each pick-up + process cycle, before computing `status()`:

1. Check each of the top rows of the visible board: a row is **exhausted** when every cell is `Void`, `Obstacle`, or `EmptyConveyor` — anything with a `Spool`, `KeySpool`, or `Conveyor` (non-empty queue) keeps it non-exhausted.
2. For each exhausted top row (top to bottom): remove it from the visible board, pull the next buffered row from `row_buffer` into the bottom of the visible board, increment `rows_cleared`.
3. Clamp `cursor_row` if it now points past the shifted board (shift up by the number of removed rows, clamped to `0..board.height`).
4. If `row_buffer` is empty when a shift would occur, don't pull anything in — the visible board just plays out to completion (fewer than 6 rows of real content remaining is fine).

**Ordering requirement**: this shift must run *before* `status()` is evaluated in the same tick. `status()`'s stuck check (`!self.board.has_selectable_spool()`) doesn't know about the row buffer — if a row empties and the shift hasn't happened yet, a real continuous game could flash `Stuck` for one frame while more content is waiting in the buffer. The engine's per-tick sequence becomes: apply pick/process → shift exhausted rows → compute status.

## Win / Stuck Semantics

- **`is_won()` needs a real change, not just reordering.** Its current logic only inspects the *visible* `board`, and a generated row can legitimately contain zero interactive cells by chance (e.g. an all-obstacle row, especially at nonzero `obstacle_percentage`) even while `row_buffer` still has content queued. Without a fix, that would end the game early while rows are still waiting to shift in. Fix: `is_won()` gains a `row_buffer.is_empty()` requirement, gated so it's a no-op for non-endless engines (where `row_buffer` is always empty by construction, since it's only populated for continuous-endless games):

  ```rust
  pub fn is_won(&self) -> bool {
      self.row_buffer.is_empty()
          && self.held_spools.is_empty()
          && self.yarn.board.iter().all(|col| col.is_empty())
          && self.board.board.iter().all(|row| { /* ...unchanged... */ })
  }
  ```

- `Stuck` still fires from board-state analysis alone, same as every other mode; it doesn't need to know about bonus counts. With 999 of every bonus in endless mode, `Stuck` becomes a soft prompt ("use a tool") rather than a real dead end — same UX as today's `Stuck` + GameOver screen, no engine change needed there.

## Scoring

- `EndlessHighScore.best_wave: usize` → `best_rows_cleared: usize`.
- Migration: `#[serde(alias = "best_wave")]` on the renamed field so existing save files deserialize without losing the recorded high score (the old wave count becomes the initial "rows cleared" baseline — imperfect but harmless, since it's immediately overwritable by real play).
- Game-over screen shows rows cleared instead of wave reached.

## Bonuses

Endless-mode engines start with 999 scissors, 999 tweezers, 999 balloons. No earning mechanism.

## Solvability

Full DFS solvability (`count_solutions`, `is_solvable`) is skipped — a 6×66 board (~396 cells) makes DFS intractable, and it's unnecessary given unlimited bonuses guarantee escapability from any board state. Only `count_balance(&board, &yarn, spool_capacity)` runs before play starts, to confirm the yarn has the correct stitch counts for the generated board's spools.

## `knitui-ni` (non-interactive driver)

- Replace `--endless-wave <N>` with a no-argument `--endless` flag that creates a fresh continuous-endless `GameEngine`.
- Drop the `DescribeWave` subcommand — there are no discrete waves left to describe.
- `--game <hash>` load/save works unchanged, since `row_buffer`/`total_rows`/`rows_cleared` ride along inside the existing `GameEngine` JSON snapshot.

## Files Changed

- `crates/loom-knit/src/endless.rs` — remove `EndlessState`'s wave/banked-bonus fields and methods; keep/adjust `EndlessHighScore`
- `crates/loom-knit/src/engine/mod.rs` — add `row_buffer`/`total_rows`/`rows_cleared` fields, row-shift logic, `GameStateSnapshot` fields, tick ordering
- `crates/loom-knit/src/renderer.rs` (or `renderer/` submodules) — "Rows: N remaining" display replacing wave display
- `crates/loom-knit/src/tui.rs` — endless flow: construct engine directly instead of via `EndlessState`, remove wave-restart logic
- `crates/loom-knit/src/bin/knitui_ni.rs` — `--endless` flag, drop `DescribeWave`

## Testing

- Engine: row-shift triggers correctly on row exhaustion; cursor clamps correctly after a shift; `is_won()` only becomes true once `row_buffer` is empty; snapshot round-trip preserves `row_buffer`/`total_rows`/`rows_cleared`; tick ordering test confirming shift-before-status.
- `knitui-ni`: `--endless` produces a valid game; `--game` round-trip across multiple commands preserves buffer state and correctly shifts rows.
- `EndlessHighScore`: old `best_wave`-keyed JSON deserializes via the alias without error.

## Out of Scope (unchanged from original)

- Interaction with campaign blessings — endless mode has never used campaign blessings and continues not to.
- `knitui_replay.rs` — no endless-specific logic exists there today; it replays generic engine state and needs no changes as long as row-shifting is deterministic and fully captured in the snapshot.
