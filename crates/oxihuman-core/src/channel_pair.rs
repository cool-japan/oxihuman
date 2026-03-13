// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::collections::VecDeque;

/// A bidirectional channel pair for message passing between two endpoints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChannelPair<T> {
    a_to_b: VecDeque<T>,
    b_to_a: VecDeque<T>,
    capacity: usize,
    total_sent: u64,
}

#[allow(dead_code)]
impl<T> ChannelPair<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            a_to_b: VecDeque::new(),
            b_to_a: VecDeque::new(),
            capacity,
            total_sent: 0,
        }
    }

    pub fn send_a_to_b(&mut self, msg: T) -> bool {
        if self.a_to_b.len() >= self.capacity {
            return false;
        }
        self.a_to_b.push_back(msg);
        self.total_sent += 1;
        true
    }

    pub fn send_b_to_a(&mut self, msg: T) -> bool {
        if self.b_to_a.len() >= self.capacity {
            return false;
        }
        self.b_to_a.push_back(msg);
        self.total_sent += 1;
        true
    }

    pub fn recv_at_b(&mut self) -> Option<T> {
        self.a_to_b.pop_front()
    }

    pub fn recv_at_a(&mut self) -> Option<T> {
        self.b_to_a.pop_front()
    }

    pub fn pending_at_b(&self) -> usize {
        self.a_to_b.len()
    }

    pub fn pending_at_a(&self) -> usize {
        self.b_to_a.len()
    }

    pub fn is_empty(&self) -> bool {
        self.a_to_b.is_empty() && self.b_to_a.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn total_sent(&self) -> u64 {
        self.total_sent
    }

    pub fn clear(&mut self) {
        self.a_to_b.clear();
        self.b_to_a.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ch: ChannelPair<i32> = ChannelPair::new(16);
        assert!(ch.is_empty());
        assert_eq!(ch.capacity(), 16);
    }

    #[test]
    fn test_send_recv_a_to_b() {
        let mut ch = ChannelPair::new(8);
        assert!(ch.send_a_to_b(42));
        assert_eq!(ch.recv_at_b(), Some(42));
    }

    #[test]
    fn test_send_recv_b_to_a() {
        let mut ch = ChannelPair::new(8);
        assert!(ch.send_b_to_a(99));
        assert_eq!(ch.recv_at_a(), Some(99));
    }

    #[test]
    fn test_capacity_limit() {
        let mut ch = ChannelPair::new(2);
        assert!(ch.send_a_to_b(1));
        assert!(ch.send_a_to_b(2));
        assert!(!ch.send_a_to_b(3));
    }

    #[test]
    fn test_pending() {
        let mut ch = ChannelPair::new(8);
        ch.send_a_to_b(1);
        ch.send_a_to_b(2);
        assert_eq!(ch.pending_at_b(), 2);
        assert_eq!(ch.pending_at_a(), 0);
    }

    #[test]
    fn test_total_sent() {
        let mut ch = ChannelPair::new(8);
        ch.send_a_to_b(1);
        ch.send_b_to_a(2);
        assert_eq!(ch.total_sent(), 2);
    }

    #[test]
    fn test_clear() {
        let mut ch = ChannelPair::new(8);
        ch.send_a_to_b(1);
        ch.send_b_to_a(2);
        ch.clear();
        assert!(ch.is_empty());
    }

    #[test]
    fn test_recv_empty() {
        let mut ch: ChannelPair<i32> = ChannelPair::new(8);
        assert_eq!(ch.recv_at_a(), None);
        assert_eq!(ch.recv_at_b(), None);
    }

    #[test]
    fn test_fifo_order() {
        let mut ch = ChannelPair::new(8);
        ch.send_a_to_b(1);
        ch.send_a_to_b(2);
        ch.send_a_to_b(3);
        assert_eq!(ch.recv_at_b(), Some(1));
        assert_eq!(ch.recv_at_b(), Some(2));
        assert_eq!(ch.recv_at_b(), Some(3));
    }

    #[test]
    fn test_bidirectional() {
        let mut ch = ChannelPair::new(8);
        ch.send_a_to_b(10);
        ch.send_b_to_a(20);
        assert_eq!(ch.recv_at_b(), Some(10));
        assert_eq!(ch.recv_at_a(), Some(20));
    }
}
