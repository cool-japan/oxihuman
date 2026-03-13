// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A pool of reusable byte buffers to reduce allocation overhead.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BytePool {
    free: Vec<Vec<u8>>,
    buffer_size: usize,
    total_allocated: usize,
}

#[allow(dead_code)]
impl BytePool {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            free: Vec::new(),
            buffer_size,
            total_allocated: 0,
        }
    }

    pub fn acquire(&mut self) -> Vec<u8> {
        if let Some(mut buf) = self.free.pop() {
            buf.clear();
            buf
        } else {
            self.total_allocated += 1;
            Vec::with_capacity(self.buffer_size)
        }
    }

    pub fn release(&mut self, buf: Vec<u8>) {
        self.free.push(buf);
    }

    pub fn free_count(&self) -> usize {
        self.free.len()
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    pub fn shrink(&mut self, max_free: usize) {
        while self.free.len() > max_free {
            self.free.pop();
        }
    }

    pub fn clear(&mut self) {
        self.free.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.free.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pool = BytePool::new(1024);
        assert_eq!(pool.buffer_size(), 1024);
        assert_eq!(pool.free_count(), 0);
    }

    #[test]
    fn test_acquire() {
        let mut pool = BytePool::new(256);
        let buf = pool.acquire();
        assert!(buf.is_empty());
        assert_eq!(pool.total_allocated(), 1);
    }

    #[test]
    fn test_release_and_reuse() {
        let mut pool = BytePool::new(256);
        let buf = pool.acquire();
        pool.release(buf);
        assert_eq!(pool.free_count(), 1);
        let _buf2 = pool.acquire();
        assert_eq!(pool.free_count(), 0);
        assert_eq!(pool.total_allocated(), 1);
    }

    #[test]
    fn test_shrink() {
        let mut pool = BytePool::new(64);
        let mut bufs = Vec::new();
        for _ in 0..5 {
            bufs.push(pool.acquire());
        }
        for buf in bufs {
            pool.release(buf);
        }
        pool.shrink(2);
        assert_eq!(pool.free_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut pool = BytePool::new(64);
        let buf = pool.acquire();
        pool.release(buf);
        pool.clear();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_multiple_acquire() {
        let mut pool = BytePool::new(64);
        let _a = pool.acquire();
        let _b = pool.acquire();
        assert_eq!(pool.total_allocated(), 2);
    }

    #[test]
    fn test_released_buffer_cleared() {
        let mut pool = BytePool::new(64);
        let mut buf = pool.acquire();
        buf.extend_from_slice(&[1, 2, 3]);
        pool.release(buf);
        let reused = pool.acquire();
        assert!(reused.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let pool = BytePool::new(64);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_shrink_to_zero() {
        let mut pool = BytePool::new(64);
        let buf = pool.acquire();
        pool.release(buf);
        pool.shrink(0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_acquire_capacity() {
        let mut pool = BytePool::new(512);
        let buf = pool.acquire();
        assert!(buf.capacity() >= 512);
    }
}
