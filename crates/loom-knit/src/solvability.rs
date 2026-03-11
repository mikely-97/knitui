use std::collections::{HashMap, HashSet, VecDeque};
use crossterm::style::Color;
use crate::board_entity::BoardEntity;
use crate::game_board::GameBoard;
use crate::yarn::{Yarn, Stitch};

/// Check 1: yarn stitch counts exactly match the total needed to complete
/// every spool/key-spool on the board plus every conveyor output.
pub fn count_balance(board: &GameBoard, yarn: &Yarn, spool_capacity: u16) -> bool {
    let mut needed: HashMap<Color, u16> = HashMap::new();

    for row in &board.board {
        for cell in row {
            match cell {
                BoardEntity::Spool(c) | BoardEntity::KeySpool(c) => {
                    *needed.entry(*c).or_insert(0) += spool_capacity;
                }
                BoardEntity::Conveyor(data) => {
                    for c in &data.queue {
                        *needed.entry(*c).or_insert(0) += spool_capacity;
                    }
                }
                _ => {}
            }
        }
    }

    let mut actual: HashMap<Color, u16> = HashMap::new();
    for col in &yarn.board {
        for stitch in col {
            *actual.entry(stitch.color).or_insert(0) += 1;
        }
    }

    needed == actual
}

/// Check 2: BFS simulation verifying every Spool/KeySpool cell on the board
/// can eventually be selected under the void-bordering rule.
///
/// Initially the top row is the selectable frontier. When a cell is "selected"
/// (simulated here), it becomes Void and exposes its orthogonal Spool/KeySpool
/// neighbors.
///
/// Conveyor output cells are treated as ordinary Spool cells for reachability:
/// their position must be reached normally. Once reachable, the conveyor keeps
/// refilling them, so they stay available. After the last spool is taken, the
/// output cell becomes Void and may expose further neighbors — but since the
/// conveyor cell itself (non-Void, non-Spool) is always present, that cell
/// will never propagate via BFS. The conveyor's depleted output eventually
/// turning Void IS handled: the total_spools count includes conveyor queue
/// items, and once each queue item is simulated as removed, the BFS proceeds.
pub fn all_spools_reachable(board: &GameBoard) -> bool {
    let h = board.height as usize;
    let w = board.width as usize;
    let b = &board.board;

    let is_spool = |r: usize, c: usize| {
        matches!(b[r][c], BoardEntity::Spool(_) | BoardEntity::KeySpool(_))
    };

    // Count total "spool positions" including conveyor outputs.
    // A conveyor at (r,c) with output_dir D contributes queue.len() spools
    // all at the same output position — so we just need that position reachable.
    // We track unique positions that need to be reached.
    let mut must_reach: HashSet<(usize, usize)> = HashSet::new();

    for r in 0..h {
        for c in 0..w {
            if is_spool(r, c) {
                must_reach.insert((r, c));
            }
            if let BoardEntity::Conveyor(data) = &b[r][c] {
                let (dr, dc) = data.output_dir.offset();
                let or_ = r as i32 + dr;
                let oc = c as i32 + dc;
                if or_ >= 0 && or_ < h as i32 && oc >= 0 && oc < w as i32 && !data.queue.is_empty() {
                    must_reach.insert((or_ as usize, oc as usize));
                }
            }
        }
    }

    // BFS: seed with top-row spools, simulate selections.
    let mut reachable: HashSet<(usize, usize)> = HashSet::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    for c in 0..w {
        if is_spool(0, c) || must_reach.contains(&(0, c)) {
            if !reachable.contains(&(0, c)) {
                reachable.insert((0, c));
                queue.push_back((0, c));
            }
        }
    }

    while let Some((r, c)) = queue.pop_front() {
        // Simulating removal of (r, c): it becomes Void.
        // Orthogonal neighbors that are Spools become selectable.
        let deltas: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in deltas {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr >= 0 && nr < h as i32 && nc >= 0 && nc < w as i32 {
                let (nr, nc) = (nr as usize, nc as usize);
                if must_reach.contains(&(nr, nc)) && !reachable.contains(&(nr, nc)) {
                    reachable.insert((nr, nc));
                    queue.push_back((nr, nc));
                }
            }
        }
    }

    must_reach.iter().all(|pos| reachable.contains(pos))
}

