# loom-engine-capi — Go bindings

`loom/loom.go` is a hand-written `cgo` package wrapping the generated
`../../include/loom.h` C ABI: a `loom.Game` struct with a `Close` method
(plus a `runtime.SetFinalizer` safety net) instead of manual
`loom_destroy` calls, and Go `error`s instead of raw C return codes.

```go
game, err := loom.New(loom.Knit)
if err != nil { ... }
defer game.Close()
for !game.ShouldQuit() {
    game.HandleKeyNamed("Up") // or game.HandleKeyChar('a')
    game.Tick()
    frame, err := game.Render(100, 40) // JSON CellGrid
}
game.SaveOnExit()
```

## Building

The Rust library must be built first:

```sh
cargo build -p loom-engine-capi
```

`loom/loom.go`'s cgo directives locate `include/loom.h` and
`target/debug/libloom_engine_capi.so` via `${SRCDIR}`-relative paths, so
`go build` works from anywhere once the library above is built — no
`CGO_CFLAGS`/`CGO_LDFLAGS` env vars needed. At **runtime**, though, the
dynamic linker still needs to find the `.so`, exactly like any other
consumer of this library (see `../../examples/smoke.c` and
`../cpp/example.cpp` for the equivalent C/C++ setup):

```sh
export LD_LIBRARY_PATH=$(cd ../../../../../target/debug && pwd)
```

## Verifying

`example/main.go` is a real, compiled program (not `go test` calling
into itself) exercising all 4 games and the error path on malformed key
JSON:

```sh
go build -o /tmp/loom_go_example ./example
LD_LIBRARY_PATH=... /tmp/loom_go_example   # see LD_LIBRARY_PATH above
```

`go vet ./...` and `gofmt -l .` are both clean.

## Notes

- Not safe for concurrent use from multiple goroutines on the same
  `*Game` — matches the underlying `Shell<G>`, which is single-threaded.
- `Close` is idempotent; call it explicitly (`defer game.Close()`) rather
  than relying on the finalizer, which only exists as a leak backstop —
  GC timing is not a substitute for `loom_save_on_exit` + `loom_destroy`
  happening when you actually mean to end the session.
