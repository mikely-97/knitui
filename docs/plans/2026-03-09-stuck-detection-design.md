# Stuck Detection Design

## Problem

When the player reaches a deadlock — no valid moves remain and processing can't make progress — the game silently freezes. The player has no feedback that they're stuck.

## Requirements

- Detect true deadlock after all active thread processing has settled
- Display "You're lost!" with options (restart / quit)
- Keep the design extensible for future recovery options (ads, powerups, undo)

## Approach: GameStatus enum + TUI state machine

### Engine: detection

Add `GameStatus` enum and `status()` method to `GameEngine`.

```rust
pub enum GameStatus {
    Playing,
    Won,
    Stuck,
}
```

`status()` replaces the current `is_won()` as the primary game state query. `is_won()` remains for backwards compatibility (used in tests, NI binary).

**Stuck conditions** (both require `!is_won()`):

1. **Front-of-queue deadlock**: `active_threads[0]` exists but cannot match any yarn column's last patch (considering lock rules)
2. **No-moves deadlock**: `active_threads` is empty and no thread/keythread on the board is selectable

Condition 1 detail — `active_threads[0]` is stuck when for every yarn column:
- Column is empty, OR
- Last patch is locked and (color doesn't match OR thread has no key), OR
- Last patch is unlocked and color doesn't match

### TUI: response

Add a `TuiState` enum to the main loop:

```rust
enum TuiState {
    Playing,
    GameOver(GameStatus), // Won or Stuck
}
```

When `GameOver(Stuck)`:
- Render board normally (player can see what went wrong)
- Overlay message: "You're lost! Press R to restart, Q to quit"
- R: create fresh `GameEngine::new(&config)`, reset to `Playing`
- Q: break out of loop, exit

When `GameOver(Won)`:
- Existing win behavior (currently none — can add "You won!" similarly)

### Detection timing

Check `engine.status()` after every `process_one_active()` call and after every `pick_up()`. This ensures stuck is detected as soon as it becomes true, without polling overhead.

### NI binary

Add `"status"` field to JSON output from `engine.status()`. The NI binary already serializes game state — this extends it naturally.

## Files changed

| File | Change |
|---|---|
| `src/engine.rs` | Add `GameStatus` enum, `status()` method, `can_process_front()` helper |
| `src/main.rs` | Add `TuiState`, check status after process/pick_up, render overlay, handle R/Q |
| `src/game_board.rs` | Add `has_selectable_thread()` helper (iterates board, returns bool) |
| `src/bin/knitui_ni.rs` | Include status in JSON output |

## Future extensibility

- `GameStatus::Stuck` can later carry data: `Stuck { recovery_options: Vec<RecoveryOption> }`
- `RecoveryOption` could be `WatchAd`, `UsePowerup(PowerupType)`, `Undo(n_moves)`
- The TUI overlay just renders whatever options are present — no hardcoded behavior
