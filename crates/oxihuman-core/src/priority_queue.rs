// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Min-heap priority queue.

#![allow(dead_code)]

/// An item in the priority queue.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PriorityItem {
    pub key: u32,
    pub priority: f32,
}

/// A min-heap priority queue ordered by priority (lowest first).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PriorityQueue {
    heap: Vec<PriorityItem>,
}

/// Create a new empty priority queue.
#[allow(dead_code)]
pub fn new_priority_queue() -> PriorityQueue {
    PriorityQueue { heap: Vec::new() }
}

/// Push an item onto the queue.
#[allow(dead_code)]
pub fn pq_push(pq: &mut PriorityQueue, key: u32, priority: f32) {
    pq.heap.push(PriorityItem { key, priority });
    sift_up(pq, pq.heap.len() - 1);
}

/// Pop the item with the lowest priority.
#[allow(dead_code)]
pub fn pq_pop(pq: &mut PriorityQueue) -> Option<PriorityItem> {
    if pq.heap.is_empty() {
        return None;
    }
    let last = pq.heap.len() - 1;
    pq.heap.swap(0, last);
    let item = pq.heap.pop();
    if !pq.heap.is_empty() {
        sift_down(pq, 0);
    }
    item
}

/// Peek at the item with the lowest priority without removing it.
#[allow(dead_code)]
pub fn pq_peek(pq: &PriorityQueue) -> Option<&PriorityItem> {
    pq.heap.first()
}

/// Return the number of items in the queue.
#[allow(dead_code)]
pub fn pq_len(pq: &PriorityQueue) -> usize {
    pq.heap.len()
}

/// Check whether the queue is empty.
#[allow(dead_code)]
pub fn pq_is_empty(pq: &PriorityQueue) -> bool {
    pq.heap.is_empty()
}

/// Clear all items.
#[allow(dead_code)]
pub fn pq_clear(pq: &mut PriorityQueue) {
    pq.heap.clear();
}

/// Check whether a key is present in the queue.
#[allow(dead_code)]
pub fn pq_contains_key(pq: &PriorityQueue, key: u32) -> bool {
    pq.heap.iter().any(|item| item.key == key)
}

fn sift_up(pq: &mut PriorityQueue, mut i: usize) {
    while i > 0 {
        let parent = (i - 1) / 2;
        if pq.heap[i].priority < pq.heap[parent].priority {
            pq.heap.swap(i, parent);
            i = parent;
        } else {
            break;
        }
    }
}

fn sift_down(pq: &mut PriorityQueue, mut i: usize) {
    let len = pq.heap.len();
    loop {
        let left = 2 * i + 1;
        let right = 2 * i + 2;
        let mut smallest = i;
        if left < len && pq.heap[left].priority < pq.heap[smallest].priority {
            smallest = left;
        }
        if right < len && pq.heap[right].priority < pq.heap[smallest].priority {
            smallest = right;
        }
        if smallest == i {
            break;
        }
        pq.heap.swap(i, smallest);
        i = smallest;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let pq = new_priority_queue();
        assert!(pq_is_empty(&pq));
        assert_eq!(pq_len(&pq), 0);
    }

    #[test]
    fn test_push_pop_min_order() {
        let mut pq = new_priority_queue();
        pq_push(&mut pq, 1, 3.0);
        pq_push(&mut pq, 2, 1.0);
        pq_push(&mut pq, 3, 2.0);
        let first = pq_pop(&mut pq).expect("should succeed");
        assert_eq!(first.key, 2);
        assert!((first.priority - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut pq = new_priority_queue();
        pq_push(&mut pq, 5, 0.5);
        let _ = pq_peek(&pq);
        assert_eq!(pq_len(&pq), 1);
    }

    #[test]
    fn test_pop_empty_returns_none() {
        let mut pq = new_priority_queue();
        assert!(pq_pop(&mut pq).is_none());
    }

    #[test]
    fn test_clear() {
        let mut pq = new_priority_queue();
        pq_push(&mut pq, 1, 1.0);
        pq_push(&mut pq, 2, 2.0);
        pq_clear(&mut pq);
        assert!(pq_is_empty(&pq));
    }

    #[test]
    fn test_contains_key() {
        let mut pq = new_priority_queue();
        pq_push(&mut pq, 42, 5.0);
        assert!(pq_contains_key(&pq, 42));
        assert!(!pq_contains_key(&pq, 99));
    }

    #[test]
    fn test_multiple_pops_in_order() {
        let mut pq = new_priority_queue();
        pq_push(&mut pq, 10, 10.0);
        pq_push(&mut pq, 20, 5.0);
        pq_push(&mut pq, 30, 7.0);
        pq_push(&mut pq, 40, 1.0);
        let mut priorities = Vec::new();
        while let Some(item) = pq_pop(&mut pq) {
            priorities.push(item.priority);
        }
        for i in 1..priorities.len() {
            assert!(priorities[i - 1] <= priorities[i]);
        }
    }

    #[test]
    fn test_len_decrements_on_pop() {
        let mut pq = new_priority_queue();
        pq_push(&mut pq, 1, 1.0);
        pq_push(&mut pq, 2, 2.0);
        assert_eq!(pq_len(&pq), 2);
        pq_pop(&mut pq);
        assert_eq!(pq_len(&pq), 1);
    }

    #[test]
    fn test_priority_item_clone() {
        let item = PriorityItem { key: 7, priority: std::f32::consts::PI };
        let item2 = item.clone();
        assert_eq!(item2.key, 7);
    }
}
