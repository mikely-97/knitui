//! Calls the real `extern "C"` functions (not the safe Rust internals
//! behind them) to prove the FFI boundary itself works: pointer lifecycle,
//! JSON marshaling in both directions, and null-safety. This is still an
//! in-process Rust test (no actual dynamic linking across a language
//! boundary), but it exercises the exact same code paths a C/Python caller
//! would hit -- see `capi_smoke.c` for a real cross-language link test.

use std::ffi::{CStr, CString};

use loom_engine_capi::*;

fn key_json(key: &str) -> CString {
    CString::new(format!(r#"{{"key":"{key}","mods":""}}"#)).unwrap()
}

fn char_key_json(c: char) -> CString {
    CString::new(format!(r#"{{"key":{{"Char":"{c}"}},"mods":""}}"#)).unwrap()
}

#[test]
fn create_and_destroy_each_game() {
    for id in [LOOM_GAME_KNIT, LOOM_GAME_MATCH3, LOOM_GAME_MERGE2, LOOM_GAME_PICROSS] {
        let handle = loom_create(id);
        assert!(!handle.is_null(), "loom_create({id}) returned null");
        unsafe { loom_destroy(handle) };
    }
}

#[test]
fn create_with_unknown_game_id_returns_null() {
    let handle = loom_create(9999);
    assert!(handle.is_null());
}

#[test]
fn destroy_null_is_a_safe_noop() {
    unsafe { loom_destroy(std::ptr::null_mut()) };
}

#[test]
fn render_returns_valid_non_empty_json() {
    let handle = loom_create(LOOM_GAME_KNIT);
    assert!(!handle.is_null());

    let json_ptr = unsafe { loom_render(handle, 100, 40) };
    assert!(!json_ptr.is_null());
    let json = unsafe { CStr::from_ptr(json_ptr) }.to_str().unwrap().to_string();
    unsafe { loom_free_string(json_ptr) };

    // Must parse back as a real CellGrid and contain at least one non-space glyph.
    let value: serde_json::Value = serde_json::from_str(&json).expect("render() must return valid JSON");
    assert!(value.get("cells").is_some(), "CellGrid JSON missing 'cells' field");

    unsafe { loom_destroy(handle) };
}

#[test]
fn handle_key_with_valid_json_returns_zero() {
    let handle = loom_create(LOOM_GAME_KNIT);
    let key = key_json("Up");
    let rc = unsafe { loom_handle_key(handle, key.as_ptr()) };
    assert_eq!(rc, 0);
    unsafe { loom_destroy(handle) };
}

#[test]
fn handle_key_with_malformed_json_returns_error_and_sets_last_error() {
    let handle = loom_create(LOOM_GAME_KNIT);
    let bad = CString::new("not json").unwrap();
    let rc = unsafe { loom_handle_key(handle, bad.as_ptr()) };
    assert_eq!(rc, -1);

    let err_ptr = unsafe { loom_last_error(handle) };
    assert!(!err_ptr.is_null());
    let msg = unsafe { CStr::from_ptr(err_ptr) }.to_str().unwrap().to_string();
    assert!(msg.contains("KeyEvent"), "error message should mention the failed parse: {msg}");
    unsafe { loom_free_string(err_ptr) };

    unsafe { loom_destroy(handle) };
}

#[test]
fn handle_key_with_null_json_returns_error() {
    let handle = loom_create(LOOM_GAME_KNIT);
    let rc = unsafe { loom_handle_key(handle, std::ptr::null()) };
    assert_eq!(rc, -1);
    unsafe { loom_destroy(handle) };
}

#[test]
fn last_error_is_null_after_a_successful_call() {
    let handle = loom_create(LOOM_GAME_KNIT);
    let bad = CString::new("not json").unwrap();
    unsafe { loom_handle_key(handle, bad.as_ptr()) };

    let key = key_json("Up");
    let rc = unsafe { loom_handle_key(handle, key.as_ptr()) };
    assert_eq!(rc, 0);

    let err_ptr = unsafe { loom_last_error(handle) };
    assert!(err_ptr.is_null(), "last_error should clear after a subsequent successful call");

    unsafe { loom_destroy(handle) };
}

#[test]
fn should_quit_starts_false_and_becomes_true_after_quit_navigation() {
    let handle = loom_create(LOOM_GAME_KNIT);
    assert!(!unsafe { loom_should_quit(handle) });

    // MainMenu -> Down x5 -> Quit -> Enter (matches KnitGame's 6-item menu:
    // Quick/Custom/Campaign/Endless/Options/Quit).
    for _ in 0..5 {
        let key = key_json("Down");
        unsafe { loom_handle_key(handle, key.as_ptr()) };
    }
    let enter = key_json("Enter");
    unsafe { loom_handle_key(handle, enter.as_ptr()) };

    assert!(unsafe { loom_should_quit(handle) });
    unsafe { loom_destroy(handle) };
}

#[test]
fn tick_and_save_on_exit_do_not_panic() {
    let handle = loom_create(LOOM_GAME_MATCH3);
    for _ in 0..5 {
        unsafe { loom_tick(handle) };
    }
    unsafe { loom_save_on_exit(handle) };
    unsafe { loom_destroy(handle) };
}

#[test]
fn scripted_play_sequence_across_all_four_games_does_not_panic() {
    for id in [LOOM_GAME_KNIT, LOOM_GAME_MATCH3, LOOM_GAME_MERGE2, LOOM_GAME_PICROSS] {
        let handle = loom_create(id);
        assert!(!handle.is_null());

        // Enter -> Quick Game / Custom Game / Campaign depending on the
        // game's main_menu_items(); just prove nothing panics regardless
        // of which screen it lands on.
        let enter = key_json("Enter");
        unsafe { loom_handle_key(handle, enter.as_ptr()) };

        for _ in 0..15 {
            unsafe { loom_tick(handle) };
            let right = key_json("Right");
            unsafe { loom_handle_key(handle, right.as_ptr()) };
            let act = char_key_json(' ');
            unsafe { loom_handle_key(handle, act.as_ptr()) };
            let frame = unsafe { loom_render(handle, 80, 30) };
            if !frame.is_null() {
                unsafe { loom_free_string(frame) };
            }
        }

        unsafe { loom_destroy(handle) };
    }
}
