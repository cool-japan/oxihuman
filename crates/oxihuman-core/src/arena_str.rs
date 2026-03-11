// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// An arena-based string allocator that stores strings in contiguous memory.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArenaStr {
    buffer: Vec<u8>,
    offsets: Vec<(usize, usize)>,
}

#[allow(dead_code)]
impl ArenaStr {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            offsets: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            offsets: Vec::new(),
        }
    }

    pub fn alloc(&mut self, s: &str) -> usize {
        let start = self.buffer.len();
        self.buffer.extend_from_slice(s.as_bytes());
        let id = self.offsets.len();
        self.offsets.push((start, s.len()));
        id
    }

    pub fn get(&self, id: usize) -> Option<&str> {
        self.offsets
            .get(id)
            .and_then(|&(start, len)| std::str::from_utf8(&self.buffer[start..start + len]).ok())
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.offsets.clear();
    }

    pub fn contains(&self, s: &str) -> bool {
        self.offsets
            .iter()
            .any(|&(start, len)| &self.buffer[start..start + len] == s.as_bytes())
    }

    pub fn avg_len(&self) -> f32 {
        if self.offsets.is_empty() {
            return 0.0;
        }
        self.buffer.len() as f32 / self.offsets.len() as f32
    }
}

impl Default for ArenaStr {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let a = ArenaStr::new();
        assert!(a.is_empty());
        assert_eq!(a.total_bytes(), 0);
    }

    #[test]
    fn test_alloc_get() {
        let mut a = ArenaStr::new();
        let id = a.alloc("hello");
        assert_eq!(a.get(id), Some("hello"));
    }

    #[test]
    fn test_multiple_allocs() {
        let mut a = ArenaStr::new();
        let id0 = a.alloc("foo");
        let id1 = a.alloc("bar");
        let id2 = a.alloc("baz");
        assert_eq!(a.get(id0), Some("foo"));
        assert_eq!(a.get(id1), Some("bar"));
        assert_eq!(a.get(id2), Some("baz"));
        assert_eq!(a.len(), 3);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let a = ArenaStr::new();
        assert_eq!(a.get(0), None);
    }

    #[test]
    fn test_total_bytes() {
        let mut a = ArenaStr::new();
        a.alloc("abc");
        a.alloc("de");
        assert_eq!(a.total_bytes(), 5);
    }

    #[test]
    fn test_clear() {
        let mut a = ArenaStr::new();
        a.alloc("test");
        a.clear();
        assert!(a.is_empty());
        assert_eq!(a.total_bytes(), 0);
    }

    #[test]
    fn test_contains() {
        let mut a = ArenaStr::new();
        a.alloc("hello");
        assert!(a.contains("hello"));
        assert!(!a.contains("world"));
    }

    #[test]
    fn test_with_capacity() {
        let a = ArenaStr::with_capacity(1024);
        assert!(a.is_empty());
    }

    #[test]
    fn test_avg_len() {
        let mut a = ArenaStr::new();
        a.alloc("ab");
        a.alloc("cdef");
        assert!((a.avg_len() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_avg_len_empty() {
        let a = ArenaStr::new();
        assert!((a.avg_len()).abs() < 1e-6);
    }
}
