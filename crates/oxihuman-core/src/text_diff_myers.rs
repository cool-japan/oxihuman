// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Myers diff algorithm for text lines.
//!
//! Implements the O(ND) algorithm from:
//! Myers, E.W. (1986). "An O(ND) Difference Algorithm and Its Variations".
//! *Algorithmica* 1(1–4): 251–266.

/// One edit operation in a diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    Equal(String),
    Insert(String),
    Delete(String),
}

/// Myers diff state.
pub struct MyersDiff {
    ops: Vec<EditOp>,
}

impl MyersDiff {
    pub fn new() -> Self {
        MyersDiff { ops: Vec::new() }
    }

    pub fn ops(&self) -> &[EditOp] {
        &self.ops
    }
}

impl Default for MyersDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the Myers O(ND) forward phase.
///
/// Returns `(d, trace)` where `d` is the edit distance and `trace[d]` is the
/// V-array snapshot taken *before* the d-th round of extensions (so that the
/// backtracker can recover which diagonal was active at each depth).
///
/// V is indexed by `k + max` to avoid negative array indices (k = x − y,
/// −max ≤ k ≤ max).  Only odd/even parities matching the current D are
/// ever written, matching the standard Myers presentation.
fn myers_forward(old: &[&str], new: &[&str]) -> (usize, Vec<Vec<i64>>) {
    let n = old.len();
    let m = new.len();
    let max = n + m;

    // V[k + max] = furthest x reached on diagonal k.
    // Allocate 2*max+1 slots; initialise to 0, but the sentinel at k=1
    // (index max+1) needs to be 0 so the first step lands at (0,0) via
    // "go down from k=1".
    let mut v: Vec<i64> = vec![0i64; 2 * max + 1];
    let mut trace: Vec<Vec<i64>> = Vec::with_capacity(max + 1);

    for d in 0..=max {
        // Snapshot V *before* this round so backtracking can use it.
        trace.push(v.clone());

        for k_raw in (-(d as i64)..=(d as i64)).step_by(2) {
            let ki = (k_raw + max as i64) as usize;

            // Choose the starting x: either extend the snake from k+1 (move
            // down → y increases, insert from new) or from k-1 (move right →
            // x increases, delete from old).
            let go_down = k_raw == -(d as i64)
                || (k_raw != d as i64
                    && v.get(ki.wrapping_sub(1)).copied().unwrap_or(i64::MIN)
                        < v.get(ki + 1).copied().unwrap_or(i64::MIN));

            let mut x: usize = if go_down {
                v[ki + 1] as usize // come from k+1: same x (insert)
            } else {
                v[ki - 1] as usize + 1 // come from k-1: advance x (delete)
            };
            let mut y = (x as i64 - k_raw) as usize;

            // Extend the snake along equal elements.
            while x < n && y < m && old[x] == new[y] {
                x += 1;
                y += 1;
            }

            v[ki] = x as i64;

            if x >= n && y >= m {
                // We have found a shortest edit path of length d.
                // Push the final V so the backtracker has it at index d+1
                // (it reads trace[d] for the state *entering* round d).
                // Actually trace already has d+1 entries (indices 0..=d);
                // the backtracker iterates d downward and reads trace[d].
                return (d, trace);
            }
        }
    }

    (max, trace)
}

