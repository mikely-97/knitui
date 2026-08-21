# Loom

A portable multi-game engine built with Rust. Currently ships four playable games — **Knit** (spool-knitting puzzle), **Match-3** (classic gem-matching), **Merge-2** (merge/order-fulfillment), and **Picross** (nonogram) — sharing one generic core (`loom-engine::shell::Shell<G>`) across three frontends: a real terminal (crossterm), a browser (WASM/canvas), and any language with a C or Python FFI. Same menus, same campaigns, same saves, wherever it runs.

Binaries:
- **loom** — game selector menu → launches Knit, Match-3, Merge-2, or Picross (terminal)
- **knitui** — launch Knit directly (interactive TUI)
- **knitui-ni** — non-interactive CLI driver for Knit (JSON in/out, for scripting and AI agents)
- **knitui-solvcheck** — independent solvability checker for Knit (reads NDJSON, runs DFS verification)

Other ways to run a game:
- **Browser** — each game crate has a `web/` dir with a WASM build (`crates/<game>/web/index.html` + a wasm-bindgen build); see that crate's `web.rs`.
- **C** — link against `crates/loom-engine-capi`'s generated header (`include/loom.h`) from C, C++, Go, or anything else with a C FFI.
- **Python** — `pip install` the `loom-py` wheel (built via maturin) for a native, memory-safe `LoomGame` class.

Clone and run:

```
cargo run                  # game selector
cargo run --bin knitui     # knit directly
```

Pass `--help` to see all options:

```
cargo run --bin knitui -- --help
```

## How to Play

The screen shows three sections (top-to-bottom in vertical layout, left-to-right in horizontal):

1. **Yarn queue** — rows of colored stitches (`▦`) showing upcoming knitting work, split into columns. Locked stitches show as `▣` and block their column until cleared with a key.
2. **Held spools** — spools you've selected from the board, waiting to be processed
3. **Game board** — a bordered grid of cells to clear. The selected cell is marked with `[` `]` bracket markers.

**Goal**: Clear all spools from the board by picking them up and processing them against the yarn queue. Each spool must be processed `--spool-capacity` times (default: 3) to complete and be discarded.

**Controls**:

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor across the board |
| Enter | Pick up the spool under the cursor |
| H | Show help overlay |
| Z | Use Scissors bonus |
| X | Use Tweezers bonus |
| C | Use Balloons bonus |
| R | Restart (on game over) |
| Esc | Cancel active bonus / Quit |

A key bar at the bottom of the screen shows all available controls and current bonus counts.

### Selectability rule

Only **exposed** spools can be picked up:
- The **top row** is always selectable.
- Any other spool is selectable only if it **borders a `Void` cell** horizontally or vertically (not diagonally).

Cells become Void when their spool is picked up. Clearing a spool exposes its neighbors, cascading inward from the top.

### Board entities

| Glyph | Entity | Behavior |
|-------|--------|----------|
| `T` (colored) | Spool | Normal selectable spool |
| `K` (colored) | Key spool | Spool that carries a key; displayed `k` in held list until key is spent |
| `X` | Obstacle | Impassable; never becomes Void |
| ` ` | Void | Empty; makes orthogonal neighbors selectable |
| `^` `V` `<` `>` (colored) | Conveyor | Arrow shows output direction. Produces spools in its adjacent output cell up to `--conveyor-capacity` times, then becomes `#` |
| `#` | Depleted conveyor | Acts like an obstacle |

### Lock / Key mechanic

A locked yarn stitch (`▣`) blocks its entire column — nothing behind it can be processed until the lock is cleared. To clear it, pick up the matching **Key spool** (`K`) from the board. The key is consumed on contact and the lock is removed as a normal wind stage.

### Bonuses

Bonuses are optional power-ups activated by hotkeys. Their counts are set at launch via CLI flags (default: 0). The bonus display shows icons, hotkeys, and remaining counts below the board (vertical layout) or to the right (horizontal layout). Bonuses with 0 remaining are greyed out.

