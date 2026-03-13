// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Patience diff algorithm stub.
//!
//! Finds unique common lines first (patience LCS), then recurses to diff
//! the regions between them. This produces more readable diffs for code.

/// A hunk in a patience diff result.
#[derive(Debug, Clone, PartialEq)]
pub struct PatienceHunk {
    pub old_start: usize,
    pub old_len: usize,
    pub new_start: usize,
    pub new_len: usize,
    pub removed: Vec<String>,
    pub added: Vec<String>,
}

/// Result of running patience diff.
#[derive(Debug, Clone)]
pub struct PatienceDiff {
    pub hunks: Vec<PatienceHunk>,
}

impl PatienceDiff {
    pub fn new() -> Self {
        Self { hunks: Vec::new() }
    }

    pub fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    pub fn is_identical(&self) -> bool {
        self.hunks.is_empty()
    }

    pub fn total_removed(&self) -> usize {
        self.hunks.iter().map(|h| h.old_len).sum()
    }

    pub fn total_added(&self) -> usize {
        self.hunks.iter().map(|h| h.new_len).sum()
    }
}

impl Default for PatienceDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Find lines that appear exactly once in both `old` and `new`.
pub fn unique_common_lines<'a>(old: &[&'a str], new: &[&'a str]) -> Vec<&'a str> {
    use std::collections::HashMap;
    let mut old_counts: HashMap<&str, usize> = HashMap::new();
    let mut new_counts: HashMap<&str, usize> = HashMap::new();
    for &l in old {
        *old_counts.entry(l).or_insert(0) += 1;
    }
    for &l in new {
        *new_counts.entry(l).or_insert(0) += 1;
    }
    old.iter()
        .filter(|&&l| old_counts.get(l) == Some(&1) && new_counts.get(l) == Some(&1))
        .copied()
        .collect()
}

/// Run patience diff on two line slices and return the result.
pub fn patience_diff(old: &[&str], new: &[&str]) -> PatienceDiff {
    let mut diff = PatienceDiff::new();
    /* Simple stub: emit one hunk per differing line position */
    let n = old.len().max(new.len());
    let mut oi = 0usize;
    let mut ni = 0usize;
    while oi < old.len() || ni < new.len() {
        let old_line = old.get(oi).copied();
        let new_line = new.get(ni).copied();
        if old_line == new_line {
            oi += 1;
            ni += 1;
            continue;
        }
        let mut hunk = PatienceHunk {
            old_start: oi,
            new_start: ni,
            old_len: 0,
            new_len: 0,
            removed: Vec::new(),
            added: Vec::new(),
        };
        if let Some(ol) = old_line {
            hunk.removed.push(ol.to_string());
            hunk.old_len = 1;
        }
        if let Some(nl) = new_line {
            hunk.added.push(nl.to_string());
            hunk.new_len = 1;
        }
        diff.hunks.push(hunk);
        if oi < old.len() {
            oi += 1;
        }
        if ni < new.len() {
            ni += 1;
        }
    }
    let _ = n; /* suppress unused warning */
    diff
}

/// Return a unified-diff-like string from a PatienceDiff.
pub fn patience_diff_to_string(diff: &PatienceDiff) -> String {
    let mut out = String::new();
    for h in &diff.hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            h.old_start, h.old_len, h.new_start, h.new_len
        ));
        for l in &h.removed {
            out.push('-');
            out.push_str(l);
            out.push('\n');
        }
        for l in &h.added {
            out.push('+');
            out.push_str(l);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_is_empty() {
        let lines = ["a", "b", "c"];
        let diff = patience_diff(&lines, &lines);
        assert!(diff.is_identical());
    }

    #[test]
    fn test_one_change() {
        let old = ["a", "b"];
        let new = ["a", "c"];
        let diff = patience_diff(&old, &new);
        assert!(!diff.is_identical());
    }

    #[test]
    fn test_hunk_count() {
        let old = ["x"];
        let new = ["y"];
        let diff = patience_diff(&old, &new);
        assert_eq!(diff.hunk_count(), 1);
    }

    #[test]
    fn test_total_removed_added() {
        let old = ["a", "b"];
        let new = ["c", "d"];
        let diff = patience_diff(&old, &new);
        assert_eq!(diff.total_removed(), diff.total_added());
    }

    #[test]
    fn test_unique_common_lines() {
        let old = ["a", "b", "c"];
        let new = ["b", "d", "e"];
        let common = unique_common_lines(&old, &new);
        assert_eq!(common, vec!["b"]);
    }

    #[test]
    fn test_unique_common_lines_empty_when_duplicate() {
        let old = ["a", "a"];
        let new = ["a"];
        let common = unique_common_lines(&old, &new);
        assert!(common.is_empty());
    }

    #[test]
    fn test_to_string_contains_at() {
        let old = ["x"];
        let new = ["y"];
        let diff = patience_diff(&old, &new);
        let s = patience_diff_to_string(&diff);
        assert!(s.contains("@@"));
    }

    #[test]
    fn test_default() {
        let d = PatienceDiff::default();
        assert!(d.is_identical());
    }

    #[test]
    fn test_added_lines_tracked() {
        let old: &[&str] = &[];
        let new = ["a", "b"];
        let diff = patience_diff(old, &new);
        assert!(diff.total_added() > 0);
    }
}