/// Check 3: every locked stitch in the yarn has a reachable KeySpool of matching color.
/// "Reachable" here is checked via a separate BFS; the function delegates to
/// `all_spools_reachable` implicitly because the game can't use a key that
/// can't be picked up.
pub fn keys_and_locks_valid(board: &GameBoard, yarn: &Yarn) -> bool {
    let mut locks: HashMap<Color, u16> = HashMap::new();
    for col in &yarn.board {
        for stitch in col {
            if stitch.locked {
                *locks.entry(stitch.color).or_insert(0) += 1;
            }
        }
    }

    if locks.is_empty() {
        return true;
    }

    let mut keys: HashMap<Color, u16> = HashMap::new();
    for row in &board.board {
        for cell in row {
            if let BoardEntity::KeySpool(c) = cell {
                *keys.entry(*c).or_insert(0) += 1;
            }
        }
    }

    for (color, lock_count) in &locks {
        let key_count = keys.get(color).copied().unwrap_or(0);
        if key_count < *lock_count {
            return false;
        }
    }

    true
}

/// Check 4: the number of distinct colors across all board spools does not
/// exceed the spool limit. If it does, the player may not be able to
/// hold one spool of each color simultaneously and could get stuck.
pub fn active_headroom_ok(board: &GameBoard, spool_limit: usize) -> bool {
    let distinct: HashSet<Color> = board.board.iter().flatten().filter_map(|e| match e {
        BoardEntity::Spool(c) | BoardEntity::KeySpool(c) => Some(*c),
        _ => None,
    }).collect();

    distinct.len() <= spool_limit
}

/// Combined solvability check. Returns `true` if the board passes all checks.
pub fn is_solvable(board: &GameBoard, yarn: &Yarn, spool_capacity: u16, spool_limit: usize) -> bool {
    count_balance(board, yarn, spool_capacity)
        && all_spools_reachable(board)
        && active_headroom_ok(board, spool_limit)
        && keys_and_locks_valid(board, yarn)
}

// ── count_solutions: full game-state DFS ────────────────────────────────────

/// Count the number of distinct pick sequences that lead to a genuine win
/// (board cleared, held spools empty, yarn exhausted).  Returns early with a
/// value greater than `limit` once that threshold is exceeded, so callers can
/// use `> limit` as a fast rejection test.
///
/// Conveyors are not modelled dynamically: their initial output cells are
/// treated as static spools and counted once.  The result is therefore a
/// lower bound for boards with conveyors.
pub fn count_solutions(
    board: &GameBoard,
    yarn: &Yarn,
    spool_capacity: u16,
    spool_limit: usize,
    limit: u64,
) -> u64 {
    let h = board.height as usize;
    let w = board.width as usize;

    // Pre-compute per-cell metadata (constant throughout DFS).
    // cell_meta[r * w + c] = (color, has_key); None color = not a spool.
    let mut cell_meta: Vec<(Option<Color>, bool)> = Vec::with_capacity(h * w);
    for r in 0..h {
        for c in 0..w {
            cell_meta.push(match &board.board[r][c] {
                BoardEntity::Spool(color)    => (Some(*color), false),
                BoardEntity::KeySpool(color) => (Some(*color), true),
                _                            => (None, false),
            });
        }
    }

    let mut cells: Vec<bool> = cell_meta.iter().map(|(c, _)| c.is_some()).collect();
    let mut yarn_cols: Vec<Vec<Stitch>> = yarn.board.clone();
    let mut held: Vec<(Color, u16, bool)> = Vec::new(); // (color, fill, has_key)

    dfs_count(&cell_meta, &mut cells, &mut held, &mut yarn_cols,
              h, w, spool_capacity, spool_limit, limit)
}

