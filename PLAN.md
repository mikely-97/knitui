# Loom Engine — Design History & Next Steps

## Completed: Multi-Game Engine Migration

knitui was refactored into a Cargo workspace called "loom" with a shared engine crate
and per-game crates, producing a single binary with a game-selector menu.

### Migration phases (all complete)

1. **Workspace scaffolding** — converted repo to Cargo workspace with `crates/` layout
2. **Extract shared types** — moved direction, palette, color_serde, settings, ad_content, generic Board<C> into loom-engine
3. **Extract campaign + endless** — generic `CampaignSaves<E>` and `EndlessHighScore` in loom-engine, parameterized by config dir
4. **Define Game + GameEngine traits** — `Game`, `GameEngine`, `GameConfig` traits plus `Action`, `GameStatus`, `RenderArea` types
5. **Extract TUI framework** — terminal setup/teardown, panic hook, `run_cli()` entry point in each game crate's `tui.rs`
6. **Port m3-tui** — match-3 game ported into `crates/loom-match3/` (124 tests passing)
7. **Single binary + game selector** — root `src/main.rs` with game selector, merge-2 stub

### Key design decisions

- **Game-specific palettes kept local**: m3 uses DARK/BRIGHT/COLORBLIND pools (different from knit's DARK/LIGHT/GREY). Each game keeps its own `palette.rs` rather than forcing a shared palette registry.
- **Game-specific settings kept local**: m3's COLOR_MODES (dark, bright, colorblind, + RGB variants) differ from knit's. Each game keeps its own `settings.rs` with hardcoded config dir paths.
- **Re-export pattern**: loom-knit re-exports `loom_engine::{palette, color_serde, settings, ad_content}` via `pub use` so internal `crate::` imports continue to work.
- **CampaignEntry trait**: generic campaign persistence uses a `CampaignEntry` trait with serde bounds. Each game defines its own campaign state struct (with game-specific bonus fields) implementing this trait.

---

## Completed: Original Feature Plan

All features from the original knitui roadmap are implemented:

- [x] CLI config via clap (unhardcoded parameters)
- [x] Animated/async background processing (step-by-step every 150ms)
- [x] Horizontal + vertical layout with auto-detection
- [x] Movement limits (cursor stays on board)
- [x] Selectability rule (exposed spools only)
- [x] Lock/key mechanic (locked yarn stitches + KeySpools)
- [x] Conveyors (queued spool output with directional placement)
- [x] Solvability checks (count balance, BFS reachability, headroom, key-lock pairing)
- [x] Bonuses: scissors, tweezers, balloons
- [x] Pseudo-ads between rounds
- [x] Campaign mode (3 tracks, 45 levels)
- [x] Endless mode (wave progression with difficulty scaling)
- [x] Non-interactive CLI driver (knitui-ni)
- [x] Solvability testing pipeline (batch-generate + DFS checker)

---

## Next Steps

### Wire up GameEngine trait (low priority)

`create_engine()` is currently `unimplemented!()` in both KnitGame and M3Game. This would allow the shared TUI framework to fully own the event loop rather than each game having its own `tui.rs`. Not blocking — both games work fine with their own event loops.

### Merge-2 game implementation

`crates/loom-merge2/` is a stub. Design the merge-2 mechanics and implement the full game.

### Further unification opportunities

- Make palette pools and COLOR_MODES game-configurable in loom-engine (trait method or associated const)
- Extract more shared menu rendering into loom-engine (main menu, options, campaign select share ~80% of code between games)
- Unify bonus frameworks (knit and m3 both have bonus inventories with different bonus types)

### Other ideas

- Puzzle editor / non-random board generation
- Online leaderboards
- Additional game modes (time attack, daily challenge)
