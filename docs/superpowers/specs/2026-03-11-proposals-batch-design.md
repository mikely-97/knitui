# KnitUI Proposals Batch Design

**Date:** 2026-03-11
**Status:** Approved

This document covers 7 proposals decomposed into 4 implementation batches.

---

## Batch 1: Quick Fixes

### 1A. Board Maximum Size 6x6

**Problem:** Board dimensions are uncapped (custom game allows up to 20x20, endless scales to 10x10). Some campaign levels exceed 6 in one dimension.

**Solution:**

- Add `pub const MAX_BOARD_DIM: u16 = 6;` in `config.rs` as single source of truth.
- In `main.rs`, change `adjust_custom_field` limits for height (field 1) and width (field 2) from `(2, 20)` to `(2, MAX_BOARD_DIM)`.
- In `endless.rs`, cap `to_config()` scaling: `.min(MAX_BOARD_DIM)` for both dimensions.
- In `campaign_levels.rs`, audit all levels. Any level with `board_height > 6` or `board_width > 6` gets clamped to 6. Redistribute removed capacity by increasing `color_number` or `conveyor_capacity` to maintain difficulty.

**Files changed:** `config.rs`, `main.rs`, `endless.rs`, `campaign_levels.rs`

---

### 1B. Lock/Key Spawning Fix

**Problem:** `BoardEntity::KeySpool` variant exists and is fully handled by the engine, solvability checker, and renderer — but `GameBoard::make_random` never places any. Keys never appear in actual gameplay.

**Root cause:** No key-placement pass in `make_random`.

**Solution:**

Add a Pass 4 to `GameBoard::make_random` after conveyor reversion:

1. Count total spools on the board (Spool + KeySpool variants).
2. Calculate `key_count = total_spools / 12` (roughly 1 key per 12 spools).
3. Only place keys when `total_spools >= 12` (tiny boards stay key-free).
4. Randomly select `key_count` cells that contain `BoardEntity::Spool(color)` and convert them to `BoardEntity::KeySpool(color)`.
5. In `Yarn::make_from_color_counter` (or immediately after yarn generation in `GameEngine::new`), for each KeySpool color on the board, mark one corresponding stitch in the yarn as `locked: true`. Choose the deepest (last) stitch of that color in any yarn column.

The existing `keys_and_locks_valid` solvability check already validates that every locked stitch has a reachable matching key, so no changes needed there.

**Files changed:** `game_board.rs`, `engine.rs` (or `yarn.rs`)

---

### 1C. Held Spool Counter

**Problem:** Hard to tell visually if the held spool rack is full or near-full. The "Stuck" condition triggers when the rack is full and no spools can be placed on yarn, but the player can't see how close they are.

**Solution:**

Add a held-spool counter to the bonus display:

- Format: `⊞ 3/7` (held count / spool_limit)
- Location: appended after the balloon entry in both `render_bonus_display_h` (vertical layout) and `render_bonus_panel` (horizontal layout).
- Color: white when safe, yellow when `held >= spool_limit - 2`, red when `held >= spool_limit - 1`.
- **Scope:** Appears in all game modes (Custom Game, Campaign, Endless). The `spool_limit` field exists in every mode's config, so the counter is always meaningful. In Endless mode (where bonuses are unlimited), the counter still helps because scissors must be actively used — the rack can still fill up if the player isn't paying attention.

**Files changed:** `renderer.rs`

---

## Batch 2: Endless Mode Rework

### Current Behavior

Endless mode is wave-based: each wave generates a new board with scaling difficulty. Player earns bonuses every 3 waves. High score = highest wave reached.

### New Behavior

Endless mode becomes a single continuous scrolling puzzle — a chill, relaxing mode.

#### Core Mechanic

- On start, generate one tall board: 6 columns x 66 rows (6 visible + 60 buffer).
- Display only the top 6 rows as the visible window.
- The yarn is generated to match the color counts of the entire 66-row board.
- Player plays normally on the visible 6x6 grid.

#### Row Shifting

After each pick-up + process cycle, check if any of the top rows are fully exhausted. A row is exhausted when it contains **no interactive entities** — i.e., every cell is one of: `Void`, `Obstacle`, `EmptyConveyor`. Any cell that is a `Spool`, `KeySpool`, `Conveyor` (with queue), `FrozenSpool`, `DecaySpool`, `DecayObstacle`, or `Slider` makes the row **not** exhausted. (If Batch 4 entities are not yet implemented when Batch 2 ships, the check simply covers the existing types and is extended when Batch 4 lands.)

