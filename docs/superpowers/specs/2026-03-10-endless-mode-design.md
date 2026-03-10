# Endless Mode Design

## Overview

Survival/single-game mode. The board regenerates each time it's cleared, with difficulty
scaling up every wave. No end point — play until stuck. High score (best wave reached)
persists across launches.

## Mechanics

### Wave Progression

Each wave is a fresh game generated from a difficulty formula keyed on the wave number.
When the board is cleared (`GameStatus::Won`), the wave advances and a new engine is
created instantly — no game-over screen, seamless transition.

When the player gets stuck (`GameStatus::Stuck`), the run ends and the dedicated
endless game-over screen is shown.

### Difficulty Formula (wave `w`)

| Parameter | Formula | Range |
|-----------|---------|-------|
| `board_height` | `min(4 + w/3, 10)` | 4–10 |
| `board_width` | `min(4 + w/3, 10)` | 4–10 |
| `color_number` | `min(2 + w/4, 8)` | 2–8 |
| `obstacle_percentage` | `min(w*2, 20)` | 0–20 |
| `generator_percentage` | `min(w*2, 20)` | 0–20 |

Display settings (scale, color mode) inherited from user settings.

### Carry-over Bonuses

One bonus is awarded on each wave advance, cycling: scissors → tweezers → balloons
(based on `wave % 3`). Banked bonuses carry into the next wave's starting config.

### High Score

Saved to `~/.config/knitui/endless.json` as `{ "best_wave": N }`.
Updated when a run ends. The game-over screen shows "New record!" if the current
wave beats the saved best, otherwise shows the previous best for comparison.

## UI Flow

1. Main menu → **Endless** → starts immediately at wave 1 (no selection screen)
2. Play — on board clear, wave advances and a fresh board appears seamlessly
3. On stuck → `render_endless_gameover()` shows wave reached + best wave
4. Keys: `R` restart fresh run · `M`/`Esc` return to menu · `Q` quit

## Files Changed

| File | Change |
|------|--------|
| `src/endless.rs` | NEW: `EndlessState` (wave, banked bonuses, difficulty formula), `EndlessHighScore` (load/save) |
| `src/lib.rs` | Added `pub mod endless;` |
| `src/renderer.rs` | Added `render_endless_gameover()` |
| `src/main.rs` | `endless_ctx`, menu item 3, wave-advance on Win, stuck handling, R/M/Q in GameOver |

## Testing

- `new_state_starts_at_wave_one` — initial state
- `advance_increments_wave_and_awards_bonus` — wave + bonus logic
- `to_config_scales_with_wave` — difficulty formula correctness
- `high_score_update_initial_record` — first record set
- `high_score_update_returns_true_for_new_record` — record tracking
- `high_score_serialization_roundtrip` — persistence
