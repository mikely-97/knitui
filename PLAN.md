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
7. **Single binary + game selector** — root `src/main.rs` with game selector for knit, match-3, merge-2, and picross

### Key design decisions

- **Game-specific palettes kept local**: m3 uses DARK/BRIGHT/COLORBLIND pools (different from knit's DARK/LIGHT/GREY). Each game keeps its own `palette.rs` rather than forcing a shared palette registry.
- **Game-specific settings kept local**: m3's COLOR_MODES (dark, bright, colorblind, + RGB variants) differ from knit's. Each game keeps its own `settings.rs` with hardcoded config dir paths.
- **Re-export pattern**: loom-knit re-exports `loom_engine::{palette, color_serde, settings, ad_content}` via `pub use` so internal `crate::` imports continue to work.
- **CampaignEntry trait**: generic campaign persistence uses a `CampaignEntry` trait with serde bounds. Each game defines its own campaign state struct (with game-specific bonus fields) implementing this trait.

---

## Completed: Merge-2 Game

`crates/loom-merge2/` is a full game, not a stub: campaign (3 tracks, blessings system),
endless mode, generators/merging mechanics, level design pass, engine tests, and
celebration/mission-summary popups on mission completion.

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

## Completed: Portable Engine Pivot (Phases 0–5)

The single-binary/multi-game workspace above was itself the starting point
for a second, larger migration: from "4 terminal apps sharing some code"
to one portable engine core driven by three real frontends (terminal,
browser, FFI). All 6 phases are done.

- **Phase 0** — decoupled `loom-engine` from crossterm entirely: new
  `render`/`input`/`storage` modules (`Surface`/`Color`/`KeyEvent`/
  `Storage` traits), crossterm itself moved to a new `loom-engine-term` crate.
- **Phase 1** — `TermSurface` (the crossterm-backed `Surface` impl) in
  `loom-engine-term`; all 4 games' renderers migrated to `Surface` calls.
  Verified visually via a pty+pyte capture technique (spawn the real
  binary in a pty, parse the raw ANSI output through the `pyte` terminal
  emulator) rather than trusting a compile pass alone.
- **Phase 2** — `loom-engine-web`: a wasm-bindgen `Surface` impl
  (`WasmSurface`) and a real, playable-in-browser gameplay loop for all 4
  games (`crates/<game>/web.rs` + `web/index.html`). Verified via a
  headless Node smoke test that actually executes the compiled `.wasm`.
  Scope was gameplay-loop-only — no menu/campaign screens in the browser
  yet (that's what Phase 3/4 delivered natively; the web side hasn't
  picked it up).
- **Phase 3** — `loom_engine::shell::{Shell<G>, chrome}`: the generic
  menu/campaign/endless/options/playing state machine, proven first on
  knit (the pilot game). Each game's `create_engine()` — `unimplemented!()`
  since the original scaffolding — got a real `GameEngine` trait adapter
  wrapping its already-working concrete engine.
- **Phase 4** — rolled `Shell<G>` out to the other 3 games (match3, merge2,
  picross). Each adapter surfaced and fixed real, previously-latent bugs
  in the process — not just mechanical porting. Two shared-infrastructure
  additions landed along the way, both needed once a game's shape didn't
  match the first 2 games: `Game::main_menu_items()` (not every game has
  all of Quick/Custom/Campaign/Endless/Options) and `GameEngine::as_any()`
  + `Game::sync_campaign_entry()` (for a game whose campaign state embeds
  live, mutating world state, not just level-index bookkeeping).
- **Phase 5** — FFI: `loom-engine-capi` (a thin C ABI, JSON in/out,
  `cbindgen`-generated header) and `loom-py` (real, memory-safe PyO3
  bindings reusing the C ABI crate's type-erasure layer directly). Both
  verified via genuine cross-language execution, not just Rust calling
  itself: a compiled-and-linked C program (`examples/smoke.c`) and an
  installed Python wheel driving the real extension module.

See `crates/loom-engine-capi/README.md` and `crates/loom-py/README.md`
for the FFI surfaces' exact shape and usage.

## Next Steps

### Completed since Phase 5: full Shell<G> web parity (2026-08-21)

All 4 games' `web.rs` now drive the same `Shell<G>` state machine
`tui.rs` does — main menu, custom game, campaign, endless, options, help
— instead of a bare gameplay loop. This needed a real prerequisite fix
first: `Shell<G>` had no injectable storage backend (its in-session saves
were hardcoded to `FsStorage`'s real files, which silently no-op on
wasm32), so a `Storage` trait was threaded through `Shell::new` and every
save call site; the web builds pass `WebStorage` (localStorage-backed),
which already existed in `loom-engine-web` but had never been wired to
anything. Along the way, a real, previously-undiscovered bug surfaced and
got fixed: `EndlessHighScore.best_wave` was only ever *read* by `Shell`
(displayed on the endless-gameover screen), never updated or saved —
across all 4 games, since Phase 3. Verified per-game via the existing
headless-Node smoke test, rewritten to exercise the real menu → play →
help → quit-to-menu flow against the actual compiled `.wasm`.

### Real remaining gaps

- No pixel-level visual verification of the web builds (the pty+pyte
  trick only covers the terminal side; the Node smoke test proves the
  `.wasm` runs without throwing, not that it looks right).
- The C ABI's C++/Go surface is intentionally just the generated header +
  docs — a polished RAII/`cgo` wrapper is deferred until a real consumer
  wants one.
- Picross's campaign is now sequential-only (see its adapter's doc
  comment) — the original let players jump to any puzzle in a track
  freely; that's a real, permanent, disclosed loss from the Shell<G>
  migration, not an oversight.

### Other ideas (unchanged from before the pivot)

- Puzzle editor / non-random board generation
- Online leaderboards
- Additional game modes (time attack, daily challenge)
- Further unify shared code (game-configurable palettes and color modes;
  knit and m3 both have bonus inventories with different bonus types)
