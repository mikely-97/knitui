# Bonuses, Help Overlay & Key Bar Design

## Overview

Add three bonuses (Scissors, Tweezers, Balloons) activated by hotkeys, a help overlay (H key), and a persistent key bar at the bottom of the screen.

## Bonuses

### Data Model

**BonusInventory** (new struct, lives on `GameEngine`):

```rust
pub struct BonusInventory {
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
    pub scissors_threads: u16,   // how many threads scissors processes
    pub balloon_lift_count: u16, // how many patches to lift per column
}
```

**BonusState** (active bonus state machine, lives on `GameEngine`):

```rust
pub enum BonusState {
    None,
    TweezersActive { saved_row: u16, saved_col: u16 },
}
```

**Balloon pseudo-columns** on `Yarn`:

```rust
pub balloon_columns: Vec<Vec<Patch>>
```

### Config (CLI flags)

| Flag | Default | Description |
|------|---------|-------------|
| `--scissors` | 0 | Starting scissors count |
| `--tweezers` | 0 | Starting tweezers count |
| `--balloons` | 0 | Starting balloons count |
| `--scissors-threads` | 1 | Threads processed per scissors use |
| `--balloon-count` | 2 | Patches lifted per column per balloons use |

### Scissors (Z key)

1. Guard: `scissors > 0`, no bonus active, `active_threads` not empty
2. Select thread(s) with the lowest `status` (least progress), up to `scissors_threads`
3. For each selected thread: deep-scan ALL patches in every yarn column (not just front) for color match. Remove matched patch from column, call `knit_on()`. Repeat until thread reaches `knit_volume` or no more matches
4. Remove completed threads from active list
5. Decrement `scissors`

### Tweezers (X key)

1. Guard: `tweezers > 0`, no bonus active
2. Save cursor position → `BonusState::TweezersActive { saved_row, saved_col }`
3. Cursor can move to ANY cell (ignoring selectability/focusability). Bracket markers render as `{ }` instead of `[ ]`
4. On `pick_up()`: ignore selectability check, pick thread, restore cursor, exit tweezers mode
5. Decrement `tweezers`
6. Esc during tweezers mode: cancel without consuming bonus

### Balloons (C key)

1. Guard: `balloons > 0`, no bonus active, `balloon_columns` is empty
2. For each yarn column: pop front `balloon_lift_count` patches → new entries in `balloon_columns`
3. `process_one` checks `balloon_columns` for matches too (same color-matching logic)
4. When a balloon column is fully consumed, remove it
5. Rendered visually distinct (above regular yarn or with `^` marker)
6. Decrement `balloons`

### Guard: no simultaneous bonuses

A bonus cannot be activated while a previous bonus is still in effect:
- Scissors: instant effect, no lingering state
- Tweezers: active until a thread is picked or Esc cancels
- Balloons: active until all `balloon_columns` are consumed

Check `BonusState != None || !balloon_columns.is_empty()` before allowing activation.

## Help Overlay (H key)

Full-screen overlay (same pattern as Won/Stuck overlays):

```
                    ═══ HELP ═══

  Movement:   ← → ↑ ↓   Move cursor
  Pick up:    Enter       Pick up thread at cursor
  Quit:       Esc / Q     Exit game
  Restart:    R            New game

  ─── Bonuses ───
  [Z] ✂ Scissors   Auto-knit threads by deep-scanning yarn
  [X] ⊹ Tweezers   Pick any thread from the board
  [C] ⊛ Balloons   Lift front patches, expose patches behind

              Press any key to close
```

New `TuiState::Help` variant. Any keypress returns to `TuiState::Playing`.

## Key Bar (bottom of screen)

A single line at the terminal bottom, always visible during play:

```
←→↑↓ Move  Enter Pick  H Help  Z ✂x2  X ⊹x1  C ⊛x3  Esc Quit
```

Bonus counts update live. Bonuses with 0 count render dimmed (grey).

## Bonus Display Area

- **Vertical layout:** Below the board: `[Z] ✂ x2   [X] ⊹ x1   [C] ⊛ x3`
- **Horizontal layout:** Right side, stacked vertically

## Architecture

All bonus logic lives on `GameEngine` (engine-integrated approach):
- `use_scissors(&mut self) -> Result<(), BonusError>`
- `use_tweezers(&mut self) -> Result<(), BonusError>` (enters tweezers mode)
- `use_balloons(&mut self) -> Result<(), BonusError>`
- `pick_up()` checks `BonusState::TweezersActive` to bypass selectability
- `move_cursor()` checks `BonusState::TweezersActive` to bypass focusability
- `process_one()` checks `balloon_columns` in addition to regular yarn columns

### Files touched

- `src/engine.rs` — BonusInventory, BonusState, bonus methods, modified pick_up/move_cursor/process
- `src/yarn.rs` — `balloon_columns` field, modified `process_one`
- `src/config.rs` — new CLI flags
- `src/main.rs` — hotkey handling (Z/X/C/H), help overlay rendering, key bar rendering, bonus display area, TuiState::Help variant
- `src/bin/knitui_ni.rs` — snapshot support for bonus state

### Snapshot/NI support

`GameStateSnapshot` gains:
- `scissors: u16, tweezers: u16, balloons: u16`
- `balloon_columns: Vec<Vec<YarnPatchSnap>>`
- `bonus_state: String` (serialized BonusState)
