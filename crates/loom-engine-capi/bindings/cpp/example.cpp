// Real cross-language link test for the C++ wrapper (mirrors
// ../../examples/smoke.c, but through loom.hpp's RAII class instead of
// raw C ABI calls). Build+run:
//
//   cargo build -p loom-engine-capi
//   g++ -std=c++17 -o /tmp/loom_cpp_example example.cpp -I ../../include -I . -L ../../../../target/debug -lloom_engine_capi
//   LD_LIBRARY_PATH=../../../../target/debug /tmp/loom_cpp_example
//
// Exercises: construction/destruction for all 4 games via RAII, move
// construction, exception on malformed key JSON, and a full scripted
// play sequence with real rendering.

#include "loom.hpp"

#include <cstdio>
#include <cstdlib>
#include <string>

#define CHECK(cond, msg)                                                     \
    do {                                                                     \
        if (!(cond)) {                                                       \
            std::fprintf(stderr, "FAIL: %s\n", msg);                        \
            std::exit(1);                                                    \
        }                                                                    \
    } while (0)

static void play_one_game(loom::GameId id, const char *name) {
    std::printf("-- %s --\n", name);

    loom::Game game(id);
    CHECK(!game.should_quit(), "should_quit must start false");

    game.handle_key_named("Enter");

    for (int i = 0; i < 10; ++i) {
        game.tick();
        game.handle_key_named("Right");
        game.handle_key_char(' ');

        std::string frame = game.render(80, 30);
        CHECK(frame.find("\"cells\"") != std::string::npos,
              "render() JSON missing cells field");
    }

    bool threw = false;
    try {
        game.handle_key("not json");
    } catch (const loom::Error &e) {
        threw = true;
        std::printf("   (expected) loom::Error: %s\n", e.what());
    }
    CHECK(threw, "handle_key with malformed JSON must throw loom::Error");

    game.save_on_exit();
    std::printf("   ok\n");
    // ~Game() runs here, calling loom_destroy exactly once.
}

int main() {
    play_one_game(loom::GameId::Knit, "knit");
    play_one_game(loom::GameId::Match3, "match3");
    play_one_game(loom::GameId::Merge2, "merge2");
    play_one_game(loom::GameId::Picross, "picross");

    // Move semantics: moving-from must not double-destroy.
    {
        loom::Game a(loom::GameId::Knit);
        loom::Game b(std::move(a));
        CHECK(!b.should_quit(), "moved-into Game must be usable");
        b.save_on_exit();
        // a's destructor runs on a null handle (safe no-op, per loom.h);
        // b's destructor runs on the real handle.
    }

    std::printf("ALL C++ WRAPPER CHECKS PASSED\n");
    return 0;
}
