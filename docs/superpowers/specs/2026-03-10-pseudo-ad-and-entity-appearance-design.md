# Pseudo-Ad Mechanic & Entity Appearance at Scale

**Date**: 2026-03-10
**Scope**: Features 1 (pseudo-ad) and 5 (entity appearance) from the roadmap

---

## 1. Renderer Extraction (prerequisite refactor)

Split `main.rs` into three concerns:

| File | Responsibility |
|------|----------------|
| `src/main.rs` | Event loop, `TuiState` enum, crossterm setup/teardown, top-level `draw()` |
| `src/renderer.rs` | All rendering functions: yarn, board, active threads, overlays, layout detection, spacing constants |
| `src/glyphs.rs` | Entity glyph lookup: `(BoardEntity, scale) → Vec<&str>` rows |

`lib.rs` gains `pub mod renderer;` and `pub mod glyphs;`.

---

## 2. Pseudo-Ad Mechanic

### Trigger conditions

- **Game-over rescue**: When `GameStatus::Stuck`, the game-over overlay offers "Watch an ad? [A]" alongside Restart [R] and Quit [Esc].
- **Player-initiated**: A hotkey (A) is available during normal play. Shown in the key bar.

### Reward

Always **+1 scissors**. No choice, no randomness.

### Limits

- **Free play**: Unlimited. The 15-second wait is the only cost.
- **Campaign** (future): Will have per-map or per-campaign limits, configured externally. The engine accepts an optional `ad_limit: Option<u16>` field (default `None` = unlimited). Not implemented now, just reserved.

### Ad content

- Loaded from an external text file. Default path: `~/.config/knitui/ads.txt`.
- Format: one quote per line. Blank lines and `#` comment lines are skipped.
- If file is missing or empty, show a single hardcoded fallback: `"You are watching a fake ad. Touch grass."`
- A random line is selected each time.
- The file path can be overridden via `--ad-file <path>` CLI flag.

### Timer and UX

- **Full-screen takeover**: Clears the game screen entirely. Centered box with:
  - Header: `✂ FREE SCISSORS ✂` (or similar thematic line)
  - The quote text, word-wrapped to fit
  - A progress bar: `░░░░░░░░░░████████ 73%` style, filling over 15 seconds
  - A countdown: `[12s remaining]`
  - Once timer hits 0: `[ Press ESC to collect your reward ]`
- **Straight timer**: No tricks. ESC does nothing until 15 seconds are up. Before that, show `"Ad still playing..."` briefly if ESC is pressed.
- Box drawn with `╔═╗║╚═╝` double-line box characters.

### Engine changes

- Add `watch_ad(&mut self)` method to `GameEngine` — increments `bonuses.scissors` by 1 and increments `ads_used` by 1.
- Add `ad_limit: Option<u16>` (default `None` = unlimited) and `ads_used: u16` (default `0`) fields to `GameEngine`.
- Add `can_watch_ad(&self) -> bool` — returns true if `ad_limit` is `None` or `ads_used < ad_limit`.
- Update `GameStateSnapshot` serialization to include `ad_limit` and `ads_used` for save/load round-tripping.
- No `--ad-limit` CLI flag for now — this will be set programmatically by the campaign system when it lands.

### TUI changes

- New `TuiState` variant: `WatchingAd { started_at: Instant, quote: String }`.
- Hotkey `A` in `Playing` state → if `can_watch_ad()`, transition to `WatchingAd`.
- Hotkey `A` in `GameOver(Stuck)` state → same.
- In `WatchingAd`: render the full-screen ad, tick the timer. After 15s, ESC → `watch_ad()`, transition back to `Playing` (or back to `GameOver` which re-evaluates status — the new scissors may un-stuck the game).
- After granting scissors from game-over: re-check `status()`. If no longer `Stuck`, return to `Playing`.

### NI binary changes

- New command: `ad` — returns `{"status":"ok","quote":"...","bonus_granted":"scissors"}` and calls `watch_ad()`. No timer (timer is a UI concept). Respects `ad_limit`.
- Error code when limit exceeded: `ad_limit_reached` with message `"ad limit reached for this game"`.

### Config changes

- New CLI flags: `--ad-file <path>` (default: `~/.config/knitui/ads.txt`).
- `Config` struct gains `ad_file: Option<PathBuf>`. When `None`, the default path `~/.config/knitui/ads.txt` is used. The fallback quote covers both "file missing" and "file empty" cases.

---

## 3. Entity Appearance at Scale 2+

### Glyph table (`src/glyphs.rs`)

A function `entity_glyph(entity: &BoardEntity, scale: u16) -> Vec<&'static str>` returns the rows for rendering. At scale 1, returns the existing single-character representations.

### Scale 2 patterns (4 chars wide × 2 rows)

