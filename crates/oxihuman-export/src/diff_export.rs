// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Diff/patch format export for config changes.

/// A single diff entry (changed line).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub line_number: usize,
    pub operation: DiffOp,
    pub content: String,
}

/// Diff operation type.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    Add,
    Remove,
    Context,
}

impl DiffOp {
    pub fn symbol(&self) -> char {
        match self {
            DiffOp::Add => '+',
            DiffOp::Remove => '-',
            DiffOp::Context => ' ',
        }
    }
}

/// A diff between two text files.
#[allow(dead_code)]
pub struct DiffExport {
    pub source_name: String,
    pub target_name: String,
    pub entries: Vec<DiffEntry>,
}

impl DiffExport {
    #[allow(dead_code)]
    pub fn new(source: &str, target: &str) -> Self {
        Self {
            source_name: source.to_string(),
            target_name: target.to_string(),
            entries: Vec::new(),
        }
    }
}

/// Compute a simple line-by-line diff between two strings.
#[allow(dead_code)]
pub fn compute_diff(source: &str, target: &str) -> DiffExport {
    let mut diff = DiffExport::new(source, target);
    let src_lines: Vec<&str> = source.lines().collect();
    let tgt_lines: Vec<&str> = target.lines().collect();
    let max = src_lines.len().max(tgt_lines.len());
    for i in 0..max {
        let src = src_lines.get(i).copied();
        let tgt = tgt_lines.get(i).copied();
        match (src, tgt) {
            (Some(s), Some(t)) if s == t => {
                diff.entries.push(DiffEntry {
                    line_number: i + 1,
                    operation: DiffOp::Context,
                    content: s.to_string(),
                });
            }
            (Some(s), Some(t)) => {
                diff.entries.push(DiffEntry {
                    line_number: i + 1,
                    operation: DiffOp::Remove,
                    content: s.to_string(),
                });
                diff.entries.push(DiffEntry {
                    line_number: i + 1,
                    operation: DiffOp::Add,
                    content: t.to_string(),
                });
            }
            (Some(s), None) => {
                diff.entries.push(DiffEntry {
                    line_number: i + 1,
                    operation: DiffOp::Remove,
                    content: s.to_string(),
                });
            }
            (None, Some(t)) => {
                diff.entries.push(DiffEntry {
                    line_number: i + 1,
                    operation: DiffOp::Add,
                    content: t.to_string(),
                });
            }
            (None, None) => {}
        }
    }
    diff
}

/// Serialize diff to unified diff format string.
#[allow(dead_code)]
pub fn export_diff_unified(diff: &DiffExport) -> String {
    let mut out = format!("--- {}\n+++ {}\n", diff.source_name, diff.target_name);
    for e in &diff.entries {
        out.push(e.operation.symbol());
        out.push_str(&e.content);
        out.push('\n');
    }
    out
}

/// Count of additions.
#[allow(dead_code)]
pub fn addition_count(diff: &DiffExport) -> usize {
    diff.entries
        .iter()
        .filter(|e| e.operation == DiffOp::Add)
        .count()
}

/// Count of removals.
#[allow(dead_code)]
pub fn removal_count(diff: &DiffExport) -> usize {
    diff.entries
        .iter()
        .filter(|e| e.operation == DiffOp::Remove)
        .count()
}

/// Whether the two texts are identical (no changes).
#[allow(dead_code)]
pub fn is_identical(diff: &DiffExport) -> bool {
    !diff.entries.iter().any(|e| e.operation != DiffOp::Context)
}

/// Total changed lines (add + remove).
#[allow(dead_code)]
pub fn changed_line_count(diff: &DiffExport) -> usize {
    addition_count(diff) + removal_count(diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_texts_no_changes() {
        let diff = compute_diff("hello\nworld", "hello\nworld");
        assert!(is_identical(&diff));
    }

    #[test]
    fn changed_line_count_one_change() {
        let diff = compute_diff("a\nb\nc", "a\nX\nc");
        assert_eq!(changed_line_count(&diff), 2);
    }

    #[test]
    fn addition_count_correct() {
        let diff = compute_diff("a\nb", "a\nb\nc");
        assert_eq!(addition_count(&diff), 1);
    }

    #[test]
    fn removal_count_correct() {
        let diff = compute_diff("a\nb\nc", "a\nb");
        assert_eq!(removal_count(&diff), 1);
    }

    #[test]
    fn unified_diff_starts_with_header() {
        let diff = compute_diff("old", "new");
        let out = export_diff_unified(&diff);
        assert!(out.starts_with("---"));
    }

    #[test]
    fn unified_diff_contains_plus() {
        let diff = compute_diff("a", "b");
        let out = export_diff_unified(&diff);
        assert!(out.contains('+'));
    }

    #[test]
    fn diff_op_symbol_correct() {
        assert_eq!(DiffOp::Add.symbol(), '+');
        assert_eq!(DiffOp::Remove.symbol(), '-');
        assert_eq!(DiffOp::Context.symbol(), ' ');
    }

    #[test]
    fn empty_strings_no_entries() {
        let diff = compute_diff("", "");
        assert_eq!(diff.entries.len(), 0);
    }

    #[test]
    fn source_target_names_stored() {
        let diff = DiffExport::new("old.cfg", "new.cfg");
        assert_eq!(diff.source_name, "old.cfg");
        assert_eq!(diff.target_name, "new.cfg");
    }

    #[test]
    fn context_lines_counted() {
        let diff = compute_diff("a\nb\nc", "a\nX\nc");
        let ctx = diff
            .entries
            .iter()
            .filter(|e| e.operation == DiffOp::Context)
            .count();
        assert_eq!(ctx, 2);
    }
}
