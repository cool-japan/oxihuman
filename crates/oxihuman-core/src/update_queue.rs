// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ordered update queue: items enqueued with a priority, drained in order.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// An item pending in the update queue.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateItem {
    pub priority: u32,
    pub id: u32,
    pub tag: String,
}

impl PartialOrd for UpdateItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UpdateItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Reverse(self.priority).cmp(&Reverse(other.priority))
    }
}

/// An update queue sorted by priority (lowest = first).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct UpdateQueue {
    heap: BinaryHeap<UpdateItem>,
    total_enqueued: u64,
}

/// Create a new `UpdateQueue`.
#[allow(dead_code)]
pub fn new_update_queue() -> UpdateQueue {
    UpdateQueue::default()
}

/// Enqueue an item.
#[allow(dead_code)]
pub fn uq_enqueue(q: &mut UpdateQueue, id: u32, priority: u32, tag: &str) {
    q.heap.push(UpdateItem {
        priority,
        id,
        tag: tag.to_string(),
    });
    q.total_enqueued += 1;
}

/// Dequeue the highest-priority item (lowest priority number).
#[allow(dead_code)]
pub fn uq_dequeue(q: &mut UpdateQueue) -> Option<UpdateItem> {
    q.heap.pop()
}

/// Peek at the next item without removing.
#[allow(dead_code)]
pub fn uq_peek(q: &UpdateQueue) -> Option<&UpdateItem> {
    q.heap.peek()
}

/// Number of items pending.
#[allow(dead_code)]
pub fn uq_len(q: &UpdateQueue) -> usize {
    q.heap.len()
}

/// Whether the queue is empty.
#[allow(dead_code)]
pub fn uq_is_empty(q: &UpdateQueue) -> bool {
    q.heap.is_empty()
}

/// Total items ever enqueued.
#[allow(dead_code)]
pub fn uq_total_enqueued(q: &UpdateQueue) -> u64 {
    q.total_enqueued
}

/// Drain all items into a sorted Vec (ascending priority).
#[allow(dead_code)]
pub fn uq_drain_all(q: &mut UpdateQueue) -> Vec<UpdateItem> {
    let mut items = Vec::new();
    while let Some(item) = q.heap.pop() {
        items.push(item);
    }
    items
}

/// Clear the queue.
#[allow(dead_code)]
pub fn uq_clear(q: &mut UpdateQueue) {
    q.heap.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let q = new_update_queue();
        assert!(uq_is_empty(&q));
    }

    #[test]
    fn test_enqueue_and_len() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 1, 5, "x");
        assert_eq!(uq_len(&q), 1);
    }

    #[test]
    fn test_priority_order() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 1, 10, "low");
        uq_enqueue(&mut q, 2, 1, "high");
        let first = uq_dequeue(&mut q).unwrap();
        assert_eq!(first.id, 2);
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 1, 3, "x");
        uq_peek(&q);
        assert_eq!(uq_len(&q), 1);
    }

    #[test]
    fn test_drain_all() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 3, 5, "c");
        uq_enqueue(&mut q, 1, 1, "a");
        uq_enqueue(&mut q, 2, 3, "b");
        let items = uq_drain_all(&mut q);
        assert_eq!(items.len(), 3);
        assert!(uq_is_empty(&q));
        assert_eq!(items[0].id, 1);
    }

    #[test]
    fn test_clear() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 1, 1, "x");
        uq_clear(&mut q);
        assert!(uq_is_empty(&q));
    }

    #[test]
    fn test_total_enqueued() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 1, 1, "a");
        uq_enqueue(&mut q, 2, 2, "b");
        assert_eq!(uq_total_enqueued(&q), 2);
    }

    #[test]
    fn test_dequeue_empty() {
        let mut q = new_update_queue();
        assert!(uq_dequeue(&mut q).is_none());
    }

    #[test]
    fn test_tag_preserved() {
        let mut q = new_update_queue();
        uq_enqueue(&mut q, 1, 0, "mytag");
        let item = uq_dequeue(&mut q).unwrap();
        assert_eq!(item.tag, "mytag".to_string());
    }
}
