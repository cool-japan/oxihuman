// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pool of reusable byte buffers to reduce allocation pressure.

/// A pool of byte buffers that can be checked out and returned.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BufferPool {
    pool: Vec<Vec<u8>>,
    buffer_size: usize,
    max_pool_size: usize,
    checkout_count: u64,
}

#[allow(dead_code)]
impl BufferPool {
    pub fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        Self {
            pool: Vec::new(),
            buffer_size,
            max_pool_size,
            checkout_count: 0,
        }
    }

    pub fn checkout(&mut self) -> Vec<u8> {
        self.checkout_count += 1;
        self.pool
            .pop()
            .unwrap_or_else(|| vec![0u8; self.buffer_size])
    }

    pub fn return_buffer(&mut self, mut buf: Vec<u8>) {
        if self.pool.len() < self.max_pool_size {
            buf.iter_mut().for_each(|b| *b = 0);
            self.pool.push(buf);
        }
    }

    pub fn available(&self) -> usize {
        self.pool.len()
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn max_pool_size(&self) -> usize {
        self.max_pool_size
    }

    pub fn total_checkouts(&self) -> u64 {
        self.checkout_count
    }

    pub fn preallocate(&mut self, count: usize) {
        let to_add = count.min(self.max_pool_size.saturating_sub(self.pool.len()));
        for _ in 0..to_add {
            self.pool.push(vec![0u8; self.buffer_size]);
        }
    }

    pub fn shrink_to(&mut self, target: usize) {
        while self.pool.len() > target {
            self.pool.pop();
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn total_memory(&self) -> usize {
        self.pool.len() * self.buffer_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_is_empty() {
        let p = BufferPool::new(1024, 8);
        assert_eq!(p.available(), 0);
    }

    #[test]
    fn checkout_creates_buffer() {
        let mut p = BufferPool::new(64, 4);
        let buf = p.checkout();
        assert_eq!(buf.len(), 64);
    }

    #[test]
    fn return_and_reuse() {
        let mut p = BufferPool::new(32, 4);
        let buf = p.checkout();
        p.return_buffer(buf);
        assert_eq!(p.available(), 1);
        let buf2 = p.checkout();
        assert_eq!(buf2.len(), 32);
        assert_eq!(p.available(), 0);
    }

    #[test]
    fn return_beyond_max_drops() {
        let mut p = BufferPool::new(16, 1);
        let b1 = p.checkout();
        let b2 = p.checkout();
        p.return_buffer(b1);
        p.return_buffer(b2);
        assert_eq!(p.available(), 1);
    }

    #[test]
    fn preallocate_fills_pool() {
        let mut p = BufferPool::new(8, 5);
        p.preallocate(3);
        assert_eq!(p.available(), 3);
    }

    #[test]
    fn shrink_to_reduces() {
        let mut p = BufferPool::new(8, 10);
        p.preallocate(5);
        p.shrink_to(2);
        assert_eq!(p.available(), 2);
    }

    #[test]
    fn clear_empties_pool() {
        let mut p = BufferPool::new(8, 4);
        p.preallocate(4);
        p.clear();
        assert_eq!(p.available(), 0);
    }

    #[test]
    fn total_memory_calculated() {
        let mut p = BufferPool::new(100, 10);
        p.preallocate(3);
        assert_eq!(p.total_memory(), 300);
    }

    #[test]
    fn checkout_count_increments() {
        let mut p = BufferPool::new(8, 4);
        let _ = p.checkout();
        let _ = p.checkout();
        assert_eq!(p.total_checkouts(), 2);
    }

    #[test]
    fn buffer_size_returns_configured() {
        let p = BufferPool::new(256, 4);
        assert_eq!(p.buffer_size(), 256);
    }
}