When exhausted rows exist:
1. Remove the exhausted top row(s) from the visible board.
2. Pull the next row(s) from the buffer into the bottom of the visible board.
3. Adjust cursor position to stay valid (shift up by the number of removed rows, clamped to valid range).
4. Decrement the remaining-rows counter.

#### Row Budget Counter

- Displayed in the bonus panel/keybar: `Rows: 47 remaining`
- Counts down as rows are pulled from the buffer.
- When buffer reaches 0: no more rows shift in. Player finishes the visible board.

#### Scoring

- High score = total rows cleared (shifted through + final visible rows cleared).
- Displayed on game-over screen, replacing wave count.
- `EndlessHighScore` changes `best_wave: usize` to `best_rows_cleared: usize`.
- **Migration:** Add `#[serde(alias = "best_wave")]` to the `best_rows_cleared` field so existing save files with `best_wave` deserialize correctly without losing high scores.

#### Bonuses

- Start with 999 scissors, 999 tweezers, 999 balloons.
- No earning mechanism — unlimited supplies for chill play.

#### Solvability

- The tall board is generated as one piece.
- **Skip full DFS solvability** (`count_solutions` and `is_solvable`): a 6x66 board has ~396 cells, making DFS intractable.
- Instead, use **color-balance-only** validation: call `count_balance(&board, &yarn, spool_capacity)` to verify the yarn has the right stitch counts for the board's spools. Skip `all_spools_reachable`, `active_headroom_ok`, and `keys_and_locks_valid`.
- Rationale: with 999 of each bonus, the player can always scissors out of stuck states, balloon past locked stitches, and tweezers reach buried spools. Full solvability guarantees are unnecessary and would make generation impossibly slow.

#### EndlessState Changes

Remove:
- `wave`, `banked_scissors`, `banked_tweezers`, `banked_balloons`
- `advance()`, `to_config()`

Add:
- `total_rows: usize` — total rows in the full board
- `rows_cleared: usize` — rows successfully shifted through
- `row_buffer: Vec<Vec<BoardEntity>>` — remaining rows not yet visible

**Files changed:** `endless.rs`, `engine.rs` (row shift logic), `renderer.rs` (row counter display), `main.rs` (endless flow rewrite)

---

## Batch 3: Blessings System

### Overview

Blessings are passive modifiers selected at campaign start that last the entire campaign run. 12 blessings across 4 tiers, with ascending unlock requirements.

### Data Model

New module: `blessings.rs`

```rust
pub enum Tier { D, C, B, A }

pub struct Blessing {
    pub id: &'static str,
    pub name: &'static str,
    pub tier: Tier,
    pub description: &'static str,
    pub ascii_art: [&'static str; 5],  // 5-line ASCII card art
}
```

All 12 blessings defined as a `const` array. No runtime registration.

### Tier Unlock Progression

- **D-tier (6 blessings):** Available from the start.
- **C-tier (3 blessings):** Unlock after completing 1 campaign track.
- **B-tier (2 blessings):** Unlock after completing 2 campaign tracks.
- **A-tier (1 blessing):** Unlock after completing all 3 campaign tracks.

Unlock state derived from `CampaignSaves` by counting tracks with `completed == true`.

### Selection Flow

1. Player selects a campaign track.
2. Before level intro, show **Blessing Selection** screen.
3. Screen displays all unlocked blessings as ASCII art cards in a scrollable grid.
4. Locked blessings shown greyed out with "Complete N tracks to unlock" text.
5. Player must pick exactly 3 blessings. On a fresh save all 6 D-tier are unlocked, so there are always at least 3 available. If resuming a campaign that already has blessings, the selection screen is skipped.
6. Selected blessings stored in `CampaignState` as `blessings: Vec<String>` (blessing IDs).
7. Blessings applied before each level via `CampaignState::to_config()` and engine flags.

### The 12 Blessings

#### D-Tier: Informational / QoL (no gameplay impact)

| ID | Name | Effect |
|----|------|--------|
| `scouts_eye` | **Scout's Eye** | Locked stitches get a visible marker in the yarn display |
| `wrap_around` | **Wrap Around** | Cursor wraps at board edges instead of stopping |
| `tidy_workspace` | **Tidy Workspace** | Held spools auto-sort by color |
| `conveyor_peek` | **Conveyor Peek** | See the next spool a conveyor will produce |
| `color_count` | **Color Count** | Display remaining spool count per color on the board |
| `match_hint` | **Match Hint** | After picking a spool, briefly highlight which yarn stitches it can match |

#### C-Tier: Tiny Numerical Edges

