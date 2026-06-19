// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Patience diff algorithm.
//!
//! Finds unique common lines first (patience LCS), then recurses to diff
//! the regions between them. This produces more readable diffs for code
//! because function boundaries act as natural anchors.

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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Given unique common lines, find the patience LCS (Longest Common Subsequence)
/// via the patience sort / LIS algorithm.
///
/// Returns pairs `(old_pos, new_pos)` for the anchors in LCS order.
fn patience_lcs(unique: &[&str], old_mid: &[&str], new_mid: &[&str]) -> Vec<(usize, usize)> {
    use std::collections::HashMap;

    if unique.is_empty() {
        return Vec::new();
    }

    // Build index: line → position in new_mid (only for unique lines)
    let mut new_pos_map: HashMap<&str, usize> = HashMap::new();
    for (i, &l) in new_mid.iter().enumerate() {
        // only unique lines appear here; insert once
        new_pos_map.insert(l, i);
    }

    // Collect (old_pos, new_pos) pairs in old_mid order
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for (oi, &l) in old_mid.iter().enumerate() {
        if let Some(&ni) = new_pos_map.get(l) {
            // Only include lines that are in the unique set
            if unique.contains(&l) {
                pairs.push((oi, ni));
            }
        }
    }

    // Now find LIS of the new_pos column using patience sort (O(n log n))
    // piles[k] = smallest new_pos at the top of pile k (length k+1 sequence)
    // predecessors[i] = index into pairs[] of the predecessor in the LIS
    let n = pairs.len();
    if n == 0 {
        return Vec::new();
    }

    let mut piles: Vec<usize> = Vec::new(); // pile tops (values are indices into pairs)
    let mut pile_tops: Vec<usize> = Vec::new(); // new_pos values at tops for binary search
    let mut predecessor: Vec<Option<usize>> = vec![None; n];

    for i in 0..n {
        let val = pairs[i].1; // new_pos value
                              // Binary search for the leftmost pile whose top >= val
        let pos = pile_tops.partition_point(|&top| top < val);
        if pos > 0 {
            predecessor[i] = Some(piles[pos - 1]);
        }
        if pos == piles.len() {
            piles.push(i);
            pile_tops.push(val);
        } else {
            piles[pos] = i;
            pile_tops[pos] = val;
        }
    }

    // Reconstruct LCS by following predecessor links from the last pile top
    let mut result_indices: Vec<usize> = Vec::new();
    let mut cur = *piles
        .last()
        .expect("invariant: piles is non-empty because unique_pairs is non-empty"); // at least one pair exists
    loop {
        result_indices.push(cur);
        match predecessor[cur] {
            Some(prev) => cur = prev,
            None => break,
        }
    }
    result_indices.reverse();

    result_indices.iter().map(|&i| pairs[i]).collect()
}

/// Fallback diff for a segment with no unique common lines.
/// Emits one hunk containing all old lines as removed and all new lines as added.
fn fallback_diff(
    old_mid: &[&str],
    new_mid: &[&str],
    old_base: usize,
    new_base: usize,
    hunks: &mut Vec<PatienceHunk>,
) {
    if old_mid.is_empty() && new_mid.is_empty() {
        return;
    }
    hunks.push(PatienceHunk {
        old_start: old_base,
        old_len: old_mid.len(),
        new_start: new_base,
        new_len: new_mid.len(),
        removed: old_mid.iter().map(|s| s.to_string()).collect(),
        added: new_mid.iter().map(|s| s.to_string()).collect(),
    });
}

