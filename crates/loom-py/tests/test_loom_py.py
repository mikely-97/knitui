"""Real cross-language verification for the PyO3 bindings -- imports and
drives the actual compiled extension module, not a mock. Build+install
first:

    cd crates/loom-py
    maturin build --release
    pip install --force-reinstall ../../target/wheels/loom_py-*.whl
    python tests/test_loom_py.py

Exercises all 4 games through a scripted play sequence (menu navigation,
tick, render, error paths for a bad key name and an unknown game id) and
confirms should_quit()/save_on_exit() don't raise.
"""

import json
import sys

import loom_py


def check(cond, msg):
    if not cond:
        raise AssertionError(f"FAIL: {msg}")


def smoke_one_game(game_id, name):
    print(f"-- {name} --")
    g = loom_py.LoomGame(game_id)
    check(g.should_quit() is False, "should_quit must start false")

    g.handle_key("Enter")

    for _ in range(10):
        g.tick()
        g.handle_key("Right")
        g.handle_key(" ")
        frame = json.loads(g.render(80, 30))
        check("cells" in frame, "render() JSON missing cells field")
        check("width" in frame and "height" in frame, "render() JSON missing width/height")

    try:
        g.handle_key("NotAKey")
        raise AssertionError("expected ValueError for a bad key name")
    except ValueError as e:
        print(f"   (expected) ValueError: {e}")

    g.save_on_exit()
    print("   ok")


def main():
    smoke_one_game(loom_py.GAME_KNIT, "knit")
    smoke_one_game(loom_py.GAME_MATCH3, "match3")
    smoke_one_game(loom_py.GAME_MERGE2, "merge2")
    smoke_one_game(loom_py.GAME_PICROSS, "picross")

    try:
        loom_py.LoomGame(9999)
        raise AssertionError("expected ValueError for unknown game_id")
    except ValueError as e:
        print(f"(expected) ValueError: {e}")

    print("ALL PYTHON SMOKE CHECKS PASSED")


if __name__ == "__main__":
    try:
        main()
    except AssertionError as e:
        print(e, file=sys.stderr)
        sys.exit(1)
