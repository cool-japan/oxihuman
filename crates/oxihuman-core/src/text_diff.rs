#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple line-based text diffing utilities.

/// A single diff operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DiffOp {
    /// A line that is equal in both texts.
    Equal(String),
    /// A line added in the new text.
    Insert(String),
    /// A line removed from the old text.
    Delete(String),
}

/// Result of computing a text diff.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct TextDiff {
    ops: Vec<DiffOp>,
}

/// Compute a line-by-line diff between `old` and `new` text.
///
/// Uses a naive LCS-based approach suitable for small texts.
#[allow(dead_code)]
pub fn compute_text_diff(old: &str, new: &str) -> TextDiff {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let m = old_lines.len();
    let n = new_lines.len();

    // Build LCS table
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    #[allow(clippy::needless_range_loop)]
    for i in 1..=m {
        for j in 1..=n {
            if old_lines[i - 1] == new_lines[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack
    let mut ops = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            ops.push(DiffOp::Equal(old_lines[i - 1].to_string()));
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || dp[i][j - 1] >= dp[i - 1][j]) {
            ops.push(DiffOp::Insert(new_lines[j - 1].to_string()));
            j -= 1;
        } else {
            ops.push(DiffOp::Delete(old_lines[i - 1].to_string()));
            i -= 1;
        }
    }
    ops.reverse();
    TextDiff { ops }
}

/// Apply a diff to reconstruct the new text.
#[allow(dead_code)]
pub fn apply_text_diff(diff: &TextDiff) -> String {
    let mut out = String::new();
    for op in &diff.ops {
        match op {
            DiffOp::Equal(line) | DiffOp::Insert(line) => {
                out.push_str(line);
                out.push('\n');
            }
            DiffOp::Delete(_) => {}
        }
    }
    out
}

/// Count total diff operations.
#[allow(dead_code)]
pub fn diff_op_count(diff: &TextDiff) -> usize {
    diff.ops.len()
}

/// Return true if there are no differences (all ops are Equal).
#[allow(dead_code)]
pub fn diff_is_empty(diff: &TextDiff) -> bool {
    diff.ops.iter().all(|op| matches!(op, DiffOp::Equal(_)))
}

/// Count lines added.
#[allow(dead_code)]
pub fn diff_lines_added(diff: &TextDiff) -> usize {
    diff.ops.iter().filter(|op| matches!(op, DiffOp::Insert(_))).count()
}

/// Count lines removed.
#[allow(dead_code)]
pub fn diff_lines_removed(diff: &TextDiff) -> usize {
    diff.ops.iter().filter(|op| matches!(op, DiffOp::Delete(_))).count()
}

/// Convert a diff to a human-readable unified-style string.
#[allow(dead_code)]
pub fn diff_to_string(diff: &TextDiff) -> String {
    let mut out = String::new();
    for op in &diff.ops {
        match op {
            DiffOp::Equal(line) => { out.push(' '); out.push_str(line); out.push('\n'); }
            DiffOp::Insert(line) => { out.push('+'); out.push_str(line); out.push('\n'); }
            DiffOp::Delete(line) => { out.push('-'); out.push_str(line); out.push('\n'); }
        }
    }
    out
}

/// Invert a diff (swap inserts and deletes).
#[allow(dead_code)]
pub fn invert_diff(diff: &TextDiff) -> TextDiff {
    let ops = diff.ops.iter().map(|op| match op {
        DiffOp::Equal(s) => DiffOp::Equal(s.clone()),
        DiffOp::Insert(s) => DiffOp::Delete(s.clone()),
        DiffOp::Delete(s) => DiffOp::Insert(s.clone()),
    }).collect();
    TextDiff { ops }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_identical() {
        let diff = compute_text_diff("hello\nworld\n", "hello\nworld\n");
        assert!(diff_is_empty(&diff));
    }

    #[test]
    fn test_compute_insert() {
        let diff = compute_text_diff("a\n", "a\nb\n");
        assert_eq!(diff_lines_added(&diff), 1);
        assert_eq!(diff_lines_removed(&diff), 0);
    }

    #[test]
    fn test_compute_delete() {
        let diff = compute_text_diff("a\nb\n", "a\n");
        assert_eq!(diff_lines_added(&diff), 0);
        assert_eq!(diff_lines_removed(&diff), 1);
    }

    #[test]
    fn test_apply_diff() {
        let diff = compute_text_diff("a\nb\n", "a\nc\n");
        let result = apply_text_diff(&diff);
        assert!(result.contains('a'));
        assert!(result.contains('c'));
        assert!(!result.contains('b'));
    }

    #[test]
    fn test_diff_op_count() {
        let diff = compute_text_diff("a\n", "b\n");
        assert!(diff_op_count(&diff) >= 1);
    }

    #[test]
    fn test_diff_to_string() {
        let diff = compute_text_diff("a\n", "a\nb\n");
        let s = diff_to_string(&diff);
        assert!(s.contains('+'));
    }

    #[test]
    fn test_invert_diff() {
        let diff = compute_text_diff("a\n", "a\nb\n");
        let inv = invert_diff(&diff);
        assert_eq!(diff_lines_added(&diff), diff_lines_removed(&inv));
        assert_eq!(diff_lines_removed(&diff), diff_lines_added(&inv));
    }

    #[test]
    fn test_empty_inputs() {
        let diff = compute_text_diff("", "");
        assert_eq!(diff_op_count(&diff), 0);
    }

    #[test]
    fn test_from_empty_to_content() {
        let diff = compute_text_diff("", "line1\nline2\n");
        assert_eq!(diff_lines_added(&diff), 2);
    }
}