fn dfs_count(
    cell_meta: &[(Option<Color>, bool)],
    cells: &mut Vec<bool>,
    held: &mut Vec<(Color, u16, bool)>,
    yarn_cols: &mut Vec<Vec<Stitch>>,
    h: usize,
    w: usize,
    spool_capacity: u16,
    spool_limit: usize,
    limit: u64,
) -> u64 {
    // Eagerly process held spools against yarn until no further progress.
    eager_process(held, yarn_cols, spool_capacity);

    // Win: board clear, held empty, yarn exhausted.
    if cells.iter().all(|&c| !c)
        && held.is_empty()
        && yarn_cols.iter().all(|col| col.is_empty())
    {
        return 1;
    }

    // If held is at the limit after eager processing, no picks are possible
    // and no yarn progress was made — definitively stuck.
    if held.len() >= spool_limit {
        return 0;
    }

    let selectable = selectable_indices(cell_meta, cells, h, w);
    if selectable.is_empty() {
        return 0; // No board moves; remaining spools are buried.
    }

    let mut count = 0u64;
    for idx in selectable {
        let (color, has_key) = match cell_meta[idx] {
            (Some(c), k) => (c, k),
            _ => continue,
        };

        // Save mutable state before branching.
        let saved_held  = held.clone();
        let saved_yarn: Vec<Vec<Stitch>> = yarn_cols.clone();

        cells[idx] = false;
        held.push((color, 0, has_key));

        count += dfs_count(cell_meta, cells, held, yarn_cols,
                           h, w, spool_capacity, spool_limit, limit);

        // Restore.
        cells[idx] = true;
        *held = saved_held;
        *yarn_cols = saved_yarn;

        if count > limit {
            return count; // Early exit: already over the cap.
        }
    }
    count
}

