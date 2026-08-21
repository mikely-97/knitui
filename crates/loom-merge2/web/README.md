# loom-merge2 web build

Browser build of merge2's `WebGame` (see `../src/web.rs`), backed by
`loom-engine-web`'s canvas `Surface` impl. Drives the full
`Shell<M2Game>` state machine -- same main menu (Custom Game/Campaign/
Endless/Options, no Quick Game item), campaign, and inventory/help
screens as the native terminal build, with saves going to `localStorage`
instead of real files. See `../../loom-knit/web/README.md` for the full
explanation.

## Build

```sh
cargo build -p loom-merge2 --lib --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir pkg ../../../target/wasm32-unknown-unknown/release/m2tui.wasm
```

## Headless smoke test

```sh
node smoke_test.mjs
```

This is the one that actually caught a real bug during development:
`energy.rs`'s wall-clock regen used `std::time::SystemTime::now()`, which
compiles fine for wasm32 but panics at runtime ("time not implemented on
this platform") the moment it's called. The fix branches on
`target_arch = "wasm32"` to use `js_sys::Date::now()` instead. A pure
"does it compile" check would never have caught this -- only actually
executing the wasm did.

## Actual browser check

```sh
python3 -m http.server 8000   # from this directory
```

Then open `http://localhost:8000/`. Same controls as the terminal build.
`M2EngineAdapter` (in `../src/game.rs`) turned out to have no real
native-only dependency once checked -- it's used here unchanged, not a
separate wasm-specific implementation.
