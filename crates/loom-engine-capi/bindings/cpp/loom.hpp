// Single-header RAII C++ wrapper around loom-engine-capi's C ABI. Not
// generated -- hand-written against `../../include/loom.h`, kept in sync
// manually since the C ABI changes rarely (see that header's own doc
// comments for the exact JSON shapes this class passes through verbatim).
//
// Usage:
//
//   loom::Game game(loom::GameId::Knit);
//   while (!game.should_quit()) {
//       game.handle_key_named("Up");           // or game.handle_key_char('a');
//       game.tick();
//       std::string frame = game.render(100, 40); // JSON CellGrid
//   }
//   game.save_on_exit();
//   // ~Game() calls loom_destroy automatically.
//
// Link against the built `libloom_engine_capi.{a,so}` exactly as
// `examples/smoke.c` does; see `bindings/cpp/example.cpp` for a full,
// compiled-and-run demonstration.

#ifndef LOOM_CAPI_CPP_HPP
#define LOOM_CAPI_CPP_HPP

#include "loom.h"

#include <stdexcept>
#include <string>
#include <utility>

namespace loom {

enum class GameId : uint32_t {
    Knit = LOOM_GAME_KNIT,
    Match3 = LOOM_GAME_MATCH3,
    Merge2 = LOOM_GAME_MERGE2,
    Picross = LOOM_GAME_PICROSS,
};

// Thrown when a call fails and the C ABI recorded a `loom_last_error`
// message (malformed key JSON, mainly -- see loom.h). Never thrown for
// null-handle cases, since a well-formed Game can't hold a null handle.
class Error : public std::runtime_error {
public:
    explicit Error(const std::string &what) : std::runtime_error(what) {}
};

namespace detail {
// Takes ownership of a `char *` from the C ABI and frees it via
// `loom_free_string`, even if constructing the `std::string` throws.
inline std::string take_c_string(char *s) {
    if (s == nullptr) {
        return std::string();
    }
    std::string result(s);
    loom_free_string(s);
    return result;
}
} // namespace detail

// RAII, move-only wrapper around one `LoomHandle`. Copying is disabled
// since a `LoomHandle` has single-owner semantics on the Rust side (one
// `loom_destroy` call per `loom_create`).
class Game {
public:
    explicit Game(GameId id) : handle_(loom_create(static_cast<uint32_t>(id))) {
        if (handle_ == nullptr) {
            throw Error("loom_create: unrecognized game id");
        }
    }

    ~Game() {
        loom_destroy(handle_);
    }

    Game(const Game &) = delete;
    Game &operator=(const Game &) = delete;

    Game(Game &&other) noexcept : handle_(other.handle_) {
        other.handle_ = nullptr;
    }

    Game &operator=(Game &&other) noexcept {
        if (this != &other) {
            loom_destroy(handle_);
            handle_ = other.handle_;
            other.handle_ = nullptr;
        }
        return *this;
    }

    // Raw JSON form -- see loom.h's `loom_handle_key` doc comment for the
    // exact shape (e.g. `{"key":"Up","mods":""}`). Throws `loom::Error`
    // with the ABI's own message on malformed JSON.
    void handle_key(const std::string &key_json) {
        int32_t rc = loom_handle_key(handle_, key_json.c_str());
        if (rc != 0) {
            throw Error("loom_handle_key: " + last_error_or("unknown error"));
        }
    }

    // Convenience for the 6 non-character keys: "Up"/"Down"/"Left"/
    // "Right"/"Enter"/"Esc".
    void handle_key_named(const std::string &name) {
        handle_key("{\"key\":\"" + name + "\",\"mods\":\"\"}");
    }

    // Convenience for a single character key.
    void handle_key_char(char c) {
        handle_key(std::string("{\"key\":{\"Char\":\"") + c + "\"},\"mods\":\"\"}");
    }

    void tick() {
        loom_tick(handle_);
    }

    // Returns the current frame as a JSON-encoded `CellGrid` (see
    // `render.rs`'s `Cell`/`Style` types for the schema).
    std::string render(uint16_t width, uint16_t height) {
        char *frame = loom_render(handle_, width, height);
        if (frame == nullptr) {
            throw Error("loom_render: " + last_error_or("unknown error"));
        }
        return detail::take_c_string(frame);
    }

    bool should_quit() const {
        return loom_should_quit(handle_);
    }

    void save_on_exit() {
        loom_save_on_exit(handle_);
    }

private:
    std::string last_error_or(const std::string &fallback) {
        char *err = loom_last_error(handle_);
        if (err == nullptr) {
            return fallback;
        }
        return detail::take_c_string(err);
    }

    LoomHandle *handle_;
};

} // namespace loom

#endif // LOOM_CAPI_CPP_HPP
