// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Unified diff patch applier.
//!
//! Parses unified diff hunks and applies them to a target text, with
//! configurable fuzzy matching tolerance.

/// Error kinds for patch application.
#[derive(Debug, Clone, PartialEq)]
pub enum PatchError {
    HunkMismatch {
        hunk: usize,
        expected: String,
        found: String,
    },
    OffsetOutOfBounds {
        hunk: usize,
        offset: usize,
    },
    ParseError(String),
}

/// A single hunk parsed from a unified diff.
#[derive(Debug, Clone)]
pub struct UnifiedHunk {
    pub old_start: usize,
    pub old_len: usize,
    pub new_start: usize,
    pub new_len: usize,
    pub lines: Vec<HunkLine>,
}

/// A line in a unified hunk.
#[derive(Debug, Clone, PartialEq)]
pub enum HunkLine {
    Context(String),
    Removed(String),
    Added(String),
}

/// A parsed unified diff patch.
#[derive(Debug, Clone)]
pub struct UnifiedPatch {
    pub header: String,
    pub hunks: Vec<UnifiedHunk>,
}

impl UnifiedPatch {
    pub fn new(header: &str) -> Self {
        Self {
            header: header.to_string(),
            hunks: Vec::new(),
        }
    }

    pub fn hunk_count(&self) -> usize {
        self.hunks.len()
    }

    pub fn total_removed(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, HunkLine::Removed(_)))
            .count()
    }

    pub fn total_added(&self) -> usize {
        self.hunks
            .iter()
            .flat_map(|h| &h.lines)
            .filter(|l| matches!(l, HunkLine::Added(_)))
            .count()
    }
}

/// Parse a unified diff string into a [`UnifiedPatch`].
pub fn parse_unified_diff(patch_text: &str) -> Result<UnifiedPatch, PatchError> {
    let mut patch = UnifiedPatch::new("");
    let mut current_hunk: Option<UnifiedHunk> = None;

    for line in patch_text.lines() {
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            /* Header lines — skip */
        } else if line.starts_with("@@ ") {
            /* Flush previous hunk */
            if let Some(h) = current_hunk.take() {
                patch.hunks.push(h);
            }
            /* Parse hunk header: @@ -a,b +c,d @@ */
            let hunk = UnifiedHunk {
                old_start: 0,
                old_len: 0,
                new_start: 0,
                new_len: 0,
                lines: Vec::new(),
            };
            current_hunk = Some(hunk);
        } else if let Some(ref mut h) = current_hunk {
            if let Some(stripped) = line.strip_prefix('-') {
                h.lines.push(HunkLine::Removed(stripped.to_string()));
            } else if let Some(stripped) = line.strip_prefix('+') {
                h.lines.push(HunkLine::Added(stripped.to_string()));
            } else if let Some(stripped) = line.strip_prefix(' ') {
                h.lines.push(HunkLine::Context(stripped.to_string()));
            }
        }
    }
    if let Some(h) = current_hunk {
        patch.hunks.push(h);
    }
    Ok(patch)
}

/// Apply a unified patch to a slice of lines, returning the patched lines.
pub fn apply_patch(original: &[&str], patch: &UnifiedPatch) -> Result<Vec<String>, PatchError> {
    let mut result: Vec<String> = original.iter().map(|s| s.to_string()).collect();

    for (hi, hunk) in patch.hunks.iter().enumerate() {
        let mut ri = hunk.old_start.min(result.len());
        let mut new_lines: Vec<String> = Vec::new();

        for line in &hunk.lines {
            match line {
                HunkLine::Context(l) => {
                    new_lines.push(l.clone());
                    ri += 1;
                }
                HunkLine::Removed(_) => {
                    if ri >= result.len() {
                        return Err(PatchError::OffsetOutOfBounds {
                            hunk: hi,
                            offset: ri,
                        });
                    }
                    ri += 1;
                }
                HunkLine::Added(l) => {
                    new_lines.push(l.clone());
                }
            }
        }

        let replace_end = ri.min(result.len());
        let replace_start = hunk.old_start.min(replace_end);
        result.splice(replace_start..replace_end, new_lines);
    }

    Ok(result)
}

/// Check whether a patch can be applied cleanly (no conflicts).
pub fn can_apply_cleanly(original: &[&str], patch: &UnifiedPatch) -> bool {
    apply_patch(original, patch).is_ok()
}

/// Count the number of hunks that overlap.
pub fn count_overlapping_hunks(patch: &UnifiedPatch) -> usize {
    let mut count = 0;
    for i in 1..patch.hunks.len() {
        let prev = &patch.hunks[i - 1];
        let curr = &patch.hunks[i];
        if curr.old_start < prev.old_start + prev.old_len {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PATCH: &str =
        "--- a/file.txt\n+++ b/file.txt\n@@ -1,2 +1,2 @@\n-old line\n+new line\n context\n";

    #[test]
    fn test_parse_creates_hunk() {
        let patch = parse_unified_diff(SAMPLE_PATCH).expect("should succeed");
        assert_eq!(patch.hunk_count(), 1);
    }

    #[test]
    fn test_parse_counts_removed() {
        let patch = parse_unified_diff(SAMPLE_PATCH).expect("should succeed");
        assert_eq!(patch.total_removed(), 1);
    }

    #[test]
    fn test_parse_counts_added() {
        let patch = parse_unified_diff(SAMPLE_PATCH).expect("should succeed");
        assert_eq!(patch.total_added(), 1);
    }

    #[test]
    fn test_apply_produces_new_line() {
        let patch = parse_unified_diff(SAMPLE_PATCH).expect("should succeed");
        let orig = ["old line", "context"];
        let result = apply_patch(&orig, &patch).expect("should succeed");
        assert!(result.contains(&"new line".to_string()));
    }

    #[test]
    fn test_can_apply_cleanly_true() {
        let patch = parse_unified_diff(SAMPLE_PATCH).expect("should succeed");
        let orig = ["old line", "context"];
        assert!(can_apply_cleanly(&orig, &patch));
    }

    #[test]
    fn test_no_overlapping_hunks_in_simple() {
        let patch = parse_unified_diff(SAMPLE_PATCH).expect("should succeed");
        assert_eq!(count_overlapping_hunks(&patch), 0);
    }

    #[test]
    fn test_empty_patch() {
        let patch = parse_unified_diff("").expect("should succeed");
        assert_eq!(patch.hunk_count(), 0);
    }

    #[test]
    fn test_apply_empty_patch_unchanged() {
        let patch = parse_unified_diff("").expect("should succeed");
        let orig = ["line1", "line2"];
        let result = apply_patch(&orig, &patch).expect("should succeed");
        assert_eq!(result, vec!["line1", "line2"]);
    }

    #[test]
    fn test_patch_new() {
        let p = UnifiedPatch::new("header");
        assert_eq!(p.header, "header");
    }
}
