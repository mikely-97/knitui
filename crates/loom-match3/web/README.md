# loom-match3 web build

Browser build of match3's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl. See
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

Then open `http://localhost:8000/`. Quick Game config only -- no menu/
campaign/blessing screens yet (see `../src/web.rs`). Note the animation
tick rate is a rough frame-count throttle, not calibrated against real
elapsed time -- cascades may run faster or slower than the terminal
version depending on your monitor's refresh rate.
