// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 3-way merge conflict resolver.

/// Result of a 3-way merge for one region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    Clean(Vec<String>),
    Conflict {
        ours: Vec<String>,
        theirs: Vec<String>,
    },
}

/// Configuration for merge resolution.
pub struct MergeConfig {
    pub label_ours: String,
    pub label_theirs: String,
    pub auto_resolve: bool,
}

impl MergeConfig {
    pub fn new(label_ours: &str, label_theirs: &str) -> Self {
        MergeConfig {
            label_ours: label_ours.to_string(),
            label_theirs: label_theirs.to_string(),
            auto_resolve: false,
        }
    }
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self::new("ours", "theirs")
    }
}

/// Perform a 3-way merge of lines.
pub fn three_way_merge(base: &[&str], ours: &[&str], theirs: &[&str]) -> MergeResult {
    /* If both sides match base, return base */
    if ours == base && theirs == base {
        return MergeResult::Clean(base.iter().map(|s| s.to_string()).collect());
    }
    /* If only ours changed, take ours */
    if theirs == base {
        return MergeResult::Clean(ours.iter().map(|s| s.to_string()).collect());
    }
    /* If only theirs changed, take theirs */
    if ours == base {
        return MergeResult::Clean(theirs.iter().map(|s| s.to_string()).collect());
    }
    /* Both changed — conflict */
    MergeResult::Conflict {
        ours: ours.iter().map(|s| s.to_string()).collect(),
        theirs: theirs.iter().map(|s| s.to_string()).collect(),
    }
}

/// Count conflicts in a merge result list.
pub fn count_conflicts(results: &[MergeResult]) -> usize {
    results
        .iter()
        .filter(|r| matches!(r, MergeResult::Conflict { .. }))
        .count()
}

/// Format a conflict block with conflict markers.
pub fn format_conflict(cfg: &MergeConfig, ours: &[String], theirs: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    out.push(format!("<<<<<<< {}", cfg.label_ours));
    out.extend(ours.iter().cloned());
    out.push("=======".to_string());
    out.extend(theirs.iter().cloned());
    out.push(format!(">>>>>>> {}", cfg.label_theirs));
    out
}

/// Auto-resolve by preferring ours.
pub fn auto_resolve_ours(result: MergeResult) -> Vec<String> {
    match result {
        MergeResult::Clean(lines) => lines,
        MergeResult::Conflict { ours, .. } => ours,
    }
}

/// Auto-resolve by preferring theirs.
pub fn auto_resolve_theirs(result: MergeResult) -> Vec<String> {
    match result {
        MergeResult::Clean(lines) => lines,
        MergeResult::Conflict { theirs, .. } => theirs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_change() {
        let r = three_way_merge(&["a"], &["a"], &["a"]);
        assert_eq!(r, MergeResult::Clean(vec!["a".to_string()]));
    }

    #[test]
    fn test_ours_only_change() {
        let r = three_way_merge(&["a"], &["b"], &["a"]);
        assert_eq!(r, MergeResult::Clean(vec!["b".to_string()]));
    }

    #[test]
    fn test_theirs_only_change() {
        let r = three_way_merge(&["a"], &["a"], &["c"]);
        assert_eq!(r, MergeResult::Clean(vec!["c".to_string()]));
    }

    #[test]
    fn test_conflict() {
        let r = three_way_merge(&["a"], &["b"], &["c"]);
        assert!(matches!(r, MergeResult::Conflict { .. }));
    }

    #[test]
    fn test_count_conflicts() {
        let results = vec![
            MergeResult::Clean(vec![]),
            MergeResult::Conflict {
                ours: vec![],
                theirs: vec![],
            },
        ];
        assert_eq!(count_conflicts(&results), 1);
    }

    #[test]
    fn test_format_conflict() {
        let cfg = MergeConfig::default();
        let lines = format_conflict(&cfg, &["ours".to_string()], &["theirs".to_string()]);
        assert!(lines[0].contains("ours"));
        assert!(lines[lines.len() - 1].contains("theirs"));
    }

    #[test]
    fn test_auto_resolve_ours() {
        let r = MergeResult::Conflict {
            ours: vec!["mine".to_string()],
            theirs: vec!["yours".to_string()],
        };
        let resolved = auto_resolve_ours(r);
        assert_eq!(resolved, vec!["mine".to_string()]);
    }

    #[test]
    fn test_auto_resolve_theirs() {
        let r = MergeResult::Conflict {
            ours: vec!["mine".to_string()],
            theirs: vec!["yours".to_string()],
        };
        let resolved = auto_resolve_theirs(r);
        assert_eq!(resolved, vec!["yours".to_string()]);
    }

    #[test]
    fn test_merge_config_default() {
        let cfg = MergeConfig::default();
        assert_eq!(cfg.label_ours, "ours");
        assert_eq!(cfg.label_theirs, "theirs");
    }

    #[test]
    fn test_clean_result_is_clean() {
        let r = three_way_merge(&["x", "y"], &["x", "y"], &["x", "y"]);
        assert!(matches!(r, MergeResult::Clean(_)));
    }
}
