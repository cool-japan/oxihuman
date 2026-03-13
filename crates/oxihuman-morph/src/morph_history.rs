// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A snapshot of morph weights at a given timestamp.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightSnapshot {
    pub weights: Vec<f32>,
    pub timestamp: u64,
}

/// Undo/redo history for morph weight changes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphHistory {
    pub snapshots: Vec<WeightSnapshot>,
    /// Points to the current snapshot index (0 = nothing undone, 1 = one undo done).
    pub cursor: usize,
    pub max_depth: usize,
}

/// Create a new MorphHistory with the given max depth.
#[allow(dead_code)]
pub fn new_morph_history(max_depth: usize) -> MorphHistory {
    MorphHistory {
        snapshots: Vec::new(),
        cursor: 0,
        max_depth: max_depth.max(1),
    }
}

/// Push a new snapshot, discarding any redo history.
#[allow(dead_code)]
pub fn push_snapshot(hist: &mut MorphHistory, weights: Vec<f32>, ts: u64) {
    // Discard everything after the current position
    if hist.cursor > 0 {
        let keep = hist.snapshots.len() - hist.cursor;
        hist.snapshots.truncate(keep);
        hist.cursor = 0;
    }
    hist.snapshots.push(WeightSnapshot { weights, timestamp: ts });
    // Enforce max depth
    while hist.snapshots.len() > hist.max_depth {
        hist.snapshots.remove(0);
    }
}

/// Move backward in history, returning the previous snapshot's weights.
#[allow(dead_code)]
pub fn undo(hist: &mut MorphHistory) -> Option<&[f32]> {
    if !can_undo(hist) {
        return None;
    }
    hist.cursor += 1;
    let idx = hist.snapshots.len() - 1 - hist.cursor;
    Some(&hist.snapshots[idx].weights)
}

/// Move forward in history, returning the next snapshot's weights.
#[allow(dead_code)]
pub fn redo(hist: &mut MorphHistory) -> Option<&[f32]> {
    if !can_redo(hist) {
        return None;
    }
    hist.cursor -= 1;
    let idx = hist.snapshots.len() - 1 - hist.cursor;
    Some(&hist.snapshots[idx].weights)
}

/// Return whether there is a previous snapshot to undo to.
#[allow(dead_code)]
pub fn can_undo(hist: &MorphHistory) -> bool {
    hist.cursor + 1 < hist.snapshots.len()
}

/// Return whether there is a future snapshot to redo to.
#[allow(dead_code)]
pub fn can_redo(hist: &MorphHistory) -> bool {
    hist.cursor > 0
}

/// A single entry in the morph history log.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub target: String,
    pub old_value: f32,
    pub new_value: f32,
}

/// Record a morph weight change, storing the old and new values.
#[allow(dead_code)]
pub fn record_change(hist: &mut MorphHistory, target: &str, old_value: f32, new_value: f32) {
    let entry = HistoryEntry {
        target: target.to_string(),
        old_value,
        new_value,
    };
    // Discard redo history
    if hist.cursor > 0 {
        let keep = hist.snapshots.len() - hist.cursor;
        hist.snapshots.truncate(keep);
        hist.cursor = 0;
    }
    push_snapshot(hist, vec![entry.old_value, entry.new_value], hist.snapshots.len() as u64);
}

/// Undo the last morph change, returning the snapshot weights.
#[allow(dead_code)]
pub fn undo_morph(hist: &mut MorphHistory) -> Option<&[f32]> {
    undo(hist)
}

/// Redo the last undone morph change, returning the snapshot weights.
#[allow(dead_code)]
pub fn redo_morph(hist: &mut MorphHistory) -> Option<&[f32]> {
    redo(hist)
}

/// Return the total number of snapshots in the history.
#[allow(dead_code)]
pub fn history_count(hist: &MorphHistory) -> usize {
    hist.snapshots.len()
}

/// Clear all history entries.
#[allow(dead_code)]
pub fn history_clear(hist: &mut MorphHistory) {
    hist.snapshots.clear();
    hist.cursor = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_history_is_empty() {
        let h = new_morph_history(10);
        assert!(!can_undo(&h));
        assert!(!can_redo(&h));
    }

    #[test]
    fn push_enables_undo() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.0], 1);
        push_snapshot(&mut h, vec![1.0], 2);
        assert!(can_undo(&h));
    }

    #[test]
    fn undo_returns_previous_weights() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.2], 1);
        push_snapshot(&mut h, vec![0.8], 2);
        let prev = undo(&mut h).expect("should succeed");
        assert!((prev[0] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn redo_after_undo() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.1], 1);
        push_snapshot(&mut h, vec![0.9], 2);
        undo(&mut h);
        assert!(can_redo(&h));
        let next = redo(&mut h).expect("should succeed");
        assert!((next[0] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn undo_when_empty_returns_none() {
        let mut h = new_morph_history(10);
        assert!(undo(&mut h).is_none());
    }

    #[test]
    fn redo_without_undo_returns_none() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.5], 1);
        assert!(redo(&mut h).is_none());
    }

    #[test]
    fn push_clears_redo() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.1], 1);
        push_snapshot(&mut h, vec![0.5], 2);
        undo(&mut h);
        push_snapshot(&mut h, vec![0.3], 3);
        assert!(!can_redo(&h));
    }

    #[test]
    fn max_depth_enforced() {
        let mut h = new_morph_history(3);
        for i in 0..6u64 {
            push_snapshot(&mut h, vec![i as f32 * 0.1], i);
        }
        assert!(h.snapshots.len() <= 3);
    }

    #[test]
    fn multiple_undos() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.1], 1);
        push_snapshot(&mut h, vec![0.2], 2);
        push_snapshot(&mut h, vec![0.3], 3);
        undo(&mut h);
        undo(&mut h);
        let w = undo(&mut h);
        assert!(w.is_none()); // can't go past the beginning
    }

    #[test]
    fn record_change_adds_snapshot() {
        let mut h = new_morph_history(10);
        record_change(&mut h, "smile", 0.0, 0.5);
        assert_eq!(history_count(&h), 1);
    }

    #[test]
    fn undo_morph_works() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.1], 1);
        push_snapshot(&mut h, vec![0.9], 2);
        let w = undo_morph(&mut h).expect("should succeed");
        assert!((w[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn redo_morph_works() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.1], 1);
        push_snapshot(&mut h, vec![0.9], 2);
        undo_morph(&mut h);
        let w = redo_morph(&mut h).expect("should succeed");
        assert!((w[0] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn history_clear_works() {
        let mut h = new_morph_history(10);
        push_snapshot(&mut h, vec![0.1], 1);
        history_clear(&mut h);
        assert_eq!(history_count(&h), 0);
    }
}