/// Backtrack through the recorded trace to reconstruct the sequence of
/// `EditOp`s.  The algorithm works backwards from (n, m) to (0, 0),
/// peeling off one edit (plus its preceding snake) per depth level.
fn reconstruct(old: &[&str], new: &[&str], trace: &[Vec<i64>], max: usize) -> Vec<EditOp> {
    let n = old.len();
    let m = new.len();

    let mut ops: Vec<EditOp> = Vec::new();
    let mut x = n as i64;
    let mut y = m as i64;

    for d in (1..trace.len()).rev() {
        let v = &trace[d];
        let k = x - y;
        let ki = (k + max as i64) as usize;

        // Mirror the choice made during the forward phase at depth d.
        let go_down = k == -(d as i64)
            || (k != d as i64
                && v.get(ki.wrapping_sub(1)).copied().unwrap_or(i64::MIN)
                    < v.get(ki + 1).copied().unwrap_or(i64::MIN));

        let prev_k = if go_down { k + 1 } else { k - 1 };
        let prev_ki = (prev_k + max as i64) as usize;
        let prev_x = v[prev_ki];
        let prev_y = prev_x - prev_k;

        // Snake: equal elements from the end of the previous edit to the start
        // of the current position.  After the edit we are at
        // (prev_x + delta_x, prev_y + delta_y) and the snake extends from
        // there up to (x, y).
        let snake_x0 = if go_down { prev_x } else { prev_x + 1 };
        let snake_y0 = snake_x0 - k;

        // Emit equal ops in reverse (we prepend or push-then-reverse).
        let mut cx = x;
        let mut cy = y;
        while cx > snake_x0 && cy > snake_y0 && cx > 0 && cy > 0 {
            cx -= 1;
            cy -= 1;
            ops.push(EditOp::Equal(old[cx as usize].to_string()));
        }

        // Emit the single edit that preceded the snake.
        if go_down {
            // Came down from diagonal k+1: insert new[prev_y].
            ops.push(EditOp::Insert(new[prev_y as usize].to_string()));
        } else {
            // Came right from diagonal k-1: delete old[prev_x].
            ops.push(EditOp::Delete(old[prev_x as usize].to_string()));
        }

        x = prev_x;
        y = prev_y;
    }

    // Any remaining (x, y) > (0, 0) after consuming all edits are equal ops
    // along the initial snake (when d=0 the whole content may be identical).
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        ops.push(EditOp::Equal(old[x as usize].to_string()));
    }

    // We built operations backwards; reverse for chronological order.
    ops.reverse();
    ops
}

/// Compute the edit distance between two line slices (O(ND) Myers).
pub fn edit_distance(a: &[&str], b: &[&str]) -> usize {
    if a == b {
        return 0;
    }
    let (d, _trace) = myers_forward(a, b);
    d
}

/// Compute Myers diff between two lists of lines.
pub fn diff_lines<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<EditOp> {
    if old.is_empty() && new.is_empty() {
        return Vec::new();
    }
    let n = old.len();
    let m = new.len();
    let max = n + m;

    let (d, trace) = myers_forward(old, new);

    if d == 0 {
        // Sequences are identical.
        return old.iter().map(|s| EditOp::Equal(s.to_string())).collect();
    }

    reconstruct(old, new, &trace, max)
}

/// Count insertions and deletions in a diff result.
pub fn diff_stats(ops: &[EditOp]) -> (usize, usize) {
    let mut ins = 0usize;
    let mut del = 0usize;
    for op in ops {
        match op {
            EditOp::Insert(_) => ins += 1,
            EditOp::Delete(_) => del += 1,
            EditOp::Equal(_) => {}
        }
    }
    (ins, del)
}

/// Return true if a and b are identical (zero diff).
pub fn is_same(a: &[&str], b: &[&str]) -> bool {
    a == b
}

