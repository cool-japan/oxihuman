#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Priority queue for task scheduling.

/// Task priority level.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskPriority(pub u32);

/// A priority queue backed by a sorted vec.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PriorityQueue<T> {
    items: Vec<(TaskPriority, T)>,
}

#[allow(dead_code)]
pub fn new_priority_queue<T>() -> PriorityQueue<T> {
    PriorityQueue { items: Vec::new() }
}

#[allow(dead_code)]
pub fn enqueue_priority<T>(q: &mut PriorityQueue<T>, priority: TaskPriority, item: T) {
    q.items.push((priority, item));
    q.items.sort_by(|a, b| b.0.cmp(&a.0));
}

#[allow(dead_code)]
pub fn dequeue_highest<T>(q: &mut PriorityQueue<T>) -> Option<T> {
    if q.items.is_empty() {
        None
    } else {
        Some(q.items.remove(0).1)
    }
}

#[allow(dead_code)]
pub fn peek_highest<T>(q: &PriorityQueue<T>) -> Option<&T> {
    q.items.first().map(|(_, v)| v)
}

#[allow(dead_code)]
pub fn priority_len<T>(q: &PriorityQueue<T>) -> usize {
    q.items.len()
}

#[allow(dead_code)]
pub fn priority_is_empty<T>(q: &PriorityQueue<T>) -> bool {
    q.items.is_empty()
}

#[allow(dead_code)]
pub fn clear_priority<T>(q: &mut PriorityQueue<T>) {
    q.items.clear();
}

#[allow(dead_code)]
pub fn change_priority<T>(q: &mut PriorityQueue<T>, index: usize, new_priority: TaskPriority) {
    if index < q.items.len() {
        q.items[index].0 = new_priority;
        q.items.sort_by(|a, b| b.0.cmp(&a.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let q: PriorityQueue<i32> = new_priority_queue();
        assert!(priority_is_empty(&q));
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut q = new_priority_queue();
        enqueue_priority(&mut q, TaskPriority(1), "low");
        enqueue_priority(&mut q, TaskPriority(10), "high");
        assert_eq!(dequeue_highest(&mut q), Some("high"));
        assert_eq!(dequeue_highest(&mut q), Some("low"));
    }

    #[test]
    fn test_peek() {
        let mut q = new_priority_queue();
        enqueue_priority(&mut q, TaskPriority(5), 42);
        assert_eq!(peek_highest(&q), Some(&42));
        assert_eq!(priority_len(&q), 1);
    }

    #[test]
    fn test_len() {
        let mut q = new_priority_queue();
        enqueue_priority(&mut q, TaskPriority(1), 'a');
        enqueue_priority(&mut q, TaskPriority(2), 'b');
        assert_eq!(priority_len(&q), 2);
    }

    #[test]
    fn test_clear() {
        let mut q = new_priority_queue();
        enqueue_priority(&mut q, TaskPriority(1), 1);
        clear_priority(&mut q);
        assert!(priority_is_empty(&q));
    }

    #[test]
    fn test_dequeue_empty() {
        let mut q: PriorityQueue<i32> = new_priority_queue();
        assert_eq!(dequeue_highest(&mut q), None);
    }

    #[test]
    fn test_change_priority() {
        let mut q = new_priority_queue();
        enqueue_priority(&mut q, TaskPriority(1), "a");
        enqueue_priority(&mut q, TaskPriority(10), "b");
        // "a" is at index 1 (sorted: b=10, a=1)
        change_priority(&mut q, 1, TaskPriority(100));
        assert_eq!(dequeue_highest(&mut q), Some("a"));
    }

    #[test]
    fn test_ordering() {
        let mut q = new_priority_queue();
        enqueue_priority(&mut q, TaskPriority(3), 3);
        enqueue_priority(&mut q, TaskPriority(1), 1);
        enqueue_priority(&mut q, TaskPriority(2), 2);
        assert_eq!(dequeue_highest(&mut q), Some(3));
        assert_eq!(dequeue_highest(&mut q), Some(2));
        assert_eq!(dequeue_highest(&mut q), Some(1));
    }

    #[test]
    fn test_peek_empty() {
        let q: PriorityQueue<i32> = new_priority_queue();
        assert_eq!(peek_highest(&q), None);
    }

    #[test]
    fn test_task_priority_ord() {
        assert!(TaskPriority(10) > TaskPriority(1));
    }
}
