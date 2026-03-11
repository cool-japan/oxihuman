//! Data diff — compute diffs between two key-value string snapshots
//! (added, removed, changed entries) and apply or invert them.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DataDiffConfig {
    pub max_entries: usize,
    pub track_unchanged: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum DiffKind {
    Added,
    Removed,
    Changed,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub key: String,
    pub kind: DiffKind,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DataDiff {
    pub config: DataDiffConfig,
    pub entries: Vec<DiffEntry>,
}

#[allow(dead_code)]
pub fn default_data_diff_config() -> DataDiffConfig {
    DataDiffConfig {
        max_entries: 1024,
        track_unchanged: false,
    }
}

/// A snapshot is a sorted list of (key, value) pairs.
#[allow(dead_code)]
pub fn compute_diff(
    before: &[(&str, &str)],
    after: &[(&str, &str)],
    cfg: &DataDiffConfig,
) -> DataDiff {
    use std::collections::HashMap;

    let before_map: HashMap<&str, &str> = before.iter().copied().collect();
    let after_map: HashMap<&str, &str> = after.iter().copied().collect();

    let mut entries = Vec::new();

    // Check for removed and changed
    for (&k, &v_old) in &before_map {
        if let Some(&v_new) = after_map.get(k) {
            if v_new != v_old {
                entries.push(DiffEntry {
                    key: k.to_string(),
                    kind: DiffKind::Changed,
                    old_value: Some(v_old.to_string()),
                    new_value: Some(v_new.to_string()),
                });
            }
        } else {
            entries.push(DiffEntry {
                key: k.to_string(),
                kind: DiffKind::Removed,
                old_value: Some(v_old.to_string()),
                new_value: None,
            });
        }
        if entries.len() >= cfg.max_entries {
            break;
        }
    }

    // Check for added
    for (&k, &v_new) in &after_map {
        if !before_map.contains_key(k) {
            entries.push(DiffEntry {
                key: k.to_string(),
                kind: DiffKind::Added,
                old_value: None,
                new_value: Some(v_new.to_string()),
            });
        }
        if entries.len() >= cfg.max_entries {
            break;
        }
    }

    // Sort by key for determinism
    entries.sort_by(|a, b| a.key.cmp(&b.key));

    DataDiff { config: cfg.clone(), entries }
}

#[allow(dead_code)]
pub fn diff_entry_count(diff: &DataDiff) -> usize {
    diff.entries.len()
}

#[allow(dead_code)]
pub fn diff_added_count(diff: &DataDiff) -> usize {
    diff.entries.iter().filter(|e| e.kind == DiffKind::Added).count()
}

#[allow(dead_code)]
pub fn diff_removed_count(diff: &DataDiff) -> usize {
    diff.entries
        .iter()
        .filter(|e| e.kind == DiffKind::Removed)
        .count()
}

#[allow(dead_code)]
pub fn diff_changed_count(diff: &DataDiff) -> usize {
    diff.entries
        .iter()
        .filter(|e| e.kind == DiffKind::Changed)
        .count()
}

#[allow(dead_code)]
pub fn diff_to_json(diff: &DataDiff) -> String {
    let entries: Vec<String> = diff
        .entries
        .iter()
        .map(|e| {
            let kind = match e.kind {
                DiffKind::Added => "added",
                DiffKind::Removed => "removed",
                DiffKind::Changed => "changed",
            };
            let old = e
                .old_value
                .as_deref()
                .map(|v| format!("\"{}\"", v))
                .unwrap_or_else(|| "null".to_string());
            let new = e
                .new_value
                .as_deref()
                .map(|v| format!("\"{}\"", v))
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{\"key\":\"{}\",\"kind\":\"{}\",\"old\":{},\"new\":{}}}",
                e.key, kind, old, new
            )
        })
        .collect();
    format!("{{\"entries\":[{}]}}", entries.join(","))
}

#[allow(dead_code)]
pub fn diff_has_changes(diff: &DataDiff) -> bool {
    !diff.entries.is_empty()
}

/// Apply diff to a mutable key-value map (represented as Vec of owned pairs).
#[allow(dead_code)]
pub fn diff_apply(snapshot: &mut Vec<(String, String)>, diff: &DataDiff) {
    for entry in &diff.entries {
        match entry.kind {
            DiffKind::Added => {
                if let Some(v) = &entry.new_value {
                    snapshot.push((entry.key.clone(), v.clone()));
                }
            }
            DiffKind::Removed => {
                snapshot.retain(|(k, _)| k != &entry.key);
            }
            DiffKind::Changed => {
                if let Some(v) = &entry.new_value {
                    for (k, val) in snapshot.iter_mut() {
                        if k == &entry.key {
                            *val = v.clone();
                        }
                    }
                }
            }
        }
    }
}

/// Invert a diff (swap added/removed, swap old/new for changed).
#[allow(dead_code)]
pub fn diff_invert(diff: &DataDiff) -> DataDiff {
    let entries = diff
        .entries
        .iter()
        .map(|e| {
            let (kind, old, new) = match e.kind {
                DiffKind::Added => (DiffKind::Removed, e.new_value.clone(), e.old_value.clone()),
                DiffKind::Removed => (DiffKind::Added, e.new_value.clone(), e.old_value.clone()),
                DiffKind::Changed => (
                    DiffKind::Changed,
                    e.new_value.clone(),
                    e.old_value.clone(),
                ),
            };
            DiffEntry {
                key: e.key.clone(),
                kind,
                old_value: old,
                new_value: new,
            }
        })
        .collect();
    DataDiff {
        config: diff.config.clone(),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> DataDiffConfig {
        default_data_diff_config()
    }

    #[test]
    fn test_added() {
        let before = vec![("a", "1")];
        let after = vec![("a", "1"), ("b", "2")];
        let d = compute_diff(&before, &after, &cfg());
        assert_eq!(diff_added_count(&d), 1);
        assert_eq!(diff_removed_count(&d), 0);
        assert_eq!(diff_changed_count(&d), 0);
    }

    #[test]
    fn test_removed() {
        let before = vec![("a", "1"), ("b", "2")];
        let after = vec![("a", "1")];
        let d = compute_diff(&before, &after, &cfg());
        assert_eq!(diff_removed_count(&d), 1);
    }

    #[test]
    fn test_changed() {
        let before = vec![("a", "1")];
        let after = vec![("a", "2")];
        let d = compute_diff(&before, &after, &cfg());
        assert_eq!(diff_changed_count(&d), 1);
    }

    #[test]
    fn test_no_changes() {
        let before = vec![("a", "1"), ("b", "2")];
        let after = before.clone();
        let d = compute_diff(&before, &after, &cfg());
        assert!(!diff_has_changes(&d));
    }

    #[test]
    fn test_to_json() {
        let before = vec![("x", "10")];
        let after = vec![("x", "20")];
        let d = compute_diff(&before, &after, &cfg());
        let j = diff_to_json(&d);
        assert!(j.contains("entries"));
        assert!(j.contains("changed"));
    }

    #[test]
    fn test_entry_count() {
        let before = vec![("a", "1"), ("b", "2")];
        let after = vec![("b", "3"), ("c", "4")];
        let d = compute_diff(&before, &after, &cfg());
        assert_eq!(diff_entry_count(&d), 3); // removed a, changed b, added c
    }

    #[test]
    fn test_apply_add() {
        let mut snapshot: Vec<(String, String)> = vec![("a".into(), "1".into())];
        let before = vec![("a", "1")];
        let after = vec![("a", "1"), ("b", "2")];
        let d = compute_diff(&before, &after, &cfg());
        diff_apply(&mut snapshot, &d);
        assert!(snapshot.iter().any(|(k, v)| k == "b" && v == "2"));
    }

    #[test]
    fn test_invert_added_becomes_removed() {
        let before = vec![];
        let after = vec![("z", "99")];
        let d = compute_diff(&before, &after, &cfg());
        let inv = diff_invert(&d);
        assert_eq!(diff_removed_count(&inv), 1);
    }

    #[test]
    fn test_invert_changed_swaps_values() {
        let before = vec![("k", "old")];
        let after = vec![("k", "new")];
        let d = compute_diff(&before, &after, &cfg());
        let inv = diff_invert(&d);
        let e = &inv.entries[0];
        assert_eq!(e.new_value.as_deref(), Some("old"));
        assert_eq!(e.old_value.as_deref(), Some("new"));
    }
}
