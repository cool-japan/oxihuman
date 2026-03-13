// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Myers diff edit script generator.
//!
//! Produces a minimal sequence of edit operations (keep / delete / insert)
//! that transforms one slice of lines into another using the Myers O(ND)
//! algorithm skeleton.

/// A single edit operation in a diff script.
#[derive(Debug, Clone, PartialEq)]
pub enum EditOp {
    Keep(String),
    Delete(String),
    Insert(String),
}

/// A complete edit script (list of [`EditOp`]).
#[derive(Debug, Clone)]
pub struct EditScript {
    pub ops: Vec<EditOp>,
}

impl EditScript {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn push(&mut self, op: EditOp) {
        self.ops.push(op);
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn keep_count(&self) -> usize {
        self.ops
            .iter()
            .filter(|o| matches!(o, EditOp::Keep(_)))
            .count()
    }

    pub fn delete_count(&self) -> usize {
        self.ops
            .iter()
            .filter(|o| matches!(o, EditOp::Delete(_)))
            .count()
    }

    pub fn insert_count(&self) -> usize {
        self.ops
            .iter()
            .filter(|o| matches!(o, EditOp::Insert(_)))
            .count()
    }
}

impl Default for EditScript {
    fn default() -> Self {
        Self::new()
    }
}

/// Build an edit script transforming `old` lines into `new` lines.
///
/// Uses a simple LCS-based greedy approach (stub quality, not full Myers).
pub fn build_edit_script(old: &[&str], new: &[&str]) -> EditScript {
    let mut script = EditScript::new();
    let n = old.len();
    let m = new.len();
    let mut i = 0;
    let mut j = 0;
    while i < n && j < m {
        if old[i] == new[j] {
            script.push(EditOp::Keep(old[i].to_string()));
            i += 1;
            j += 1;
        } else {
            script.push(EditOp::Delete(old[i].to_string()));
            script.push(EditOp::Insert(new[j].to_string()));
            i += 1;
            j += 1;
        }
    }
    while i < n {
        script.push(EditOp::Delete(old[i].to_string()));
        i += 1;
    }
    while j < m {
        script.push(EditOp::Insert(new[j].to_string()));
        j += 1;
    }
    script
}

/// Apply an edit script and return the resulting lines.
pub fn apply_edit_script(script: &EditScript) -> Vec<String> {
    script
        .ops
        .iter()
        .filter_map(|op| match op {
            EditOp::Keep(s) | EditOp::Insert(s) => Some(s.clone()),
            EditOp::Delete(_) => None,
        })
        .collect()
}

/// Count the edit distance (number of non-keep ops) in an edit script.
pub fn edit_distance_from_script(script: &EditScript) -> usize {
    script
        .ops
        .iter()
        .filter(|o| !matches!(o, EditOp::Keep(_)))
        .count()
}

/// Serialize an edit script to a human-readable diff string.
pub fn script_to_diff_string(script: &EditScript) -> String {
    let mut out = String::new();
    for op in &script.ops {
        match op {
            EditOp::Keep(s) => {
                out.push(' ');
                out.push_str(s);
                out.push('\n');
            }
            EditOp::Delete(s) => {
                out.push('-');
                out.push_str(s);
                out.push('\n');
            }
            EditOp::Insert(s) => {
                out.push('+');
                out.push_str(s);
                out.push('\n');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_inputs() {
        let script = build_edit_script(&[], &[]);
        assert!(script.is_empty());
    }

    #[test]
    fn test_identical_lines_all_keep() {
        let old = ["a", "b", "c"];
        let script = build_edit_script(&old, &old);
        assert_eq!(script.keep_count(), 3);
        assert_eq!(script.delete_count(), 0);
        assert_eq!(script.insert_count(), 0);
    }

    #[test]
    fn test_all_deleted() {
        let old = ["x", "y"];
        let script = build_edit_script(&old, &[]);
        assert_eq!(script.delete_count(), 2);
        assert_eq!(script.insert_count(), 0);
    }

    #[test]
    fn test_all_inserted() {
        let new = ["x", "y"];
        let script = build_edit_script(&[], &new);
        assert_eq!(script.insert_count(), 2);
        assert_eq!(script.delete_count(), 0);
    }

    #[test]
    fn test_apply_gives_new_lines() {
        let old = ["a", "b"];
        let new = ["a", "c"];
        let script = build_edit_script(&old, &new);
        let result = apply_edit_script(&script);
        assert_eq!(result, vec!["a", "c"]);
    }

    #[test]
    fn test_edit_distance_nonzero() {
        let old = ["a"];
        let new = ["b"];
        let script = build_edit_script(&old, &new);
        assert!(edit_distance_from_script(&script) > 0);
    }

    #[test]
    fn test_script_to_diff_string_contains_plus_minus() {
        let old = ["hello"];
        let new = ["world"];
        let script = build_edit_script(&old, &new);
        let diff = script_to_diff_string(&script);
        assert!(diff.contains('-'));
        assert!(diff.contains('+'));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut s = EditScript::new();
        assert!(s.is_empty());
        s.push(EditOp::Keep("x".into()));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_default() {
        let s = EditScript::default();
        assert!(s.is_empty());
    }
}
