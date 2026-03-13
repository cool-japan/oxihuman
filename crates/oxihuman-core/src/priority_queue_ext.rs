// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Extended priority queue with decrease-key via BinaryHeap + HashMap.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

/// An entry in the priority queue.
#[derive(Debug, Clone, PartialEq)]
pub struct PqEntry {
    pub key: String,
    pub priority: i64,
}

impl Eq for PqEntry {}

impl PartialOrd for PqEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PqEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority) /* natural order; Reverse wrapper makes min-heap */
    }
}

/// Extended priority queue supporting decrease-key.
pub struct PriorityQueueExt {
    heap: BinaryHeap<Reverse<PqEntry>>,
    current: HashMap<String, i64>,
}

/// Construct a new PriorityQueueExt.
pub fn new_priority_queue_ext() -> PriorityQueueExt {
    PriorityQueueExt {
        heap: BinaryHeap::new(),
        current: HashMap::new(),
    }
}

impl PriorityQueueExt {
    /// Insert or update the priority of `key`.
    pub fn insert(&mut self, key: &str, priority: i64) {
        self.current.insert(key.to_string(), priority);
        self.heap.push(Reverse(PqEntry {
            key: key.to_string(),
            priority,
        }));
    }

    /// Decrease the priority of `key` (lower value = higher priority).
    pub fn decrease_key(&mut self, key: &str, new_priority: i64) {
        if let Some(p) = self.current.get_mut(key) {
            if new_priority < *p {
                *p = new_priority;
                self.heap.push(Reverse(PqEntry {
                    key: key.to_string(),
                    priority: new_priority,
                }));
            }
        }
    }

    /// Pop the entry with the lowest priority value.
    pub fn pop_min(&mut self) -> Option<PqEntry> {
        while let Some(Reverse(entry)) = self.heap.pop() {
            if let Some(&cur) = self.current.get(&entry.key) {
                if cur == entry.priority {
                    self.current.remove(&entry.key);
                    return Some(entry);
                }
                /* stale entry, skip */
            }
        }
        None
    }

    /// Peek at the minimum entry without removing.
    pub fn peek_min(&self) -> Option<(&str, i64)> {
        self.current
            .iter()
            .min_by_key(|(_, &p)| p)
            .map(|(k, &p)| (k.as_str(), p))
    }

    /// Number of live entries.
    pub fn len(&self) -> usize {
        self.current.len()
    }

    /// Whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    /// Check whether `key` is in the queue.
    pub fn contains(&self, key: &str) -> bool {
        self.current.contains_key(key)
    }

    /// Get the current priority of `key`.
    pub fn priority_of(&self, key: &str) -> Option<i64> {
        self.current.get(key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        /* new queue is empty */
        let pq = new_priority_queue_ext();
        assert!(pq.is_empty());
        assert_eq!(pq.len(), 0);
    }

    #[test]
    fn test_insert_and_pop() {
        /* insert then pop returns the entry */
        let mut pq = new_priority_queue_ext();
        pq.insert("a", 10);
        let e = pq.pop_min().expect("should succeed");
        assert_eq!(e.key, "a");
        assert_eq!(e.priority, 10);
    }

    #[test]
    fn test_min_heap_order() {
        /* pop_min returns entries in ascending priority order */
        let mut pq = new_priority_queue_ext();
        pq.insert("c", 30);
        pq.insert("a", 10);
        pq.insert("b", 20);
        let first = pq.pop_min().expect("should succeed");
        assert_eq!(first.priority, 10);
    }

    #[test]
    fn test_decrease_key() {
        /* decrease_key lowers priority and affects pop order */
        let mut pq = new_priority_queue_ext();
        pq.insert("a", 50);
        pq.insert("b", 20);
        pq.decrease_key("a", 5);
        let first = pq.pop_min().expect("should succeed");
        assert_eq!(first.key, "a");
        assert_eq!(first.priority, 5);
    }

    #[test]
    fn test_contains() {
        /* contains returns correct bool */
        let mut pq = new_priority_queue_ext();
        pq.insert("x", 1);
        assert!(pq.contains("x"));
        assert!(!pq.contains("y"));
    }

    #[test]
    fn test_priority_of() {
        /* priority_of returns current priority */
        let mut pq = new_priority_queue_ext();
        pq.insert("z", 42);
        assert_eq!(pq.priority_of("z"), Some(42));
        assert_eq!(pq.priority_of("missing"), None);
    }

    #[test]
    fn test_pop_removes_from_len() {
        /* len decreases after pop */
        let mut pq = new_priority_queue_ext();
        pq.insert("a", 1);
        pq.insert("b", 2);
        assert_eq!(pq.len(), 2);
        pq.pop_min();
        assert_eq!(pq.len(), 1);
    }

    #[test]
    fn test_decrease_key_no_increase() {
        /* decrease_key with larger value is a no-op */
        let mut pq = new_priority_queue_ext();
        pq.insert("a", 10);
        pq.decrease_key("a", 20);
        assert_eq!(pq.priority_of("a"), Some(10));
    }
}
