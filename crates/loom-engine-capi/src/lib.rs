//! C ABI for the loom portable game engine (Phase 5). Host-owns-the-loop:
//! the FFI caller is a frontend exactly like `loom-engine-term`'s tui.rs or
//! `loom-engine-web`'s web.rs, just written in another language -- it
//! drives `Shell<G>` in single steps (create -> handle_key/tick/render in
//! a loop -> destroy) and is responsible for its own timing, key mapping,
//! and actually drawing the returned frame.
//!
//! Config, key events, and render frames cross the boundary as JSON
//! strings, matching the serde_json patterns already used for local
//! save/load throughout this codebase (see the pivot's confirmed
//! architecture decisions) -- no bespoke binary struct layout to keep in
//! sync between languages.
//!
//! Every `*mut LoomHandle` returned by `loom_create` must eventually be
//! passed to exactly one `loom_destroy` call. Every `*mut c_char` returned
//! by `loom_render`/`loom_last_error` must eventually be passed to exactly
//! one `loom_free_string` call. Passing a pointer to any other loom_*
//! function afterwards, or to the wrong free function, is undefined
//! behavior -- this is a thin, unsafe C ABI, not a memory-safe wrapper;
//! see the per-language binding crates (planned: PyO3) for a safe surface.

mod erased;

use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};

use loom_engine::game::GameId;
use loom_engine::input::KeyEvent;

use erased::{create_shell, ErasedShell};

/// Opaque handle to a running `Shell<G>` instance, for one of the 4 games.
pub struct LoomHandle {
    shell: Box<dyn ErasedShell>,
    /// Set on the last call that failed, readable via `loom_last_error`
    /// without needing a second out-parameter on every function.
    last_error: RefCell<Option<CString>>,
}

/// Game identifiers for `loom_create`. Must stay in sync with
/// `loom_engine::game::GameId` -- see `game_id_from_u32` below, the one
/// place that mapping is defined.
pub const LOOM_GAME_KNIT: u32 = 0;
pub const LOOM_GAME_MATCH3: u32 = 1;
pub const LOOM_GAME_MERGE2: u32 = 2;
pub const LOOM_GAME_PICROSS: u32 = 3;

fn game_id_from_u32(id: u32) -> Option<GameId> {
    match id {
        LOOM_GAME_KNIT => Some(GameId::Knit),
        LOOM_GAME_MATCH3 => Some(GameId::Match3),
        LOOM_GAME_MERGE2 => Some(GameId::Merge2),
        LOOM_GAME_PICROSS => Some(GameId::Picross),
        _ => None,
    }
}

/// Create a new game instance. `game_id` is one of the `LOOM_GAME_*`
/// constants. Loads real persisted settings/campaign/high-score state from
/// disk (same location the native terminal build uses for that game).
/// Returns null if `game_id` is unrecognized.
///
/// # Safety
/// The returned pointer, if non-null, must eventually be passed to exactly
/// one `loom_destroy` call and to no other function after that.
#[unsafe(no_mangle)]
pub extern "C" fn loom_create(game_id: u32) -> *mut LoomHandle {
    let Some(id) = game_id_from_u32(game_id) else {
        return std::ptr::null_mut();
    };
    let handle = LoomHandle {
        shell: create_shell(id),
        last_error: RefCell::new(None),
    };
    Box::into_raw(Box::new(handle))
}

/// Destroy a handle created by `loom_create`. Does *not* call
/// `loom_save_on_exit` implicitly -- call that first if the in-progress
/// campaign run should be persisted.
///
/// # Safety
/// `handle` must be a pointer previously returned by `loom_create` and not
/// already destroyed. Passing null is safe (no-op).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_destroy(handle: *mut LoomHandle) {
    if handle.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(handle) });
}

