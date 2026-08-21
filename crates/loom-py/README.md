# loom-py

PyO3 bindings for the loom portable game engine (Phase 5, second
sub-deliverable after [`loom-engine-capi`](../loom-engine-capi)'s C ABI).
A real, memory-safe Python class — no raw pointers, no manual free calls,
no JSON gymnastics for input.

## Shape

Host-owns-the-loop, same as every native frontend and the C ABI:

```python
import json
import loom_py

g = loom_py.LoomGame(loom_py.GAME_KNIT)
while not g.should_quit():
    # ... read a keypress from wherever the host gets input ...
    g.handle_key("Up")          # or "a", " ", "Enter", "Esc", ...
    g.tick()
    frame = json.loads(g.render(100, 40))  # {"width", "height", "cells": [...]}
    # ... draw `frame` somehow ...
g.save_on_exit()
```

`render()` returns the same JSON `CellGrid` shape as the C ABI (see
`loom-engine-capi`'s docs) — `loom_py.pyi` documents the exact fields.
Unlike the C ABI, `handle_key` takes a plain key name string, not
hand-built `KeyEvent` JSON (which has a real gotcha: bitflags' serde
format for the `mods` field is a string, not an int — this binding avoids
that entirely by not exposing modifiers, since no game currently reads
them).

Internally this reuses `loom-engine-capi`'s `ErasedShell`/`create_shell`
type-erasure layer directly rather than re-deriving it.

## Building

```
cd crates/loom-py
pip install maturin
maturin build --release   # -> ../../target/wheels/loom_py-*.whl
pip install ../../target/wheels/loom_py-*.whl
```

Or for local development (editable install into the active venv):

```
maturin develop
```

## Verifying

```
python tests/test_loom_py.py
```

A real Python program importing and driving the actual compiled
extension through all 4 games, including error paths (bad key name,
unknown game id) — not a mock.

## Scope

`LoomGame` is intentionally `unsendable` (PyO3 term: can only be touched
from the thread that created it) — `Shell<G>` isn't proven `Send`/`Sync`
across all 4 game instantiations, and every native frontend already
drives it from a single thread anyway ("host owns the loop"). A
multi-threaded/async-safe wrapper is future work if a real consumer needs
one, matching this pivot's "defer polish until someone needs it"
principle already applied to the C ABI's C++/Go surface.
