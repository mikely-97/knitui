# loom-picross web build

Browser build of picross's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl. Drives the full
`Shell<PicrossGame>` state machine -- main menu (Campaign + Quit only --
picross has no Quick/Custom/Endless), track select, level intro, and help,
same as the native terminal build, with saves going to `localStorage`
instead of real files. See `../../loom-knit/web/README.md` for the full
explanation -- the workflow is identical here.

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

Then open `http://localhost:8000/`. Same controls as the terminal build.
Campaign play is sequential-only, same as native (see `../src/game.rs`'s
`current_puzzle` doc comment for why) -- no free puzzle-jumping.
