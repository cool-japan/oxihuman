#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generational resource handles with a handle pool.

/// A generational handle to a resource.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle {
    index: u32,
    generation: u32,
}

/// Pool that allocates and recycles generational handles.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HandlePool {
    generations: Vec<u32>,
    free_list: Vec<u32>,
    capacity: usize,
}

#[allow(dead_code)]
pub fn new_handle_pool(capacity: usize) -> HandlePool {
    HandlePool {
        generations: Vec::with_capacity(capacity),
        free_list: Vec::new(),
        capacity,
    }
}

#[allow(dead_code)]
pub fn allocate_handle(pool: &mut HandlePool) -> Option<ResourceHandle> {
    if let Some(idx) = pool.free_list.pop() {
        pool.generations[idx as usize] += 1;
        Some(ResourceHandle {
            index: idx,
            generation: pool.generations[idx as usize],
        })
    } else if pool.generations.len() < pool.capacity {
        let idx = pool.generations.len() as u32;
        pool.generations.push(1);
        Some(ResourceHandle {
            index: idx,
            generation: 1,
        })
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn release_handle(pool: &mut HandlePool, handle: ResourceHandle) {
    let idx = handle.index as usize;
    if idx < pool.generations.len() && pool.generations[idx] == handle.generation {
        pool.generations[idx] += 1;
        pool.free_list.push(handle.index);
    }
}

#[allow(dead_code)]
pub fn handle_is_valid(pool: &HandlePool, handle: ResourceHandle) -> bool {
    let idx = handle.index as usize;
    idx < pool.generations.len() && pool.generations[idx] == handle.generation
}

#[allow(dead_code)]
pub fn handle_count(pool: &HandlePool) -> usize {
    pool.generations.len() - pool.free_list.len()
}

#[allow(dead_code)]
pub fn handle_generation(handle: ResourceHandle) -> u32 {
    handle.generation
}

#[allow(dead_code)]
pub fn handle_index(handle: ResourceHandle) -> u32 {
    handle.index
}

#[allow(dead_code)]
pub fn pool_capacity(pool: &HandlePool) -> usize {
    pool.capacity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = new_handle_pool(10);
        assert_eq!(pool_capacity(&pool), 10);
        assert_eq!(handle_count(&pool), 0);
    }

    #[test]
    fn test_allocate() {
        let mut pool = new_handle_pool(10);
        let h = allocate_handle(&mut pool).unwrap();
        assert_eq!(handle_index(h), 0);
        assert_eq!(handle_generation(h), 1);
    }

    #[test]
    fn test_valid() {
        let mut pool = new_handle_pool(10);
        let h = allocate_handle(&mut pool).unwrap();
        assert!(handle_is_valid(&pool, h));
    }

    #[test]
    fn test_release_invalidates() {
        let mut pool = new_handle_pool(10);
        let h = allocate_handle(&mut pool).unwrap();
        release_handle(&mut pool, h);
        assert!(!handle_is_valid(&pool, h));
    }

    #[test]
    fn test_reuse_slot() {
        let mut pool = new_handle_pool(10);
        let h1 = allocate_handle(&mut pool).unwrap();
        release_handle(&mut pool, h1);
        let h2 = allocate_handle(&mut pool).unwrap();
        assert_eq!(handle_index(h2), 0);
        assert_eq!(handle_generation(h2), 3); // gen bumped on release and alloc
    }

    #[test]
    fn test_handle_count() {
        let mut pool = new_handle_pool(10);
        allocate_handle(&mut pool);
        allocate_handle(&mut pool);
        assert_eq!(handle_count(&pool), 2);
    }

    #[test]
    fn test_capacity_limit() {
        let mut pool = new_handle_pool(1);
        let _ = allocate_handle(&mut pool).unwrap();
        assert!(allocate_handle(&mut pool).is_none());
    }

    #[test]
    fn test_double_release() {
        let mut pool = new_handle_pool(10);
        let h = allocate_handle(&mut pool).unwrap();
        release_handle(&mut pool, h);
        release_handle(&mut pool, h); // no-op, generation mismatch
        assert_eq!(handle_count(&pool), 0);
    }

    #[test]
    fn test_multiple_allocations() {
        let mut pool = new_handle_pool(5);
        let handles: Vec<_> = (0..5).filter_map(|_| allocate_handle(&mut pool)).collect();
        assert_eq!(handles.len(), 5);
    }

    #[test]
    fn test_handle_equality() {
        let h1 = ResourceHandle { index: 0, generation: 1 };
        let h2 = ResourceHandle { index: 0, generation: 1 };
        assert_eq!(h1, h2);
    }
}
