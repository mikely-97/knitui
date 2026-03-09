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
       #[arg(long, default_value_t = 3)] pub generator_capacity: u16,
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

---

## 4.5. Selectability: Only Exposed Threads Are Pickable

**Current state**: Any thread on the board can be selected.

**Desired state**: A thread is selectable only if it is in the top row **or** borders a `Void` cell orthogonally (not diagonally). Obstacles never become Void, so they do not unlock neighbors.

### Implementation

Add `is_selectable(board: &GameBoard, row: usize, col: usize) -> bool` in `game_board.rs`:

```rust
pub fn is_selectable(board: &GameBoard, row: usize, col: usize) -> bool {
    if !matches!(board.board[row][col], BoardEntity::Thread(_) | BoardEntity::KeyThread(_)) {
        return false;
    }
    if row == 0 {
        return true;
    }
    let h = board.height as usize;
    let w = board.width as usize;
    let is_void = |r: usize, c: usize| matches!(board.board[r][c], BoardEntity::Void);
    (row > 0   && is_void(row - 1, col)) ||
    (row+1 < h && is_void(row + 1, col)) ||
    (col > 0   && is_void(row, col - 1)) ||
    (col+1 < w && is_void(row, col + 1))
}
```

In the Enter handler (`main.rs`): check `is_selectable` before adding a thread to active_threads.

In the cursor movement handler: skip cells where `is_selectable` is false (advance until a selectable cell is found, or stop at edge).

### Obstacle concern

An Obstacle in the top row permanently prevents any cell directly below it from being unlocked through that column. Board generation must ensure no thread is completely surrounded by Obstacles and non-top-row positions (i.e., reachability must be checked). See §6.

---

## 5. Complex Board Entities: Locks, Keys, Generators

**Why**: The current board only has `Thread`, `Obstacle`, and `Void`. Adding interactive cell types increases strategic depth.

### 5a. Lock / Key

**Design (option A — locks are on yarn, keys are on board threads):**

- `Lock(Color)`: a `Patch` on the yarn that is locked. It blocks its entire yarn column — no patches behind it can be popped until it is cleared. It can only be cleared by a thread that carries the matching Key.
- `KeyThread(Color)`: a board entity — a thread with a key attached. Displayed with a distinct glyph (e.g. `K` instead of `T`). When added to active threads, it carries `has_key: true`. When processed against the yarn, if the last patch in a column is a `Lock` of matching color, the lock is popped (unlocked + consumed as a knit stage) and the key is spent.

**Data model changes:**

`src/yarn.rs` — add `locked` field to `Patch`:
```rust
pub struct Patch {
    pub color: Color,
    pub locked: bool,
}
```

`src/active_threads.rs` — add `has_key` to `Thread`:
```rust
pub struct Thread {
    pub color: Color,
    pub status: u16,
    pub has_key: bool,
}
```

`src/board_entity.rs` — add `KeyThread` variant:
```rust
pub enum BoardEntity {
    Thread(Color),
    KeyThread(Color),       // thread with a key attached
    Obstacle,
    Void,
    Generator(GeneratorData),
    DepletedGenerator,
}
```

**`process_one` logic** (`src/yarn.rs`):

For each column, inspect the last patch:
- Locked patch, any color, thread has no key → **skip column entirely** (blocked)
- Locked patch, color matches, thread has key → pop patch, `knit_on()`, clear `has_key`
- Unlocked patch, color matches → pop patch, `knit_on()` (unchanged)
- Unlocked patch, color mismatch → skip (unchanged)

**Yarn generation**: locked patches are placed at known positions (puzzle design time), paired with a `KeyThread` of the same color somewhere reachable on the board.

**Files**: `src/board_entity.rs`, `src/active_threads.rs`, `src/yarn.rs`, `src/main.rs`

---

### 5b. Generator

**Design:**

- `Generator(GeneratorData)`: a board cell with a fixed adjacent output cell. The output cell starts with the first thread from the generator's queue. When a player selects and clears the output cell, the generator places the next thread from its queue in that cell. After the queue is exhausted, the generator becomes `DepletedGenerator` (acts as `Obstacle`); the output cell remains `Void`.
- The output direction (which of the 4 orthogonal neighbors is the output cell) is set at puzzle generation time and stored in `GeneratorData`.
- The queue is **not random** — it is defined at puzzle generation time to ensure solvability.

**Data model:**

```rust
pub enum Direction { Up, Down, Left, Right }

pub struct GeneratorData {
    pub color: Color,         // color of generated threads (or use queue for varied colors)
    pub output_dir: Direction,
    pub queue: Vec<Color>,    // remaining outputs; front = next to generate
}
```

`GeneratorData::output_pos(gen_row, gen_col) -> (usize, usize)` computes the output cell from the direction.

