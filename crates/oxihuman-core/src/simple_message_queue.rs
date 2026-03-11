// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::VecDeque;

pub struct SimpleMessageQueue {
    pub messages: VecDeque<String>,
    pub capacity: usize,
}

impl SimpleMessageQueue {
    pub fn new(capacity: usize) -> Self {
        SimpleMessageQueue {
            messages: VecDeque::new(),
            capacity,
        }
    }
}

pub fn new_message_queue_simple(capacity: usize) -> SimpleMessageQueue {
    SimpleMessageQueue::new(capacity)
}

pub fn smq_push(q: &mut SimpleMessageQueue, msg: &str) -> bool {
    if q.messages.len() >= q.capacity {
        return false;
    }
    q.messages.push_back(msg.to_string());
    true
}

pub fn smq_pop(q: &mut SimpleMessageQueue) -> Option<String> {
    q.messages.pop_front()
}

pub fn smq_peek(q: &SimpleMessageQueue) -> Option<&str> {
    q.messages.front().map(|s| s.as_str())
}

pub fn smq_len(q: &SimpleMessageQueue) -> usize {
    q.messages.len()
}

pub fn smq_is_empty(q: &SimpleMessageQueue) -> bool {
    q.messages.is_empty()
}

pub fn smq_is_full(q: &SimpleMessageQueue) -> bool {
    q.messages.len() >= q.capacity
}

pub fn smq_clear(q: &mut SimpleMessageQueue) {
    q.messages.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new queue is empty */
        let q = new_message_queue_simple(5);
        assert!(smq_is_empty(&q));
        assert_eq!(smq_len(&q), 0);
    }

    #[test]
    fn test_push_and_pop() {
        /* push then pop returns message in FIFO order */
        let mut q = new_message_queue_simple(5);
        smq_push(&mut q, "hello");
        smq_push(&mut q, "world");
        assert_eq!(smq_pop(&mut q).unwrap(), "hello");
        assert_eq!(smq_pop(&mut q).unwrap(), "world");
    }

    #[test]
    fn test_push_full() {
        /* push on full queue returns false */
        let mut q = new_message_queue_simple(2);
        smq_push(&mut q, "a");
        smq_push(&mut q, "b");
        assert!(!smq_push(&mut q, "c"));
    }

    #[test]
    fn test_peek() {
        /* peek returns front without removing */
        let mut q = new_message_queue_simple(5);
        smq_push(&mut q, "front");
        assert_eq!(smq_peek(&q), Some("front"));
        assert_eq!(smq_len(&q), 1);
    }

    #[test]
    fn test_is_full() {
        /* is_full at capacity */
        let mut q = new_message_queue_simple(1);
        smq_push(&mut q, "x");
        assert!(smq_is_full(&q));
    }

    #[test]
    fn test_clear() {
        /* clear empties the queue */
        let mut q = new_message_queue_simple(5);
        smq_push(&mut q, "m");
        smq_clear(&mut q);
        assert!(smq_is_empty(&q));
    }

    #[test]
    fn test_pop_empty() {
        /* pop on empty returns None */
        let mut q = new_message_queue_simple(5);
        assert!(smq_pop(&mut q).is_none());
    }

    #[test]
    fn test_len() {
        /* len tracks correctly */
        let mut q = new_message_queue_simple(10);
        smq_push(&mut q, "a");
        smq_push(&mut q, "b");
        smq_push(&mut q, "c");
        assert_eq!(smq_len(&q), 3);
    }
}
