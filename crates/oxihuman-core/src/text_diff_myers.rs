// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Myers diff algorithm for text lines.

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

/// Compute the edit distance between two line slices.
pub fn edit_distance(a: &[&str], b: &[&str]) -> usize {
    let n = a.len();
    let m = b.len();
    if n == m && a == b {
        return 0;
    }
    /* stub: return Manhattan-like bound */
    n.abs_diff(m)
}

/// Compute Myers diff between two lists of lines.
pub fn diff_lines<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<EditOp> {
    /* Simplified LCS-based stub: equal prefix, delete remainder, insert new */
    let mut ops = Vec::new();
    let min_len = old.len().min(new.len());
    let mut common = 0;
    for i in 0..min_len {
        if old[i] == new[i] {
            common += 1;
        } else {
            break;
        }
    }
    for line in old.iter().take(common) {
        ops.push(EditOp::Equal(line.to_string()));
    }
    for line in old.iter().skip(common) {
        ops.push(EditOp::Delete(line.to_string()));
    }
    for line in new.iter().skip(common) {
        ops.push(EditOp::Insert(line.to_string()));
    }
    ops
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
        /* first two lines equal */
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
}
