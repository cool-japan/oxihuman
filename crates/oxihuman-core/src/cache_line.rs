// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Represents a single cache line with key, value, and metadata.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CacheLine {
    key: String,
    data: Vec<u8>,
    access_count: u64,
    dirty: bool,
    valid: bool,
}

#[allow(dead_code)]
impl CacheLine {
    pub fn new(key: &str, data: Vec<u8>) -> Self {
        Self {
            key: key.to_string(),
            data,
            access_count: 0,
            dirty: false,
            valid: true,
        }
    }

    pub fn invalid() -> Self {
        Self {
            key: String::new(),
            data: Vec::new(),
            access_count: 0,
            dirty: false,
            valid: false,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn access(&mut self) -> &[u8] {
        self.access_count += 1;
        &self.data
    }

    pub fn write(&mut self, data: Vec<u8>) {
        self.data = data;
        self.dirty = true;
        self.access_count += 1;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    pub fn access_count(&self) -> u64 {
        self.access_count
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn matches_key(&self, key: &str) -> bool {
        self.valid && self.key == key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cl = CacheLine::new("key1", vec![1, 2, 3]);
        assert_eq!(cl.key(), "key1");
        assert_eq!(cl.data(), &[1, 2, 3]);
        assert!(cl.is_valid());
        assert!(!cl.is_dirty());
    }

    #[test]
    fn test_invalid() {
        let cl = CacheLine::invalid();
        assert!(!cl.is_valid());
    }

    #[test]
    fn test_access_increments_count() {
        let mut cl = CacheLine::new("k", vec![10]);
        assert_eq!(cl.access_count(), 0);
        let _ = cl.access();
        assert_eq!(cl.access_count(), 1);
    }

    #[test]
    fn test_write_marks_dirty() {
        let mut cl = CacheLine::new("k", vec![]);
        cl.write(vec![5, 6]);
        assert!(cl.is_dirty());
        assert_eq!(cl.data(), &[5, 6]);
    }

    #[test]
    fn test_invalidate() {
        let mut cl = CacheLine::new("k", vec![1]);
        cl.invalidate();
        assert!(!cl.is_valid());
    }

    #[test]
    fn test_size() {
        let cl = CacheLine::new("k", vec![0; 100]);
        assert_eq!(cl.size(), 100);
    }

    #[test]
    fn test_mark_clean() {
        let mut cl = CacheLine::new("k", vec![]);
        cl.write(vec![1]);
        cl.mark_clean();
        assert!(!cl.is_dirty());
    }

    #[test]
    fn test_matches_key() {
        let cl = CacheLine::new("abc", vec![]);
        assert!(cl.matches_key("abc"));
        assert!(!cl.matches_key("xyz"));
    }

    #[test]
    fn test_matches_key_invalid() {
        let mut cl = CacheLine::new("abc", vec![]);
        cl.invalidate();
        assert!(!cl.matches_key("abc"));
    }

    #[test]
    fn test_write_increments_access() {
        let mut cl = CacheLine::new("k", vec![]);
        cl.write(vec![1]);
        cl.write(vec![2]);
        assert_eq!(cl.access_count(), 2);
    }
}
