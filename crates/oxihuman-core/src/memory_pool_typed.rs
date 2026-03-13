// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Typed object memory pool — pre-allocates a fixed number of typed slots
//! and hands them out as indices, avoiding per-allocation overhead.

/// A typed memory pool storing `T` in a flat Vec, recycling freed indices.
pub struct MemoryPoolTyped<T> {
    slots: Vec<Option<T>>,
    free: Vec<usize>,
    capacity: usize,
}

impl<T: Default> MemoryPoolTyped<T> {
    /// Create a new pool with the given capacity.
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity).map(|_| None).collect();
        let free = (0..capacity).rev().collect();
        Self {
            slots,
            free,
            capacity,
        }
    }

    /// Allocate a slot and initialise with `value`. Returns the slot index.
    pub fn alloc(&mut self, value: T) -> Option<usize> {
        let idx = self.free.pop()?;
        self.slots[idx] = Some(value);
        Some(idx)
    }

    /// Free a slot by index, reclaiming it for future allocations.
    pub fn free_slot(&mut self, idx: usize) {
        if idx < self.slots.len() {
            self.slots[idx] = None;
            self.free.push(idx);
        }
    }

    /// Borrow a reference to the value at `idx`.
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.slots.get(idx)?.as_ref()
    }

    /// Borrow a mutable reference to the value at `idx`.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.slots.get_mut(idx)?.as_mut()
    }

    /// Number of live (allocated) slots.
    pub fn live_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Number of free slots remaining.
    pub fn free_count(&self) -> usize {
        self.free.len()
    }

    /// Pool capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Construct a pool with the given capacity.
pub fn new_memory_pool_typed<T: Default>(capacity: usize) -> MemoryPoolTyped<T> {
    MemoryPoolTyped::new(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_get() {
        let mut pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(8);
        let idx = pool.alloc(42).expect("should succeed");
        assert_eq!(pool.get(idx), Some(&42)); /* value stored correctly */
    }

    #[test]
    fn test_free_and_reuse() {
        let mut pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(4);
        let idx = pool.alloc(10).expect("should succeed");
        pool.free_slot(idx);
        assert_eq!(pool.get(idx), None); /* slot cleared after free */
        let idx2 = pool.alloc(20).expect("should succeed");
        assert_eq!(idx2, idx); /* slot was reused */
    }

    #[test]
    fn test_capacity_exhaustion() {
        let mut pool: MemoryPoolTyped<u8> = MemoryPoolTyped::new(2);
        assert!(pool.alloc(1).is_some()); /* first alloc ok */
        assert!(pool.alloc(2).is_some()); /* second alloc ok */
        assert!(pool.alloc(3).is_none()); /* pool full */
    }

    #[test]
    fn test_live_count() {
        let mut pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(8);
        pool.alloc(1);
        pool.alloc(2);
        assert_eq!(pool.live_count(), 2); /* two slots live */
    }

    #[test]
    fn test_free_count() {
        let mut pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(4);
        pool.alloc(1);
        assert_eq!(pool.free_count(), 3); /* one consumed */
    }

    #[test]
    fn test_get_mut() {
        let mut pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(4);
        let idx = pool.alloc(5).expect("should succeed");
        *pool.get_mut(idx).expect("should succeed") = 99;
        assert_eq!(pool.get(idx), Some(&99)); /* mutation visible */
    }

    #[test]
    fn test_capacity() {
        let pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(16);
        assert_eq!(pool.capacity(), 16); /* capacity matches request */
    }

    #[test]
    fn test_out_of_bounds_free() {
        let mut pool: MemoryPoolTyped<i32> = MemoryPoolTyped::new(4);
        pool.free_slot(100); /* should not panic */
        assert_eq!(pool.free_count(), 4); /* pool unchanged */
    }

    #[test]
    fn test_new_helper() {
        let pool = new_memory_pool_typed::<u32>(10);
        assert_eq!(pool.capacity(), 10); /* helper works */
    }
}
