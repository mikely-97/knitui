"""Type stub for the loom_py native extension (see src/lib.rs for the
authoritative Rust source -- keep this in sync by hand when the PyO3
API changes; PyO3 doesn't generate stubs automatically)."""

GAME_KNIT: int
GAME_MATCH3: int
GAME_MERGE2: int
GAME_PICROSS: int

class LoomGame:
    """A running game instance. Construct with one of the GAME_* module
    constants; loads real persisted settings/campaign/high-score state
    from disk, same as the native terminal build for that game."""

    def __init__(self, game_id: int) -> None: ...

    def handle_key(self, key: str) -> None:
        """key: one of "Up"/"Down"/"Left"/"Right"/"Enter"/"Esc", or a
        single character (e.g. "a", " "). Raises ValueError otherwise."""
        ...

    def tick(self) -> None:
        """Advance background/animation state by one tick."""
        ...

    def render(self, width: int, height: int) -> str:
        """Returns a JSON string: {"width": int, "height": int,
        "cells": [{"glyph": str, "style": {...}}, ...]} (row-major).
        Use json.loads() to parse."""
        ...

    def should_quit(self) -> bool: ...

    def save_on_exit(self) -> None:
        """Persist any in-progress campaign run. Call once before letting
        this object be garbage-collected."""
        ...
