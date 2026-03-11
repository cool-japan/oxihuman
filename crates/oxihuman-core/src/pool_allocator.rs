// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fixed-size object pool allocator.

/// A slot in the pool.
#[derive(Debug, Clone)]
pub struct PoolSlot<T> {
    pub value: T,
    pub generation: u32,
}

/// Handle into the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolHandle {
    pub index: usize,
    pub generation: u32,
}

/// Fixed-capacity pool allocator.
pub struct PoolAllocator<T> {
    slots: Vec<Option<PoolSlot<T>>>,
    free: Vec<usize>,
    generations: Vec<u32>,
    capacity: usize,
    live_count: usize,
}

#[allow(dead_code)]
impl<T: Clone> PoolAllocator<T> {
    pub fn new(capacity: usize) -> Self {
        let slots = (0..capacity).map(|_| None).collect();
        let free = (0..capacity).rev().collect();
        let generations = vec![0u32; capacity];
        PoolAllocator {
            slots,
            free,
            generations,
            capacity,
            live_count: 0,
        }
    }

    pub fn alloc(&mut self, value: T) -> Option<PoolHandle> {
        let index = self.free.pop()?;
        self.generations[index] += 1;
        let gen = self.generations[index];
        self.slots[index] = Some(PoolSlot {
            value,
            generation: gen,
        });
        self.live_count += 1;
        Some(PoolHandle {
            index,
            generation: gen,
        })
    }

    pub fn free(&mut self, handle: PoolHandle) -> bool {
        if handle.index >= self.capacity {
            return false;
        }
        match &self.slots[handle.index] {
            Some(s) if s.generation == handle.generation => {
                self.slots[handle.index] = None;
                self.free.push(handle.index);
                self.live_count -= 1;
                true
            }
            _ => false,
        }
    }

    pub fn get(&self, handle: PoolHandle) -> Option<&T> {
        if handle.index >= self.capacity {
            return None;
        }
        self.slots[handle.index]
            .as_ref()
            .filter(|s| s.generation == handle.generation)
            .map(|s| &s.value)
    }

    pub fn get_mut(&mut self, handle: PoolHandle) -> Option<&mut T> {
        if handle.index >= self.capacity {
            return None;
        }
        self.slots[handle.index]
            .as_mut()
            .filter(|s| s.generation == handle.generation)
            .map(|s| &mut s.value)
    }

    pub fn is_valid(&self, handle: PoolHandle) -> bool {
        handle.index < self.capacity
            && self.slots[handle.index]
                .as_ref()
                .is_some_and(|s| s.generation == handle.generation)
    }

    pub fn live_count(&self) -> usize {
        self.live_count
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn free_count(&self) -> usize {
        self.free.len()
    }

    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
        self.free.clear();
        for i in (0..self.capacity).rev() {
            self.free.push(i);
        }
        self.live_count = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = (PoolHandle, &T)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            slot.as_ref().map(|s| {
                (
                    PoolHandle {
                        index: i,
                        generation: s.generation,
                    },
                    &s.value,
                )
            })
        })
    }
}

pub fn new_pool<T: Clone>(capacity: usize) -> PoolAllocator<T> {
    PoolAllocator::new(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_get() {
        let mut pool: PoolAllocator<i32> = new_pool(4);
        let h = pool.alloc(42).unwrap();
        assert_eq!(*pool.get(h).unwrap(), 42);
    }

    #[test]
    fn free_invalidates_handle() {
        let mut pool: PoolAllocator<i32> = new_pool(4);
        let h = pool.alloc(10).unwrap();
        assert!(pool.free(h));
        assert!(pool.get(h).is_none());
    }

    #[test]
    fn generation_prevents_use_after_free() {
        let mut pool: PoolAllocator<i32> = new_pool(4);
        let h = pool.alloc(1).unwrap();
        pool.free(h);
        let h2 = pool.alloc(2).unwrap();
        assert_eq!(h2.index, h.index);
        assert!(pool.get(h).is_none());
        assert_eq!(*pool.get(h2).unwrap(), 2);
    }

    #[test]
    fn live_count_tracking() {
        let mut pool: PoolAllocator<u8> = new_pool(8);
        let h1 = pool.alloc(1).unwrap();
        let h2 = pool.alloc(2).unwrap();
        assert_eq!(pool.live_count(), 2);
        pool.free(h1);
        assert_eq!(pool.live_count(), 1);
        pool.free(h2);
        assert_eq!(pool.live_count(), 0);
    }

    #[test]
    fn capacity_exhausted() {
        let mut pool: PoolAllocator<u8> = new_pool(2);
        pool.alloc(1).unwrap();
        pool.alloc(2).unwrap();
        assert!(pool.alloc(3).is_none());
    }

    #[test]
    fn reset_restores_capacity() {
        let mut pool: PoolAllocator<u8> = new_pool(2);
        pool.alloc(1).unwrap();
        pool.alloc(2).unwrap();
        pool.reset();
        assert_eq!(pool.free_count(), 2);
        assert!(pool.alloc(3).is_some());
    }

    #[test]
    fn get_mut_modifies_value() {
        let mut pool: PoolAllocator<i32> = new_pool(4);
        let h = pool.alloc(10).unwrap();
        *pool.get_mut(h).unwrap() = 99;
        assert_eq!(*pool.get(h).unwrap(), 99);
    }

    #[test]
    fn iter_live_items() {
        let mut pool: PoolAllocator<i32> = new_pool(4);
        let h1 = pool.alloc(1).unwrap();
        let _h2 = pool.alloc(2).unwrap();
        pool.free(h1);
        let vals: Vec<i32> = pool.iter().map(|(_, v)| *v).collect();
        assert_eq!(vals, vec![2]);
    }

    #[test]
    fn is_valid_check() {
        let mut pool: PoolAllocator<i32> = new_pool(4);
        let h = pool.alloc(5).unwrap();
        assert!(pool.is_valid(h));
        pool.free(h);
        assert!(!pool.is_valid(h));
    }
}
