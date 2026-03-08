# Implementation Plan

Features from the TODO list, ordered roughly from easiest to most complex. Each section covers what the feature means, what needs to change, and concrete implementation steps.

---

## 1. Unhardcoded Config

**Why**: All game parameters are hardcoded constants. Players can't adjust difficulty, board size, or color mode without recompiling.

**Approach**: CLI arguments via the `clap` crate (standard Rust choice). A config file (e.g. `knitui.toml`) would also work but CLI args are simpler and more discoverable.

**Changes**:

1. Add `clap = { version = "4", features = ["derive"] }` to `Cargo.toml`.
2. Create `src/config.rs`:
   ```rust
   #[derive(clap::Parser)]
   pub struct Config {
       #[arg(long, default_value_t = 6)] pub board_height: u16,
       #[arg(long, default_value_t = 6)] pub board_width: u16,
       #[arg(long, default_value_t = 6)] pub color_number: u16,
       #[arg(long, default_value = "dark")] pub color_mode: String,
       #[arg(long, default_value_t = 7)] pub active_threads_limit: usize,
       #[arg(long, default_value_t = 3)] pub knit_volume: u16,
       #[arg(long, default_value_t = 4)] pub yarn_lines: u16,
       #[arg(long, default_value_t = 5)] pub obstacle_percentage: u16,
       #[arg(long, default_value_t = 6)] pub visible_patches: u16,
   }
   ```
3. In `main.rs`: replace all constants with `let config = Config::parse();` and thread the values through.
4. Parse `color_mode` string → `ColorMode` enum in `palette.rs`.

**Files**: `Cargo.toml`, new `src/config.rs`, `src/main.rs`, `src/palette.rs`

---

## 2. Animated / Async Processing of Knits

**Why**: Pressing Backspace currently processes all active threads instantly. An animation would make the game feel responsive and easier to follow.

**Preferred approach** (no heavy async runtime needed): step-by-step processing inside the existing event loop using a state machine.

**Design**:

- Add a `ProcessingState` enum to `main.rs`:
  ```rust
  enum ProcessingState {
      Idle,
      Processing { remaining: Vec<Thread> },
  }
  ```
- When Backspace is pressed, move active_threads into `ProcessingState::Processing`.
- On each poll timeout (the `else` branch currently does nothing), process one thread from `remaining`, call `render()`, then sleep briefly — producing a visual "one by one" effect.
- When `remaining` is empty, transition back to `Idle`.

**Alternative** (true async with `tokio`): More complex, not necessary for a single-player turn-based game. Avoid unless the game needs to do something truly concurrent (e.g. animated board events running while the player moves).

**Files**: `src/main.rs` (state machine), `src/yarn.rs` (already has `process_one`)

---

## 3. Horizontal Layout

**Why**: The current vertical stack (yarn → threads → board) is tall. A side-by-side view could fit better on wide terminals.

**Design**:

```
[ yarn columns ]  [ game board ]
[ active threads             ]
```

Or:

```
[ game board ] | [ yarn columns ]
               | [ active threads ]
```

**Changes**:

1. Add a `Layout` enum: `Vertical | Horizontal`.
2. Extract constants `yarn_offset`, `active_offset`, `minimal_y` into a layout helper (or compute them from layout mode).
3. In `render()`, use `MoveTo(col_offset + x, row_offset + y)` per section instead of assuming column 0.
4. Horizontal rendering of yarn: each column becomes a row chunk beside the board.

**Key challenge**: The yarn `Display` impl writes `\n\r` — for horizontal layout, yarn columns need to be written into fixed terminal columns instead. The `Display` impl will need a rewrite or an alternative rendering path that uses cursor positioning directly.

**Files**: `src/main.rs`, `src/yarn.rs` (render path)

---

## 4. Movement Limits (Cursor Stays on Board)

**Current state**: Arrow keys are already bounded (`saturating_sub`, `min`), but the cursor can still land on yarn/active-thread rows if `y` starts too high, and it can move over `Obstacle` or `Void` cells.

**Improvements**:

1. **Enforce board-only cursor**: Clamp `y` to `[minimal_y, minimal_y + board_height - 1]` and `x` to `[0, board_width - 1]` — this is almost done, verify the off-by-one math.
2. **Skip void/obstacle cells**: When moving, advance the cursor past cells that can't be selected until a `Thread` cell is found (or hit the edge). This prevents the player from sitting on an unselectable cell.
3. **Visual feedback**: Highlight the cell under the cursor differently when it's a `Thread` vs `Void/Obstacle`.

**Files**: `src/main.rs` (key handler), `src/board_entity.rs` (maybe add a helper `is_selectable()`)

