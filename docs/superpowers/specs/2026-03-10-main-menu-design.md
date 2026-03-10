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
| 4 | Quit | Exit program |

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
    pub scale: u16,
    pub scissors: u16,
    pub tweezers: u16,
    pub balloons: u16,
}
```

### Named presets

| Preset | Board | Colors | Obstacles | Generators | Scale | Bonuses |
|--------|-------|--------|-----------|------------|-------|---------|
| Small | 4×4 | 4 | 0% | 0% | 1 | 0/0/0 |
| Medium | 6×6 | 6 | 5% | 5% | 1 | 0/0/0 |
| Large | 8×8 | 8 | 10% | 10% | 1 | 1/1/1 |
| Chaos | 10×10 | 8 | 20% | 15% | 1 | 2/2/2 |

### to_config() conversion

`GamePreset::to_config(&self, base: &Config) -> Config` — creates a `Config` by applying preset values on top of the CLI-parsed `Config`. Fields not in the preset (like `color_mode`, `yarn_lines`, `knit_volume`, `ad_file`, etc.) are inherited from the base `Config`. This means CLI flags like `--color-mode dark-rgb` or `--scale 2` still take effect.

### Custom Game screen

The `CustomGame` state shows:

1. Preset selector at top: `← Small | [Medium] | Large | Chaos →` — switch with `←`/`→`
2. Editable field list below (9 fields):
   - Board Height, Board Width, Color Count, Obstacle %, Generator %, Scale, Scissors, Tweezers, Balloons
3. Navigate fields with `↑`/`↓`, adjust values with `←`/`→` (decrement/increment by 1)
4. `Enter`: start game with current values
5. `Esc`: return to main menu

Selecting a different preset resets all fields to that preset's values. Tweaking any field is independent — no "modified" indicator needed.

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
     Scale            1
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
| `src/preset.rs` | **New** — `GamePreset` struct, `PRESETS` array, `to_config()` |
| `src/main.rs` | Add `MainMenu`, `CustomGame` to `TuiState`; start in `MainMenu`; menu/custom keybindings; game-over `M` option; `Esc` in `Playing` → menu |
| `src/renderer.rs` | Add `render_main_menu()`, `render_custom_game()` |
| `src/lib.rs` | Add `pub mod preset;` |

---

## 8. Testing

- `preset::to_config()` — preset values override base config correctly
- `preset::to_config()` — non-preset fields (color_mode, ad_file, etc.) inherited from base
- `preset::PRESETS` — all presets have valid field ranges (board ≥ 2, colors ≥ 2, etc.)
- State transitions — unit-testable if we extract transition logic into functions

---

## 9. Out of Scope

- Campaign mode implementation (placeholder only)
- Endless mode implementation (placeholder only)
- Saving/loading game sessions from menu
- In-game settings screen
- Key rebinding