/// Core recursive patience diff engine.
/// `old_base` and `new_base` are the absolute line offsets of the slices in
/// the original documents (used for hunk headers).
fn diff_recursive(
    old: &[&str],
    new: &[&str],
    old_base: usize,
    new_base: usize,
    hunks: &mut Vec<PatienceHunk>,
) {
    // ── 1. Trim common prefix ───────────────────────────────────────────────
    let common_pre = old
        .iter()
        .zip(new.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let old = &old[common_pre..];
    let new = &new[common_pre..];
    let old_base = old_base + common_pre;
    let new_base = new_base + common_pre;

    // ── 2. Trim common suffix ───────────────────────────────────────────────
    let common_suf = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let old_mid = &old[..old.len() - common_suf];
    let new_mid = &new[..new.len() - common_suf];

    if old_mid.is_empty() && new_mid.is_empty() {
        return; // everything matched
    }

    // ── 3. Find unique common lines in the middle portions ──────────────────
    let unique = unique_common_lines(old_mid, new_mid);

    if unique.is_empty() {
        fallback_diff(old_mid, new_mid, old_base, new_base, hunks);
        return;
    }

    // ── 4. Patience LCS of unique lines ────────────────────────────────────
    let anchors = patience_lcs(&unique, old_mid, new_mid);

    if anchors.is_empty() {
        fallback_diff(old_mid, new_mid, old_base, new_base, hunks);
        return;
    }

    // ── 5. Recurse into each gap between consecutive anchors ────────────────
    let mut prev_old = 0usize;
    let mut prev_new = 0usize;

    for (anchor_old, anchor_new) in &anchors {
        let seg_old = &old_mid[prev_old..*anchor_old];
        let seg_new = &new_mid[prev_new..*anchor_new];
        diff_recursive(
            seg_old,
            seg_new,
            old_base + prev_old,
            new_base + prev_new,
            hunks,
        );
        // Skip the anchor line itself (it is identical)
        prev_old = anchor_old + 1;
        prev_new = anchor_new + 1;
    }

    // Segment after the last anchor
    let seg_old = &old_mid[prev_old..];
    let seg_new = &new_mid[prev_new..];
    diff_recursive(
        seg_old,
        seg_new,
        old_base + prev_old,
        new_base + prev_new,
        hunks,
    );
}

/// Merge adjacent hunks that are contiguous (no unchanged lines between them).
fn merge_adjacent_hunks(mut hunks: Vec<PatienceHunk>) -> Vec<PatienceHunk> {
    if hunks.len() < 2 {
        return hunks;
    }
    let mut merged: Vec<PatienceHunk> = Vec::with_capacity(hunks.len());
    merged.push(hunks.remove(0));
    for h in hunks {
        let last = merged
            .last_mut()
            .expect("invariant: merged is non-empty because we pushed before the loop");
        let old_contiguous = last.old_start + last.old_len == h.old_start;
        let new_contiguous = last.new_start + last.new_len == h.new_start;
        if old_contiguous && new_contiguous {
            last.old_len += h.old_len;
            last.new_len += h.new_len;
            last.removed.extend(h.removed);
            last.added.extend(h.added);
        } else {
            merged.push(h);
        }
    }
    merged
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run patience diff on two line slices and return the result.
pub fn patience_diff(old: &[&str], new: &[&str]) -> PatienceDiff {
    let mut hunks: Vec<PatienceHunk> = Vec::new();
    diff_recursive(old, new, 0, 0, &mut hunks);
    let hunks = merge_adjacent_hunks(hunks);
    PatienceDiff { hunks }
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

    /// The key correctness test: two isolated function changes must produce
    /// exactly 2 hunks, not one large blob.
    #[test]
    fn test_two_isolated_function_changes() {
        let old = [
            "fn foo() {",
            "  let x = 1;",
            "  let y = 2;",
            "}",
            "fn bar() {",
            "  let a = 1;",
            "}",
            "",
        ];
        let new = [
            "fn foo() {",
            "  let x = 1;",
            "  let y = 3;",
            "}",
            "fn bar() {",
            "  let a = 2;",
            "}",
            "",
        ];
        let diff = patience_diff(&old, &new);
        // Patience diff must recognise fn foo / fn bar / } as anchors and
        // isolate each change to its own hunk.
        assert_eq!(
            diff.hunk_count(),
            2,
            "expected 2 hunks (one per function change), got {}:\n{:#?}",
            diff.hunk_count(),
            diff.hunks
        );
        assert_eq!(diff.total_removed(), 2);
        assert_eq!(diff.total_added(), 2);
    }

    #[test]
    fn test_insert_only() {
        let old = ["a", "b", "c"];
        let new = ["a", "x", "y", "b", "c"];
        let diff = patience_diff(&old, &new);
        assert_eq!(diff.total_added(), 2);
        assert_eq!(diff.total_removed(), 0);
    }

    #[test]
    fn test_delete_only() {
        let old = ["a", "b", "c", "d"];
        let new = ["a", "d"];
        let diff = patience_diff(&old, &new);
        assert_eq!(diff.total_removed(), 2);
        assert_eq!(diff.total_added(), 0);
    }

    #[test]
    fn test_empty_old() {
        let old: &[&str] = &[];
        let new = ["a", "b", "c"];
        let diff = patience_diff(old, &new);
        assert_eq!(diff.total_added(), 3);
        assert_eq!(diff.total_removed(), 0);
    }

    #[test]
    fn test_empty_new() {
        let old = ["a", "b", "c"];
        let new: &[&str] = &[];
        let diff = patience_diff(&old, new);
        assert_eq!(diff.total_removed(), 3);
        assert_eq!(diff.total_added(), 0);
    }

    #[test]
    fn test_total_lines_conserved() {
        let old = ["a", "b", "c", "d"];
        let new = ["a", "x", "c", "y"];
        let diff = patience_diff(&old, &new);
        // lines not covered by hunks are common, so:
        // common_lines = old.len() - total_removed = new.len() - total_added
        let common_via_old = old.len() - diff.total_removed();
        let common_via_new = new.len() - diff.total_added();
        assert_eq!(common_via_old, common_via_new);
    }
}
