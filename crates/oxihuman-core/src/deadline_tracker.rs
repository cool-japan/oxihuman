// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Deadline/SLA tracker.

use std::collections::HashMap;

/// Status of a tracked deadline.
#[derive(Debug, Clone, PartialEq)]
pub enum DeadlineStatus {
    Pending,
    Met,
    Breached,
}

/// A single deadline entry.
#[derive(Debug, Clone)]
pub struct DeadlineEntry {
    pub id: u64,
    pub name: String,
    pub deadline_ms: u64,
    pub created_ms: u64,
    pub completed_ms: Option<u64>,
    pub status: DeadlineStatus,
}

impl DeadlineEntry {
    pub fn remaining_ms(&self, now_ms: u64) -> Option<i64> {
        if self.status == DeadlineStatus::Pending {
            Some(self.deadline_ms as i64 - now_ms as i64)
        } else {
            None
        }
    }

    pub fn is_overdue(&self, now_ms: u64) -> bool {
        self.status == DeadlineStatus::Pending && now_ms > self.deadline_ms
    }
}

/// Tracks multiple deadlines and their SLA compliance.
#[derive(Debug, Default)]
pub struct DeadlineTracker {
    entries: HashMap<u64, DeadlineEntry>,
    next_id: u64,
}

impl DeadlineTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: &str, created_ms: u64, deadline_ms: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(
            id,
            DeadlineEntry {
                id,
                name: name.to_string(),
                deadline_ms,
                created_ms,
                completed_ms: None,
                status: DeadlineStatus::Pending,
            },
        );
        id
    }

    pub fn complete(&mut self, id: u64, now_ms: u64) -> bool {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.completed_ms = Some(now_ms);
            entry.status = if now_ms <= entry.deadline_ms {
                DeadlineStatus::Met
            } else {
                DeadlineStatus::Breached
            };
            true
        } else {
            false
        }
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn overdue_count(&self, now_ms: u64) -> usize {
        self.entries
            .values()
            .filter(|e| e.is_overdue(now_ms))
            .count()
    }

    pub fn met_count(&self) -> usize {
        self.entries
            .values()
            .filter(|e| e.status == DeadlineStatus::Met)
            .count()
    }

    pub fn breached_count(&self) -> usize {
        self.entries
            .values()
            .filter(|e| e.status == DeadlineStatus::Breached)
            .count()
    }

    pub fn get(&self, id: u64) -> Option<&DeadlineEntry> {
        self.entries.get(&id)
    }
}

pub fn new_deadline_tracker() -> DeadlineTracker {
    DeadlineTracker::new()
}

pub fn dt_add(tracker: &mut DeadlineTracker, name: &str, created_ms: u64, deadline_ms: u64) -> u64 {
    tracker.add(name, created_ms, deadline_ms)
}

pub fn dt_complete(tracker: &mut DeadlineTracker, id: u64, now_ms: u64) -> bool {
    tracker.complete(id, now_ms)
}

pub fn dt_count(tracker: &DeadlineTracker) -> usize {
    tracker.count()
}

pub fn dt_overdue_count(tracker: &DeadlineTracker, now_ms: u64) -> usize {
    tracker.overdue_count(now_ms)
}

pub fn dt_met_count(tracker: &DeadlineTracker) -> usize {
    tracker.met_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_count() {
        let mut t = new_deadline_tracker();
        dt_add(&mut t, "task1", 0, 1000);
        assert_eq!(dt_count(&t), 1);
    }

    #[test]
    fn test_complete_on_time() {
        let mut t = new_deadline_tracker();
        let id = dt_add(&mut t, "fast", 0, 1000);
        dt_complete(&mut t, id, 500);
        assert_eq!(dt_met_count(&t), 1);
    }

    #[test]
    fn test_complete_late_is_breached() {
        let mut t = new_deadline_tracker();
        let id = dt_add(&mut t, "slow", 0, 500);
        dt_complete(&mut t, id, 1000);
        assert_eq!(t.breached_count(), 1);
    }

    #[test]
    fn test_overdue_detection() {
        let mut t = new_deadline_tracker();
        dt_add(&mut t, "x", 0, 100);
        assert_eq!(dt_overdue_count(&t, 200), 1);
    }

    #[test]
    fn test_not_overdue_before_deadline() {
        let mut t = new_deadline_tracker();
        dt_add(&mut t, "x", 0, 100);
        assert_eq!(dt_overdue_count(&t, 50), 0);
    }

    #[test]
    fn test_remaining_ms() {
        let mut t = new_deadline_tracker();
        let id = dt_add(&mut t, "x", 0, 1000);
        let remaining = t.get(id).unwrap().remaining_ms(400);
        assert_eq!(remaining, Some(600));
    }

    #[test]
    fn test_complete_unknown_id() {
        let mut t = new_deadline_tracker();
        assert!(!dt_complete(&mut t, 999, 0));
    }

    #[test]
    fn test_met_on_exact_deadline() {
        let mut t = new_deadline_tracker();
        let id = dt_add(&mut t, "exact", 0, 500);
        dt_complete(&mut t, id, 500);
        assert_eq!(dt_met_count(&t), 1);
    }

    #[test]
    fn test_multiple_deadlines() {
        let mut t = new_deadline_tracker();
        let id1 = dt_add(&mut t, "a", 0, 100);
        let id2 = dt_add(&mut t, "b", 0, 200);
        dt_complete(&mut t, id1, 50);
        dt_complete(&mut t, id2, 300);
        assert_eq!(dt_met_count(&t), 1);
        assert_eq!(t.breached_count(), 1);
    }
}
