/// Integration tests for the knitui-ni binary.
/// Each test invokes the binary via std::process::Command and inspects JSON output.
use std::process::Command;
use serde_json::Value;

fn bin_path() -> String {
    // cargo test builds binaries in target/debug/
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("knitui-ni");
    path.to_string_lossy().into_owned()
}

/// Run knitui-ni with the given args, return (stdout, stderr, exit_code).
fn run(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(bin_path())
        .args(args)
        .output()
        .expect("failed to run knitui-ni");
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.code().unwrap_or(-1),
    )
}

fn parse_ok(stdout: &str) -> Value {
    let v: Value = serde_json::from_str(stdout)
        .unwrap_or_else(|e| panic!("failed to parse JSON: {e}\nstdout: {stdout}"));
    assert_eq!(v["status"], "ok", "expected ok response: {stdout}");
    v
}

fn parse_err(stderr: &str) -> Value {
    let v: Value = serde_json::from_str(stderr)
        .unwrap_or_else(|e| panic!("failed to parse error JSON: {e}\nstderr: {stderr}"));
    assert_eq!(v["status"], "error", "expected error response: {stderr}");
    v
}

/// Helper: create a new game and return (hash, full_response).
fn create_game(extra_args: &[&str]) -> (String, Value) {
    let (stdout, _, code) = run(extra_args);
    assert_eq!(code, 0, "game creation failed: {stdout}");
    let v = parse_ok(&stdout);
    let hash = v["game"].as_str().unwrap().to_string();
    (hash, v)
}

// ── Creation tests ──────────────────────────────────────────────────────────

#[test]
fn test_create_game_default() {
    let (hash, v) = create_game(&[]);
    assert_eq!(hash.len(), 8);
    assert!(hash.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(v["won"], false);
    // state should have board, yarn, held_spools
    let state = &v["state"];
    assert!(state["board"].is_array());
    assert!(state["yarn"].is_array());
    assert!(state["held_spools"].is_array());
    assert_eq!(state["board_height"], 6);
    assert_eq!(state["board_width"], 6);
}

#[test]
fn test_create_game_custom_options() {
    let (_, v) = create_game(&[
        "--board-height", "3", "--board-width", "4", "--spool-capacity", "2",
    ]);
    let state = &v["state"];
    assert_eq!(state["board_height"], 3);
    assert_eq!(state["board_width"], 4);
    assert_eq!(state["spool_capacity"], 2);
    assert_eq!(state["board"].as_array().unwrap().len(), 3);
    assert_eq!(state["board"][0].as_array().unwrap().len(), 4);
}

// ── Move tests ──────────────────────────────────────────────────────────────

#[test]
fn test_move_cursor_right() {
    let (hash, _) = create_game(&["--obstacle-percentage", "0", "--conveyor-percentage", "0"]);
    let (stdout, _, code) = run(&["--game", &hash, "move", "right"]);
    assert_eq!(code, 0);
    let v = parse_ok(&stdout);
    assert_eq!(v["state"]["cursor_col"], 1);
    assert_eq!(v["state"]["cursor_row"], 0);
}

#[test]
fn test_move_cursor_boundary() {
    let (hash, _) = create_game(&["--obstacle-percentage", "0", "--conveyor-percentage", "0"]);
    // Cursor starts at (0,0), moving left should fail
    let (_, stderr, code) = run(&["--game", &hash, "move", "left"]);
    assert_ne!(code, 0);
    let v = parse_err(&stderr);
    assert_eq!(v["code"], "out_of_bounds");
}

// ── Pick tests ──────────────────────────────────────────────────────────────

#[test]
fn test_pick_up_spool() {
    let (hash, _) = create_game(&["--obstacle-percentage", "0", "--conveyor-percentage", "0"]);
    // With 0% obstacles and 0% conveyors, (0,0) should be a Spool — top row is always selectable
    let (stdout, _, code) = run(&["--game", &hash, "pick"]);
    assert_eq!(code, 0);
    let v = parse_ok(&stdout);
    assert_eq!(v["state"]["held_spools"].as_array().unwrap().len(), 1);
}

#[test]
fn test_pick_up_at_obstacle() {
    let (hash, _) = create_game(&["--obstacle-percentage", "100"]);
    // 100% obstacles: picking up should fail with not_a_spool
    let (_, stderr, code) = run(&["--game", &hash, "pick"]);
    assert_ne!(code, 0);
    let v = parse_err(&stderr);
    assert_eq!(v["code"], "not_a_spool");
}

// ── Process tests ───────────────────────────────────────────────────────────

#[test]
fn test_process_spools() {
    let (hash, _) = create_game(&["--obstacle-percentage", "0", "--conveyor-percentage", "0"]);
    // Pick a spool first
    let (_, _, code) = run(&["--game", &hash, "pick"]);
    assert_eq!(code, 0);
    // Then process
    let (stdout, _, code) = run(&["--game", &hash, "process"]);
    assert_eq!(code, 0);
    let v = parse_ok(&stdout);
    // held_spools may still have the spool (if spool_capacity > 1) or be empty
    assert!(v["state"]["held_spools"].is_array());
}

// ── Persistence tests ───────────────────────────────────────────────────────

#[test]
fn test_game_persistence() {
    let (hash, _) = create_game(&["--obstacle-percentage", "0", "--conveyor-percentage", "0"]);
    // Move right
    let (_, _, code) = run(&["--game", &hash, "move", "right"]);
    assert_eq!(code, 0);
    // Move right again — should start from col=1, end at col=2
    let (stdout, _, code) = run(&["--game", &hash, "move", "right"]);
    assert_eq!(code, 0);
    let v = parse_ok(&stdout);
    assert_eq!(v["state"]["cursor_col"], 2);
}

// ── Error cases ─────────────────────────────────────────────────────────────

#[test]
fn test_missing_game_hash() {
    let (_, stderr, code) = run(&["--game", "nonexistent_hash_xyz", "move", "up"]);
    assert_ne!(code, 0);
    let v = parse_err(&stderr);
    assert_eq!(v["code"], "load_failed");
}

#[test]
fn test_no_command_with_game() {
    let (hash, _) = create_game(&[]);
    let (_, stderr, code) = run(&["--game", &hash]);
    assert_ne!(code, 0);
    let v = parse_err(&stderr);
    assert_eq!(v["code"], "no_command");
}
