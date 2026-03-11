# Loom

A multi-game terminal puzzle engine built with Rust and crossterm. Currently ships two playable games — **Knit** (spool-knitting puzzle) and **Match-3** (classic gem-matching) — plus a **Merge-2** stub, all selectable from a single binary.

Binaries:
- **loom** — game selector menu → launches Knit, Match-3, or Merge-2
- **knitui** — launch Knit directly (interactive TUI)
- **knitui-ni** — non-interactive CLI driver for Knit (JSON in/out, for scripting and AI agents)
- **knitui-solvcheck** — independent solvability checker for Knit (reads NDJSON, runs DFS verification)

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

Loom is a Cargo workspace with a shared engine crate and per-game crates:

```
Cargo.toml                  — workspace root + loom binary
src/
└── main.rs                 — game selector menu → dispatches to game crates

crates/
├── loom-engine/            — shared framework (lib: loom_engine)
│   └── src/
│       ├── game.rs         — Game, GameEngine, GameConfig traits
│       ├── board.rs        — generic Board<C> 2D grid
│       ├── direction.rs    — Direction enum + offset()
│       ├── palette.rs      — 6 palettes (Dark/Bright/Colorblind × ANSI/RGB)
│       ├── color_serde.rs  — crossterm::Color serde helpers
│       ├── settings.rs     — UserSettings persistence
│       ├── campaign.rs     — CampaignSaves<E> generic campaign framework
│       ├── endless.rs      — EndlessHighScore generic persistence
│       ├── renderer.rs     — layout detection, box drawing, menu chrome
│       ├── glyphs.rs       — shared glyph utilities
│       ├── bonus.rs        — BonusInventory generic framework
│       └── ad_content.rs   — pseudo-ad quotes
│
├── loom-knit/              — Knit game (lib: knitui)
│   └── src/
│       ├── engine.rs       — KnitEngine: board + yarn + held_spools + processing
│       ├── game_board.rs   — BoardEntity, random generation, selectability
│       ├── yarn.rs         — Yarn, Stitch, lock/key mechanics
│       ├── spool.rs        — Spool struct
│       ├── solvability.rs  — 4 board validation checks
│       ├── renderer.rs     — knit-specific TUI rendering
│       ├── tui.rs          — knit TUI event loop + menus
│       ├── game.rs         — impl Game for KnitGame
│       └── ...             — config, campaign_levels, preset, glyphs, etc.
│
├── loom-match3/            — Match-3 game (lib: m3tui)
│   └── src/
│       ├── engine.rs       — M3Engine: phase machine (swap → cascade → refill)
│       ├── board.rs        — Cell, CellContent, SpecialPiece, TileModifier
│       ├── matches.rs      — match detection + shape classification
│       ├── renderer.rs     — m3-specific TUI rendering
│       ├── tui.rs          — match-3 TUI event loop + menus
│       ├── game.rs         — impl Game for M3Game
│       └── ...             — bonuses, campaign_levels, config, etc.
│
└── loom-merge2/            — Merge-2 game stub (lib: m2tui)
    └── src/
        └── game.rs         — impl Game for M2Game (placeholder)
```

### Core traits (loom-engine)

- **`Game`** — identity, config, campaign/endless level data, presets, help text
- **`GameEngine`** — handle_key, tick, render, status, score
- **`GameConfig`** — board_width/height, color_count, scale, color_mode

Each game crate implements these traits, and the shared TUI framework in each game's `tui.rs` drives the event loop, menus, campaign/endless persistence, and rendering.

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
cargo test --workspace          # all tests across all crates
cargo build --release           # build all binaries
```

**Dependencies**: `crossterm 0.27`, `rand 0.9.2`, `clap 4`, `serde 1`, `serde_json 1`, `dirs 5`

## TODO

- [ ] Wire up `GameEngine` trait implementations (currently `create_engine()` is stubbed)
- [ ] Merge-2 game implementation
- [ ] Puzzle editor / non-random board generation
- [ ] Further unify shared code (game-configurable palettes and color modes)

See [PLAN.md](PLAN.md) for design history and migration notes.
