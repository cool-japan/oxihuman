// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Generate unified diff patches.

/// A hunk in a unified diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<String>,
}

/// A unified diff patch.
#[derive(Debug, Clone, Default)]
pub struct UnifiedPatch {
    pub old_file: String,
    pub new_file: String,
    pub hunks: Vec<DiffHunk>,
}

impl UnifiedPatch {
    pub fn new(old_file: &str, new_file: &str) -> Self {
        UnifiedPatch {
            old_file: old_file.to_string(),
            new_file: new_file.to_string(),
            hunks: Vec::new(),
        }
    }

    pub fn add_hunk(&mut self, hunk: DiffHunk) {
        self.hunks.push(hunk);
    }

    pub fn is_empty(&self) -> bool {
        self.hunks.is_empty()
    }
}

/// Generate a unified patch from old/new line slices.
pub fn generate_patch(old_file: &str, new_file: &str, old: &[&str], new: &[&str]) -> UnifiedPatch {
    let mut patch = UnifiedPatch::new(old_file, new_file);
    if old == new {
        return patch;
    }
    let mut hunk_lines = Vec::new();
    for line in old {
        hunk_lines.push(format!("-{}", line));
    }
    for line in new {
        hunk_lines.push(format!("+{}", line));
    }
    patch.add_hunk(DiffHunk {
        old_start: 1,
        old_count: old.len(),
        new_start: 1,
        new_count: new.len(),
        lines: hunk_lines,
    });
    patch
}

/// Serialize a patch to unified diff format string.
pub fn serialize_patch(patch: &UnifiedPatch) -> String {
    let mut out = String::new();
    out.push_str(&format!("--- {}\n", patch.old_file));
    out.push_str(&format!("+++ {}\n", patch.new_file));
    for hunk in &patch.hunks {
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
        ));
        for line in &hunk.lines {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Count total changed lines in a patch.
pub fn total_changed_lines(patch: &UnifiedPatch) -> usize {
    patch
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .filter(|l| l.starts_with('+') || l.starts_with('-'))
        .count()
}

/// Apply a patch to old lines to produce new lines (stub).
pub fn apply_patch_stub(old: &[&str], patch: &UnifiedPatch) -> Vec<String> {
    if patch.is_empty() {
        return old.iter().map(|s| s.to_string()).collect();
    }
    /* Stub: extract '+' lines from patch */
    patch
        .hunks
        .iter()
        .flat_map(|h| {
            h.lines
                .iter()
                .filter(|l| l.starts_with('+'))
                .map(|l| l[1..].to_string())
        })
        .collect()
}

/// Return true if patch is a no-op (empty).
pub fn is_identity_patch(patch: &UnifiedPatch) -> bool {
    patch.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_patch_for_same_files() {
        let p = generate_patch("a.txt", "b.txt", &["x"], &["x"]);
        assert!(p.is_empty());
    }

    #[test]
    fn test_patch_has_hunk_for_diff() {
        let p = generate_patch("a.txt", "b.txt", &["x"], &["y"]);
        assert!(!p.is_empty());
        assert_eq!(p.hunks.len(), 1);
    }

    #[test]
    fn test_serialize_contains_header() {
        let p = generate_patch("old.txt", "new.txt", &["a"], &["b"]);
        let s = serialize_patch(&p);
        assert!(s.contains("--- old.txt"));
        assert!(s.contains("+++ new.txt"));
    }

    #[test]
    fn test_total_changed_lines() {
        let p = generate_patch("a", "b", &["x"], &["y"]);
        assert_eq!(total_changed_lines(&p), 2); /* one delete, one insert */
    }

    #[test]
    fn test_apply_patch_stub() {
        let p = generate_patch("a", "b", &["old"], &["new"]);
        let result = apply_patch_stub(&["old"], &p);
        assert_eq!(result, vec!["new".to_string()]);
    }

    #[test]
    fn test_identity_patch() {
        let p = generate_patch("a", "b", &["same"], &["same"]);
        assert!(is_identity_patch(&p));
    }

    #[test]
    fn test_patch_new_default() {
        let p = UnifiedPatch::default();
        assert!(p.is_empty());
    }

    #[test]
    fn test_hunk_line_counts() {
        let p = generate_patch("f", "g", &["a", "b"], &["c"]);
        assert_eq!(p.hunks[0].old_count, 2);
        assert_eq!(p.hunks[0].new_count, 1);
    }

    #[test]
    fn test_serialize_empty_patch() {
        let p = UnifiedPatch::new("a", "b");
        let s = serialize_patch(&p);
        assert!(s.contains("--- a"));
    }

    #[test]
    fn test_apply_empty_patch() {
        let p = UnifiedPatch::new("a", "b");
        let result = apply_patch_stub(&["line1"], &p);
        assert_eq!(result, vec!["line1".to_string()]);
    }
}