**Board entity enum** (full, post-5a+5b):
```rust
pub enum BoardEntity {
    Thread(Color),
    KeyThread(Color),
    Obstacle,
    Void,
    Generator(GeneratorData),
    DepletedGenerator,
}
```

**Enter handler changes** (`main.rs`): when the selected cell is the output cell of a Generator, after setting it to `Void`, call `generator.queue.pop_front()` and if `Some(color)`, place `Thread(color)` back at the output cell. If queue is now empty, set the generator cell to `DepletedGenerator`.

**Yarn generation implication**: the yarn must include patches for all threads the generator will ever produce: `sum over generator queues of len(queue) × knit_volume` patches per color.

**Files**: `src/board_entity.rs`, `src/game_board.rs`, `src/main.rs`

---

## 6. Solvability Checks

**Why**: Random board generation can produce unwinnable boards — e.g. threads that can never be reached due to the void-bordering rule, or locked patches with no reachable key.

### Checks required

**Check 1 — Count balance (per color C):**
```
yarn_patches(C) == (board_threads(C) + sum_of_generator_queue_outputs(C)) × knit_volume
```
where `board_threads(C)` counts `Thread(C)` and `KeyThread(C)`, and generator outputs count the C-colored items across all generator queues.

This is almost guaranteed by construction (the current `count_knits` approach) but must be extended to include generator queues.

**Check 2 — Thread reachability (BFS simulation):**

Simulate the selection process on the static board using BFS:
1. Seed queue with all top-row `Thread`/`KeyThread` cells.
2. For each dequeued cell `(r, c)`: mark as reachable, then "remove" it (simulate → Void).
3. For each orthogonal neighbor of `(r, c)` that is `Thread`/`KeyThread` and not yet reachable: add to queue (it now borders a Void).
4. For generator output cells: the output cell is selectable once reachable. Each time it is cleared, the generator places a new Thread there (same position). The output cell becomes permanently Void only after the queue is exhausted — so treat it as "always available" once it first becomes reachable, until its queue runs out.
5. After BFS: all `Thread`/`KeyThread` cells on the board (plus total generator outputs as a count) must have been reachable. If any thread is unreachable, the board is unsolvable.

**Note on obstacles**: an Obstacle in the top row permanently blocks the column below it if no other Void path exists to those cells. The BFS catches this automatically.

**Check 3 — Key-lock pairing:**

For every `Lock(C)` patch in the yarn:
- There must be exactly one `KeyThread(C)` on the board.
- That `KeyThread(C)` must be reachable (from check 2) before the locked patch would be needed.

The ordering constraint ("reachable before needed") is hard to check statically without simulating the full game. A sufficient relaxation: every `KeyThread` is reachable (check 2 already ensures this). If that holds, a player who picks up the key before pressing Backspace will be able to unlock the patch.

**Check 4 — Active threads headroom:**

At any point in the game, the player's active thread list must be able to make progress. A conservative check: `distinct_colors_on_board ≤ active_threads_limit`. If a player needs to hold one thread of every color simultaneously to make progress, and there are more colors than slots, the board may be unwinnable.

### Implementation

1. Add `src/solvability.rs` with:
   - `fn count_balance(board: &GameBoard, yarn: &Yarn) -> bool`
   - `fn all_threads_reachable(board: &GameBoard) -> bool` (BFS)
   - `fn keys_and_locks_valid(board: &GameBoard, yarn: &Yarn) -> bool`
   - `fn active_headroom_ok(board: &GameBoard, active_limit: usize) -> bool`
   - `fn is_solvable(board: &GameBoard, yarn: &Yarn, active_limit: usize) -> bool` (calls all four)

2. In `main.rs`, after generation: loop regenerating until `is_solvable(...)` returns true. Cap at 100 retries; if exhausted, either panic with a helpful message or relax the config (e.g. reduce obstacle percentage).

**Files**: new `src/solvability.rs`, `src/main.rs`, `src/game_board.rs`

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
Config (1) ──────────────────── enables all features to be configurable
Movement (4, 4.5) ───────────── quick win; 4.5 is prerequisite for meaningful solvability
Solvability basic (6, checks 1+2+4) ── before complex boards get dangerous
Animation (2) ───────────────── UX polish, independent
Horizontal layout (3) ───────── UX polish, independent
Locks/Keys (5a) ─────────────── builds on yarn.rs and board_entity.rs
Generators (5b) ─────────────── builds on board_entity.rs; needs solvability
Solvability full (6, check 3) ── extend to cover locks/keys after 5a
Bonuses (7) ─────────────────── builds on everything above
```

**Suggested order**: 4 → 4.5 → 1 → 6(basic) → 2 → 3 → 5a → 5b → 6(full) → 7

Start with movement (tiny, no deps), then the selectable rule (changes the core interaction), then config (unlocks tuning), then basic solvability (safe foundation for the rest), then the advanced entities.
