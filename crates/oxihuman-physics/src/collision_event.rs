#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Collision event queue.

/// A collision event between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub body_a: u32,
    pub body_b: u32,
    pub normal: [f32; 3],
    pub depth: f32,
}

/// Queue of collision events.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollisionEventQueue {
    events: Vec<CollisionEvent>,
}

#[allow(dead_code)]
pub fn new_collision_event_queue() -> CollisionEventQueue {
    CollisionEventQueue { events: Vec::new() }
}

#[allow(dead_code)]
pub fn push_collision_event(
    q: &mut CollisionEventQueue,
    body_a: u32, body_b: u32,
    normal: [f32; 3], depth: f32,
) {
    q.events.push(CollisionEvent { body_a, body_b, normal, depth });
}

#[allow(dead_code)]
pub fn pop_collision_event(q: &mut CollisionEventQueue) -> Option<CollisionEvent> {
    if q.events.is_empty() {
        None
    } else {
        Some(q.events.remove(0))
    }
}

#[allow(dead_code)]
pub fn event_count(q: &CollisionEventQueue) -> usize {
    q.events.len()
}

#[allow(dead_code)]
pub fn clear_events(q: &mut CollisionEventQueue) {
    q.events.clear();
}

#[allow(dead_code)]
pub fn peek_event(q: &CollisionEventQueue) -> Option<&CollisionEvent> {
    q.events.first()
}

#[allow(dead_code)]
pub fn events_between(q: &CollisionEventQueue, a: u32, b: u32) -> Vec<&CollisionEvent> {
    q.events.iter().filter(|e| {
        (e.body_a == a && e.body_b == b) || (e.body_a == b && e.body_b == a)
    }).collect()
}

#[allow(dead_code)]
pub fn has_events(q: &CollisionEventQueue) -> bool {
    !q.events.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let q = new_collision_event_queue();
        assert!(!has_events(&q));
    }

    #[test]
    fn test_push_pop() {
        let mut q = new_collision_event_queue();
        push_collision_event(&mut q, 1, 2, [0.0, 1.0, 0.0], 0.1);
        let e = pop_collision_event(&mut q).expect("should succeed");
        assert_eq!(e.body_a, 1);
        assert_eq!(e.body_b, 2);
    }

    #[test]
    fn test_count() {
        let mut q = new_collision_event_queue();
        push_collision_event(&mut q, 1, 2, [0.0, 1.0, 0.0], 0.1);
        push_collision_event(&mut q, 3, 4, [0.0, 0.0, 1.0], 0.2);
        assert_eq!(event_count(&q), 2);
    }

    #[test]
    fn test_clear() {
        let mut q = new_collision_event_queue();
        push_collision_event(&mut q, 1, 2, [0.0, 1.0, 0.0], 0.1);
        clear_events(&mut q);
        assert!(!has_events(&q));
    }

    #[test]
    fn test_peek() {
        let mut q = new_collision_event_queue();
        push_collision_event(&mut q, 5, 6, [1.0, 0.0, 0.0], 0.5);
        let e = peek_event(&q).expect("should succeed");
        assert_eq!(e.body_a, 5);
        assert_eq!(event_count(&q), 1);
    }

    #[test]
    fn test_events_between() {
        let mut q = new_collision_event_queue();
        push_collision_event(&mut q, 1, 2, [0.0, 1.0, 0.0], 0.1);
        push_collision_event(&mut q, 3, 4, [0.0, 1.0, 0.0], 0.2);
        push_collision_event(&mut q, 2, 1, [0.0, 1.0, 0.0], 0.3);
        let evts = events_between(&q, 1, 2);
        assert_eq!(evts.len(), 2);
    }

    #[test]
    fn test_pop_empty() {
        let mut q = new_collision_event_queue();
        assert!(pop_collision_event(&mut q).is_none());
    }

    #[test]
    fn test_has_events() {
        let mut q = new_collision_event_queue();
        assert!(!has_events(&q));
        push_collision_event(&mut q, 1, 2, [0.0, 1.0, 0.0], 0.1);
        assert!(has_events(&q));
    }

    #[test]
    fn test_events_between_none() {
        let q = new_collision_event_queue();
        let evts = events_between(&q, 1, 2);
        assert!(evts.is_empty());
    }

    #[test]
    fn test_depth() {
        let mut q = new_collision_event_queue();
        push_collision_event(&mut q, 1, 2, [0.0, 1.0, 0.0], 0.75);
        let e = pop_collision_event(&mut q).expect("should succeed");
        assert!((e.depth - 0.75).abs() < 1e-6);
    }
}