## 4.5. Movement Limits (Only Void-Bordering Or Upper Row Cells Are Selectable)

**Current state**: Cursor can select all cells on the board
**Desired state**: for it to be a puzzle, only the upper row can be selectable initially. Then, as Knit becomes Void, knits that border it horizontally or vertically (NOT DIAGONALLY) can be selected too. Only Void unlocks them, nothing else!

---

## 5. Complex Board Entities: Locks, Keys, Generators

**Why**: The current board only has `Thread`, `Obstacle`, and `Void`. Adding interactive cell types increases strategic depth.

### 5a. Lock / Key [EDITED]

- `Lock(Color)`: a patch on the yarn can be locked, in which case it cannot be processed until unlocked. It blocks its yarn's column too.
- `Key(Color)`: a key is appended to a knit on the board that has the same color as the locked yarn. The key should be visible.


### 5b. Generator [EDITED]

- `Generator(Color)`: has an output cell that borders it horizontally or vertically. Initially a knit is located in that cell. When removed, the generator creates another knit in its place. This happens up to <GENERATOR_CAPACITY> times (should define that in config), after which it becomes a DepletedGenerator and effectively becomes a form of Obstacle. As for implementation, the queue should NOT be random, because solvability of the puzzle depends on what's in the queue.


---

## 6. Solvability Checks

**Why**: A randomly generated board might be unwinnable — e.g. active thread limit too low to clear groups of threads, or yarn distribution makes certain colors unreachable.

**Current guarantee**: `count_knits()` × `knit_volume` patches are added to the yarn, so the total patch count always matches. The question is whether the *order* allows the player to make progress.

### What to check

1. **Basic count match**: Already guaranteed by construction. Verify: `yarn total per color == board threads per color × knit_volume`.
2. **Active threads headroom**: If the board has more distinct colors than `active_threads_limit`, a player might get stuck holding threads they can't match. Check: `distinct_colors_on_board <= active_threads_limit`.
3. **Yarn column distribution**: If all patches of one color end up at the bottom of the same column, hidden behind many other colors, the game is still technically solvable but very hard. A relaxed check: ensure each color appears in at least 2 yarn columns (or warn if not).
4. [EDITED] make sure to account for special entities, such as generators and locks/keys.

### Implementation

1. Add `fn is_solvable(board: &GameBoard, yarn: &Yarn, active_limit: usize) -> bool` in a new `src/solvability.rs`.
2. Check #1: count board threads per color, compare to yarn patch counts.
3. Check #2: count distinct colors on board, compare to `active_limit`.
4. In `main.rs`, after board+yarn generation, loop: if `!is_solvable(...)`, regenerate.

**Cap retries** at e.g. 100 to avoid infinite loops on degenerate configs.

**Files**: new `src/solvability.rs`, `src/main.rs`

---

## 7. Bonuses and Power-Ups

**Why**: Extra mechanics to add variety and fun.

### Ideas

| Bonus | Effect | Implementation |
|-------|--------|----------------|
| **Wildcard patch** | Matches any thread color | `Patch::Wild` variant in `yarn.rs`; `process_one` matches it for any thread |
| **Double-knit patch** | One patch counts as 2 knit stages | `Patch::Double(Color)`; `thread.knit_on()` called twice |
| **Color-clear patch** | Removes all patches of one color from yarn | Appears rarely; clears a full color from the queue |
| **Board shuffle** | Randomizes thread positions on board | Triggered by special key or power-up item |
| **Obstacle breaker** | Converts `Obstacle` to `Thread` of random color | Selectable like a thread |

### Pseudo-ads

A fun in-game joke mechanic: between rounds (when the board is cleared), show a fake ASCII advertisement for a fictional product for 3 seconds before the next round starts.

**Implementation**: A `Vec<String>` of fake ad strings in a `src/ads.rs`; displayed in the render after board-clear detection.

**Files**: `src/yarn.rs` (Patch variants), `src/board_entity.rs` (bonus cells), new `src/ads.rs`, `src/main.rs`

---

## Dependency / Ordering

```
Config (1) ─────────────────────────── enables all features to be configurable
Movement (4) ──────────────────────── polish, quick win
Solvability (6) ────────────────────── needed before complex boards get too weird
Animation (2) ──────────────────────── UX polish, independent
Horizontal layout (3) ──────────────── UX polish, independent
Complex boards (5) ─────────────────── builds on solvability (6)
Bonuses (7) ────────────────────────── builds on complex boards (5)
```

**Suggested order**: 4 → 1 → 6 → 2 → 3 → 5 → 7

Start with movement limits (tiny, no deps), then config (unlocks everything being tunable), then solvability (foundation for safe random generation), then the rest.
