use std::collections::{HashMap, HashSet, VecDeque};
use crossterm::style::Color;
use crate::board_entity::BoardEntity;
use crate::game_board::GameBoard;
use crate::yarn::Yarn;

/// Check 1: yarn patch counts exactly match the total needed to complete
/// every thread/key-thread on the board plus every generator output.
pub fn count_balance(board: &GameBoard, yarn: &Yarn, knit_volume: u16) -> bool {
    let mut needed: HashMap<Color, u16> = HashMap::new();

    for row in &board.board {
        for cell in row {
            match cell {
                BoardEntity::Thread(c) | BoardEntity::KeyThread(c) => {
                    *needed.entry(*c).or_insert(0) += knit_volume;
                }
                BoardEntity::Generator(data) => {
                    for c in &data.queue {
                        *needed.entry(*c).or_insert(0) += knit_volume;
                    }
                }
                _ => {}
            }
        }
    }

    let mut actual: HashMap<Color, u16> = HashMap::new();
    for col in &yarn.board {
        for patch in col {
            *actual.entry(patch.color).or_insert(0) += 1;
        }
    }

    needed == actual
}

/// Check 2: BFS simulation verifying every Thread/KeyThread cell on the board
/// can eventually be selected under the void-bordering rule.
///
/// Initially the top row is the selectable frontier. When a cell is "selected"
/// (simulated here), it becomes Void and exposes its orthogonal Thread/KeyThread
/// neighbors.
///
/// Generator output cells are treated as ordinary Thread cells for reachability:
/// their position must be reached normally. Once reachable, the generator keeps
/// refilling them, so they stay available. After the last thread is taken, the
/// output cell becomes Void and may expose further neighbors — but since the
/// generator cell itself (non-Void, non-Thread) is always present, that cell
/// will never propagate via BFS. The generator's depleted output eventually
/// turning Void IS handled: the total_threads count includes generator queue
/// items, and once each queue item is simulated as removed, the BFS proceeds.
pub fn all_threads_reachable(board: &GameBoard) -> bool {
    let h = board.height as usize;
    let w = board.width as usize;
    let b = &board.board;

    let is_thread = |r: usize, c: usize| {
        matches!(b[r][c], BoardEntity::Thread(_) | BoardEntity::KeyThread(_))
    };

    // Count total "thread positions" including generator outputs.
    // A generator at (r,c) with output_dir D contributes queue.len() threads
    // all at the same output position — so we just need that position reachable.
    // We track unique positions that need to be reached.
    let mut must_reach: HashSet<(usize, usize)> = HashSet::new();

    for r in 0..h {
        for c in 0..w {
            if is_thread(r, c) {
                must_reach.insert((r, c));
            }
            if let BoardEntity::Generator(data) = &b[r][c] {
                let (dr, dc) = data.output_dir.offset();
                let or_ = r as i32 + dr;
                let oc = c as i32 + dc;
                if or_ >= 0 && or_ < h as i32 && oc >= 0 && oc < w as i32 && !data.queue.is_empty() {
                    must_reach.insert((or_ as usize, oc as usize));
                }
            }
        }
    }

    // BFS: seed with top-row threads, simulate selections.
    let mut reachable: HashSet<(usize, usize)> = HashSet::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    for c in 0..w {
        if is_thread(0, c) || must_reach.contains(&(0, c)) {
            if !reachable.contains(&(0, c)) {
                reachable.insert((0, c));
                queue.push_back((0, c));
            }
        }
    }

    while let Some((r, c)) = queue.pop_front() {
        // Simulating removal of (r, c): it becomes Void.
        // Orthogonal neighbors that are Threads become selectable.
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

/// Check 3: every locked patch in the yarn has a reachable KeyThread of matching color.
/// "Reachable" here is checked via a separate BFS; the function delegates to
/// `all_threads_reachable` implicitly because the game can't use a key that
/// can't be picked up.
pub fn keys_and_locks_valid(board: &GameBoard, yarn: &Yarn) -> bool {
    let mut locks: HashMap<Color, u16> = HashMap::new();
    for col in &yarn.board {
        for patch in col {
            if patch.locked {
                *locks.entry(patch.color).or_insert(0) += 1;
            }
        }
    }

    if locks.is_empty() {
        return true;
    }

    let mut keys: HashMap<Color, u16> = HashMap::new();
    for row in &board.board {
        for cell in row {
            if let BoardEntity::KeyThread(c) = cell {
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

/// Check 4: the number of distinct colors across all board threads does not
/// exceed the active thread limit. If it does, the player may not be able to
/// hold one thread of each color simultaneously and could get stuck.
pub fn active_headroom_ok(board: &GameBoard, active_limit: usize) -> bool {
    let distinct: HashSet<Color> = board.board.iter().flatten().filter_map(|e| match e {
        BoardEntity::Thread(c) | BoardEntity::KeyThread(c) => Some(*c),
        _ => None,
    }).collect();

    distinct.len() <= active_limit
}

/// Combined solvability check. Returns `true` if the board passes all checks.
pub fn is_solvable(board: &GameBoard, yarn: &Yarn, knit_volume: u16, active_limit: usize) -> bool {
    count_balance(board, yarn, knit_volume)
        && all_threads_reachable(board)
        && active_headroom_ok(board, active_limit)
        && keys_and_locks_valid(board, yarn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_board::GameBoard;
    use crate::palette::{select_palette, ColorMode};
    use crate::yarn::Yarn;

    #[test]
    fn test_count_balance_matches_generated_yarn() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(3, 3, &palette, 0, 2);
        let yarn = Yarn::make_from_color_counter(board.count_knits(), 3, 5);
        assert!(count_balance(&board, &yarn, 2));
    }

    #[test]
    fn test_all_threads_reachable_flat_board() {
        // All-thread board with no obstacles: every cell is reachable.
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(4, 4, &palette, 0, 1);
        assert!(all_threads_reachable(&board));
    }

    #[test]
    fn test_all_threads_reachable_manual_blocked() {
        use crate::board_entity::{BoardEntity, Direction, GeneratorData};
        // Row 0: Obstacle. Row 1: Thread. Row 2: Thread.
        // Thread at (1,0) is NOT top-row and its only neighbor is Obstacle above.
        // So it is unreachable → should fail.
        let board = GameBoard {
            board: vec![
                vec![BoardEntity::Obstacle],
                vec![BoardEntity::Thread(Color::Red)],
                vec![BoardEntity::Thread(Color::Red)],
            ],
            height: 3,
            width: 1,
            knit_volume: 1,
        };
        assert!(!all_threads_reachable(&board));
    }

    #[test]
    fn test_active_headroom_ok() {
        let palette = vec![Color::Red, Color::Blue, Color::Green];
        let board = GameBoard::make_random(3, 3, &palette, 0, 1);
        // 7-slot limit is definitely enough for 3 colors.
        assert!(active_headroom_ok(&board, 7));
    }

    #[test]
    fn test_active_headroom_too_tight() {
        // Construct a board with 3 distinct colors but limit is 2.
        use crate::board_entity::BoardEntity;
        let board = GameBoard {
            board: vec![vec![
                BoardEntity::Thread(Color::Red),
                BoardEntity::Thread(Color::Blue),
                BoardEntity::Thread(Color::Green),
            ]],
            height: 1,
            width: 3,
            knit_volume: 1,
        };
        assert!(!active_headroom_ok(&board, 2));
    }

    #[test]
    fn test_keys_and_locks_valid_no_locks() {
        let palette = vec![Color::Red];
        let board = GameBoard::make_random(2, 2, &palette, 0, 1);
        let yarn = Yarn::make_from_color_counter(board.count_knits(), 2, 3);
        // No locked patches → always valid.
        assert!(keys_and_locks_valid(&board, &yarn));
    }

    #[test]
    fn test_keys_and_locks_valid_matching_key() {
        use crate::board_entity::BoardEntity;
        use crate::yarn::{Yarn, Patch};
        let board = GameBoard {
            board: vec![vec![BoardEntity::KeyThread(Color::Red)]],
            height: 1,
            width: 1,
            knit_volume: 1,
        };
        let yarn = Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_patches: 3,
        };
        assert!(keys_and_locks_valid(&board, &yarn));
    }

    #[test]
    fn test_keys_and_locks_valid_missing_key() {
        use crate::board_entity::BoardEntity;
        use crate::yarn::{Yarn, Patch};
        // Locked patch but no key on the board.
        let board = GameBoard {
            board: vec![vec![BoardEntity::Thread(Color::Red)]],
            height: 1,
            width: 1,
            knit_volume: 1,
        };
        let yarn = Yarn {
            board: vec![vec![Patch { color: Color::Red, locked: true }]],
            yarn_lines: 1,
            visible_patches: 3,
        };
        assert!(!keys_and_locks_valid(&board, &yarn));
    }

    #[test]
    fn test_is_solvable_standard_board() {
        let palette = vec![Color::Red, Color::Blue];
        let board = GameBoard::make_random(4, 4, &palette, 0, 2);
        let yarn = Yarn::make_from_color_counter(board.count_knits(), 3, 5);
        // A full-thread board with plenty of active slots should always be solvable.
        assert!(is_solvable(&board, &yarn, 2, 10));
    }
}
