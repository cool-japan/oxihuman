#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Priority-based event queue.

/// An event with an associated priority.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PriorityEvent {
    pub priority: u32,
    pub name: String,
    pub payload: String,
}

/// A queue of priority events, highest priority first.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EventPriorityQueue {
    events: Vec<PriorityEvent>,
}

#[allow(dead_code)]
pub fn new_event_priority_queue() -> EventPriorityQueue {
    EventPriorityQueue { events: Vec::new() }
}

#[allow(dead_code)]
pub fn push_event(q: &mut EventPriorityQueue, priority: u32, name: &str, payload: &str) {
    q.events.push(PriorityEvent {
        priority,
        name: name.to_string(),
        payload: payload.to_string(),
    });
    q.events.sort_by(|a, b| b.priority.cmp(&a.priority));
}

#[allow(dead_code)]
pub fn pop_event(q: &mut EventPriorityQueue) -> Option<PriorityEvent> {
    if q.events.is_empty() {
        None
    } else {
        Some(q.events.remove(0))
    }
}

#[allow(dead_code)]
pub fn event_queue_len(q: &EventPriorityQueue) -> usize {
    q.events.len()
}

#[allow(dead_code)]
pub fn peek_event(q: &EventPriorityQueue) -> Option<&PriorityEvent> {
    q.events.first()
}

#[allow(dead_code)]
pub fn drain_events(q: &mut EventPriorityQueue) -> Vec<PriorityEvent> {
    let drained = q.events.drain(..).collect();
    drained
}

#[allow(dead_code)]
pub fn event_queue_is_empty(q: &EventPriorityQueue) -> bool {
    q.events.is_empty()
}

#[allow(dead_code)]
pub fn event_priority_at(q: &EventPriorityQueue, index: usize) -> Option<u32> {
    q.events.get(index).map(|e| e.priority)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let q = new_event_priority_queue();
        assert!(event_queue_is_empty(&q));
    }

    #[test]
    fn test_push_pop() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 5, "evt", "data");
        let e = pop_event(&mut q).unwrap();
        assert_eq!(e.priority, 5);
        assert_eq!(e.name, "evt");
    }

    #[test]
    fn test_priority_order() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 1, "low", "");
        push_event(&mut q, 10, "high", "");
        push_event(&mut q, 5, "mid", "");
        assert_eq!(pop_event(&mut q).unwrap().name, "high");
        assert_eq!(pop_event(&mut q).unwrap().name, "mid");
        assert_eq!(pop_event(&mut q).unwrap().name, "low");
    }

    #[test]
    fn test_len() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 1, "a", "");
        push_event(&mut q, 2, "b", "");
        assert_eq!(event_queue_len(&q), 2);
    }

    #[test]
    fn test_peek() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 10, "top", "");
        let e = peek_event(&q).unwrap();
        assert_eq!(e.name, "top");
        assert_eq!(event_queue_len(&q), 1); // not consumed
    }

    #[test]
    fn test_drain() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 1, "a", "");
        push_event(&mut q, 2, "b", "");
        let all = drain_events(&mut q);
        assert_eq!(all.len(), 2);
        assert!(event_queue_is_empty(&q));
    }

    #[test]
    fn test_pop_empty() {
        let mut q = new_event_priority_queue();
        assert!(pop_event(&mut q).is_none());
    }

    #[test]
    fn test_event_priority_at() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 7, "x", "");
        assert_eq!(event_priority_at(&q, 0), Some(7));
        assert_eq!(event_priority_at(&q, 1), None);
    }

    #[test]
    fn test_payload() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 1, "e", "payload_data");
        let e = pop_event(&mut q).unwrap();
        assert_eq!(e.payload, "payload_data");
    }

    #[test]
    fn test_same_priority() {
        let mut q = new_event_priority_queue();
        push_event(&mut q, 5, "first", "");
        push_event(&mut q, 5, "second", "");
        assert_eq!(event_queue_len(&q), 2);
    }
}
