#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact queue for deferred contact processing.

use std::collections::VecDeque;

/// A queued contact between two bodies.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QueuedContact {
    pub body_a: u32,
    pub body_b: u32,
    pub normal: [f32; 3],
    pub depth: f32,
}

/// A FIFO queue of contacts.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ContactQueue {
    queue: VecDeque<QueuedContact>,
}

/// Create a new empty `ContactQueue`.
#[allow(dead_code)]
pub fn new_contact_queue() -> ContactQueue {
    ContactQueue::default()
}

/// Add a contact to the end of the queue.
#[allow(dead_code)]
pub fn enqueue_contact(q: &mut ContactQueue, contact: QueuedContact) {
    q.queue.push_back(contact);
}

/// Remove and return the first contact in the queue.
#[allow(dead_code)]
pub fn dequeue_contact(q: &mut ContactQueue) -> Option<QueuedContact> {
    q.queue.pop_front()
}

/// Return the number of contacts in the queue.
#[allow(dead_code)]
pub fn queue_len(q: &ContactQueue) -> usize {
    q.queue.len()
}

/// Return true if the queue is empty.
#[allow(dead_code)]
pub fn queue_is_empty(q: &ContactQueue) -> bool {
    q.queue.is_empty()
}

/// Clear all contacts from the queue.
#[allow(dead_code)]
pub fn clear_contact_queue(q: &mut ContactQueue) {
    q.queue.clear();
}

/// Peek at the first contact without removing it.
#[allow(dead_code)]
pub fn peek_contact(q: &ContactQueue) -> Option<&QueuedContact> {
    q.queue.front()
}

/// Drain all contacts from the queue and return them.
#[allow(dead_code)]
pub fn contact_queue_flush(q: &mut ContactQueue) -> Vec<QueuedContact> {
    q.queue.drain(..).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contact(a: u32, b: u32) -> QueuedContact {
        QueuedContact { body_a: a, body_b: b, normal: [0.0, 1.0, 0.0], depth: 0.01 }
    }

    #[test]
    fn test_new_contact_queue() {
        let q = new_contact_queue();
        assert!(queue_is_empty(&q));
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut q = new_contact_queue();
        enqueue_contact(&mut q, make_contact(0, 1));
        let c = dequeue_contact(&mut q).unwrap();
        assert_eq!(c.body_a, 0);
        assert_eq!(c.body_b, 1);
    }

    #[test]
    fn test_queue_len() {
        let mut q = new_contact_queue();
        enqueue_contact(&mut q, make_contact(0, 1));
        enqueue_contact(&mut q, make_contact(1, 2));
        assert_eq!(queue_len(&q), 2);
    }

    #[test]
    fn test_queue_is_empty() {
        let mut q = new_contact_queue();
        assert!(queue_is_empty(&q));
        enqueue_contact(&mut q, make_contact(0, 1));
        assert!(!queue_is_empty(&q));
    }

    #[test]
    fn test_clear_contact_queue() {
        let mut q = new_contact_queue();
        enqueue_contact(&mut q, make_contact(0, 1));
        clear_contact_queue(&mut q);
        assert!(queue_is_empty(&q));
    }

    #[test]
    fn test_peek_contact() {
        let mut q = new_contact_queue();
        enqueue_contact(&mut q, make_contact(5, 6));
        let p = peek_contact(&q).unwrap();
        assert_eq!(p.body_a, 5);
        assert_eq!(queue_len(&q), 1); // not removed
    }

    #[test]
    fn test_fifo_order() {
        let mut q = new_contact_queue();
        enqueue_contact(&mut q, make_contact(1, 2));
        enqueue_contact(&mut q, make_contact(3, 4));
        assert_eq!(dequeue_contact(&mut q).unwrap().body_a, 1);
        assert_eq!(dequeue_contact(&mut q).unwrap().body_a, 3);
    }

    #[test]
    fn test_contact_queue_flush() {
        let mut q = new_contact_queue();
        enqueue_contact(&mut q, make_contact(0, 1));
        enqueue_contact(&mut q, make_contact(2, 3));
        let all = contact_queue_flush(&mut q);
        assert_eq!(all.len(), 2);
        assert!(queue_is_empty(&q));
    }
}