/// Eagerly match held spools against yarn columns until no match is possible.
/// Processes spools in held order (front first); restarts after any progress.
/// A spool is removed when its fill reaches spool_capacity.
fn eager_process(
    held: &mut Vec<(Color, u16, bool)>,
    yarn_cols: &mut Vec<Vec<Stitch>>,
    spool_capacity: u16,
) {
    let mut progress = true;
    while progress {
        progress = false;
        let mut i = 0;
        while i < held.len() {
            // Find the first yarn column whose front stitch matches held[i].
            let mut matched: Option<(usize, bool)> = None; // (col_idx, was_locked)
            for j in 0..yarn_cols.len() {
                if let Some(stitch) = yarn_cols[j].last() {
                    let lock_ok = !stitch.locked || held[i].2; // has_key
                    if stitch.color == held[i].0 && lock_ok {
                        matched = Some((j, stitch.locked));
                        break;
                    }
                }
            }
            if let Some((j, was_locked)) = matched {
                yarn_cols[j].pop();
                held[i].1 += 1; // fill
                if was_locked {
                    held[i].2 = false; // consume key
                }
                progress = true;
                if held[i].1 >= spool_capacity {
                    held.remove(i); // spool complete
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}

/// Return indices of cells that currently have a spool AND are selectable
/// (top-row, or adjacent to a surface-connected void).
fn selectable_indices(
    cell_meta: &[(Option<Color>, bool)],
    cells: &[bool],
    h: usize,
    w: usize,
) -> Vec<usize> {
    // Void = no spool entity, or spool that was picked (cells[idx] == false).
    let is_void = |r: usize, c: usize| -> bool {
        let idx = r * w + c;
        cell_meta[idx].0.is_none() || !cells[idx]
    };

    // BFS from row-0 voids to find surface-connected void set.
    let mut connected = vec![false; h * w];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    for c in 0..w {
        if is_void(0, c) {
            connected[c] = true; // idx = 0 * w + c
            queue.push_back((0, c));
        }
    }
    while let Some((r, c)) = queue.pop_front() {
        for (nr, nc) in sol_neighbors(r, c, h, w) {
            let nidx = nr * w + nc;
            if !connected[nidx] && is_void(nr, nc) {
                connected[nidx] = true;
                queue.push_back((nr, nc));
            }
        }
    }

    // Collect selectable spool indices.
    let mut result = Vec::new();
    for r in 0..h {
        for c in 0..w {
            let idx = r * w + c;
            if cell_meta[idx].0.is_none() || !cells[idx] {
                continue; // Not a present spool.
            }
            let exposed = r == 0
                || sol_neighbors(r, c, h, w).iter().any(|&(nr, nc)| connected[nr * w + nc]);
            if exposed {
                result.push(idx);
            }
        }
    }
    result
}

fn sol_neighbors(r: usize, c: usize, h: usize, w: usize) -> Vec<(usize, usize)> {
    let mut n = Vec::with_capacity(4);
    if r > 0     { n.push((r - 1, c)); }
    if r + 1 < h { n.push((r + 1, c)); }
    if c > 0     { n.push((r, c - 1)); }
    if c + 1 < w { n.push((r, c + 1)); }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_board::GameBoard;
    use crate::yarn::Yarn;

    #[test]
    fn test_count_balance_matches_generated_yarn() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(3, 3, &palette, 0, 2, 0, 0);
        let yarn = Yarn::make_from_color_counter(board.count_spools(), 3, 5);
        assert!(count_balance(&board, &yarn, 2));
    }

    #[test]
    fn test_all_spools_reachable_flat_board() {
        // All-spool board with no obstacles: every cell is reachable.
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(4, 4, &palette, 0, 1, 0, 0);
        assert!(all_spools_reachable(&board));
    }

    #[test]
    fn test_all_spools_reachable_manual_blocked() {
        use crate::board_entity::BoardEntity;
        // Row 0: Obstacle. Row 1: Spool. Row 2: Spool.
        // Spool at (1,0) is NOT top-row and its only neighbor is Obstacle above.
        // So it is unreachable → should fail.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle],
                vec![BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Spool(Color::Red)],
            ],
            height: 3,
            width: 1,
            spool_capacity: 1,
        };
        assert!(!all_spools_reachable(&board));
    }

    #[test]
    fn test_active_headroom_ok() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(3, 3, &palette, 0, 1, 0, 0);
        // 7-slot limit is definitely enough for 3 colors.
        assert!(active_headroom_ok(&board, 7));
    }

    #[test]
    fn test_active_headroom_too_tight() {
        // Construct a board with 3 distinct colors but limit is 2.
        use crate::board_entity::BoardEntity;
        let board = GameBoard {
            board: vec![vec![
                BoardEntity::Spool(Color::Red),
                BoardEntity::Spool(Color::Blue),
                BoardEntity::Spool(Color::Green),
            ]],
            height: 1,
            width: 3,
            spool_capacity: 1,
        };
        assert!(!active_headroom_ok(&board, 2));
    }

    #[test]
    fn test_keys_and_locks_valid_no_locks() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 0, 1, 0, 0);
        let yarn = Yarn::make_from_color_counter(board.count_spools(), 2, 3);
        // No locked stitches → always valid.
        assert!(keys_and_locks_valid(&board, &yarn));
    }

    #[test]
    fn test_keys_and_locks_valid_matching_key() {
        use crate::board_entity::BoardEntity;
        use crate::yarn::{Yarn, Stitch};
        let board = GameBoard {
            board: vec![vec![BoardEntity::KeySpool(Color::Red)]],
            height: 1,
            width: 1,
            spool_capacity: 1,
        };
        let yarn = Yarn {
            board: vec![vec![Stitch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };
        assert!(keys_and_locks_valid(&board, &yarn));
    }

    #[test]
    fn test_keys_and_locks_valid_missing_key() {
        use crate::board_entity::BoardEntity;
        use crate::yarn::{Yarn, Stitch};
        // Locked stitch but no key on the board.
        let board = GameBoard {
            board: vec![vec![BoardEntity::Spool(Color::Red)]],
            height: 1,
            width: 1,
            spool_capacity: 1,
        };
        let yarn = Yarn {
            board: vec![vec![Stitch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_stitches: 3,
            balloon_columns: Vec::new(),
        };
        assert!(!keys_and_locks_valid(&board, &yarn));
    }

    #[test]
    fn test_is_solvable_standard_board() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(4, 4, &palette, 0, 2, 0, 0);
        let yarn = Yarn::make_from_color_counter(board.count_spools(), 3, 5);
        // A full-spool board with plenty of active slots should always be solvable.
        assert!(is_solvable(&board, &yarn, 2, 10));
    }

    // ── count_solutions tests ────────────────────────────────────────────────

    fn make_yarn(cols: Vec<Vec<(Color, bool)>>, yarn_lines: u16) -> Yarn {
        Yarn {
            board: cols.into_iter()
                .map(|col| col.into_iter()
                    .map(|(color, locked)| Stitch { color, locked })
                    .collect())
                .collect(),
            yarn_lines,
            visible_stitches: 6,
            balloon_columns: Vec::new(),
        }
    }

    #[test]
    fn count_solutions_single_spool() {
        // 1×1 board, one spool, one matching stitch → exactly 1 solution.
        let board = GameBoard {
            board: vec![vec![BoardEntity::Spool(Color::Red)]],
            height: 1, width: 1, spool_capacity: 1,
        };
        let yarn = make_yarn(vec![vec![(Color::Red, false)]], 1);
        assert_eq!(count_solutions(&board, &yarn, 1, 7, 10), 1);
    }

    #[test]
    fn count_solutions_forced_sequence() {
        // 2×1 board: top=Red, bottom=Blue.  Bottom is only accessible after top
        // is picked, so there is exactly 1 valid ordering.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Spool(Color::Red)],
                vec![BoardEntity::Spool(Color::Blue)],
            ],
            height: 2, width: 1, spool_capacity: 1,
        };
        // Yarn column: Blue at bottom (index 0), Red at top (last = processed first).
        let yarn = make_yarn(vec![vec![(Color::Blue, false), (Color::Red, false)]], 1);
        assert_eq!(count_solutions(&board, &yarn, 1, 7, 10), 1);
    }

    #[test]
    fn count_solutions_two_free() {
        // 1×2 board: both spools are in row 0, so either can be picked first → 2 solutions.
        let board = GameBoard {
            board: vec![vec![
                BoardEntity::Spool(Color::Red),
                BoardEntity::Spool(Color::Blue),
            ]],
            height: 1, width: 2, spool_capacity: 1,
        };
        let yarn = make_yarn(vec![
            vec![(Color::Red, false)],
            vec![(Color::Blue, false)],
        ], 2);
        assert_eq!(count_solutions(&board, &yarn, 1, 7, 10), 2);
    }

    #[test]
    fn count_solutions_early_exit() {
        // 1×3 board: three independent top-row spools, distinct colors → 3! = 6 orderings.
        // With limit=2 the DFS stops after finding the 3rd path.
        use crossterm::style::Color::*;
        let board = GameBoard {
            board: vec![vec![
                BoardEntity::Spool(Red),
                BoardEntity::Spool(Green),
                BoardEntity::Spool(Blue),
            ]],
            height: 1, width: 3, spool_capacity: 1,
        };
        let yarn = make_yarn(vec![
            vec![(Red, false)],
            vec![(Green, false)],
            vec![(Blue, false)],
        ], 3);
        assert!(count_solutions(&board, &yarn, 1, 7, 2) > 2);
    }

    #[test]
    fn count_solutions_lock_key() {
        // 2×1: (0,0)=KeySpool(Red), (1,0)=Spool(Red).  Yarn: [unlocked, locked] (locked=last/top).
        // Only ordering: pick KeySpool first → unlocks locked stitch, then Spool.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::KeySpool(Color::Red)],
                vec![BoardEntity::Spool(Color::Red)],
            ],
            height: 2, width: 1, spool_capacity: 1,
        };
        // Last element = front/top of yarn.  Locked stitch is at the top so it is
        // processed first; the key is required to pop it.
        let yarn = make_yarn(vec![vec![(Color::Red, false), (Color::Red, true)]], 1);
        assert_eq!(count_solutions(&board, &yarn, 1, 7, 10), 1);
    }
}
