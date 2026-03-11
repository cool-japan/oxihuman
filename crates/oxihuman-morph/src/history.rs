// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::params::ParamState;

/// A single recorded state in the history.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub params: ParamState,
    /// Optional label describing what changed (e.g. "Set height to 0.7").
    pub label: Option<String>,
}

impl HistoryEntry {
    pub fn new(params: ParamState) -> Self {
        Self {
            params,
            label: None,
        }
    }

    pub fn with_label(params: ParamState, label: impl Into<String>) -> Self {
        Self {
            params,
            label: Some(label.into()),
        }
    }
}

/// Undo/redo history stack for ParamState.
/// Maintains a current index into a vec of entries.
/// Max capacity prevents unbounded memory growth.
pub struct History {
    entries: Vec<HistoryEntry>,
    /// Index of the current state (0-based). Meaningful only when entries is non-empty.
    current: usize,
    /// Maximum number of entries to retain.
    max_entries: usize,
}

impl History {
    /// Create a new history with the given capacity.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            current: 0,
            max_entries,
        }
    }

    /// Create a new history with default capacity of 50.
    pub fn with_default_capacity() -> Self {
        Self::new(50)
    }

    /// Push a new state. Discards any redo states beyond current.
    /// If at capacity, drops the oldest entry.
    pub fn push(&mut self, entry: HistoryEntry) {
        // Discard all entries after current (truncate redo states)
        if !self.entries.is_empty() && self.current < self.entries.len() - 1 {
            self.entries.truncate(self.current + 1);
        }
        self.entries.push(entry);
        // If over capacity, drop oldest entry
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        self.current = self.entries.len() - 1;
    }

    /// Undo: move back one step. Returns the previous state, or None if at start.
    pub fn undo(&mut self) -> Option<&ParamState> {
        if self.entries.is_empty() || self.current == 0 {
            return None;
        }
        self.current -= 1;
        Some(&self.entries[self.current].params)
    }

    /// Redo: move forward one step. Returns the next state, or None if at end.
    pub fn redo(&mut self) -> Option<&ParamState> {
        if self.entries.is_empty() || self.current >= self.entries.len() - 1 {
            return None;
        }
        self.current += 1;
        Some(&self.entries[self.current].params)
    }

    /// Current state, or None if history is empty.
    pub fn current(&self) -> Option<&ParamState> {
        if self.entries.is_empty() {
            None
        } else {
            Some(&self.entries[self.current].params)
        }
    }

    /// Number of entries in history.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if history is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// True if undo is available (current > 0 and history non-empty).
    pub fn can_undo(&self) -> bool {
        !self.entries.is_empty() && self.current > 0
    }

    /// True if redo is available (current < len - 1).
    pub fn can_redo(&self) -> bool {
        !self.entries.is_empty() && self.current < self.entries.len() - 1
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current = 0;
    }

    /// All entry labels (for displaying a history list to the user).
    pub fn labels(&self) -> Vec<Option<&str>> {
        self.entries.iter().map(|e| e.label.as_deref()).collect()
    }

    /// Jump directly to a specific history index. Returns the state or None if out of bounds.
    pub fn jump_to(&mut self, index: usize) -> Option<&ParamState> {
        if index >= self.entries.len() {
            return None;
        }
        self.current = index;
        Some(&self.entries[self.current].params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(height: f32) -> HistoryEntry {
        HistoryEntry::new(ParamState::new(height, 0.5, 0.5, 0.5))
    }

    fn make_labeled(height: f32, label: &str) -> HistoryEntry {
        HistoryEntry::with_label(ParamState::new(height, 0.5, 0.5, 0.5), label)
    }

    #[test]
    fn push_and_current() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.7));
        let state = h.current().expect("should have current");
        assert!((state.height - 0.7).abs() < 1e-6);
    }

    #[test]
    fn undo_returns_previous() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.3)); // A
        h.push(make_entry(0.8)); // B
        let prev = h.undo().expect("should undo");
        assert!((prev.height - 0.3).abs() < 1e-6);
    }

    #[test]
    fn redo_after_undo() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.3)); // A
        h.push(make_entry(0.8)); // B
        h.undo();
        let next = h.redo().expect("should redo");
        assert!((next.height - 0.8).abs() < 1e-6);
    }

    #[test]
    fn undo_at_start_returns_none() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.5));
        assert!(h.undo().is_none());
    }

    #[test]
    fn redo_at_end_returns_none() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.5));
        assert!(h.redo().is_none());
    }

    #[test]
    fn push_clears_redo() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.1)); // A
        h.push(make_entry(0.2)); // B
        h.undo(); // back to A
        h.push(make_entry(0.9)); // C (discards B)
        assert!(h.redo().is_none());
    }

    #[test]
    fn capacity_limit_respected() {
        let mut h = History::new(50);
        for i in 0..55 {
            h.push(make_entry(i as f32 / 55.0));
        }
        assert_eq!(h.len(), 50);
    }

    #[test]
    fn can_undo_false_when_empty() {
        let h = History::with_default_capacity();
        assert!(!h.can_undo());
    }

    #[test]
    fn can_redo_true_after_undo() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.1)); // A
        h.push(make_entry(0.2)); // B
        h.undo();
        assert!(h.can_redo());
    }

    #[test]
    fn jump_to_valid_index() {
        let mut h = History::with_default_capacity();
        h.push(make_entry(0.1)); // index 0
        h.push(make_entry(0.5)); // index 1
        h.push(make_entry(0.9)); // index 2
        let state = h.jump_to(0).expect("should jump");
        assert!((state.height - 0.1).abs() < 1e-6);
    }

    #[test]
    fn labels_all_entries() {
        let mut h = History::with_default_capacity();
        h.push(make_labeled(0.1, "first"));
        h.push(make_labeled(0.5, "second"));
        h.push(make_labeled(0.9, "third"));
        assert_eq!(h.labels().len(), 3);
    }
}
