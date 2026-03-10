# Main Menu Design

**Date**: 2026-03-10
**Scope**: Main menu, presets, custom game configuration screen

---

## 1. Overview

Add a main menu as the entry point for the TUI. Currently the game launches straight into gameplay. The menu provides access to Quick Game, Custom Game, and placeholder entries for Campaign and Endless modes.

---

## 2. TuiState Changes

Add two new variants to the `TuiState` enum:

```
MainMenu { selected: usize }
CustomGame { preset_idx: usize, editing_field: Option<usize>, config: Config }
```

**Initial state:** `MainMenu { selected: 0 }` (was `Playing`).

### Menu items (indexed)

| Index | Label | Action |
|-------|-------|--------|
| 0 | Quick Game | Create engine from CLI defaults, transition to `Playing` |
| 1 | Custom Game | Transition to `CustomGame` with Medium preset |
| 2 | Campaign | Show "Coming soon!" flash, stay in `MainMenu` |
| 3 | Endless | Show "Coming soon!" flash, stay in `MainMenu` |
| 4 | Options | Transition to `Options` screen (scale, color mode) |
| 5 | Quit | Exit program |

### Navigation

- `↑`/`↓`: move highlight
- `Enter`: select item
- `Esc`/`Q`: quit (same as selecting Quit)

### "Coming soon" flash

When Campaign or Endless is selected, set a transient message string on the menu state. Render it below the menu items. Clear it on next keypress. No separate overlay needed.

---

## 3. Presets & Custom Game

### GamePreset struct

A `GamePreset` holds the player-facing configuration fields with a name:

```rust
pub struct GamePreset {
    pub name: &'static str,
    pub board_height: u16,
    pub board_width: u16,
    pub color_number: u16,
    pub obstacle_percentage: u16,
    pub generator_percentage: u16,
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
}
```

### Named presets

| Preset | Board | Colors | Obstacles | Generators | Bonuses |
|--------|-------|--------|-----------|------------|---------|
| Small | 4×4 | 4 | 0% | 0% | 0/0/0 |
| Medium | 6×6 | 6 | 5% | 5% | 0/0/0 |
| Large | 8×8 | 8 | 10% | 10% | 1/1/1 |
| Chaos | 10×10 | 8 | 20% | 15% | 2/2/2 |

### to_config() conversion

`GamePreset::to_config(&self, base: &Config) -> Config` — creates a `Config` by applying preset values on top of the CLI-parsed `Config`. Fields not in the preset (like `color_mode`, `yarn_lines`, `knit_volume`, `ad_file`, etc.) are inherited from the base `Config`. This means CLI flags like `--color-mode dark-rgb` or `--scale 2` still take effect.

### Custom Game screen

The `CustomGame` state shows:

1. Preset selector at top: `← Small | [Medium] | Large | Chaos →` — switch with `←`/`→`
2. Editable field list below (8 fields):
   - Board Height, Board Width, Color Count, Obstacle %, Generator %, Scissors, Tweezers, Balloons
   - Note: Scale and Color Mode are display preferences managed in Options, not per-game settings
3. Navigate fields with `↑`/`↓`, adjust values with `←`/`→` (decrement/increment by 1)
4. `Enter`: start game with current values
5. `Esc`: return to main menu

Selecting a different preset resets all fields to that preset's values. Tweaking any field is independent — no "modified" indicator needed.

---

## 3a. Options & Persistent Settings

### Options screen

Accessible from main menu index 4. Shows two settings:
- **Scale** (1–5): `←`/`→` to adjust
- **Color Mode**: cycles through `dark → bright → colorblind → dark-rgb → bright-rgb → colorblind-rgb`

`↑`/`↓` navigates, `Esc` saves and returns to menu. Changes take effect immediately.

### Persistence

Settings saved to `~/.config/knitui/settings.json` via `UserSettings` struct (serde_json). Loaded at startup. Three-tier merge:

1. Hard defaults (scale=1, color_mode="dark")
2. Saved settings file overrides defaults
3. CLI args (`--scale`, `--color-mode`) override saved settings for that session only

### CLI skip-menu behavior

Game-specific args (`--board-height`, `--scissors`, etc.) skip the menu and launch directly. Display-only args (`--scale`, `--color-mode`) do not skip the menu — they override the saved settings for that session.

---

## 4. Game-Over Changes

Current overlay: `R` restart, `Q` quit.

New overlay: `R` restart (same config), `M` menu (return to `MainMenu`), `Q` quit.

The ad hotkey `A` remains available when stuck.

---

## 5. Esc Behavior in Playing

Currently `Esc` in `Playing` quits the program. With the menu, `Esc` in `Playing` returns to `MainMenu`. `Q` also returns to menu (same as `Esc`). The only way to exit the program is from the menu's Quit option or `Q`/`Esc` from the menu itself.

---

## 6. Rendering

### render_main_menu()

Centered text list. Selected item in reverse video (same technique as cursor highlighting). Unselected items in normal text. Flash message (if any) rendered below in dim text.

```
          ═══ KNITUI ═══

        > Quick Game
          Custom Game
          Campaign
          Endless
          Options
          Quit

        Coming soon!
```

### render_custom_game()

Two-column layout. Preset selector at top. Fields below with current values. Selected field highlighted.

```
     ═══ CUSTOM GAME ═══

     Preset: ← [Medium] →

     Board Height     6
     Board Width      6
   > Color Count      6
     Obstacle %       5
     Generator %      5
     Scissors         0
     Tweezers         0
     Balloons         0

     Enter: Start  Esc: Back
```

Both screens are minimal text — no box drawing, consistent with help overlay.

---

## 7. Files Changed

| File | Change |
|------|--------|
| `src/preset.rs` | **New** — `GamePreset` struct (no scale), `PRESETS` array, `to_config()` |
| `src/settings.rs` | **New** — `UserSettings` with `load()`/`save()` to `~/.config/knitui/settings.json`, color mode cycling |
| `src/main.rs` | Add `MainMenu`, `CustomGame`, `Options` to `TuiState`; settings merge; start in `MainMenu`; CLI skip-menu detection; game-over `M` option; `Esc` in `Playing` → menu |
| `src/renderer.rs` | Add `render_main_menu()`, `render_custom_game()`, `render_options()` |
| `src/lib.rs` | Add `pub mod preset; pub mod settings;` |

---

## 8. Testing

- `preset::to_config()` — preset values override base config correctly
- `preset::to_config()` — non-preset fields (color_mode, scale, ad_file, etc.) inherited from base
- `preset::to_config()` — scale and color_mode preserved from base config
- `preset::PRESETS` — all presets have valid field ranges (board ≥ 2, colors ≥ 2, etc.)
- `settings::UserSettings` — serialization roundtrip
- `settings::next_color_mode()` / `prev_color_mode()` — cycling works correctly
- State transitions — unit-testable if we extract transition logic into functions

---

## 9. Out of Scope

- Campaign mode implementation (placeholder only)
- Endless mode implementation (placeholder only)
- Saving/loading game sessions from menu
- In-game settings screen
- Key rebinding
