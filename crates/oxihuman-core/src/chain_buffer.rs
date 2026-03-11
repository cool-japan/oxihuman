// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! A chain of fixed-size buffers for sequential data storage.

/// One link in the chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ChainLink {
    data: Vec<u8>,
    used: usize,
}

/// A chain of fixed-size byte buffers.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainBuffer {
    links: Vec<ChainLink>,
    link_capacity: usize,
}

#[allow(dead_code)]
impl ChainBuffer {
    pub fn new(link_capacity: usize) -> Self {
        let cap = if link_capacity == 0 {
            64
        } else {
            link_capacity
        };
        Self {
            links: Vec::new(),
            link_capacity: cap,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        let mut offset = 0;
        while offset < data.len() {
            if self.links.is_empty()
                || self
                    .links
                    .last()
                    .is_none_or(|l| l.used >= self.link_capacity)
            {
                self.links.push(ChainLink {
                    data: vec![0u8; self.link_capacity],
                    used: 0,
                });
            }
            let Some(link) = self.links.last_mut() else {
                break;
            };
            let space = self.link_capacity - link.used;
            let to_copy = space.min(data.len() - offset);
            link.data[link.used..link.used + to_copy]
                .copy_from_slice(&data[offset..offset + to_copy]);
            link.used += to_copy;
            offset += to_copy;
        }
    }

    pub fn total_bytes(&self) -> usize {
        self.links.iter().map(|l| l.used).sum()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    pub fn link_capacity(&self) -> usize {
        self.link_capacity
    }

    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.total_bytes());
        for link in &self.links {
            out.extend_from_slice(&link.data[..link.used]);
        }
        out
    }

    pub fn clear(&mut self) {
        self.links.clear();
    }

    pub fn read_at(&self, offset: usize) -> Option<u8> {
        let mut remaining = offset;
        for link in &self.links {
            if remaining < link.used {
                return Some(link.data[remaining]);
            }
            remaining -= link.used;
        }
        None
    }

    pub fn wasted_bytes(&self) -> usize {
        self.links.iter().map(|l| self.link_capacity - l.used).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chain_is_empty() {
        let c = ChainBuffer::new(64);
        assert!(c.is_empty());
        assert_eq!(c.total_bytes(), 0);
    }

    #[test]
    fn write_small_data() {
        let mut c = ChainBuffer::new(16);
        c.write(b"hello");
        assert_eq!(c.total_bytes(), 5);
        assert_eq!(c.link_count(), 1);
    }

    #[test]
    fn write_spans_multiple_links() {
        let mut c = ChainBuffer::new(4);
        c.write(b"abcdefghij"); // 10 bytes, 4 per link -> 3 links
        assert_eq!(c.link_count(), 3);
        assert_eq!(c.total_bytes(), 10);
    }

    #[test]
    fn to_vec_reconstructs_data() {
        let mut c = ChainBuffer::new(3);
        c.write(b"hello world");
        assert_eq!(c.to_vec(), b"hello world");
    }

    #[test]
    fn read_at_returns_byte() {
        let mut c = ChainBuffer::new(4);
        c.write(b"abcdef");
        assert_eq!(c.read_at(0), Some(b'a'));
        assert_eq!(c.read_at(4), Some(b'e'));
        assert_eq!(c.read_at(10), None);
    }

    #[test]
    fn clear_empties() {
        let mut c = ChainBuffer::new(8);
        c.write(b"data");
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn link_capacity_returns_configured() {
        let c = ChainBuffer::new(128);
        assert_eq!(c.link_capacity(), 128);
    }

    #[test]
    fn zero_capacity_defaults() {
        let c = ChainBuffer::new(0);
        assert_eq!(c.link_capacity(), 64);
    }

    #[test]
    fn wasted_bytes_calculated() {
        let mut c = ChainBuffer::new(8);
        c.write(b"abc"); // 3 used of 8
        assert_eq!(c.wasted_bytes(), 5);
    }

    #[test]
    fn multiple_writes_append() {
        let mut c = ChainBuffer::new(16);
        c.write(b"foo");
        c.write(b"bar");
        assert_eq!(c.to_vec(), b"foobar");
    }
}