/// Reconstruct the "new" file from a diff.
pub fn apply_diff(ops: &[EditOp]) -> Vec<String> {
    let mut result = Vec::new();
    for op in ops {
        match op {
            EditOp::Equal(s) | EditOp::Insert(s) => result.push(s.clone()),
            EditOp::Delete(_) => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_lines() {
        let a = vec!["hello", "world"];
        assert!(is_same(&a, &a));
    }

    #[test]
    fn test_diff_empty() {
        let ops = diff_lines(&[], &[]);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_diff_insert_only() {
        let ops = diff_lines(&[], &["new"]);
        let (ins, del) = diff_stats(&ops);
        assert_eq!(ins, 1);
        assert_eq!(del, 0);
    }

    #[test]
    fn test_diff_delete_only() {
        let ops = diff_lines(&["old"], &[]);
        let (ins, del) = diff_stats(&ops);
        assert_eq!(ins, 0);
        assert_eq!(del, 1);
    }

    #[test]
    fn test_diff_common_prefix() {
        let ops = diff_lines(&["a", "b", "c"], &["a", "b", "d"]);
        // First two lines are equal.
        let eq_count = ops.iter().filter(|o| matches!(o, EditOp::Equal(_))).count();
        assert_eq!(eq_count, 2);
    }

    #[test]
    fn test_apply_diff_roundtrip() {
        let ops = diff_lines(&["x"], &["x", "y"]);
        let result = apply_diff(&ops);
        assert!(result.contains(&"x".to_string()));
        assert!(result.contains(&"y".to_string()));
    }

    #[test]
    fn test_edit_distance_zero() {
        assert_eq!(edit_distance(&["a"], &["a"]), 0);
    }

    #[test]
    fn test_edit_distance_non_zero() {
        let d = edit_distance(&["a", "b"], &["c"]);
        assert!(d > 0);
    }

    #[test]
    fn test_myers_diff_new() {
        let md = MyersDiff::new();
        assert!(md.ops().is_empty());
    }

    #[test]
    fn test_diff_stats_equal_only() {
        let ops = diff_lines(&["a"], &["a"]);
        let (ins, del) = diff_stats(&ops);
        assert_eq!(ins, 0);
        assert_eq!(del, 0);
    }

    /// Myers must find the optimal (shortest) edit path, not a naive
    /// prefix-truncation.  old=["a","b","c","d"], new=["a","c","d","e"]:
    /// the shortest edit is Delete("b") + Insert("e"), keeping a, c, d equal.
    #[test]
    fn test_diff_interleaved() {
        let old = ["a", "b", "c", "d"];
        let new = ["a", "c", "d", "e"];
        let ops = diff_lines(&old, &new);

        // Verify the reconstructed new sequence is correct.
        let applied = apply_diff(&ops);
        assert_eq!(applied, vec!["a", "c", "d", "e"]);

        let (ins, del) = diff_stats(&ops);
        // Optimal edit: 1 delete ("b") + 1 insert ("e").
        assert_eq!(del, 1, "expected exactly 1 delete, got {del}");
        assert_eq!(ins, 1, "expected exactly 1 insert, got {ins}");

        // The three elements a, c, d must appear as Equal ops.
        let eq_lines: Vec<&str> = ops
            .iter()
            .filter_map(|o| {
                if let EditOp::Equal(s) = o {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(eq_lines, vec!["a", "c", "d"]);
    }

    /// Deleting a single middle element: ["a","b","c"] → ["a","c"] has
    /// edit distance 1 (just delete "b").
    #[test]
    fn test_edit_distance_abc_to_ac() {
        assert_eq!(edit_distance(&["a", "b", "c"], &["a", "c"]), 1);
    }

    /// Verify that apply_diff(diff_lines(old, new)) always reconstructs `new`
    /// for a richer example.
    #[test]
    fn test_apply_diff_reconstructs_new() {
        let old = ["line1", "line2", "line3", "line4", "line5"];
        let new = ["line1", "lineX", "line3", "lineY", "line5"];
        let ops = diff_lines(&old, &new);
        let result = apply_diff(&ops);
        assert_eq!(result, new.to_vec());
    }

    /// Completely disjoint sequences: all old lines deleted, all new inserted.
    #[test]
    fn test_diff_completely_different() {
        let old = ["a", "b"];
        let new = ["c", "d"];
        let ops = diff_lines(&old, &new);
        let applied = apply_diff(&ops);
        assert_eq!(applied, vec!["c", "d"]);
        let (ins, del) = diff_stats(&ops);
        assert_eq!(ins, 2);
        assert_eq!(del, 2);
    }
}
