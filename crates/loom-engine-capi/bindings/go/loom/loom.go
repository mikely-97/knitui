// Package loom provides cgo bindings to loom-engine-capi's C ABI. Not
// generated -- hand-written against ../../../include/loom.h, kept in
// sync manually since the C ABI changes rarely (see that header's own
// doc comments for the exact JSON shapes this package passes through
// verbatim).
//
// The library must be built first (produces libloom_engine_capi.so):
//
//	cargo build -p loom-engine-capi
//
// and its directory must be on LD_LIBRARY_PATH at runtime, same as any
// other consumer of this dynamic library (see ../../../examples/smoke.c
// and ../../cpp/example.cpp for the equivalent C/C++ setup).
//
// Usage:
//
//	game, err := loom.New(loom.Knit)
//	if err != nil { ... }
//	defer game.Close()
//	for !game.ShouldQuit() {
//	    game.HandleKeyNamed("Up") // or game.HandleKeyChar('a')
//	    game.Tick()
//	    frame, err := game.Render(100, 40) // JSON CellGrid
//	}
//	game.SaveOnExit()
package loom

/*
#cgo CFLAGS: -I${SRCDIR}/../../../include
#cgo LDFLAGS: -L${SRCDIR}/../../../../../target/debug -lloom_engine_capi
#include "loom.h"
#include <stdlib.h>
*/
import "C"

import (
	"errors"
	"runtime"
	"unsafe"
)

// GameID selects which of the 4 games New creates.
type GameID uint32

const (
	Knit    GameID = C.LOOM_GAME_KNIT
	Match3  GameID = C.LOOM_GAME_MATCH3
	Merge2  GameID = C.LOOM_GAME_MERGE2
	Picross GameID = C.LOOM_GAME_PICROSS
)

// Game wraps one LoomHandle. Not safe for concurrent use from multiple
// goroutines (matches the underlying Shell<G>, which is single-threaded).
type Game struct {
	handle *C.LoomHandle
}

// New creates a running game instance, loading real persisted
// settings/campaign/high-score state from disk (same location the
// native terminal build uses for that game). Call Close (or defer it)
// exactly once when done; a finalizer also calls Close as a safety net,
// but relying on GC timing for a live game session is not recommended.
func New(id GameID) (*Game, error) {
	h := C.loom_create(C.uint32_t(id))
	if h == nil {
		return nil, errors.New("loom_create: unrecognized game id")
	}
	g := &Game{handle: h}
	runtime.SetFinalizer(g, (*Game).Close)
	return g, nil
}

// Close destroys the underlying handle. Safe to call more than once;
// calls after the first are no-ops.
func (g *Game) Close() {
	if g.handle != nil {
		C.loom_destroy(g.handle)
		g.handle = nil
		runtime.SetFinalizer(g, nil)
	}
}

// HandleKeyJSON handles one key event, JSON-encoded as
// loom_engine::input::KeyEvent -- see loom.h's loom_handle_key doc
// comment for the exact shape. Prefer HandleKeyNamed/HandleKeyChar for
// the common cases.
func (g *Game) HandleKeyJSON(keyJSON string) error {
	cs := C.CString(keyJSON)
	defer C.free(unsafe.Pointer(cs))
	rc := C.loom_handle_key(g.handle, cs)
	if rc != 0 {
		return errors.New("loom_handle_key: " + g.lastErrorOr("unknown error"))
	}
	return nil
}

// HandleKeyNamed handles one of the 6 non-character keys: "Up", "Down",
// "Left", "Right", "Enter", "Esc".
func (g *Game) HandleKeyNamed(name string) error {
	return g.HandleKeyJSON(`{"key":"` + name + `","mods":""}`)
}

// HandleKeyChar handles a single character key.
func (g *Game) HandleKeyChar(c rune) error {
	return g.HandleKeyJSON(`{"key":{"Char":"` + string(c) + `"},"mods":""}`)
}

// Tick advances background/animation state by one tick (call at
// whatever cadence the host frontend polls at).
func (g *Game) Tick() {
	C.loom_tick(g.handle)
}

// Render returns the current frame at width x height cells, as a
// JSON-encoded loom_engine::render::CellGrid.
func (g *Game) Render(width, height uint16) (string, error) {
	frame := C.loom_render(g.handle, C.uint16_t(width), C.uint16_t(height))
	if frame == nil {
		return "", errors.New("loom_render: " + g.lastErrorOr("unknown error"))
	}
	defer C.loom_free_string(frame)
	return C.GoString(frame), nil
}

// ShouldQuit reports whether the game has requested to quit (player
// selected Quit from the main menu, or equivalent).
func (g *Game) ShouldQuit() bool {
	return bool(C.loom_should_quit(g.handle))
}

// SaveOnExit persists any in-progress campaign run. Call once, right
// before Close, mirroring every native tui.rs's exit sequence.
func (g *Game) SaveOnExit() {
	C.loom_save_on_exit(g.handle)
}

func (g *Game) lastErrorOr(fallback string) string {
	err := C.loom_last_error(g.handle)
	if err == nil {
		return fallback
	}
	defer C.loom_free_string(err)
	return C.GoString(err)
}
