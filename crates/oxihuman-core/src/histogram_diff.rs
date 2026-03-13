// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Histogram diff algorithm stub.
//!
//! Builds a frequency histogram of lines and anchors matching on the
//! least-frequent common line, giving high-quality diffs for structured text.

use std::collections::HashMap;

/// Configuration for histogram diff.
#[derive(Debug, Clone)]
pub struct HistogramDiffConfig {
    /// Maximum line frequency to consider as an anchor candidate.
    pub max_anchor_freq: usize,
    pub context_lines: usize,
}

impl Default for HistogramDiffConfig {
    fn default() -> Self {
        Self {
            max_anchor_freq: 64,
            context_lines: 3,
        }
    }
}

/// A change hunk produced by histogram diff.
#[derive(Debug, Clone)]
pub struct HistogramHunk {
    pub old_range: (usize, usize),
    pub new_range: (usize, usize),
    pub removed: Vec<String>,
    pub added: Vec<String>,
}

/// Result of histogram diff.
#[derive(Debug, Clone)]
pub struct HistogramDiff {
    pub hunks: Vec<HistogramHunk>,
}

impl HistogramDiff {
    pub fn new() -> Self {
        Self { hunks: Vec::new() }
    }

    pub fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    pub fn is_identical(&self) -> bool {
        self.hunks.is_empty()
    }

    pub fn changed_lines(&self) -> usize {
        self.hunks
            .iter()
            .map(|h| h.removed.len() + h.added.len())
            .sum()
    }
}

impl Default for HistogramDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a frequency histogram of the given lines.
pub fn build_histogram<'a>(lines: &[&'a str]) -> HashMap<&'a str, usize> {
    let mut map: HashMap<&str, usize> = HashMap::new();
    for &l in lines {
        *map.entry(l).or_insert(0) += 1;
    }
    map
}

/// Run histogram diff on two line slices.
pub fn histogram_diff(old: &[&str], new: &[&str], cfg: &HistogramDiffConfig) -> HistogramDiff {
    let mut result = HistogramDiff::new();
    let old_hist = build_histogram(old);
    let new_hist = build_histogram(new);

    let mut oi = 0usize;
    let mut ni = 0usize;

    while oi < old.len() || ni < new.len() {
        let o = old.get(oi).copied();
        let n = new.get(ni).copied();
        if o == n {
            oi += 1;
            ni += 1;
            continue;
        }
        /* Check anchor eligibility by frequency */
        let o_eligible = o
            .map(|l| old_hist.get(l).copied().unwrap_or(0) <= cfg.max_anchor_freq)
            .unwrap_or(false);
        let n_eligible = n
            .map(|l| new_hist.get(l).copied().unwrap_or(0) <= cfg.max_anchor_freq)
            .unwrap_or(false);

        let mut hunk = HistogramHunk {
            old_range: (oi, oi),
            new_range: (ni, ni),
            removed: Vec::new(),
            added: Vec::new(),
        };

        if o_eligible || o.is_none() {
            if let Some(ol) = o {
                hunk.removed.push(ol.to_string());
                oi += 1;
            }
        } else if let Some(ol) = o {
            hunk.removed.push(ol.to_string());
            oi += 1;
        }
        if n_eligible || n.is_none() {
            if let Some(nl) = n {
                hunk.added.push(nl.to_string());
                ni += 1;
            }
        } else if let Some(nl) = n {
            hunk.added.push(nl.to_string());
            ni += 1;
        }

        hunk.old_range.1 = oi;
        hunk.new_range.1 = ni;
        result.hunks.push(hunk);
    }
    result
}

/// Format histogram diff as a string.
pub fn histogram_diff_to_string(diff: &HistogramDiff) -> String {
    let mut out = String::new();
    for h in &diff.hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            h.old_range.0,
            h.old_range.1.saturating_sub(h.old_range.0),
            h.new_range.0,
            h.new_range.1.saturating_sub(h.new_range.0),
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

    fn default_cfg() -> HistogramDiffConfig {
        HistogramDiffConfig::default()
    }

    #[test]
    fn test_identical_empty_hunks() {
        let lines = ["a", "b"];
        let d = histogram_diff(&lines, &lines, &default_cfg());
        assert!(d.is_identical());
    }

    #[test]
    fn test_single_change() {
        let old = ["a", "b"];
        let new = ["a", "c"];
        let d = histogram_diff(&old, &new, &default_cfg());
        assert!(!d.is_identical());
    }

    #[test]
    fn test_hunk_count_one() {
        let old = ["x"];
        let new = ["y"];
        let d = histogram_diff(&old, &new, &default_cfg());
        assert_eq!(d.hunk_count(), 1);
    }

    #[test]
    fn test_changed_lines_nonzero() {
        let old = ["p"];
        let new = ["q"];
        let d = histogram_diff(&old, &new, &default_cfg());
        assert!(d.changed_lines() > 0);
    }

    #[test]
    fn test_build_histogram_counts() {
        let lines = ["a", "b", "a"];
        let h = build_histogram(&lines);
        assert_eq!(h[&"a"], 2);
        assert_eq!(h[&"b"], 1);
    }

    #[test]
    fn test_all_added() {
        let old: &[&str] = &[];
        let new = ["x", "y"];
        let d = histogram_diff(old, &new, &default_cfg());
        assert!(d.changed_lines() > 0);
    }

    #[test]
    fn test_to_string_has_at() {
        let old = ["a"];
        let new = ["b"];
        let d = histogram_diff(&old, &new, &default_cfg());
        let s = histogram_diff_to_string(&d);
        assert!(s.contains("@@"));
    }

    #[test]
    fn test_default_config() {
        let cfg = HistogramDiffConfig::default();
        assert_eq!(cfg.context_lines, 3);
    }

    #[test]
    fn test_default_histogram_diff() {
        let d = HistogramDiff::default();
        assert!(d.is_identical());
    }
}
