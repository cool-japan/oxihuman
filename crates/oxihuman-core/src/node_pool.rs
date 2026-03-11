// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Node pool: typed object pool with stable handle-based access.

/// Opaque node handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct NodeHandle(u32, u32); // (index, generation)

/// Node slot.
#[derive(Debug)]
#[allow(dead_code)]
enum NodeSlot<T> {
    Occupied {
        value: T,
        generation: u32,
    },
    Free {
        next_free: Option<u32>,
        generation: u32,
    },
}

/// Node pool.
#[derive(Debug)]
#[allow(dead_code)]
pub struct NodePool<T> {
    slots: Vec<NodeSlot<T>>,
    free_head: Option<u32>,
    count: usize,
}

/// Create a new NodePool.
#[allow(dead_code)]
pub fn new_node_pool<T>() -> NodePool<T> {
    NodePool {
        slots: Vec::new(),
        free_head: None,
        count: 0,
    }
}

/// Allocate a node; returns its handle.
#[allow(dead_code)]
pub fn np_alloc<T>(pool: &mut NodePool<T>, value: T) -> NodeHandle {
    if let Some(idx) = pool.free_head {
        let idx_usize = idx as usize;
        let gen = match &pool.slots[idx_usize] {
            NodeSlot::Free {
                generation,
                next_free,
            } => {
                pool.free_head = *next_free;
                *generation
            }
            _ => unreachable!(),
        };
        pool.slots[idx_usize] = NodeSlot::Occupied {
            value,
            generation: gen,
        };
        pool.count += 1;
        NodeHandle(idx, gen)
    } else {
        let idx = pool.slots.len() as u32;
        pool.slots.push(NodeSlot::Occupied {
            value,
            generation: 0,
        });
        pool.count += 1;
        NodeHandle(idx, 0)
    }
}

/// Free a node by handle; returns the value if handle was valid.
#[allow(dead_code)]
pub fn np_free<T>(pool: &mut NodePool<T>, handle: NodeHandle) -> Option<T> {
    let idx = handle.0 as usize;
    if idx >= pool.slots.len() {
        return None;
    }
    let gen = handle.1;
    let old = std::mem::replace(
        &mut pool.slots[idx],
        NodeSlot::Free {
            next_free: pool.free_head,
            generation: gen + 1,
        },
    );
    if let NodeSlot::Occupied { value, generation } = old {
        if generation == gen {
            pool.free_head = Some(handle.0);
            pool.count -= 1;
            return Some(value);
        }
        // wrong generation – restore
        pool.slots[idx] = NodeSlot::Occupied { value, generation };
    } else {
        pool.slots[idx] = old;
    }
    None
}

/// Get reference by handle.
#[allow(dead_code)]
pub fn np_get<T>(pool: &NodePool<T>, handle: NodeHandle) -> Option<&T> {
    let idx = handle.0 as usize;
    if let Some(NodeSlot::Occupied { value, generation }) = pool.slots.get(idx) {
        if *generation == handle.1 {
            return Some(value);
        }
    }
    None
}

/// Get mutable reference by handle.
#[allow(dead_code)]
pub fn np_get_mut<T>(pool: &mut NodePool<T>, handle: NodeHandle) -> Option<&mut T> {
    let idx = handle.0 as usize;
    if let Some(NodeSlot::Occupied { value, generation }) = pool.slots.get_mut(idx) {
        if *generation == handle.1 {
            return Some(value);
        }
    }
    None
}

/// Whether handle is valid.
#[allow(dead_code)]
pub fn np_is_valid<T>(pool: &NodePool<T>, handle: NodeHandle) -> bool {
    np_get(pool, handle).is_some()
}

/// Number of live nodes.
#[allow(dead_code)]
pub fn np_count<T>(pool: &NodePool<T>) -> usize {
    pool.count
}

/// Total capacity (including free slots).
#[allow(dead_code)]
pub fn np_capacity<T>(pool: &NodePool<T>) -> usize {
    pool.slots.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_get() {
        let mut pool = new_node_pool();
        let h = np_alloc(&mut pool, 42u32);
        assert_eq!(np_get(&pool, h), Some(&42));
    }

    #[test]
    fn test_free_and_reuse() {
        let mut pool = new_node_pool();
        let h1 = np_alloc(&mut pool, 1u32);
        np_free(&mut pool, h1);
        let h2 = np_alloc(&mut pool, 2u32);
        assert_eq!(h2.0, h1.0);
        assert_ne!(h2.1, h1.1); // generation bumped
    }

    #[test]
    fn test_stale_handle() {
        let mut pool = new_node_pool();
        let h1 = np_alloc(&mut pool, 1u32);
        np_free(&mut pool, h1);
        np_alloc(&mut pool, 2u32);
        assert_eq!(np_get(&pool, h1), None); // stale
    }

    #[test]
    fn test_count() {
        let mut pool = new_node_pool();
        let h = np_alloc(&mut pool, 1u32);
        np_alloc(&mut pool, 2u32);
        assert_eq!(np_count(&pool), 2);
        np_free(&mut pool, h);
        assert_eq!(np_count(&pool), 1);
    }

    #[test]
    fn test_get_mut() {
        let mut pool = new_node_pool();
        let h = np_alloc(&mut pool, 5u32);
        if let Some(v) = np_get_mut(&mut pool, h) {
            *v = 99;
        }
        assert_eq!(np_get(&pool, h), Some(&99));
    }

    #[test]
    fn test_invalid_handle() {
        let pool: NodePool<u32> = new_node_pool();
        assert!(!np_is_valid(&pool, NodeHandle(0, 0)));
    }

    #[test]
    fn test_capacity_grows() {
        let mut pool = new_node_pool();
        np_alloc(&mut pool, 1u32);
        np_alloc(&mut pool, 2u32);
        np_alloc(&mut pool, 3u32);
        assert_eq!(np_capacity(&pool), 3);
    }

    #[test]
    fn test_free_out_of_bounds() {
        let mut pool: NodePool<u32> = new_node_pool();
        assert_eq!(np_free(&mut pool, NodeHandle(99, 0)), None);
    }

    #[test]
    fn test_is_valid() {
        let mut pool = new_node_pool();
        let h = np_alloc(&mut pool, 0u32);
        assert!(np_is_valid(&pool, h));
    }
}
