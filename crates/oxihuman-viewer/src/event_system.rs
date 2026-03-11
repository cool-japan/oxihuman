// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]
//! Simple viewer event system.

#[allow(dead_code)]
pub enum ViewerEventKind {
    MouseClick,
    MouseMove,
    KeyPress,
    Resize,
    Scroll,
}

#[allow(dead_code)]
pub struct ViewerEvent {
    pub kind: ViewerEventKind,
    pub x: f32,
    pub y: f32,
    pub value: f32,
}

#[allow(dead_code)]
pub struct EventQueue {
    pub events: Vec<ViewerEvent>,
    pub max_events: usize,
}

#[allow(dead_code)]
pub fn new_event_queue(max_events: usize) -> EventQueue {
    EventQueue { events: Vec::new(), max_events }
}

#[allow(dead_code)]
pub fn eq_push(q: &mut EventQueue, event: ViewerEvent) {
    if q.events.len() >= q.max_events {
        q.events.remove(0);
    }
    q.events.push(event);
}

#[allow(dead_code)]
pub fn eq_pop(q: &mut EventQueue) -> Option<ViewerEvent> {
    if q.events.is_empty() {
        None
    } else {
        Some(q.events.remove(0))
    }
}

#[allow(dead_code)]
pub fn eq_count(q: &EventQueue) -> usize {
    q.events.len()
}

#[allow(dead_code)]
pub fn eq_clear(q: &mut EventQueue) {
    q.events.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut q = new_event_queue(10);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 0.0, y: 0.0, value: 0.0 });
        assert_eq!(eq_count(&q), 1);
    }

    #[test]
    fn test_pop() {
        let mut q = new_event_queue(10);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::KeyPress, x: 0.0, y: 0.0, value: 65.0 });
        let e = eq_pop(&mut q);
        assert!(e.is_some());
        assert_eq!(eq_count(&q), 0);
    }

    #[test]
    fn test_pop_empty() {
        let mut q = new_event_queue(10);
        assert!(eq_pop(&mut q).is_none());
    }

    #[test]
    fn test_count() {
        let mut q = new_event_queue(10);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::Scroll, x: 0.0, y: 0.0, value: 1.0 });
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseMove, x: 1.0, y: 2.0, value: 0.0 });
        assert_eq!(eq_count(&q), 2);
    }

    #[test]
    fn test_clear() {
        let mut q = new_event_queue(10);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::Resize, x: 0.0, y: 0.0, value: 0.0 });
        eq_clear(&mut q);
        assert_eq!(eq_count(&q), 0);
    }

    #[test]
    fn test_max_events_enforced() {
        let mut q = new_event_queue(2);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 0.0, y: 0.0, value: 0.0 });
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 1.0, y: 0.0, value: 0.0 });
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 2.0, y: 0.0, value: 0.0 });
        assert_eq!(eq_count(&q), 2);
    }

    #[test]
    fn test_max_events_drops_oldest() {
        let mut q = new_event_queue(2);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 1.0, y: 0.0, value: 0.0 });
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 2.0, y: 0.0, value: 0.0 });
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 3.0, y: 0.0, value: 0.0 });
        let e = eq_pop(&mut q).unwrap();
        assert!((e.x - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_fifo_order() {
        let mut q = new_event_queue(10);
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 10.0, y: 0.0, value: 0.0 });
        eq_push(&mut q, ViewerEvent { kind: ViewerEventKind::MouseClick, x: 20.0, y: 0.0, value: 0.0 });
        let e = eq_pop(&mut q).unwrap();
        assert!((e.x - 10.0).abs() < 1e-5);
    }
}
