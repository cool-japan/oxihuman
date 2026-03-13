// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A bounded MPSC channel implemented with a ring buffer for fixed-capacity message passing.

use std::collections::VecDeque;

/// Error returned when the channel is full.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelError {
    Full,
    Empty,
    Closed,
}

/// A bounded channel with a fixed capacity ring buffer.
#[allow(dead_code)]
#[derive(Debug)]
pub struct BoundedChannel<T> {
    buffer: VecDeque<T>,
    capacity: usize,
    closed: bool,
    total_sent: u64,
    total_received: u64,
}

#[allow(dead_code)]
impl<T> BoundedChannel<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be > 0");
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            closed: false,
            total_sent: 0,
            total_received: 0,
        }
    }

    pub fn send(&mut self, value: T) -> Result<(), ChannelError> {
        if self.closed {
            return Err(ChannelError::Closed);
        }
        if self.buffer.len() >= self.capacity {
            return Err(ChannelError::Full);
        }
        self.buffer.push_back(value);
        self.total_sent += 1;
        Ok(())
    }

    pub fn receive(&mut self) -> Result<T, ChannelError> {
        if let Some(val) = self.buffer.pop_front() {
            self.total_received += 1;
            Ok(val)
        } else if self.closed {
            Err(ChannelError::Closed)
        } else {
            Err(ChannelError::Empty)
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.buffer.front()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn close(&mut self) {
        self.closed = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn total_sent(&self) -> u64 {
        self.total_sent
    }

    pub fn total_received(&self) -> u64 {
        self.total_received
    }

    pub fn drain_all(&mut self) -> Vec<T> {
        let mut out = Vec::with_capacity(self.buffer.len());
        while let Some(v) = self.buffer.pop_front() {
            self.total_received += 1;
            out.push(v);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_receive() {
        let mut ch = BoundedChannel::new(4);
        ch.send(1).expect("should succeed");
        ch.send(2).expect("should succeed");
        assert_eq!(ch.receive().expect("should succeed"), 1);
        assert_eq!(ch.receive().expect("should succeed"), 2);
    }

    #[test]
    fn test_full() {
        let mut ch = BoundedChannel::new(2);
        ch.send(1).expect("should succeed");
        ch.send(2).expect("should succeed");
        assert_eq!(ch.send(3), Err(ChannelError::Full));
        assert!(ch.is_full());
    }

    #[test]
    fn test_empty() {
        let mut ch: BoundedChannel<i32> = BoundedChannel::new(2);
        assert_eq!(ch.receive(), Err(ChannelError::Empty));
    }

    #[test]
    fn test_close() {
        let mut ch = BoundedChannel::new(4);
        ch.send(10).expect("should succeed");
        ch.close();
        assert_eq!(ch.send(20), Err(ChannelError::Closed));
        assert_eq!(ch.receive().expect("should succeed"), 10);
        assert_eq!(ch.receive(), Err(ChannelError::Closed));
    }

    #[test]
    fn test_peek() {
        let mut ch = BoundedChannel::new(4);
        assert_eq!(ch.peek(), None);
        ch.send(42).expect("should succeed");
        assert_eq!(ch.peek(), Some(&42));
    }

    #[test]
    fn test_stats() {
        let mut ch = BoundedChannel::new(4);
        ch.send(1).expect("should succeed");
        ch.send(2).expect("should succeed");
        ch.receive().expect("should succeed");
        assert_eq!(ch.total_sent(), 2);
        assert_eq!(ch.total_received(), 1);
    }

    #[test]
    fn test_drain() {
        let mut ch = BoundedChannel::new(4);
        ch.send(1).expect("should succeed");
        ch.send(2).expect("should succeed");
        ch.send(3).expect("should succeed");
        let all = ch.drain_all();
        assert_eq!(all, vec![1, 2, 3]);
        assert!(ch.is_empty());
    }

    #[test]
    fn test_capacity() {
        let ch: BoundedChannel<i32> = BoundedChannel::new(8);
        assert_eq!(ch.capacity(), 8);
        assert_eq!(ch.len(), 0);
    }

    #[test]
    fn test_fifo_order() {
        let mut ch = BoundedChannel::new(10);
        for i in 0..5 {
            ch.send(i).expect("should succeed");
        }
        for i in 0..5 {
            assert_eq!(ch.receive().expect("should succeed"), i);
        }
    }

    #[test]
    fn test_wrap_around() {
        let mut ch = BoundedChannel::new(3);
        ch.send(1).expect("should succeed");
        ch.send(2).expect("should succeed");
        ch.receive().expect("should succeed");
        ch.send(3).expect("should succeed");
        ch.send(4).expect("should succeed");
        assert_eq!(ch.receive().expect("should succeed"), 2);
        assert_eq!(ch.receive().expect("should succeed"), 3);
        assert_eq!(ch.receive().expect("should succeed"), 4);
    }
}
