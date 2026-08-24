# pixel-verify

Real-browser pixel-level verification for all 4 loom web builds. Each
game's `web/smoke_test.mjs` (committed alongside its `web/` build)
proves the compiled `.wasm` runs end-to-end without throwing, using
synthesized JS objects standing in for `CanvasRenderingContext2d` and
`KeyboardEvent` -- it does not prove anything renders correctly, since
Node has no real canvas. This tool closes that gap: it drives each
game's actual `index.html` in real headless Chromium (via
[Playwright](https://playwright.dev)), dispatches real keydown events,
and screenshots the actual canvas at each meaningful state.

It does **not** replace looking at the screenshots. The script only
catches console/page errors and unexpected HTTP failures automatically
-- judging whether the board actually *looks* right (correct colors,
readable text, sane layout) still needs a human, or an AI agent that can
view images, to open the PNGs in `./shots/`.

## One-time setup

```sh
npm install
npx playwright install chromium
```

If `verify.mjs` then fails to launch with "Executable doesn't exist at
.../chrome-headless-shell-*" (a stale system-wide Playwright browser
cache not matching the `playwright` npm package version pinned in
`package.json`), that install command above is what fixes it -- it
downloads the exact matching revision.

## Every run

The wasm-bindgen output (`crates/<game>/web/pkg/`) isn't committed
(regenerated, like any other build artifact) -- build it first:

```sh
cd ../..   # repo root
cargo build -p loom-knit -p loom-match3 -p loom-merge2 -p loom-picross \
  --lib --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir crates/loom-knit/web/pkg    target/wasm32-unknown-unknown/release/knitui.wasm
wasm-bindgen --target web --out-dir crates/loom-match3/web/pkg target/wasm32-unknown-unknown/release/m3tui.wasm
wasm-bindgen --target web --out-dir crates/loom-merge2/web/pkg target/wasm32-unknown-unknown/release/m2tui.wasm
wasm-bindgen --target web --out-dir crates/loom-picross/web/pkg target/wasm32-unknown-unknown/release/pictui.wasm
```

Then, from this directory:

```sh
node verify.mjs
```

Screenshots land in `./shots/<game>_<step>.png` (gitignored -- generated
output, same as `pkg/`).

## What it exercises per game

Each game's flow mirrors its own `web/smoke_test.mjs` key sequence
(menu shape and screens genuinely differ per game -- see each `Game`
impl's `main_menu_items()`, e.g. merge2 has no Quick Game, picross has
only Campaign + Quit): main menu, entering play, a help-screen
toggle, any game-specific sub-state (merge2's inventory, picross's
fill/cross), and quit-to-menu.

## Last verified

2026-08-24: all 4 games -- correct menus (including each game's
customized menu subset), correct gameplay boards (color-coded cells,
cursor highlighting, HUD/keybar text), correct help overlays, and clean
quit-to-menu round trips. No console or page errors beyond the expected
one-time `favicon.ico` 404 (none of these apps serve a favicon; the
browser requests it automatically regardless).