| Bonus | Key | Icon | Effect |
|-------|-----|------|--------|
| **Scissors** | Z | ✂ | Instantly auto-winds the least-progressed held spool by deep-scanning ALL stitches in the yarn (not just the front). Ignores queue order. |
| **Tweezers** | X | ⊹ | Enter free-cursor mode: move to any cell and pick up any spool regardless of selectability. Cursor shows `{ }` brackets. Press Esc to cancel without consuming. |
| **Balloons** | C | ⊛ | Lifts the front N stitches from each yarn column into separate pseudo-columns, exposing the stitches behind them. Pseudo-columns are also matchable. |

Guards: only one bonus can be active at a time. Scissors requires held spools. Balloons requires previous balloon columns to be fully consumed first.

### Background processing

Held spools are processed automatically in the background (one step every 150 ms). You can continue moving and picking up spools while processing runs. Each spool is matched against the yarn one at a time so you can see what matches and what doesn't.

## Non-interactive mode (knitui-ni)

`knitui-ni` drives the same game engine via CLI commands. Game state persists as JSON files in `~/.local/share/knitui/`.

### Create a game

```bash
cargo run --bin knitui-ni                    # default options
cargo run --bin knitui-ni -- --board-height 3 --board-width 4  # custom
```

Output: JSON with `"status": "ok"`, `"game": "<8-char hash>"`, and full `"state"`.

### Create from campaign / endless mode

```bash
cargo run --bin knitui-ni -- --campaign --track 0 --level 5     # campaign level
cargo run --bin knitui-ni -- --endless-wave 7                    # endless wave
cargo run --bin knitui-ni -- --max-solutions 1                   # force single-solution puzzle
cargo run --bin knitui-ni -- --ad-limit 3                        # set ad limit (campaign)
```

### Execute commands

```bash
cargo run --bin knitui-ni -- --game <HASH> move <up|down|left|right>
cargo run --bin knitui-ni -- --game <HASH> pick
cargo run --bin knitui-ni -- --game <HASH> process
cargo run --bin knitui-ni -- --game <HASH> scissors
cargo run --bin knitui-ni -- --game <HASH> tweezers
cargo run --bin knitui-ni -- --game <HASH> cancel-tweezers
cargo run --bin knitui-ni -- --game <HASH> balloons
cargo run --bin knitui-ni -- --game <HASH> ad
```

Success response:
```json
{"status":"ok","game":"abc123xy","won":false,"game_status":"playing","state":{...}}
```

Error response (to stderr, exit code 1):
```json
{"status":"error","code":"not_selectable","message":"spool is not exposed"}
```

Error codes: `out_of_bounds`, `not_selectable`, `not_a_spool`, `active_full`, `bonus_failed`, `ad_limit_reached`, `load_failed`, `save_failed`, `no_command`.

### Query and batch subcommands

```bash
cargo run --bin knitui-ni -- list-campaign              # JSON dump of all campaign tracks/levels
cargo run --bin knitui-ni -- describe-wave 10            # config for endless wave 10
cargo run --bin knitui-ni -- batch-generate --count 100  # generate 100 boards as NDJSON
cargo run --bin knitui-ni -- --campaign --track 1 --level 3 batch-generate --count 50
```

## Solvability testing pipeline

An independent pipeline verifies that every generated board is solvable without bonuses. It uses `knitui-ni batch-generate` to produce boards and `knitui-solvcheck` to verify each one via full DFS.

```bash
cargo build --release
bash scripts/test_solvability.sh        # default: 50 boards per config
bash scripts/test_solvability.sh 200    # 200 boards per config
```

The script tests:
1. All campaign levels (3 tracks, 45 levels) with zero bonuses
2. Endless waves 1–30 with zero bonuses
3. Full parameter sweep (heights × widths × colors × obstacle%)
4. Conveyor configurations

You can also run the checker manually:

```bash
cargo run --bin knitui-ni -- batch-generate --count 100 | cargo run --bin knitui-solvcheck
```

## Configuration

All parameters are settable via CLI flags (both binaries). Defaults:

