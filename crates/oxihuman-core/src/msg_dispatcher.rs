// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::VecDeque;

/// Message type with a topic and payload.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub topic: String,
    pub payload: String,
    pub priority: u32,
}

/// Dispatches messages to topic-based queues.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MsgDispatcher {
    queue: VecDeque<Message>,
    max_size: usize,
    dropped: u64,
}

#[allow(dead_code)]
impl MsgDispatcher {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: VecDeque::new(),
            max_size,
            dropped: 0,
        }
    }

    pub fn send(&mut self, topic: &str, payload: &str, priority: u32) -> bool {
        if self.queue.len() >= self.max_size {
            self.dropped += 1;
            return false;
        }
        self.queue.push_back(Message {
            topic: topic.to_string(),
            payload: payload.to_string(),
            priority,
        });
        true
    }

    pub fn receive(&mut self) -> Option<Message> {
        self.queue.pop_front()
    }

    pub fn receive_by_topic(&mut self, topic: &str) -> Option<Message> {
        let pos = self.queue.iter().position(|m| m.topic == topic)?;
        self.queue.remove(pos)
    }

    pub fn peek(&self) -> Option<&Message> {
        self.queue.front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.max_size
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn count_by_topic(&self, topic: &str) -> usize {
        self.queue.iter().filter(|m| m.topic == topic).count()
    }

    pub fn drain_topic(&mut self, topic: &str) -> Vec<Message> {
        let mut result = Vec::new();
        let mut remaining = VecDeque::new();
        while let Some(msg) = self.queue.pop_front() {
            if msg.topic == topic {
                result.push(msg);
            } else {
                remaining.push_back(msg);
            }
        }
        self.queue = remaining;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let d = MsgDispatcher::new(10);
        assert!(d.is_empty());
    }

    #[test]
    fn test_send_receive() {
        let mut d = MsgDispatcher::new(10);
        d.send("topic1", "hello", 0);
        let msg = d.receive().unwrap();
        assert_eq!(msg.topic, "topic1");
        assert_eq!(msg.payload, "hello");
    }

    #[test]
    fn test_fifo_order() {
        let mut d = MsgDispatcher::new(10);
        d.send("t", "first", 0);
        d.send("t", "second", 0);
        assert_eq!(d.receive().unwrap().payload, "first");
        assert_eq!(d.receive().unwrap().payload, "second");
    }

    #[test]
    fn test_overflow() {
        let mut d = MsgDispatcher::new(2);
        assert!(d.send("t", "a", 0));
        assert!(d.send("t", "b", 0));
        assert!(!d.send("t", "c", 0));
        assert_eq!(d.dropped_count(), 1);
    }

    #[test]
    fn test_receive_by_topic() {
        let mut d = MsgDispatcher::new(10);
        d.send("a", "msg_a", 0);
        d.send("b", "msg_b", 0);
        let msg = d.receive_by_topic("b").unwrap();
        assert_eq!(msg.payload, "msg_b");
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn test_peek() {
        let mut d = MsgDispatcher::new(10);
        d.send("t", "peek_me", 0);
        assert_eq!(d.peek().unwrap().payload, "peek_me");
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut d = MsgDispatcher::new(10);
        d.send("t", "data", 0);
        d.clear();
        assert!(d.is_empty());
    }

    #[test]
    fn test_count_by_topic() {
        let mut d = MsgDispatcher::new(10);
        d.send("x", "1", 0);
        d.send("y", "2", 0);
        d.send("x", "3", 0);
        assert_eq!(d.count_by_topic("x"), 2);
    }

    #[test]
    fn test_drain_topic() {
        let mut d = MsgDispatcher::new(10);
        d.send("a", "1", 0);
        d.send("b", "2", 0);
        d.send("a", "3", 0);
        let drained = d.drain_topic("a");
        assert_eq!(drained.len(), 2);
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn test_is_full() {
        let mut d = MsgDispatcher::new(1);
        d.send("t", "x", 0);
        assert!(d.is_full());
    }
}
