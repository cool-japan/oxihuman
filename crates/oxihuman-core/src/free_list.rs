// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A free-list allocator that reuses indices from a pool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FreeList {
    free_indices: Vec<usize>,
    next_fresh: usize,
    allocated_count: usize,
}

#[allow(dead_code)]
impl FreeList {
    pub fn new() -> Self {
        Self {
            free_indices: Vec::new(),
            next_fresh: 0,
            allocated_count: 0,
        }
    }

    pub fn allocate(&mut self) -> usize {
        self.allocated_count += 1;
        if let Some(idx) = self.free_indices.pop() {
            idx
        } else {
            let idx = self.next_fresh;
            self.next_fresh += 1;
            idx
        }
    }

    pub fn deallocate(&mut self, index: usize) -> bool {
        if index >= self.next_fresh {
            return false;
        }
        if self.free_indices.contains(&index) {
            return false;
        }
        self.free_indices.push(index);
        self.allocated_count -= 1;
        true
    }

    pub fn is_allocated(&self, index: usize) -> bool {
        index < self.next_fresh && !self.free_indices.contains(&index)
    }

    pub fn allocated_count(&self) -> usize {
        self.allocated_count
    }

    pub fn free_count(&self) -> usize {
        self.free_indices.len()
    }

    pub fn total_capacity(&self) -> usize {
        self.next_fresh
    }

    pub fn is_empty(&self) -> bool {
        self.allocated_count == 0
    }

    pub fn reset(&mut self) {
        self.free_indices.clear();
        self.next_fresh = 0;
        self.allocated_count = 0;
    }

    pub fn utilization(&self) -> f32 {
        if self.next_fresh == 0 {
            return 0.0;
        }
        self.allocated_count as f32 / self.next_fresh as f32
    }

    pub fn peek_next(&self) -> usize {
        self.free_indices.last().copied().unwrap_or(self.next_fresh)
    }
}

impl Default for FreeList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_sequential() {
        let mut fl = FreeList::new();
        assert_eq!(fl.allocate(), 0);
        assert_eq!(fl.allocate(), 1);
        assert_eq!(fl.allocate(), 2);
    }

    #[test]
    fn test_deallocate_and_reuse() {
        let mut fl = FreeList::new();
        fl.allocate(); // 0
        fl.allocate(); // 1
        fl.deallocate(0);
        let reused = fl.allocate();
        assert_eq!(reused, 0);
    }

    #[test]
    fn test_is_allocated() {
        let mut fl = FreeList::new();
        fl.allocate();
        assert!(fl.is_allocated(0));
        fl.deallocate(0);
        assert!(!fl.is_allocated(0));
    }

    #[test]
    fn test_allocated_count() {
        let mut fl = FreeList::new();
        fl.allocate();
        fl.allocate();
        fl.deallocate(0);
        assert_eq!(fl.allocated_count(), 1);
    }

    #[test]
    fn test_free_count() {
        let mut fl = FreeList::new();
        fl.allocate();
        fl.allocate();
        fl.deallocate(1);
        assert_eq!(fl.free_count(), 1);
    }

    #[test]
    fn test_total_capacity() {
        let mut fl = FreeList::new();
        fl.allocate();
        fl.allocate();
        assert_eq!(fl.total_capacity(), 2);
    }

    #[test]
    fn test_reset() {
        let mut fl = FreeList::new();
        fl.allocate();
        fl.allocate();
        fl.reset();
        assert!(fl.is_empty());
        assert_eq!(fl.total_capacity(), 0);
    }

    #[test]
    fn test_double_deallocate() {
        let mut fl = FreeList::new();
        fl.allocate();
        assert!(fl.deallocate(0));
        assert!(!fl.deallocate(0));
    }

    #[test]
    fn test_utilization() {
        let mut fl = FreeList::new();
        fl.allocate();
        fl.allocate();
        fl.deallocate(0);
        assert!((fl.utilization() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_peek_next() {
        let mut fl = FreeList::new();
        assert_eq!(fl.peek_next(), 0);
        fl.allocate();
        fl.allocate();
        fl.deallocate(0);
        assert_eq!(fl.peek_next(), 0);
    }
}
