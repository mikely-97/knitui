//! PyO3 bindings for the loom portable game engine (Phase 5, second
//! sub-deliverable after `loom-engine-capi`'s C ABI). Reuses that crate's
//! `ErasedShell`/`create_shell` type-erasure layer directly -- no need to
//! re-derive it -- but gives Python a real, memory-safe class instead of
//! raw pointers: PyO3 handles the object lifetime, so there's no
//! `loom_destroy`/`loom_free_string` equivalent to get wrong, and
//! `handle_key` takes a plain key name instead of requiring the caller to
//! hand-build JSON (including bitflags' string-typed `mods` field, a real
//! gotcha documented in `loom-engine-capi`'s README).
//!
//! Still host-owns-the-loop, same as the C ABI: the Python caller is a
//! frontend, driving `create -> handle_key/tick/render in a loop -> quit`
//! exactly like every native frontend does.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use loom_engine::input::{Key, KeyEvent};
use loom_engine_capi::erased::{create_shell, ErasedShell};
use loom_engine_capi::game_id_from_u32;

/// A running game instance. Construct with one of the `GAME_*` module
/// constants; loads real persisted settings/campaign/high-score state from
/// disk, same as the native terminal build for that game.
///
/// `unsendable`: `Box<dyn ErasedShell>` isn't `Send`/`Sync` (the trait
/// doesn't require it, and proving every `Shell<G>` instantiation actually
/// is would be real, unjustified extra work for a "host owns the loop,
/// single-threaded frontend" use case -- exactly how every native
/// frontend already drives `Shell<G>` today). This means a `LoomGame`
/// object can only be touched from the Python thread that created it;
/// PyO3 raises a clear `RuntimeError` if that's ever violated, rather than
/// silently allowing a data race.
#[pyclass(unsendable)]
struct LoomGame {
    shell: Box<dyn ErasedShell>,
}

#[pymethods]
impl LoomGame {
    #[new]
    fn new(game_id: u32) -> PyResult<Self> {
        let id = game_id_from_u32(game_id)
            .ok_or_else(|| PyValueError::new_err(format!("unknown game_id: {game_id}")))?;
        Ok(Self { shell: create_shell(id) })
    }

    /// Handle one keypress. `key` is one of `"Up"`, `"Down"`, `"Left"`,
    /// `"Right"`, `"Enter"`, `"Esc"`, or a single character (e.g. `"a"`,
    /// `" "`). Raises `ValueError` for anything else.
    fn handle_key(&mut self, key: &str) -> PyResult<()> {
        let k = match key {
            "Up" => Key::Up,
            "Down" => Key::Down,
            "Left" => Key::Left,
            "Right" => Key::Right,
            "Enter" => Key::Enter,
            "Esc" => Key::Esc,
            s if s.chars().count() == 1 => Key::Char(s.chars().next().unwrap()),
            other => {
                return Err(PyValueError::new_err(format!(
                    "unrecognized key {other:?}: expected Up/Down/Left/Right/Enter/Esc or a single character"
                )));
            }
        };
        self.shell.handle_key(KeyEvent::new(k));
        Ok(())
    }

    /// Advance background/animation state by one tick (call at whatever
    /// cadence the host polls at).
    fn tick(&mut self) {
        self.shell.tick();
    }

    /// Render the current frame at `width`x`height` cells, returned as a
    /// JSON string (`loom_engine::render::CellGrid`'s shape: `{"width":
    /// ..,"height": ..,"cells": [{"glyph": "A","style": {"fg": .., "bg":
    /// .., "attrs": ..}}, ...]}`, row-major). Use `json.loads()` on the
    /// Python side.
    fn render(&self, width: u16, height: u16) -> PyResult<String> {
        let grid = self.shell.render(width, height);
        serde_json::to_string(&grid)
            .map_err(|e| PyValueError::new_err(format!("failed to serialize frame: {e}")))
    }

    /// True once the game has requested to quit (player selected Quit from
    /// the main menu, or equivalent). The host should stop its loop and
    /// call `save_on_exit()`.
    fn should_quit(&self) -> bool {
        self.shell.should_quit()
    }

    /// Persist any in-progress campaign run. Call once before dropping the
    /// object, mirroring every native frontend's exit sequence.
    fn save_on_exit(&mut self) {
        self.shell.save_on_exit();
    }
}

#[pymodule]
fn loom_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<LoomGame>()?;
    m.add("GAME_KNIT", loom_engine_capi::LOOM_GAME_KNIT)?;
    m.add("GAME_MATCH3", loom_engine_capi::LOOM_GAME_MATCH3)?;
    m.add("GAME_MERGE2", loom_engine_capi::LOOM_GAME_MERGE2)?;
    m.add("GAME_PICROSS", loom_engine_capi::LOOM_GAME_PICROSS)?;
    Ok(())
}
