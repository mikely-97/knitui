# knitui engine + knitui-ni design

## Goal

Separate game logic from presentation so that:
1. The TUI (`knitui`) remains the main interactive binary
2. A new non-interactive CLI (`knitui-ni`) can drive the same logic via commands

## Structure

Same crate, two `[[bin]]` targets (`src/main.rs` + `src/bin/knitui_ni.rs`).
Game state persists to `~/.local/share/knitui/<hash>.json`.
Output format: JSON.

## GameEngine (src/engine.rs)

Central shared layer. Owns all mutable game state:

```rust
pub struct GameEngine {
    pub board: GameBoard,
    pub yarn: Yarn,
    pub active_threads: Vec<Thread>,
    pub cursor: (u16, u16),
    pub knit_volume: u16,
    pub active_threads_limit: usize,
}
```

Public actions (return `Result<_, _>` — errors are serialisable strings):

- `GameEngine::new(config: &Config) -> Self` — generation + retry loop
- `move_cursor(dir: Direction) -> Result<(), MoveError>`
- `pick_up() -> Result<PickResult, PickError>`
- `process_one() -> ProcessStepResult` — one thread (TUI animation)
- `process_all() -> Vec<ProcessStepResult>` — all threads (NI convenience)
- `is_won() -> bool`

Derives `serde::Serialize/Deserialize`. Requires a `color_serde` helper
since `crossterm::Color` doesn't impl those traits natively.

## knitui-ni commands

```
knitui-ni [OPTIONS]            create game, print state, save to XDG
knitui-ni --game <hash> move <up|down|left|right>
knitui-ni --game <hash> pick
knitui-ni --game <hash> process
```

Success output:
```json
{"status":"ok","game":"abc123","state":{...}}
```

Error output:
```json
{"status":"error","code":"not_selectable","message":"..."}
```

## Files changed

| File | Change |
|------|--------|
| `Cargo.toml` | add serde, serde_json, dirs; add `[[bin]]` knitui-ni |
| `src/color_serde.rs` | new: serde for crossterm::Color |
| `src/engine.rs` | new: GameEngine struct + actions |
| `src/bin/knitui_ni.rs` | new: NI binary |
| `src/board_entity.rs` | derive Serialize/Deserialize |
| `src/yarn.rs` | derive Serialize/Deserialize |
| `src/active_threads.rs` | derive Serialize/Deserialize |
| `src/game_board.rs` | derive Serialize/Deserialize |
| `src/lib.rs` | add new modules |
| `src/main.rs` | use GameEngine, remove duplicated logic |
