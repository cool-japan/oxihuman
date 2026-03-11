// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Tracks changes (diffs) to named properties over time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub property: String,
    pub old_value: String,
    pub new_value: String,
    pub timestamp: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffTracker {
    entries: Vec<DiffEntry>,
    clock: u64,
    max_entries: usize,
}

#[allow(dead_code)]
impl DiffTracker {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            clock: 0,
            max_entries: max_entries.max(1),
        }
    }

    pub fn advance_clock(&mut self, dt: u64) {
        self.clock += dt;
    }

    pub fn record(&mut self, property: &str, old_value: &str, new_value: &str) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(DiffEntry {
            property: property.to_string(),
            old_value: old_value.to_string(),
            new_value: new_value.to_string(),
            timestamp: self.clock,
        });
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn latest(&self) -> Option<&DiffEntry> {
        self.entries.last()
    }

    pub fn by_property(&self, property: &str) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.property == property)
            .collect()
    }

    pub fn since(&self, timestamp: u64) -> Vec<&DiffEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp >= timestamp)
            .collect()
    }

    pub fn changed_properties(&self) -> Vec<String> {
        let mut props: Vec<String> = self.entries.iter().map(|e| e.property.clone()).collect();
        props.sort();
        props.dedup();
        props
    }

    pub fn has_changes_for(&self, property: &str) -> bool {
        self.entries.iter().any(|e| e.property == property)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn revert_latest(&mut self) -> Option<DiffEntry> {
        self.entries.pop()
    }

    pub fn all_entries(&self) -> &[DiffEntry] {
        &self.entries
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dt = DiffTracker::new(100);
        assert!(dt.is_empty());
    }

    #[test]
    fn test_record() {
        let mut dt = DiffTracker::new(100);
        dt.record("color", "red", "blue");
        assert_eq!(dt.count(), 1);
    }

    #[test]
    fn test_latest() {
        let mut dt = DiffTracker::new(100);
        dt.record("a", "1", "2");
        dt.record("b", "3", "4");
        assert_eq!(dt.latest().unwrap().property, "b");
    }

    #[test]
    fn test_by_property() {
        let mut dt = DiffTracker::new(100);
        dt.record("x", "1", "2");
        dt.record("y", "3", "4");
        dt.record("x", "2", "3");
        assert_eq!(dt.by_property("x").len(), 2);
    }

    #[test]
    fn test_since() {
        let mut dt = DiffTracker::new(100);
        dt.record("a", "1", "2");
        dt.advance_clock(10);
        dt.record("b", "3", "4");
        assert_eq!(dt.since(5).len(), 1);
    }

    #[test]
    fn test_changed_properties() {
        let mut dt = DiffTracker::new(100);
        dt.record("b", "1", "2");
        dt.record("a", "3", "4");
        dt.record("b", "2", "3");
        let props = dt.changed_properties();
        assert_eq!(props, vec!["a", "b"]);
    }

    #[test]
    fn test_max_entries_eviction() {
        let mut dt = DiffTracker::new(2);
        dt.record("a", "1", "2");
        dt.record("b", "3", "4");
        dt.record("c", "5", "6");
        assert_eq!(dt.count(), 2);
        assert!(!dt.has_changes_for("a"));
    }

    #[test]
    fn test_revert_latest() {
        let mut dt = DiffTracker::new(100);
        dt.record("x", "old", "new");
        let reverted = dt.revert_latest().unwrap();
        assert_eq!(reverted.property, "x");
        assert!(dt.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut dt = DiffTracker::new(100);
        dt.record("a", "1", "2");
        dt.clear();
        assert!(dt.is_empty());
    }

    #[test]
    fn test_has_changes_for() {
        let mut dt = DiffTracker::new(100);
        assert!(!dt.has_changes_for("x"));
        dt.record("x", "1", "2");
        assert!(dt.has_changes_for("x"));
    }
}
