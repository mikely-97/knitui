/* Real cross-language link test for the C ABI (not run by `cargo test`,
 * which only proves the Rust side calls itself correctly). Build+run:
 *
 *   cargo build -p loom-engine-capi
 *   cc -o /tmp/loom_smoke examples/smoke.c -I include \
 *      -L ../../target/debug -lloom_engine_capi
 *   LD_LIBRARY_PATH=../../target/debug /tmp/loom_smoke
 *
 * Exercises: create/destroy for all 4 games, handle_key with real JSON,
 * tick, render returning parseable JSON, should_quit, save_on_exit,
 * last_error on a malformed key, and string ownership (free every
 * returned string exactly once). Exits non-zero and prints which check
 * failed on the first failure. */

#include "loom.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define CHECK(cond, msg) \
    do { if (!(cond)) { fprintf(stderr, "FAIL: %s\n", msg); return 1; } } while (0)

static int smoke_one_game(uint32_t game_id, const char *name) {
    printf("-- %s --\n", name);

    struct LoomHandle *h = loom_create(game_id);
    CHECK(h != NULL, "loom_create returned null for a valid game id");

    CHECK(!loom_should_quit(h), "should_quit must start false");

    /* Enter -> whatever the first main-menu item does for this game. */
    int rc = loom_handle_key(h, "{\"key\":\"Enter\",\"mods\":\"\"}");
    CHECK(rc == 0, "handle_key(Enter) should succeed");

    for (int i = 0; i < 10; i++) {
        loom_tick(h);
        loom_handle_key(h, "{\"key\":\"Right\",\"mods\":\"\"}");
        loom_handle_key(h, "{\"key\":{\"Char\":\" \"},\"mods\":\"\"}");

        char *frame = loom_render(h, 80, 30);
        if (frame != NULL) {
            CHECK(strstr(frame, "\"cells\"") != NULL, "render() JSON missing cells field");
            loom_free_string(frame);
        }
    }

    /* Malformed key should error and set last_error, without crashing. */
    rc = loom_handle_key(h, "not json");
    CHECK(rc == -1, "handle_key with malformed JSON must return -1");
    char *err = loom_last_error(h);
    CHECK(err != NULL, "last_error must be set after a failed handle_key");
    printf("   (expected) last_error: %s\n", err);
    loom_free_string(err);

    loom_save_on_exit(h);
    loom_destroy(h);
    printf("   ok\n");
    return 0;
}

int main(void) {
    if (smoke_one_game(LOOM_GAME_KNIT, "knit")) return 1;
    if (smoke_one_game(LOOM_GAME_MATCH3, "match3")) return 1;
    if (smoke_one_game(LOOM_GAME_MERGE2, "merge2")) return 1;
    if (smoke_one_game(LOOM_GAME_PICROSS, "picross")) return 1;

    /* Null-safety: every function must tolerate a null handle. */
    loom_destroy(NULL);
    loom_tick(NULL);
    loom_save_on_exit(NULL);
    CHECK(loom_should_quit(NULL) == true, "should_quit(null) should report true (safe default)");
    CHECK(loom_render(NULL, 10, 10) == NULL, "render(null) should return null");
    CHECK(loom_handle_key(NULL, "{\"key\":\"Up\",\"mods\":\"\"}") == -1, "handle_key(null) should return -1");
    CHECK(loom_last_error(NULL) == NULL, "last_error(null) should return null");
    CHECK(loom_create(9999) == NULL, "create with an unknown game id should return null");

    printf("ALL SMOKE CHECKS PASSED\n");
    return 0;
}
