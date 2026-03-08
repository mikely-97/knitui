# knitui

A terminal-based puzzle game inspired by mobile yarn/knitting games. Match colored threads on the board against a scrolling yarn queue to clear the board.

Clone and run:

```
cargo run
```

## How to Play

The screen is divided into three sections from top to bottom:

1. **Yarn queue** — rows of colored patches (`▦`) showing upcoming knitting work, split into columns
2. **Active threads** — the threads you've currently selected from the board
3. **Game board** — the grid of colored thread cells to clear

**Goal**: Clear all threads from the board by selecting them and processing them against the yarn queue. Each thread must be processed `knit_volume` times (default: 3) to be completed.

**Controls**:

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor across the board |
| Enter | Pick up the thread under the cursor (adds to active threads) |
| Backspace | Process all active threads against the yarn queue |
| Esc | Quit |

**Picking up a thread** removes it from the board and adds it to your active list (up to 7 at a time). **Processing** tries to find a matching colored patch at the end of each yarn column for each active thread. If a match is found, the patch is consumed and the thread advances one stage. Threads that complete all stages (status > `knit_volume`) are discarded.

## Architecture

```
src/
├── main.rs           — game loop, terminal rendering, keyboard input
├── lib.rs            — module declarations
├── game_board.rs     — board generation (random grid with threads and obstacles)
├── board_entity.rs   — enum for board cells: Thread(Color) | Obstacle | Void
├── yarn.rs           — yarn queue: Patch, Yarn, process_one, process_sequence
├── active_threads.rs — Thread struct tracking color and knitting progress (status)
├── color_counter.rs  — ColorCounter: HashMap of Color → count, shuffled queue
└── palette.rs        — color palettes: Dark | Bright | Colorblind (8 colors each)
```

### Key data flow

```
select_palette()
    → GameBoard::make_random()
        → game_board.count_knits()          (ColorCounter: color → n × knit_volume)
            → Yarn::make_from_color_counter()
                → shuffled_queue distributed across yarn columns
```

### Hardcoded configuration (in `main.rs`)

| Constant | Default | Description |
|----------|---------|-------------|
| `board_height` | 6 | Grid rows |
| `board_width` | 6 | Grid columns |
| `color_number` | 6 | Number of distinct colors used |
| `color_mode` | Dark | Palette: Dark / Bright / Colorblind |
| `active_threads_limit` | 7 | Max threads held at once |
| `knit_volume` | 3 | Times each thread must be processed |
| `yarn_lines` | 4 | Number of yarn columns |
| `obstacle_percentage` | 5 | % chance each cell is an obstacle |
| `visible_patches` | 6 | How many yarn rows are shown |

## Development

```bash
cargo run          # play the game
cargo test         # run all tests (39 unit + 16 integration)
cargo build        # build binary
```

**Dependencies**: `crossterm 0.27` (terminal I/O), `rand 0.9.2`

Tests live alongside source in `#[cfg(test)]` blocks and in `tests/` (edge cases and game flow integration tests).

## TODO

- [ ] Unhardcoded config (CLI args or config file)
- [ ] Async/animated processing of knits
- [ ] Horizontal layout option
- [ ] Movement limits refinement (cursor stays within board only)
- [ ] More complex boards: lock/key cells, generator cells
- [ ] Solvability checks on board generation
- [ ] Bonuses and power-ups

See [PLAN.md](PLAN.md) for implementation details on each item.
