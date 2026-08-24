// Real cross-language link test for the Go package (mirrors
// ../../examples/smoke.c and ../../cpp/example.cpp, but through the cgo
// wrapper instead of raw C ABI calls / the C++ RAII class). Build+run:
//
//	cd ..; cargo build -p loom-engine-capi   # from bindings/go, builds the workspace's target/debug lib
//	go build -o /tmp/loom_go_example ./example
//	LD_LIBRARY_PATH=$(cd ../../../../../target/debug && pwd) /tmp/loom_go_example
//
// Exercises: New/Close for all 4 games, an error on malformed key JSON,
// and a full scripted play sequence with real rendering.
package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/mikely-97/knitui/crates/loom-engine-capi/bindings/go/loom"
)

func check(cond bool, msg string) {
	if !cond {
		fmt.Fprintf(os.Stderr, "FAIL: %s\n", msg)
		os.Exit(1)
	}
}

func playOneGame(id loom.GameID, name string) {
	fmt.Printf("-- %s --\n", name)

	game, err := loom.New(id)
	check(err == nil, "New must succeed for a valid game id")
	defer game.Close()

	check(!game.ShouldQuit(), "ShouldQuit must start false")
	check(game.HandleKeyNamed("Enter") == nil, "HandleKeyNamed(Enter) should succeed")

	for i := 0; i < 10; i++ {
		game.Tick()
		_ = game.HandleKeyNamed("Right")
		_ = game.HandleKeyChar(' ')

		frame, err := game.Render(80, 30)
		check(err == nil, "Render must succeed")
		check(strings.Contains(frame, `"cells"`), "render() JSON missing cells field")
	}

	err = game.HandleKeyJSON("not json")
	check(err != nil, "HandleKeyJSON with malformed JSON must return an error")
	fmt.Printf("   (expected) error: %v\n", err)

	game.SaveOnExit()
	fmt.Println("   ok")
}

func main() {
	playOneGame(loom.Knit, "knit")
	playOneGame(loom.Match3, "match3")
	playOneGame(loom.Merge2, "merge2")
	playOneGame(loom.Picross, "picross")

	// Unknown game id must error, not panic.
	_, err := loom.New(loom.GameID(9999))
	check(err != nil, "New with an unknown game id should return an error")

	fmt.Println("ALL GO WRAPPER CHECKS PASSED")
}
