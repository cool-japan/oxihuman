//! Change tracker — records which fields/properties have been modified since the last flush.

// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A record of a single field change.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeRecord {
    /// Name of the field that changed.
    pub field: String,
    /// Whether the field is currently dirty.
    pub dirty: bool,
}

/// Configuration for the change tracker.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChangeTrackerConfig {
    /// Maximum number of records to keep (0 = unlimited).
    pub max_records: usize,
}

/// Tracks which named fields have been marked as changed.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChangeTracker {
    pub config: ChangeTrackerConfig,
    pub records: Vec<ChangeRecord>,
}

/// Returns a default `ChangeTrackerConfig`.
#[allow(dead_code)]
pub fn default_change_tracker_config() -> ChangeTrackerConfig {
    ChangeTrackerConfig { max_records: 0 }
}

/// Creates a new `ChangeTracker` with the given config.
#[allow(dead_code)]
pub fn new_change_tracker(cfg: &ChangeTrackerConfig) -> ChangeTracker {
    ChangeTracker {
        config: cfg.clone(),
        records: Vec::new(),
    }
}

/// Marks `field` as changed. If the field is already tracked, sets its dirty flag.
#[allow(dead_code)]
pub fn tracker_mark_changed(tracker: &mut ChangeTracker, field: &str) {
    if let Some(rec) = tracker.records.iter_mut().find(|r| r.field == field) {
        rec.dirty = true;
    } else {
        tracker.records.push(ChangeRecord {
            field: field.to_string(),
            dirty: true,
        });
    }
}

/// Returns true if `field` is currently marked as changed (dirty).
#[allow(dead_code)]
pub fn tracker_is_changed(tracker: &ChangeTracker, field: &str) -> bool {
    tracker
        .records
        .iter()
        .any(|r| r.field == field && r.dirty)
}

/// Returns names of all currently dirty fields.
#[allow(dead_code)]
pub fn tracker_changed_fields(tracker: &ChangeTracker) -> Vec<&str> {
    tracker
        .records
        .iter()
        .filter(|r| r.dirty)
        .map(|r| r.field.as_str())
        .collect()
}

/// Marks all dirty fields as clean (not dirty) without removing records.
#[allow(dead_code)]
pub fn tracker_flush(tracker: &mut ChangeTracker) {
    for rec in &mut tracker.records {
        rec.dirty = false;
    }
}

/// Returns the number of currently dirty fields.
#[allow(dead_code)]
pub fn tracker_change_count(tracker: &ChangeTracker) -> usize {
    tracker.records.iter().filter(|r| r.dirty).count()
}

/// Returns true if any field is currently dirty.
#[allow(dead_code)]
pub fn tracker_any_changed(tracker: &ChangeTracker) -> bool {
    tracker.records.iter().any(|r| r.dirty)
}

/// Sets all records to clean without removing them.
#[allow(dead_code)]
pub fn tracker_mark_all_clean(tracker: &mut ChangeTracker) {
    tracker_flush(tracker);
}

/// Returns the total number of records (dirty or clean).
#[allow(dead_code)]
pub fn tracker_record_count(tracker: &ChangeTracker) -> usize {
    tracker.records.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_change_tracker_config();
        assert_eq!(cfg.max_records, 0);
    }

    #[test]
    fn test_new_tracker_empty() {
        let cfg = default_change_tracker_config();
        let tracker = new_change_tracker(&cfg);
        assert_eq!(tracker_record_count(&tracker), 0);
        assert!(!tracker_any_changed(&tracker));
    }

    #[test]
    fn test_mark_and_is_changed() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "position");
        assert!(tracker_is_changed(&tracker, "position"));
        assert!(!tracker_is_changed(&tracker, "rotation"));
    }

    #[test]
    fn test_changed_fields() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "x");
        tracker_mark_changed(&mut tracker, "y");
        let fields = tracker_changed_fields(&tracker);
        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"x"));
        assert!(fields.contains(&"y"));
    }

    #[test]
    fn test_flush() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "scale");
        assert!(tracker_any_changed(&tracker));
        tracker_flush(&mut tracker);
        assert!(!tracker_any_changed(&tracker));
        // Record still exists but is clean
        assert_eq!(tracker_record_count(&tracker), 1);
    }

    #[test]
    fn test_change_count() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "a");
        tracker_mark_changed(&mut tracker, "b");
        tracker_mark_changed(&mut tracker, "c");
        assert_eq!(tracker_change_count(&tracker), 3);
    }

    #[test]
    fn test_mark_all_clean() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "alpha");
        tracker_mark_changed(&mut tracker, "beta");
        tracker_mark_all_clean(&mut tracker);
        assert_eq!(tracker_change_count(&tracker), 0);
        assert_eq!(tracker_record_count(&tracker), 2);
    }

    #[test]
    fn test_mark_changed_idempotent() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "x");
        tracker_mark_changed(&mut tracker, "x"); // second call should not add duplicate
        assert_eq!(tracker_record_count(&tracker), 1);
        assert_eq!(tracker_change_count(&tracker), 1);
    }

    #[test]
    fn test_any_changed_false_after_flush() {
        let cfg = default_change_tracker_config();
        let mut tracker = new_change_tracker(&cfg);
        tracker_mark_changed(&mut tracker, "color");
        tracker_flush(&mut tracker);
        assert!(!tracker_any_changed(&tracker));
        // Mark again after flush
        tracker_mark_changed(&mut tracker, "color");
        assert!(tracker_any_changed(&tracker));
    }
}