| ID | Name | Effect |
|----|------|--------|
| `lucky_find` | **Lucky Find** | 3% of obstacles spawn as Voids instead |
| `apprentices_kit` | **Apprentice's Kit** | Start the campaign with 1 extra scissors (total, not per level) |
| `light_pockets` | **Light Pockets** | Start the campaign with 1 extra balloon (total, not per level) |

#### B-Tier: Small Recurring Buffs

| ID | Name | Effect |
|----|------|--------|
| `extra_slot` | **Extra Slot** | +1 to spool_limit |
| `sharp_start` | **Sharp Start** | +1 scissors at the start of each level |

#### A-Tier: Tool Upgrade

| ID | Name | Effect |
|----|------|--------|
| `double_cut` | **Double Cut** | Scissors destroy 2 spools instead of 1 |

### Implementation Categories

**Config modifiers** (applied in `CampaignState::to_config()`):
- `lucky_find`: reduce obstacle_percentage by 3
- `apprentices_kit`: +1 scissors at campaign start (banked)
- `light_pockets`: +1 balloons at campaign start (banked)
- `extra_slot`: +1 spool_limit
- `sharp_start`: +1 scissors per level

**Engine flags** (checked during gameplay):
- `scouts_eye`: renderer flag to show lock markers
- `wrap_around`: cursor movement wraps instead of clamping
- `tidy_workspace`: sort held_spools by color after each pick
- `conveyor_peek`: renderer shows next queue element
- `color_count`: renderer shows per-color counts
- `match_hint`: renderer highlights matching stitches after pick
- `double_cut`: scissors_spools = 2

### ASCII Art Card Format

Each blessing rendered as a bordered card:

```
┌─────────────┐
│   (art L1)  │
│   (art L2)  │
│   (art L3)  │
│   (art L4)  │
│   (art L5)  │
│  CARD NAME  │
│  ─ X Tier ─ │
│  description │
└─────────────┘
```

Cards displayed in a grid (3 per row). Selected cards get a highlight border. Locked cards greyed out.

### Persistence

`CampaignState` gains a new field:
```rust
pub blessings: Vec<String>,  // e.g., ["scouts_eye", "extra_slot", "double_cut"]
```

Default: empty vec (backwards-compatible with existing saves via `#[serde(default)]`).

### Blessing Interactions with Batch 4 Entities

