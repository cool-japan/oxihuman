#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

/// A queue of pending items awaiting processing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PendingQueue {
    items: VecDeque<String>,
}

#[allow(dead_code)]
pub fn new_pending_queue() -> PendingQueue {
    PendingQueue {
        items: VecDeque::new(),
    }
}

#[allow(dead_code)]
pub fn enqueue_pending(q: &mut PendingQueue, item: &str) {
    q.items.push_back(item.to_string());
}

#[allow(dead_code)]
pub fn process_pending(q: &mut PendingQueue) -> Option<String> {
    q.items.pop_front()
}

#[allow(dead_code)]
pub fn pending_count(q: &PendingQueue) -> usize {
    q.items.len()
}

#[allow(dead_code)]
pub fn pending_is_empty(q: &PendingQueue) -> bool {
    q.items.is_empty()
}

#[allow(dead_code)]
pub fn peek_pending(q: &PendingQueue) -> Option<&str> {
    q.items.front().map(|s| s.as_str())
}

#[allow(dead_code)]
pub fn cancel_pending(q: &mut PendingQueue, item: &str) -> bool {
    if let Some(pos) = q.items.iter().position(|s| s == item) {
        q.items.remove(pos);
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn pending_to_json(q: &PendingQueue) -> String {
    let items: Vec<String> = q.items.iter().map(|s| format!("\"{}\"", s)).collect();
    format!("{{\"pending\":[{}]}}", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pending_queue() {
        let q = new_pending_queue();
        assert!(pending_is_empty(&q));
    }

    #[test]
    fn test_enqueue_pending() {
        let mut q = new_pending_queue();
        enqueue_pending(&mut q, "task1");
        assert_eq!(pending_count(&q), 1);
    }

    #[test]
    fn test_process_pending() {
        let mut q = new_pending_queue();
        enqueue_pending(&mut q, "task1");
        assert_eq!(process_pending(&mut q), Some("task1".to_string()));
    }

    #[test]
    fn test_process_empty() {
        let mut q = new_pending_queue();
        assert!(process_pending(&mut q).is_none());
    }

    #[test]
    fn test_pending_count() {
        let mut q = new_pending_queue();
        enqueue_pending(&mut q, "a");
        enqueue_pending(&mut q, "b");
        assert_eq!(pending_count(&q), 2);
    }

    #[test]
    fn test_pending_is_empty() {
        let q = new_pending_queue();
        assert!(pending_is_empty(&q));
    }

    #[test]
    fn test_peek_pending() {
        let mut q = new_pending_queue();
        enqueue_pending(&mut q, "first");
        assert_eq!(peek_pending(&q), Some("first"));
    }

    #[test]
    fn test_cancel_pending() {
        let mut q = new_pending_queue();
        enqueue_pending(&mut q, "task1");
        assert!(cancel_pending(&mut q, "task1"));
        assert!(pending_is_empty(&q));
    }

    #[test]
    fn test_cancel_not_found() {
        let mut q = new_pending_queue();
        assert!(!cancel_pending(&mut q, "missing"));
    }

    #[test]
    fn test_pending_to_json() {
        let mut q = new_pending_queue();
        enqueue_pending(&mut q, "a");
        let json = pending_to_json(&q);
        assert!(json.contains("\"a\""));
    }
}
