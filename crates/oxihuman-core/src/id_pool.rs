// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ID pool: recycles released IDs before allocating new ones.

/// An opaque ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]
pub struct Id(pub u64);

/// ID pool.
#[derive(Debug)]
#[allow(dead_code)]
pub struct IdPool {
    next: u64,
    recycled: Vec<u64>,
    active: std::collections::HashSet<u64>,
}

/// Create a new IdPool starting at `start`.
#[allow(dead_code)]
pub fn new_id_pool(start: u64) -> IdPool {
    IdPool {
        next: start,
        recycled: Vec::new(),
        active: std::collections::HashSet::new(),
    }
}

/// Allocate an ID.
#[allow(dead_code)]
pub fn id_alloc(pool: &mut IdPool) -> Id {
    if let Some(id) = pool.recycled.pop() {
        pool.active.insert(id);
        Id(id)
    } else {
        let id = pool.next;
        pool.next += 1;
        pool.active.insert(id);
        Id(id)
    }
}

/// Release an ID back to the pool.
#[allow(dead_code)]
pub fn id_release(pool: &mut IdPool, id: Id) -> bool {
    if pool.active.remove(&id.0) {
        pool.recycled.push(id.0);
        true
    } else {
        false
    }
}

/// Whether an ID is currently active.
#[allow(dead_code)]
pub fn id_is_active(pool: &IdPool, id: Id) -> bool {
    pool.active.contains(&id.0)
}

/// Count of active IDs.
#[allow(dead_code)]
pub fn id_active_count(pool: &IdPool) -> usize {
    pool.active.len()
}

/// Count of recycled (waiting) IDs.
#[allow(dead_code)]
pub fn id_recycled_count(pool: &IdPool) -> usize {
    pool.recycled.len()
}

/// Peek at the next ID that would be allocated without recycling.
#[allow(dead_code)]
pub fn id_peek_next(pool: &IdPool) -> u64 {
    pool.next
}

/// Release all active IDs at once.
#[allow(dead_code)]
pub fn id_release_all(pool: &mut IdPool) {
    let ids: Vec<u64> = pool.active.drain().collect();
    pool.recycled.extend(ids);
}

/// Total IDs ever allocated (next - start not tracked, use active + recycled).
#[allow(dead_code)]
pub fn id_total(pool: &IdPool) -> usize {
    pool.active.len() + pool.recycled.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_sequential() {
        let mut pool = new_id_pool(1);
        let a = id_alloc(&mut pool);
        let b = id_alloc(&mut pool);
        assert_eq!(a.0, 1);
        assert_eq!(b.0, 2);
    }

    #[test]
    fn test_release_and_reuse() {
        let mut pool = new_id_pool(0);
        let a = id_alloc(&mut pool);
        id_release(&mut pool, a);
        let b = id_alloc(&mut pool);
        assert_eq!(b.0, a.0);
    }

    #[test]
    fn test_is_active() {
        let mut pool = new_id_pool(0);
        let id = id_alloc(&mut pool);
        assert!(id_is_active(&pool, id));
        id_release(&mut pool, id);
        assert!(!id_is_active(&pool, id));
    }

    #[test]
    fn test_double_release() {
        let mut pool = new_id_pool(0);
        let id = id_alloc(&mut pool);
        assert!(id_release(&mut pool, id));
        assert!(!id_release(&mut pool, id));
    }

    #[test]
    fn test_active_count() {
        let mut pool = new_id_pool(0);
        id_alloc(&mut pool);
        id_alloc(&mut pool);
        assert_eq!(id_active_count(&pool), 2);
    }

    #[test]
    fn test_recycled_count() {
        let mut pool = new_id_pool(0);
        let id = id_alloc(&mut pool);
        id_release(&mut pool, id);
        assert_eq!(id_recycled_count(&pool), 1);
    }

    #[test]
    fn test_release_all() {
        let mut pool = new_id_pool(0);
        id_alloc(&mut pool);
        id_alloc(&mut pool);
        id_release_all(&mut pool);
        assert_eq!(id_active_count(&pool), 0);
        assert_eq!(id_recycled_count(&pool), 2);
    }

    #[test]
    fn test_peek_next() {
        let pool = new_id_pool(10);
        assert_eq!(id_peek_next(&pool), 10);
    }

    #[test]
    fn test_total() {
        let mut pool = new_id_pool(0);
        id_alloc(&mut pool);
        let b = id_alloc(&mut pool);
        id_release(&mut pool, b);
        assert_eq!(id_total(&pool), 2);
    }
}
