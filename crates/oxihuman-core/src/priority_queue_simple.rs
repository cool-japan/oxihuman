#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Min-priority queue (simple sorted vec).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PqItem<T: Clone + Ord> {
    pub priority: i64,
    pub data: T,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PriorityQueueSimple {
    pub items: Vec<PqItem<String>>,
}

#[allow(dead_code)]
pub fn new_priority_queue_simple() -> PriorityQueueSimple {
    PriorityQueueSimple { items: Vec::new() }
}

#[allow(dead_code)]
pub fn pqs_push(pq: &mut PriorityQueueSimple, data: &str, priority: i64) {
    let item = PqItem {
        priority,
        data: data.to_string(),
    };
    let pos = pq.items.partition_point(|x| x.priority <= priority);
    pq.items.insert(pos, item);
}

#[allow(dead_code)]
pub fn pqs_pop(pq: &mut PriorityQueueSimple) -> Option<String> {
    if pq.items.is_empty() {
        return None;
    }
    Some(pq.items.remove(0).data)
}

#[allow(dead_code)]
pub fn pqs_peek(pq: &PriorityQueueSimple) -> Option<&str> {
    pq.items.first().map(|x| x.data.as_str())
}

#[allow(dead_code)]
pub fn pqs_len(pq: &PriorityQueueSimple) -> usize {
    pq.items.len()
}

#[allow(dead_code)]
pub fn pqs_is_empty(pq: &PriorityQueueSimple) -> bool {
    pq.items.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let pq = new_priority_queue_simple();
        assert!(pqs_is_empty(&pq));
        assert_eq!(pqs_len(&pq), 0);
    }

    #[test]
    fn push_and_peek() {
        let mut pq = new_priority_queue_simple();
        pqs_push(&mut pq, "hello", 5);
        assert_eq!(pqs_peek(&pq), Some("hello"));
    }

    #[test]
    fn min_priority_first() {
        let mut pq = new_priority_queue_simple();
        pqs_push(&mut pq, "low", 10);
        pqs_push(&mut pq, "high", 1);
        assert_eq!(pqs_peek(&pq), Some("high"));
    }

    #[test]
    fn pop_returns_min() {
        let mut pq = new_priority_queue_simple();
        pqs_push(&mut pq, "c", 30);
        pqs_push(&mut pq, "a", 10);
        pqs_push(&mut pq, "b", 20);
        assert_eq!(pqs_pop(&mut pq), Some("a".to_string()));
        assert_eq!(pqs_pop(&mut pq), Some("b".to_string()));
        assert_eq!(pqs_pop(&mut pq), Some("c".to_string()));
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut pq = new_priority_queue_simple();
        assert!(pqs_pop(&mut pq).is_none());
    }

    #[test]
    fn len_tracks_count() {
        let mut pq = new_priority_queue_simple();
        pqs_push(&mut pq, "x", 1);
        pqs_push(&mut pq, "y", 2);
        assert_eq!(pqs_len(&pq), 2);
        pqs_pop(&mut pq);
        assert_eq!(pqs_len(&pq), 1);
    }

    #[test]
    fn same_priority_stable() {
        let mut pq = new_priority_queue_simple();
        pqs_push(&mut pq, "first", 5);
        pqs_push(&mut pq, "second", 5);
        let a = pqs_pop(&mut pq).expect("should succeed");
        let b = pqs_pop(&mut pq).expect("should succeed");
        // Both should be returned
        assert!(a == "first" || a == "second");
        assert!(b == "first" || b == "second");
        assert_ne!(a, b);
    }

    #[test]
    fn negative_priority() {
        let mut pq = new_priority_queue_simple();
        pqs_push(&mut pq, "neg", -5);
        pqs_push(&mut pq, "pos", 5);
        assert_eq!(pqs_peek(&pq), Some("neg"));
    }

    #[test]
    fn peek_empty_returns_none() {
        let pq = new_priority_queue_simple();
        assert!(pqs_peek(&pq).is_none());
    }

    #[test]
    fn push_many() {
        let mut pq = new_priority_queue_simple();
        for i in (0..10i64).rev() {
            pqs_push(&mut pq, &i.to_string(), i);
        }
        assert_eq!(pqs_len(&pq), 10);
        let first = pqs_pop(&mut pq).expect("should succeed");
        assert_eq!(first, "0");
    }
}