| Entity | Row 1 | Row 2 | Notes |
|--------|-------|-------|-------|
| Spool | `╲╱╲╱` | `╱╲╱╲` | Cross-stitch, colored with spool color |
| KeySpool | `╲╱⚷╱` | `╱╲╲╱` | Cross-stitch with key symbol embedded |
| Obstacle | `░░░░` | `░░░░` | Light shade, grey/dim |
| Conveyor(→) | `⊞──▸` | `⊞──▸` | Source box + arrow in direction |
| Conveyor(←) | `◂──⊞` | `◂──⊞` | Mirrored |
| Conveyor(↓) | `·⊞⊞·` | `·▾▾·` | Source on top, arrow below, centered in 4-wide cell |
| Conveyor(↑) | `·▴▴·` | `·⊞⊞·` | Arrow on top, source below, centered in 4-wide cell |
| EmptyConveyor | `⊞──·` | `⊞──·` | Source box + dots (exhausted, no arrow) |
| Void | `    ` | `    ` | Empty space |

### Scale 3 patterns (6 chars wide × 3 rows)

Tile/extend the scale 2 patterns to fill the 6-wide × 3-row grid:

| Entity | Row 1 | Row 2 | Row 3 |
|--------|-------|-------|-------|
| Spool | `╲╱╲╱╲╱` | `╱╲╱╲╱╲` | `╲╱╲╱╲╱` |
| KeySpool | `╲╱╲╱╲╱` | `╱╲⚷╲╱╲` | `╲╱╲╱╲╱` |
| Obstacle | `░░░░░░` | `░░░░░░` | `░░░░░░` |
| Conveyor(→) | `⊞────▸` | `⊞────▸` | `⊞────▸` |
| Conveyor(←) | `◂────⊞` | `◂────⊞` | `◂────⊞` |
| Conveyor(↓) | `··⊞⊞··` | `··╏╏··` | `··▾▾··` |
| Conveyor(↑) | `··▴▴··` | `··╏╏··` | `··⊞⊞··` |
| EmptyConveyor | `⊞──··` | `⊞──··` | `⊞──··` |
| Void | `      ` | `      ` | `      ` |

### Coloring

- Spool and KeySpool glyphs are rendered in the spool's `Color`, same as scale 1.
- Conveyor glyphs use the first color in the conveyor's queue (or dim/grey if depleted).
- Obstacle glyphs use dim grey (`Color::DarkGrey`).

### Cursor brackets at scale 2+

At scale > 1, the `[` `]` cursor markers span the full height of the cell. For scale 2, `[` is drawn on both rows of the left edge, `]` on both rows of the right edge. Scale 3: all 3 rows. This replaces the first/last character column of the glyph area (no extra width needed).

### Rendering integration

`renderer.rs` calls `entity_glyph()` when drawing board cells at scale > 1. At scale 1, the existing single-char rendering path is unchanged.

### Key bar and help overlay updates

- Key bar gains `A:Ad` entry showing the ad hotkey.
- Help overlay gains a line explaining the ad mechanic: `A — Watch a fake ad for +1 scissors`.

---

## 4. Files changed summary

| File | Change |
|------|--------|
| `src/main.rs` | Extract rendering code out; add `WatchingAd` state, ad hotkey handling, ad file loading |
| `src/renderer.rs` | **New** — all rendering functions moved from `main.rs`, plus ad overlay renderer |
| `src/glyphs.rs` | **New** — entity glyph lookup table |
| `src/engine.rs` | Add `watch_ad()`, `can_watch_ad()`, `ad_limit`, `ads_used` fields |
| `src/ad_content.rs` | **New** — ad quote file parser with fallback |
| `src/config.rs` | Add `--ad-file` flag |
| `src/bin/knitui_ni.rs` | Add `ad` command |
| `src/lib.rs` | Add `pub mod renderer; pub mod glyphs; pub mod ad_content;` |

---

## 5. Renderer extraction detail

Functions moving from `main.rs` to `renderer.rs`:
- `detect_layout()`, `FlankSide` enum, spacing constants (`YARN_HGAP`, `YARN_VGAP`, `THREAD_GAP`, `COMP_GAP`)
- `render_yarn()`, `render_balloon_columns()`, `render_balloon_flank()`
- `render_board()` (grid drawing)
- `render_held_spools()`
- `render_help_overlay()`, `render_game_over_overlay()`
- New: `render_ad_overlay()`

Staying in `main.rs`:
- `TuiState` enum and state transitions
- `main()` function: crossterm init, event loop, `draw()` dispatch
- `draw()` top-level function that calls into `renderer::*`

---

## 6. Testing

- `engine::watch_ad()` — grants scissors, increments `ads_used`
- `engine::can_watch_ad()` — returns true when unlimited, false when limit reached
- `engine::watch_ad()` with limit — errors when `ads_used >= ad_limit`
- `glyphs::entity_glyph()` — returns correct dimensions for each scale (1→1×1, 2→2×4, 3→3×6)
- `glyphs::entity_glyph()` — all `BoardEntity` variants produce valid output at all scales
- Ad file parser — handles missing file (fallback), empty file (fallback), comment lines, blank lines
- Snapshot round-trip — `ad_limit` and `ads_used` survive serialize/deserialize

---

## 7. Out of scope

- Campaign ad limits (field reserved, logic deferred to campaign feature)
- Main menu (separate feature)
- Endless mode (separate feature)
- Multi-game umbrella (separate feature)
- Scale 1 appearance changes (fine as-is)
