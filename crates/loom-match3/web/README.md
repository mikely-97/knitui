# loom-match3 web build

Browser build of match3's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl. Drives the full
`Shell<M3Game>` state machine -- same main menu, custom game, campaign,
endless, options, and help screens as the native terminal build, with
saves going to `localStorage` instead of real files. See
`../../loom-knit/web/README.md` for the full explanation.

## Build

```sh
cargo build -p loom-match3 --lib --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg ../../../target/wasm32-unknown-unknown/release/m3tui.wasm
```

## Headless smoke test

```sh
node smoke_test.mjs
```

## Actual browser check

```sh
python3 -m http.server 8000   # from this directory
```

Then open `http://localhost:8000/`. Same controls as the terminal build.
Note the tick rate is every animation frame (~60fps), not the terminal
build's ~50ms poll cadence -- cascades/animations run faster here.
