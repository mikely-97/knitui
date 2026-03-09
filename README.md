# knitui

A terminal-based puzzle game inspired by mobile yarn/knitting games. Match colored threads on the board against a scrolling yarn queue to clear the board.

Two binaries:
- **knitui** — interactive TUI (crossterm)
- **knitui-ni** — non-interactive CLI driver (JSON in/out, for scripting and AI agents)

Clone and run:

```
cargo run --bin knitui
```

Pass `--help` to see all options:

```
cargo run --bin knitui -- --help
```

## How to Play

The screen shows three sections (top-to-bottom in vertical layout, left-to-right in horizontal):

1. **Yarn queue** — rows of colored patches (`▦`) showing upcoming knitting work, split into columns. Locked patches show as `▣` and block their column until cleared with a key.
2. **Active threads** — threads you've selected from the board, waiting to be processed
3. **Game board** — a bordered grid of cells to clear. The selected cell is marked with `[` `]` bracket markers.

**Goal**: Clear all threads from the board by picking them up and processing them against the yarn queue. Each thread must be processed `--knit-volume` times (default: 3) to complete and be discarded.

**Controls**:

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor across the board |
| Enter | Pick up the thread under the cursor |
| H | Show help overlay |
| Z | Use Scissors bonus |
| X | Use Tweezers bonus |
| C | Use Balloons bonus |
| R | Restart (on game over) |
| Esc | Cancel active bonus / Quit |

A key bar at the bottom of the screen shows all available controls and current bonus counts.

### Selectability rule

Only **exposed** threads can be picked up:
- The **top row** is always selectable.
- Any other thread is selectable only if it **borders a `Void` cell** horizontally or vertically (not diagonally).

Cells become Void when their thread is picked up. Clearing a thread exposes its neighbors, cascading inward from the top.

### Board entities

| Glyph | Entity | Behavior |
|-------|--------|----------|
| `T` (colored) | Thread | Normal selectable thread |
| `K` (colored) | Key thread | Thread that carries a key; displayed `k` in active list until key is spent |
| `X` | Obstacle | Impassable; never becomes Void |
| ` ` | Void | Empty; makes orthogonal neighbors selectable |
| `G` (colored) | Generator | Produces threads in its adjacent output cell up to `--generator-capacity` times, then becomes `#` |
| `#` | Depleted generator | Acts like an obstacle |

### Lock / Key mechanic

A locked yarn patch (`▣`) blocks its entire column — nothing behind it can be processed until the lock is cleared. To clear it, pick up the matching **Key thread** (`K`) from the board. The key is consumed on contact and the lock is removed as a normal knit stage.

### Bonuses

Bonuses are optional power-ups activated by hotkeys. Their counts are set at launch via CLI flags (default: 0). The bonus display shows icons, hotkeys, and remaining counts below the board (vertical layout) or to the right (horizontal layout). Bonuses with 0 remaining are greyed out.

| Bonus | Key | Icon | Effect |
|-------|-----|------|--------|
| **Scissors** | Z | ✂ | Instantly auto-knits the least-progressed active thread by deep-scanning ALL patches in the yarn (not just the front). Ignores queue order. |
| **Tweezers** | X | ⊹ | Enter free-cursor mode: move to any cell and pick up any thread regardless of selectability. Cursor shows `{ }` brackets. Press Esc to cancel without consuming. |
| **Balloons** | C | ⊛ | Lifts the front N patches from each yarn column into separate pseudo-columns, exposing the patches behind them. Pseudo-columns are also matchable. |

Guards: only one bonus can be active at a time. Scissors requires active threads. Balloons requires previous balloon columns to be fully consumed first.

### Background processing

Active threads are processed automatically in the background (one step every 150 ms). You can continue moving and picking up threads while processing runs. Each thread is matched against the yarn one at a time so you can see what matches and what doesn't.

## Non-interactive mode (knitui-ni)

`knitui-ni` drives the same game engine via CLI commands. Game state persists as JSON files in `~/.local/share/knitui/`.

### Create a game

```bash
cargo run --bin knitui-ni                    # default options
cargo run --bin knitui-ni -- --board-height 3 --board-width 4  # custom
```

Output: JSON with `"status": "ok"`, `"game": "<8-char hash>"`, and full `"state"`.

### Execute commands

```bash
cargo run --bin knitui-ni -- --game <HASH> move <up|down|left|right>
cargo run --bin knitui-ni -- --game <HASH> pick
cargo run --bin knitui-ni -- --game <HASH> process
cargo run --bin knitui-ni -- --game <HASH> scissors
cargo run --bin knitui-ni -- --game <HASH> tweezers
cargo run --bin knitui-ni -- --game <HASH> balloons
```

Success response:
```json
{"status":"ok","game":"abc123xy","won":false,"game_status":"playing","state":{...}}
```

Error response (to stderr, exit code 1):
```json
{"status":"error","code":"not_selectable","message":"thread is not exposed"}
```

Error codes: `out_of_bounds`, `not_selectable`, `not_a_thread`, `active_full`, `bonus_failed`, `load_failed`, `save_failed`, `no_command`.

