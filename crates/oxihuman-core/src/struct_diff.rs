// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A flat key-value snapshot.
#[allow(dead_code)]
pub struct Snapshot {
    pub fields: Vec<(String, String)>,
}

/// Describes a change between two snapshots for a single field.
#[allow(dead_code)]
pub struct FieldDiff {
    pub key: String,
    pub old_val: Option<String>,
    pub new_val: Option<String>,
}

/// Create a new empty snapshot.
#[allow(dead_code)]
pub fn new_snapshot() -> Snapshot {
    Snapshot { fields: Vec::new() }
}

/// Set a key-value pair in the snapshot (updates if key exists).
#[allow(dead_code)]
pub fn snap_set(s: &mut Snapshot, key: &str, val: &str) {
    if let Some(entry) = s.fields.iter_mut().find(|(k, _)| k == key) {
        entry.1 = val.to_string();
    } else {
        s.fields.push((key.to_string(), val.to_string()));
    }
}

/// Get the value for a key from a snapshot.
#[allow(dead_code)]
pub fn snap_get<'a>(s: &'a Snapshot, key: &str) -> Option<&'a str> {
    s.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// Compute the diff between two snapshots.
#[allow(dead_code)]
pub fn diff_snapshots(old: &Snapshot, new: &Snapshot) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    // Check for changed or removed fields.
    for (old_key, old_val) in &old.fields {
        let new_val = snap_get(new, old_key);
        if new_val != Some(old_val.as_str()) {
            diffs.push(FieldDiff {
                key: old_key.clone(),
                old_val: Some(old_val.clone()),
                new_val: new_val.map(|v| v.to_string()),
            });
        }
    }

    // Check for added fields.
    for (new_key, new_val) in &new.fields {
        if snap_get(old, new_key).is_none() {
            diffs.push(FieldDiff {
                key: new_key.clone(),
                old_val: None,
                new_val: Some(new_val.clone()),
            });
        }
    }

    diffs
}

/// Return the number of diffs.
#[allow(dead_code)]
pub fn diff_count(diffs: &[FieldDiff]) -> usize {
    diffs.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_snapshots_no_diff() {
        let old = new_snapshot();
        let new = new_snapshot();
        assert_eq!(diff_count(&diff_snapshots(&old, &new)), 0);
    }

    #[test]
    fn identical_snapshots_no_diff() {
        let mut old = new_snapshot();
        snap_set(&mut old, "x", "1");
        let mut new = new_snapshot();
        snap_set(&mut new, "x", "1");
        assert_eq!(diff_count(&diff_snapshots(&old, &new)), 0);
    }

    #[test]
    fn changed_value_produces_diff() {
        let mut old = new_snapshot();
        snap_set(&mut old, "x", "1");
        let mut new = new_snapshot();
        snap_set(&mut new, "x", "2");
        let diffs = diff_snapshots(&old, &new);
        assert_eq!(diff_count(&diffs), 1);
        assert_eq!(diffs[0].old_val.as_deref(), Some("1"));
        assert_eq!(diffs[0].new_val.as_deref(), Some("2"));
    }

    #[test]
    fn added_field_produces_diff() {
        let old = new_snapshot();
        let mut new = new_snapshot();
        snap_set(&mut new, "y", "hello");
        let diffs = diff_snapshots(&old, &new);
        assert_eq!(diff_count(&diffs), 1);
        assert!(diffs[0].old_val.is_none());
    }

    #[test]
    fn removed_field_produces_diff() {
        let mut old = new_snapshot();
        snap_set(&mut old, "z", "val");
        let new = new_snapshot();
        let diffs = diff_snapshots(&old, &new);
        assert_eq!(diff_count(&diffs), 1);
        assert!(diffs[0].new_val.is_none());
    }

    #[test]
    fn snap_set_updates_existing() {
        let mut s = new_snapshot();
        snap_set(&mut s, "k", "a");
        snap_set(&mut s, "k", "b");
        assert_eq!(snap_get(&s, "k"), Some("b"));
        assert_eq!(s.fields.len(), 1);
    }

    #[test]
    fn snap_get_missing_returns_none() {
        let s = new_snapshot();
        assert!(snap_get(&s, "missing").is_none());
    }

    #[test]
    fn multiple_diffs() {
        let mut old = new_snapshot();
        snap_set(&mut old, "a", "1");
        snap_set(&mut old, "b", "2");
        let mut new = new_snapshot();
        snap_set(&mut new, "a", "10");
        snap_set(&mut new, "c", "3");
        let diffs = diff_snapshots(&old, &new);
        // "a" changed, "b" removed, "c" added
        assert_eq!(diff_count(&diffs), 3);
    }

    #[test]
    fn diff_key_preserved() {
        let mut old = new_snapshot();
        snap_set(&mut old, "mykey", "v1");
        let mut new = new_snapshot();
        snap_set(&mut new, "mykey", "v2");
        let diffs = diff_snapshots(&old, &new);
        assert_eq!(diffs[0].key, "mykey");
    }
}
