use std::io::{self, BufRead};
use knitui::engine::GameEngine;
use knitui::solvability::{is_solvable, count_solutions};

fn main() {
    let stdin = io::stdin();
    let mut total = 0u64;
    let mut heuristic_pass = 0u64;
    let mut heuristic_fail = 0u64;
    let mut dfs_solvable = 0u64;
    let mut dfs_unsolvable = 0u64;
    let mut max_attempts_hit = 0u64;
    let mut failures: Vec<String> = Vec::new();

    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("read error: {e}");
                continue;
            }
        };
        let line = line.trim();
        if line.is_empty() { continue; }

        total += 1;

        // Parse the NDJSON line from batch-generate
        let parsed: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[board {total}] JSON parse error: {e}");
                failures.push(format!("board {total}: parse error"));
                continue;
            }
        };

        // Extract the state sub-object
        let state_val = if parsed.get("state").is_some() {
            &parsed["state"]
        } else {
            &parsed
        };

        let state_json = serde_json::to_string(state_val).unwrap();
        let engine = match GameEngine::from_json(&state_json) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[board {total}] deserialization error: {e}");
                failures.push(format!("board {total}: deserialization error: {e}"));
                continue;
            }
        };

        let gen_attempts = parsed
            .pointer("/state/generation_attempts")
            .or_else(|| parsed.get("generation_attempts"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if gen_attempts >= 100 {
            max_attempts_hit += 1;
        }

        let config_desc = parsed.get("config")
            .map(|c| format!("{}x{} {}c {}%obs",
                c["board_height"], c["board_width"],
                c["color_number"], c["obstacle_percentage"]))
            .unwrap_or_else(|| format!("{}x{} cap={}",
                engine.board.height, engine.board.width, engine.spool_capacity));

        // Heuristic check
        let heuristic = is_solvable(
            &engine.board, &engine.yarn,
            engine.spool_capacity, engine.spool_limit,
        );
        if heuristic {
            heuristic_pass += 1;
        } else {
            heuristic_fail += 1;
        }

        // Authoritative DFS check (limit=1: just need to know if at least 1 solution)
        let solutions = count_solutions(
            &engine.board, &engine.yarn,
            engine.spool_capacity, engine.spool_limit,
            1,
        );

        if solutions >= 1 {
            dfs_solvable += 1;
        } else {
            dfs_unsolvable += 1;
            let detail = format!(
                "board {total}: UNSOLVABLE ({config_desc}, gen_attempts={gen_attempts}, heuristic={heuristic})"
            );
            eprintln!("{detail}");
            failures.push(detail);
        }

        // Per-board JSON output
        let result = serde_json::json!({
            "board": total,
            "heuristic_pass": heuristic,
            "dfs_solvable": solutions >= 1,
            "generation_attempts": gen_attempts,
            "config": config_desc,
        });
        println!("{}", serde_json::to_string(&result).unwrap());
    }

    // Summary
    eprintln!();
    eprintln!("=== Solvability Check Summary ===");
    eprintln!("Total boards checked:    {total}");
    eprintln!("Heuristic pass:          {heuristic_pass}");
    eprintln!("Heuristic fail:          {heuristic_fail}");
    eprintln!("DFS solvable:            {dfs_solvable}");
    eprintln!("DFS UNSOLVABLE:          {dfs_unsolvable}");
    eprintln!("Generator hit 100 tries: {max_attempts_hit}");
    if !failures.is_empty() {
        eprintln!();
        eprintln!("Failures:");
        for f in &failures {
            eprintln!("  {f}");
        }
    }
    if dfs_unsolvable > 0 {
        std::process::exit(1);
    }
}