## Configuration

All parameters are settable via CLI flags (both binaries). Defaults:

| Flag | Default | Description |
|------|---------|-------------|
| `--board-height` | 6 | Grid rows |
| `--board-width` | 6 | Grid columns |
| `--color-number` | 6 | Distinct colors used |
| `--color-mode` | `dark` | Palette: `dark` \| `bright` \| `colorblind` \| `dark-rgb` \| `bright-rgb` \| `colorblind-rgb` |
| `--active-threads-limit` | 7 | Max threads held at once |
| `--knit-volume` | 3 | Times each thread must be processed |
| `--yarn-lines` | 4 | Yarn columns |
| `--obstacle-percentage` | 5 | % chance each cell is an obstacle |
| `--visible-patches` | 6 | Yarn rows shown on screen |
| `--generator-capacity` | 3 | Threads each generator produces |
| `--layout` | `auto` | Layout: `auto` \| `horizontal` \| `vertical` |
| `--scale` | 1 | Cell scale factor (1–3): render each entity as N×N characters |
| `--scissors` | 0 | Starting scissors bonus count |
| `--tweezers` | 0 | Starting tweezers bonus count |
| `--balloons` | 0 | Starting balloons bonus count |
| `--scissors-threads` | 1 | Threads processed per scissors use |
| `--balloon-count` | 2 | Patches lifted per yarn column per balloons use |

The `-rgb` color modes use 24-bit true color escapes, which are immune to terminal theme overrides (useful for kitty, alacritty, etc. that remap ANSI palette slots).

`--layout auto` picks vertical if the terminal is tall enough, otherwise horizontal. At `--scale 2` or `3`, each cell is rendered as a 2×2 or 3×3 block inside a box-drawing grid.

Example — play with bonuses:

```
cargo run --bin knitui -- --scissors 3 --tweezers 2 --balloons 2
```

Example — a bigger, harder board:

```
cargo run --bin knitui -- --board-height 8 --board-width 10 --knit-volume 5 --color-mode bright
```

Example — scaled cells with RGB colors in horizontal layout:

```
cargo run --bin knitui -- --scale 2 --color-mode dark-rgb --layout horizontal
```

## Architecture

```
src/
├── main.rs           — TUI binary: rendering (vertical/horizontal layout,
│                       scaled cells, box-drawn grid, bracket cursor), input
├── bin/
│   └── knitui_ni.rs  — NI binary: CLI arg parsing, JSON I/O, XDG persistence
├── lib.rs            — module declarations
├── engine.rs         — GameEngine: owns all game state, action methods,
│                       JSON snapshot serialisation, generator helpers
├── config.rs         — CLI config (clap)
├── game_board.rs     — board generation, is_selectable, count_knits
├── board_entity.rs   — BoardEntity enum: Thread | KeyThread | Obstacle | Void
│                       | Generator(GeneratorData) | DepletedGenerator
│                       Direction enum and GeneratorData struct
├── yarn.rs           — Patch (with locked flag), Yarn, process_one with lock logic
├── active_threads.rs — Thread: color, status, has_key
├── color_counter.rs  — ColorCounter: HashMap of Color → count, shuffled queue
├── color_serde.rs    — serialize/deserialize crossterm::Color as strings
├── palette.rs        — color palettes: Dark | Bright | Colorblind, each with
│                       ANSI and RGB variants (8 colors each)
└── solvability.rs    — board solvability checks (count balance, BFS reachability,
                        active headroom, key-lock pairing)
```

### Key data flow

```
Config
  → GameEngine::new()
      → select_palette()
      → GameBoard::make_random()               (retry loop until is_solvable)
          → game_board.count_knits()           (color → threads × knit_volume,
                                                includes generator queues)
              → Yarn::make_from_color_counter() (shuffled patches across columns)

TUI (main.rs):   GameEngine + crossterm rendering + ProcessingState animation
NI  (knitui_ni): GameEngine + JSON snapshot ↔ ~/.local/share/knitui/<hash>.json
```

### Solvability checks (run on every generated board)

1. **Count balance** — yarn patches per color == board threads × knit_volume (including all generator outputs)
2. **Thread reachability** — BFS from top row, simulating selections; every thread must be reachable via the void-bordering cascade
3. **Active headroom** — distinct colors on board ≤ active thread limit
4. **Key-lock pairing** — every locked yarn patch has a matching Key thread on the board

Boards that fail any check are regenerated (up to 100 retries).

## Development

```bash
cargo run --bin knitui     # play the interactive game
cargo run --bin knitui-ni  # create a non-interactive game
cargo test                 # 145 tests (114 unit + 31 integration)
cargo build --release      # build both binaries
```

**Dependencies**: `crossterm 0.27`, `rand 0.9.2`, `clap 4`, `serde 1`, `serde_json 1`, `dirs 5`

## TODO

- [ ] Puzzle editor / non-random board generation (needed to actually place generators and locks)
- [x] Bonuses: scissors, tweezers, balloons (hotkey-activated, configurable counts)
- [ ] In-game pseudo-ads between rounds

See [PLAN.md](PLAN.md) for design notes on remaining features.
