// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Region/zone allocator stub — divides a fixed address space into named
//! regions and tracks allocations within each region independently.

use std::collections::HashMap;

/// A named region with a start offset and size limit.
#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
    pub start: usize,
    pub size: usize,
    pub used: usize,
}

impl Region {
    fn new(name: &str, start: usize, size: usize) -> Self {
        Self {
            name: name.to_string(),
            start,
            size,
            used: 0,
        }
    }

    /// Allocate `bytes` from this region. Returns offset into the full address space.
    fn alloc(&mut self, bytes: usize) -> Option<usize> {
        if self.used + bytes > self.size {
            return None;
        }
        let addr = self.start + self.used;
        self.used += bytes;
        Some(addr)
    }

    /// Reset this region's usage counter.
    fn reset(&mut self) {
        self.used = 0;
    }

    /// Free bytes remaining in this region.
    pub fn free_bytes(&self) -> usize {
        self.size.saturating_sub(self.used)
    }
}

/// Manages multiple named regions within a total address space.
pub struct RegionAllocator {
    regions: HashMap<String, Region>,
    total_size: usize,
}

impl RegionAllocator {
    /// Create a region allocator over a total address space of `total_size` bytes.
    pub fn new(total_size: usize) -> Self {
        Self {
            regions: HashMap::new(),
            total_size,
        }
    }

    /// Register a named region starting at `start` with `size` bytes.
    pub fn add_region(&mut self, name: &str, start: usize, size: usize) -> bool {
        if start + size > self.total_size {
            return false;
        }
        self.regions
            .insert(name.to_string(), Region::new(name, start, size));
        true
    }

    /// Allocate `bytes` from region `name`. Returns absolute address or None.
    pub fn alloc_in(&mut self, name: &str, bytes: usize) -> Option<usize> {
        self.regions.get_mut(name)?.alloc(bytes)
    }

    /// Reset a named region's allocation cursor.
    pub fn reset_region(&mut self, name: &str) {
        if let Some(r) = self.regions.get_mut(name) {
            r.reset();
        }
    }

    /// Free bytes remaining in region `name`.
    pub fn free_in(&self, name: &str) -> usize {
        self.regions.get(name).map(|r| r.free_bytes()).unwrap_or(0)
    }

    /// Number of registered regions.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Total capacity of the address space.
    pub fn total_size(&self) -> usize {
        self.total_size
    }
}

/// Create a new region allocator with the given total size.
pub fn new_region_allocator(total_size: usize) -> RegionAllocator {
    RegionAllocator::new(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_region() {
        let mut ra = RegionAllocator::new(1024);
        assert!(ra.add_region("heap", 0, 512)); /* valid region */
    }

    #[test]
    fn test_add_region_overflow() {
        let mut ra = RegionAllocator::new(256);
        assert!(!ra.add_region("bad", 200, 100)); /* would overflow */
    }

    #[test]
    fn test_alloc_in() {
        let mut ra = RegionAllocator::new(1024);
        ra.add_region("heap", 0, 512);
        let addr = ra.alloc_in("heap", 64);
        assert_eq!(addr, Some(0)); /* first alloc at region start */
    }

    #[test]
    fn test_alloc_sequential() {
        let mut ra = RegionAllocator::new(1024);
        ra.add_region("heap", 0, 512);
        ra.alloc_in("heap", 64);
        let addr = ra.alloc_in("heap", 128).unwrap();
        assert_eq!(addr, 64); /* second alloc after first */
    }

    #[test]
    fn test_alloc_exhausted() {
        let mut ra = RegionAllocator::new(1024);
        ra.add_region("tiny", 0, 16);
        ra.alloc_in("tiny", 16);
        assert!(ra.alloc_in("tiny", 1).is_none()); /* region full */
    }

    #[test]
    fn test_reset_region() {
        let mut ra = RegionAllocator::new(1024);
        ra.add_region("heap", 0, 512);
        ra.alloc_in("heap", 256);
        ra.reset_region("heap");
        assert_eq!(ra.free_in("heap"), 512); /* all bytes free after reset */
    }

    #[test]
    fn test_free_in() {
        let mut ra = RegionAllocator::new(1024);
        ra.add_region("heap", 0, 512);
        ra.alloc_in("heap", 100);
        assert_eq!(ra.free_in("heap"), 412); /* correct free count */
    }

    #[test]
    fn test_region_count() {
        let mut ra = RegionAllocator::new(4096);
        ra.add_region("a", 0, 1024);
        ra.add_region("b", 1024, 1024);
        assert_eq!(ra.region_count(), 2); /* two regions registered */
    }

    #[test]
    fn test_total_size() {
        let ra = RegionAllocator::new(8192);
        assert_eq!(ra.total_size(), 8192); /* accessor correct */
    }

    #[test]
    fn test_new_helper() {
        let ra = new_region_allocator(2048);
        assert_eq!(ra.total_size(), 2048); /* helper works */
    }
}
