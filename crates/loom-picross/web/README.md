# loom-picross web build

Browser build of picross's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl. See
`../../loom-knit/web/README.md` for the full explanation -- the workflow
is identical here.

## Build

```sh
cargo build -p loom-picross --lib --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg ../../../target/wasm32-unknown-unknown/release/pictui.wasm
```

## Headless smoke test

```sh
node smoke_test.mjs
```

## Actual browser check

```sh
python3 -m http.server 8000   # from this directory
```

Then open `http://localhost:8000/`. Only the first built-in puzzle is
playable -- no menu/track-select yet (see `../src/web.rs`).
