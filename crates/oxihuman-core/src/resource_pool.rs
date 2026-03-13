// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Resource pool with borrowing semantics.

/// State of a pooled resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    Available,
    InUse,
}

/// A single resource slot.
#[derive(Debug)]
pub struct ResourceSlot<T> {
    pub resource: T,
    pub state: ResourceState,
    pub borrow_count: u32,
}

/// Pool of resources with acquire/release semantics.
pub struct ResourcePool<T> {
    slots: Vec<ResourceSlot<T>>,
}

#[allow(dead_code)]
impl<T: Clone> ResourcePool<T> {
    pub fn new(resources: Vec<T>) -> Self {
        let slots = resources
            .into_iter()
            .map(|r| ResourceSlot {
                resource: r,
                state: ResourceState::Available,
                borrow_count: 0,
            })
            .collect();
        ResourcePool { slots }
    }

    pub fn acquire(&mut self) -> Option<usize> {
        self.slots.iter_mut().enumerate().find_map(|(i, s)| {
            if s.state == ResourceState::Available {
                s.state = ResourceState::InUse;
                s.borrow_count += 1;
                Some(i)
            } else {
                None
            }
        })
    }

    pub fn release(&mut self, index: usize) -> bool {
        if index >= self.slots.len() {
            return false;
        }
        if self.slots[index].state == ResourceState::InUse {
            self.slots[index].state = ResourceState::Available;
            true
        } else {
            false
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index).map(|s| &s.resource)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index).map(|s| &mut s.resource)
    }

    pub fn is_in_use(&self, index: usize) -> bool {
        self.slots
            .get(index)
            .is_some_and(|s| s.state == ResourceState::InUse)
    }

    pub fn available_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.state == ResourceState::Available)
            .count()
    }

    pub fn in_use_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.state == ResourceState::InUse)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.slots.len()
    }

    pub fn borrow_count(&self, index: usize) -> u32 {
        self.slots.get(index).map(|s| s.borrow_count).unwrap_or(0)
    }

    pub fn release_all(&mut self) {
        for s in &mut self.slots {
            s.state = ResourceState::Available;
        }
    }

    pub fn total_borrows(&self) -> u64 {
        self.slots.iter().map(|s| s.borrow_count as u64).sum()
    }
}

pub fn new_resource_pool<T: Clone>(resources: Vec<T>) -> ResourcePool<T> {
    ResourcePool::new(resources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_available() {
        let mut pool = new_resource_pool(vec![10i32, 20, 30]);
        let idx = pool.acquire().expect("should succeed");
        assert!(pool.is_in_use(idx));
    }

    #[test]
    fn release_makes_available() {
        let mut pool = new_resource_pool(vec![1i32]);
        let idx = pool.acquire().expect("should succeed");
        assert!(pool.release(idx));
        assert!(!pool.is_in_use(idx));
    }

    #[test]
    fn acquire_all_then_none() {
        let mut pool = new_resource_pool(vec![1i32, 2]);
        pool.acquire().expect("should succeed");
        pool.acquire().expect("should succeed");
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn available_count() {
        let mut pool = new_resource_pool(vec![1i32, 2, 3]);
        pool.acquire();
        assert_eq!(pool.available_count(), 2);
        assert_eq!(pool.in_use_count(), 1);
    }

    #[test]
    fn get_resource() {
        let pool = new_resource_pool(vec![42i32]);
        assert_eq!(*pool.get(0).expect("should succeed"), 42);
    }

    #[test]
    fn borrow_count_tracking() {
        let mut pool = new_resource_pool(vec![1i32]);
        pool.acquire();
        pool.release(0);
        pool.acquire();
        assert_eq!(pool.borrow_count(0), 2);
    }

    #[test]
    fn release_all() {
        let mut pool = new_resource_pool(vec![1i32, 2, 3]);
        pool.acquire();
        pool.acquire();
        pool.release_all();
        assert_eq!(pool.available_count(), 3);
    }

    #[test]
    fn total_borrows() {
        let mut pool = new_resource_pool(vec![1i32, 2]);
        pool.acquire();
        pool.release(0);
        pool.acquire();
        pool.acquire();
        assert_eq!(pool.total_borrows(), 3);
    }

    #[test]
    fn out_of_bounds_release() {
        let mut pool = new_resource_pool(vec![1i32]);
        assert!(!pool.release(99));
    }
}