| Flag | Default | Description |
|------|---------|-------------|
| `--board-height` | 6 | Grid rows |
| `--board-width` | 6 | Grid columns |
| `--color-number` | 6 | Distinct colors used |
| `--color-mode` | `dark` | Palette: `dark` \| `bright` \| `colorblind` \| `dark-rgb` \| `bright-rgb` \| `colorblind-rgb` |
| `--spool-limit` | 7 | Max spools held at once |
| `--spool-capacity` | 3 | Times each spool must be wound to complete |
| `--yarn-lines` | 4 | Yarn columns |
| `--obstacle-percentage` | 5 | % chance each cell is an obstacle |
| `--visible-stitches` | 6 | Yarn rows shown on screen |
| `--conveyor-capacity` | 3 | Spools each conveyor produces |
| `--conveyor-percentage` | 5 | % chance each cell becomes a conveyor |
| `--layout` | `auto` | Layout: `auto` \| `horizontal` \| `vertical` |
| `--scale` | 1 | Cell scale factor (1–3): render each entity as N×N characters |
| `--scissors` | 0 | Starting scissors bonus count |
| `--tweezers` | 0 | Starting tweezers bonus count |
| `--balloons` | 0 | Starting balloons bonus count |
| `--scissors-spools` | 1 | Spools wound per scissors use |
| `--balloon-count` | 2 | Stitches lifted per yarn column per balloons use |
| `--max-solutions` | — | Max distinct winning pick sequences (slower generation for small values) |

The `-rgb` color modes use 24-bit true color escapes, which are immune to terminal theme overrides (useful for kitty, alacritty, etc. that remap ANSI palette slots).

`--layout auto` picks vertical if the terminal is tall enough, otherwise horizontal. At `--scale 2` or `3`, each cell is rendered as a 2×2 or 3×3 block inside a box-drawing grid.

Example — play with bonuses:

```
cargo run --bin knitui -- --scissors 3 --tweezers 2 --balloons 2
```

Example — a bigger, harder board:

```
cargo run --bin knitui -- --board-height 8 --board-width 10 --spool-capacity 5 --color-mode bright
```

Example — scaled cells with RGB colors in horizontal layout:

```
cargo run --bin knitui -- --scale 2 --color-mode dark-rgb --layout horizontal
```

## Architecture

Loom is a Cargo workspace built around one portable core crate, three frontend
crates that drive it (terminal, web, FFI), and one crate per game:

```
Cargo.toml                  — workspace root + loom binary
src/
└── main.rs                 — game selector menu → dispatches to game crates

crates/
├── loom-engine/             — portable core, no I/O (lib: loom_engine)
│   └── src/
│       ├── game.rs          — Game, GameEngine, GameConfig traits; MenuItem
│       ├── shell/
│       │   ├── mod.rs       — Shell<G>: the generic menu/campaign/endless/
│       │   │                  options/playing state machine every game
│       │   │                  drives through, single-step (handle_key/
│       │   │                  tick/render), no blocking loop of its own
│       │   └── chrome.rs    — generic Surface-only UI screens (main menu,
│       │                      campaign select, options, blessing selection, ...)
│       ├── render.rs        — Color/Style/Cell/CellGrid/Surface — the
│       │                      universal cell-grid rendering model every
│       │                      frontend blits from
│       ├── input.rs         — Key/KeyEvent — portable, crossterm-free
│       ├── campaign.rs      — CampaignEntry trait + CampaignSaves<E>
│       ├── endless.rs       — EndlessHighScore generic persistence
│       ├── settings.rs      — UserSettings persistence
│       ├── storage.rs       — Storage trait (native-storage feature: FsStorage)
│       ├── blessings.rs     — Blessing type + is_unlocked()
│       ├── anim.rs          — AnimOverlay (merge-dissolve/rise animations)
│       └── ad_content.rs    — pseudo-ad quotes
│
├── loom-engine-term/         — crossterm-backed Surface impl (TermSurface) +
│                               terminal init/restore; the native frontend
├── loom-engine-web/          — wasm-bindgen Surface impl (WasmSurface) +
│                               browser key mapping; the web frontend
├── loom-engine-capi/         — C ABI: create/handle_key/tick/render/
│                               should_quit/save_on_exit/destroy over JSON;
│                               generated header at include/loom.h
├── loom-py/                  — PyO3 bindings: a real, memory-safe LoomGame
│                               Python class, reusing loom-engine-capi's
│                               type-erasure layer directly
│
├── loom-knit/                — Knit game (lib: knitui)
│   └── src/
│       ├── engine.rs         — KnitEngine: board + yarn + held_spools + processing
│       ├── game_board.rs     — BoardEntity, random generation, selectability
│       ├── yarn.rs           — Yarn, Stitch, lock/key mechanics
│       ├── spool.rs          — Spool struct
│       ├── solvability.rs    — 4 board validation checks
│       ├── renderer/         — knit-specific Surface-only rendering
│       ├── tui.rs            — thin: CLI parsing + crossterm event loop,
│       │                       driving Shell<KnitGame>
│       ├── web.rs            — wasm32-only gameplay-loop entry point
│       ├── game.rs           — impl Game for KnitGame + the GameEngine
│       │                       trait adapter wrapping KnitEngine
│       └── ...                — config, campaign_levels, preset, glyphs, etc.
│
├── loom-match3/               — Match-3 game (lib: m3tui), same shape as loom-knit
├── loom-merge2/                — Merge-2 game (lib: m2tui), same shape
└── loom-picross/               — Picross/nonogram game (lib: pictui), same shape
```