When Batch 4 is implemented, blessings interact as follows:
- `lucky_find` (3% obstacles → voids): applies only to initial `Obstacle` cells during generation, NOT to `DecayObstacle` (which is a runtime state, not generated).
- `color_count`: counts FrozenSpool and DecaySpool colors (they are spools that will eventually be available).
- `match_hint`: does NOT highlight FrozenSpools (they can't be picked yet). Does highlight DecaySpools (they can be picked while countdown > 0).
- `double_cut` (scissors destroy 2): works on any pickable spool. Does NOT work on FrozenSpools (can't be targeted). Can target DecaySpools if they haven't decayed yet.
- `scouts_eye`, `wrap_around`, `tidy_workspace`, `conveyor_peek`: no entity interaction issues.

**Files changed:** new `blessings.rs`, `campaign.rs`, `engine.rs`, `renderer.rs`, `main.rs`, `lib.rs`

---

## Batch 4: New Board Entities

### Overview

Three new entity types that add strategic depth across spatial complexity, time pressure, and spatial manipulation axes.

### Entity 1: Frozen Spool

**Concept:** A spool encased in ice that must be thawed before pickup.

**Mechanics:**
- Represented as `BoardEntity::FrozenSpool(Color, u8)` where `u8` is the ice level (starts at 2).
- Cannot be picked up while frozen.
- Each time an orthogonally adjacent spool is picked up, ice level decrements by 1.
- When ice reaches 0, converts to a regular `BoardEntity::Spool(Color)`.
- Is focusable (cursor can land on it) but not selectable (cannot be picked).
- Rendered with a special glyph (e.g., `*` or a blue-tinted spool character) showing ice level.

**Solvability:**
- Solvability checker treats frozen spools as regular spools that require adjacent picks first. The BFS reachability check considers them reachable if their neighbors are reachable.
- The `count_solutions` DFS models thawing: a frozen spool becomes pickable after enough adjacent picks.

**Generation:**
- In `make_random`, after Pass 4 (keys), add Pass 5: convert ~5% of regular spools to frozen (on boards with >= 16 spools).
- Campaign levels can specify `frozen_percentage` for fine control.

### Entity 2: Decay Spool

**Concept:** A spool with a countdown that temporarily becomes an obstacle, then thaws when fully exposed.

**Mechanics:**
- Represented as `BoardEntity::DecaySpool(Color, u8)` where `u8` is the remaining countdown.
- Starts with countdown = 4 (configurable).
- Each time the player picks up any spool anywhere on the board, all decay spools decrement by 1.
- When countdown reaches 0: converts to `BoardEntity::DecayObstacle(Color)` — a temporary obstacle that remembers its color.
- **Thaw condition:** When all orthogonal neighbors of a `DecayObstacle` are Void, Obstacle, EmptyConveyor, or other non-spool entities (i.e., it's fully exposed with no adjacent spools), it thaws back into `BoardEntity::Spool(Color)`.
- This ensures the puzzle is always completable: the player might need to clear around the decayed spool first, but it will eventually become available again.

**Solvability:**
- Checker treats decay spools as regular spools with delayed availability.
- **Deadlock prevention:** Multiple adjacent DecaySpools could theoretically all decay into DecayObstacles simultaneously, and since each requires all neighbors to be non-spool to thaw, they'd block each other. To prevent this: during generation (Pass 6), never place two DecaySpools in orthogonally adjacent cells. This is enforced by checking neighbors before conversion. With this constraint, a DecayObstacle's neighbors will always be regular spools, voids, or obstacles — never another DecayObstacle — so it will thaw once those neighbors are cleared.

**Generation:**
- In `make_random`, Pass 6: convert ~3% of spools to decay (on boards with >= 20 spools).
- Campaign levels can specify `decay_percentage`.

### Entity 3: Slider

**Concept:** A directional cell that slides spools through it when a void opens.

**Mechanics:**
- Represented as `BoardEntity::Slider(Direction)` — has a fixed direction arrow.
- The slider itself is not a spool and cannot be picked up. It acts like a transparent conduit.
- **Trigger:** When the cell the slider points toward becomes Void, AND the cell opposite the slider's direction contains a Spool/KeySpool/FrozenSpool/DecaySpool:
  - The spool behind the slider slides through, ending up in the void cell.
  - The cell behind the slider becomes Void.
  - The slider remains in place.
- Rendered with a directional arrow (→, ←, ↑, ↓) in a distinct color.
- **Chain reaction semantics:** Slider activation is processed in a single pass after each pick-up. When a spool is picked, all sliders are checked once (in row-major order). A slider that fires may create a new void, but that new void does NOT trigger other sliders in the same pass. Chain reactions happen on the **next** pick-up event. This keeps the state change predictable (one slider fires per pick per location) and makes solvability checking tractable.

**Solvability:**
- Slider effects are deterministic (fixed direction) and can be simulated in the `count_solutions` DFS.
- The checker simulates: "if picking spool X creates a void at position Y, and position Y is where slider Z points, then spool W slides from behind Z into Y."

**Generation:**
- In `make_random`, Pass 7: place sliders on ~3% of cells (replacing a spool). Only place if both the "from" and "to" neighbors exist and contain spools.
- Campaign levels can specify `slider_percentage`.

### BoardEntity Enum Changes

```rust
pub enum BoardEntity {
    Spool(Color),
    KeySpool(Color),
    Obstacle,
    Void,
    Conveyor(ConveyorData),
    EmptyConveyor,
    // New:
    FrozenSpool(Color, u8),       // color, ice_level
    DecaySpool(Color, u8),        // color, countdown
    DecayObstacle(Color),         // color (remembers for thaw)
    Slider(Direction),            // fixed direction
}
```

### Config Changes

New optional fields (used by campaign levels, defaults to 0 for custom game):
```rust
pub frozen_percentage: u16,   // % chance spool becomes frozen
pub decay_percentage: u16,    // % chance spool becomes decay
pub slider_percentage: u16,   // % cells become sliders
```

**Files changed:** `board_entity.rs`, `game_board.rs`, `engine.rs`, `solvability.rs`, `renderer.rs`, `config.rs`, `campaign_levels.rs`

---

## Implementation Order

1. **Batch 1** (Quick Fixes): 1A board cap → 1B lock/key fix → 1C held counter
2. **Batch 2** (Endless Rework): rewrite endless.rs → engine row-shift logic → renderer counter → main.rs flow
3. **Batch 3** (Blessings): blessings.rs module → campaign integration → selection UI → renderer cards
4. **Batch 4** (New Entities): board_entity variants → make_random passes → engine mechanics → solvability updates → renderer glyphs → campaign level integration

Each batch is independently shippable and testable.

**Note on pass numbering:** Batch 4's generation passes (Frozen=Pass 5, Decay=Pass 6, Slider=Pass 7) assume Batch 1B's key pass (Pass 4) is already implemented. If Batch 4 is implemented before Batch 1B, renumber accordingly. The ordering within `make_random` should be: obstacles → spools → conveyors → revert bad conveyors → keys → frozen → decay → sliders.
