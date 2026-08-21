# loom-knit web build

Browser build of loom-knit's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl. Drives the full
`Shell<KnitGame>` state machine -- same main menu, custom game, campaign,
endless, options, and help screens as the native terminal build, with
settings/campaign/high-score saves going to `localStorage` (via
`loom-engine-web`'s `WebStorage`) instead of real files.

## Build

```sh
cargo build -p loom-knit --lib --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg ../../../target/wasm32-unknown-unknown/release/knitui.wasm
```

(`pkg/` is generated -- gitignored, not committed. Requires `wasm32-unknown-unknown`
via `rustup target add wasm32-unknown-unknown`, and `wasm-bindgen-cli` matching
the `wasm-bindgen` version in `Cargo.toml` -- `cargo install wasm-bindgen-cli
--version <matching version> --locked`. `wasm-pack` is not required.)

## Headless smoke test (no browser needed)

```sh
node smoke_test.mjs
```

Runs `WebGame` under Node with mock `CanvasRenderingContext2d`/`KeyboardEvent`
objects. Verifies the actual compiled `.wasm` constructs and runs the full
menu -> Quick Game -> play -> help -> quit-to-menu flow without throwing
(exercising the solvability retry loop, `getRandomValues` via the
`wasm_js` getrandom backend, and the generic `Shell<KnitGame>` state
machine end to end). It does **not** check pixel output, and `WebStorage`
silently no-ops under Node (no real `localStorage`) -- only a real browser
can confirm rendering and persistence.

## Actual browser check

```sh
python3 -m http.server 8000   # from this directory; ES module imports need http://, not file://
```

Then open `http://localhost:8000/`. Same controls as the terminal build:
arrow keys navigate menus and move the board cursor, Enter selects/picks
up, Esc backs out. Settings/campaign/high-score persist across reloads via
`localStorage`.