### Core traits (loom-engine)

- **`Game`** — identity, config, campaign/endless level data, presets, help
  text, `main_menu_items()` (which of Quick/Custom/Campaign/Endless/
  Options/Quit a game actually offers — not every game has all five)
- **`GameEngine`** — handle_key, tick, render, status, score, `as_any()`
  (for the rare game whose campaign state needs live-engine sync back —
  see `Game::sync_campaign_entry`)
- **`GameConfig`** — board_width/height, color_count, scale, color_mode,
  custom-game field editing

Each game crate implements these traits via a `GameEngine` trait adapter
(in `game.rs`) that wraps its own already-working concrete engine — the
adapter is the only place a game's specific quirks (win conditions,
per-mode key bindings, live vs. rebuilt-per-attempt state) get reconciled
against the generic `Shell<G>` shape. Every frontend (`tui.rs`, `web.rs`,
the C ABI, the Python bindings) drives the *same* `Shell<G>` in single
steps; none of them own game logic themselves.

### Knit data flow

```
Config → KnitEngine::new()
  → select_palette()
  → GameBoard::make_random()            (retry loop until is_solvable)
      → count_spools()                  (color → spools × spool_capacity)
          → Yarn::make_from_color_counter()
```

### Knit solvability checks (run on every generated board)

1. **Count balance** — yarn stitches per color == board spools × spool_capacity (including conveyor outputs)
2. **Spool reachability** — BFS from top row; every spool reachable via void-bordering cascade
3. **Active headroom** — distinct colors on board ≤ spool limit
4. **Key-lock pairing** — every locked yarn stitch has a matching Key spool

Boards that fail any check are regenerated (up to 100 retries).

## Development

```bash
cargo run                       # game selector
cargo run --bin knitui          # play knit directly
cargo run --bin knitui-ni       # non-interactive knit driver
cargo test --workspace          # all tests across all crates (native)
cargo build --release           # build all binaries

# Web (per game crate, from that crate's directory):
wasm-bindgen-cli ...            # see crates/<game>/web/ for the exact build steps

# C ABI:
cargo build -p loom-engine-capi --release
# see crates/loom-engine-capi/README.md for header regeneration + the C smoke test

# Python bindings:
cd crates/loom-py && maturin build --release
# see crates/loom-py/README.md
```

**Dependencies**: `crossterm 0.27`, `rand 0.9.2`, `clap 4`, `serde 1`, `serde_json 1`, `dirs 5`, `bitflags 2`; `wasm-bindgen` (web); `pyo3` (Python bindings)

## TODO

- [ ] Puzzle editor / non-random board generation
- [ ] Real in-browser visual verification (the terminal side has a pty+pyte capture technique; the web side has a headless Node smoke test executing the real `.wasm`, but no pixel-level check yet)
- [ ] Polished C++ RAII wrapper / Go `cgo` package for the C ABI (deferred until a real consumer wants one; the generated header + docs are the whole surface today)

See [PLAN.md](PLAN.md) for design history and migration notes.
