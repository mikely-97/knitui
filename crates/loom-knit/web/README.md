# loom-knit web build

Browser build of loom-knit's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl.

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
objects. Verifies the actual compiled `.wasm` constructs, generates a board
(exercising the solvability retry loop and `getRandomValues` via the
`wasm_js` getrandom backend), renders, and responds to cursor-move/pick-up
input without throwing. It does **not** check pixel output -- only a real
browser can confirm that.

## Actual browser check

```sh
python3 -m http.server 8000   # from this directory; ES module imports need http://, not file://
```

Then open `http://localhost:8000/`. Arrow keys move the cursor, Enter picks
up. The board is fixed at the default `Config` (6x6, scale 1) -- there's no
menu yet, this is the core gameplay loop only (see `../src/web.rs`'s doc
comment for why).