/// Handle one key event, JSON-encoded as `loom_engine::input::KeyEvent`.
/// `key` is either a plain string for the 6 non-character keys --
/// `"Up"`/`"Down"`/`"Left"`/`"Right"`/`"Enter"`/`"Esc"` -- or
/// `{"Char":"a"}` for a character key. `mods` is `""` for no modifiers, or
/// bitflags' `serde` string format for combinations, e.g. `"CTRL | SHIFT"`
/// (note: no game currently reads modifiers on any key, so `""` always
/// works in practice). Examples:
/// `{"key":"Up","mods":""}`, `{"key":{"Char":"a"},"mods":""}`.
/// Returns 0 on success, -1 if `handle`/`key_json` is null or the JSON is
/// malformed (check `loom_last_error` for why).
///
/// # Safety
/// `handle` must be a live pointer from `loom_create`. `key_json` must be
/// null or point to a valid, NUL-terminated UTF-8 C string for the
/// duration of this call (not retained afterwards).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_handle_key(handle: *mut LoomHandle, key_json: *const c_char) -> i32 {
    let Some(handle) = (unsafe { handle.as_mut() }) else { return -1 };
    let Some(json) = c_str_to_str(key_json) else {
        set_error(handle, "loom_handle_key: null or invalid UTF-8 key_json");
        return -1;
    };
    match serde_json::from_str::<KeyEvent>(json) {
        Ok(key) => {
            handle.shell.handle_key(key);
            clear_error(handle);
            0
        }
        Err(e) => {
            set_error(handle, &format!("loom_handle_key: invalid KeyEvent JSON: {e}"));
            -1
        }
    }
}

/// Advance background/animation state by one tick (call at whatever
/// cadence the host frontend polls at -- matches how every native tui.rs
/// drives `Shell::tick()` today).
///
/// # Safety
/// `handle` must be a live pointer from `loom_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_tick(handle: *mut LoomHandle) {
    if let Some(handle) = unsafe { handle.as_mut() } {
        handle.shell.tick();
    }
}

/// Render the current frame at `width`x`height` cells, returned as
/// JSON-encoded `loom_engine::render::CellGrid`. Returns null on error
/// (null handle) -- check `loom_last_error`. The returned string must be
/// freed with `loom_free_string`.
///
/// # Safety
/// `handle` must be a live pointer from `loom_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_render(handle: *mut LoomHandle, width: u16, height: u16) -> *mut c_char {
    let Some(handle) = (unsafe { handle.as_mut() }) else { return std::ptr::null_mut() };
    let grid = handle.shell.render(width, height);
    match serde_json::to_string(&grid) {
        Ok(json) => {
            clear_error(handle);
            string_to_c(json)
        }
        Err(e) => {
            set_error(handle, &format!("loom_render: failed to serialize frame: {e}"));
            std::ptr::null_mut()
        }
    }
}

/// True once the game has requested to quit (player selected Quit from the
/// main menu, or equivalent). The host frontend should stop its loop and
/// call `loom_save_on_exit` + `loom_destroy`.
///
/// # Safety
/// `handle` must be a live pointer from `loom_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_should_quit(handle: *mut LoomHandle) -> bool {
    match unsafe { handle.as_ref() } {
        Some(handle) => handle.shell.should_quit(),
        None => true,
    }
}

/// Persist any in-progress campaign run. Call once, right before
/// `loom_destroy`, mirroring every native tui.rs's exit sequence.
///
/// # Safety
/// `handle` must be a live pointer from `loom_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_save_on_exit(handle: *mut LoomHandle) {
    if let Some(handle) = unsafe { handle.as_mut() } {
        handle.shell.save_on_exit();
    }
}

/// Last error message recorded by a call on this handle, or null if the
/// most recent call succeeded. The returned string must be freed with
/// `loom_free_string`.
///
/// # Safety
/// `handle` must be a live pointer from `loom_create`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_last_error(handle: *mut LoomHandle) -> *mut c_char {
    let Some(handle) = (unsafe { handle.as_ref() }) else { return std::ptr::null_mut() };
    match handle.last_error.borrow().as_ref() {
        Some(msg) => msg.clone().into_raw(),
        None => std::ptr::null_mut(),
    }
}

/// Free a string returned by `loom_render` or `loom_last_error`.
///
/// # Safety
/// `s` must be a pointer previously returned by `loom_render` or
/// `loom_last_error` and not already freed. Passing null is safe (no-op).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn loom_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}

fn c_str_to_str<'a>(s: *const c_char) -> Option<&'a str> {
    if s.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(s) }.to_str().ok()
}

fn string_to_c(s: String) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(), // embedded NUL -- shouldn't happen for our JSON output
    }
}

fn set_error(handle: &LoomHandle, msg: &str) {
    *handle.last_error.borrow_mut() = CString::new(msg).ok();
}

fn clear_error(handle: &LoomHandle) {
    *handle.last_error.borrow_mut() = None;
}
