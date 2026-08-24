# loom-engine-capi — C++ bindings

`loom.hpp` is a single-header, hand-written RAII wrapper around the
generated `../../include/loom.h` C ABI: a move-only `loom::Game` class
that calls `loom_destroy` in its destructor and throws `loom::Error`
(carrying the ABI's own `loom_last_error` message) instead of returning
raw error codes.

```cpp
#include "loom.hpp"

loom::Game game(loom::GameId::Knit);
while (!game.should_quit()) {
    game.handle_key_named("Up");      // or game.handle_key_char('a')
    game.tick();
    std::string frame = game.render(100, 40); // JSON CellGrid
}
game.save_on_exit();
// ~Game() calls loom_destroy automatically.
```

## Building

`loom.hpp` only needs `../../include/loom.h` (C++11 or later; uses
`std::string`/RAII/move semantics, nothing fancier) and the built
library:

```sh
cargo build -p loom-engine-capi
```

## Verifying

`example.cpp` is a real, compiled-and-linked program (not run by `cargo
test`) exercising all 4 games, move construction, and the exception path
on malformed key JSON:

```sh
g++ -std=c++17 -o /tmp/loom_cpp_example example.cpp -I ../../include -I . -L ../../../../target/debug -lloom_engine_capi
LD_LIBRARY_PATH=../../../../target/debug /tmp/loom_cpp_example
```

## Note on the generated header

`../../include/loom.h` is cbindgen-generated with `cpp_compat = true`
(`cbindgen.toml`), which wraps its declarations in `extern "C" { ... }`
guarded by `#ifdef __cplusplus`. Without that, a C++ compiler mangles the
declared names and every symbol fails to link against the `extern "C"`
functions Rust actually exports — regenerate with the `cbindgen.toml` in
this crate, not a bare `cbindgen --crate loom-engine-capi`, if you ever
touch the header by hand.
