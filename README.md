# knitui

A terminal-based puzzle game inspired by mobile yarn/knitting games. Match colored threads on the board against a scrolling yarn queue to clear the board.

Clone and run:

```
cargo run
```

Pass `--help` to see all options:

```
cargo run -- --help
```

## How to Play

The screen is divided into three sections from top to bottom:

1. **Yarn queue** — rows of colored patches (`▦`) showing upcoming knitting work, split into columns. Locked patches show as `▣` and block their column until cleared with a key.
2. **Active threads** — threads you've selected from the board, waiting to be processed
3. **Game board** — the grid of cells to clear

**Goal**: Clear all threads from the board by picking them up and processing them against the yarn queue. Each thread must be processed `--knit-volume` times (default: 3) to complete and be discarded.

**Controls**:

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor across the board |
| Enter | Pick up the thread under the cursor |
| Backspace | Process active threads (animated, one at a time) |
| Esc | Quit |

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

A locked yarn patch (`▣`) blocks its entire column — nothing behind it can be processed until the lock is cleared. To clear it, pick up the matching **Key thread** (`K`) from the board and press Backspace. The key is consumed on contact and the lock is removed as a normal knit stage.

### Processing animation

Pressing Backspace queues all active threads for sequential processing. Each thread is processed against the yarn one at a time (150 ms per step) so you can see what matches and what doesn't. Input is paused during the animation.

## Configuration

All parameters are settable via CLI flags. Defaults:

| Flag | Default | Description |
|------|---------|-------------|
| `--board-height` | 6 | Grid rows |
| `--board-width` | 6 | Grid columns |
| `--color-number` | 6 | Distinct colors used |
| `--color-mode` | `dark` | Palette: `dark` \| `bright` \| `colorblind` |
| `--active-threads-limit` | 7 | Max threads held at once |
| `--knit-volume` | 3 | Times each thread must be processed |
| `--yarn-lines` | 4 | Yarn columns |
| `--obstacle-percentage` | 5 | % chance each cell is an obstacle |
| `--visible-patches` | 6 | Yarn rows shown on screen |
| `--generator-capacity` | 3 | Threads each generator produces |

Example — a bigger, harder board:

```
cargo run -- --board-height 8 --board-width 10 --knit-volume 5 --color-mode bright
```

## Architecture

```
src/
├── main.rs           — game loop, rendering, input, animation state machine
├── lib.rs            — module declarations
├── config.rs         — CLI config (clap)
├── game_board.rs     — board generation, is_selectable, count_knits
├── board_entity.rs   — BoardEntity enum: Thread | KeyThread | Obstacle | Void
│                       | Generator(GeneratorData) | DepletedGenerator
│                       Direction enum and GeneratorData struct
├── yarn.rs           — Patch (with locked flag), Yarn, process_one with lock logic
├── active_threads.rs — Thread: color, status, has_key
├── color_counter.rs  — ColorCounter: HashMap of Color → count, shuffled queue
├── palette.rs        — color palettes: Dark | Bright | Colorblind (8 colors each)
└── solvability.rs    — board solvability checks (count balance, BFS reachability,
                        active headroom, key-lock pairing)
```

### Key data flow

```
Config::parse()
    → select_palette()
        → GameBoard::make_random()               (retry loop until is_solvable)
            → game_board.count_knits()           (color → threads × knit_volume,
                                                  includes generator queues)
                → Yarn::make_from_color_counter() (shuffled patches across columns)
```

### Solvability checks (run on every generated board)

1. **Count balance** — yarn patches per color == board threads × knit_volume (including all generator outputs)
2. **Thread reachability** — BFS from top row, simulating selections; every thread must be reachable via the void-bordering cascade
3. **Active headroom** — distinct colors on board ≤ active thread limit
4. **Key-lock pairing** — every locked yarn patch has a matching Key thread on the board

Boards that fail any check are regenerated (up to 100 retries).

## Development

```bash
cargo run          # play the game
cargo test         # 76 tests (60 unit + 16 integration)
cargo build        # build binary
```

**Dependencies**: `crossterm 0.27`, `rand 0.9.2`, `clap 4`

## TODO

- [ ] Horizontal layout option
- [ ] Puzzle editor / non-random board generation (needed to actually place generators and locks)
- [ ] Bonuses and power-ups (wildcard patches, double-knit, etc.)
- [ ] In-game pseudo-ads between rounds

See [PLAN.md](PLAN.md) for design notes on remaining features.
