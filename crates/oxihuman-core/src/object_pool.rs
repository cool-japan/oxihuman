// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Fixed-size object pool allocator for frequent allocation/deallocation patterns.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub capacity: usize,
    pub auto_grow: bool,
    pub grow_factor: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoolSlot {
    pub index: usize,
    pub generation: u64,
    pub occupied: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoolHandle {
    pub index: usize,
    pub generation: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectPool {
    pub config: PoolConfig,
    pub slots: Vec<PoolSlot>,
    pub free_list: Vec<usize>,
    pub alloc_count: u64,
}

#[allow(dead_code)]
pub fn default_pool_config(capacity: usize) -> PoolConfig {
    PoolConfig {
        capacity,
        auto_grow: false,
        grow_factor: 2,
    }
}

#[allow(dead_code)]
pub fn new_object_pool(cfg: PoolConfig) -> ObjectPool {
    let capacity = cfg.capacity;
    let slots: Vec<PoolSlot> = (0..capacity)
        .map(|i| PoolSlot {
            index: i,
            generation: 0,
            occupied: false,
        })
        .collect();
    let free_list: Vec<usize> = (0..capacity).collect();
    ObjectPool {
        config: cfg,
        slots,
        free_list,
        alloc_count: 0,
    }
}

#[allow(dead_code)]
pub fn pool_alloc(pool: &mut ObjectPool) -> Option<PoolHandle> {
    if pool.free_list.is_empty() {
        if pool.config.auto_grow {
            let old_cap = pool.slots.len();
            let new_cap = (old_cap * pool.config.grow_factor).max(old_cap + 1);
            for i in old_cap..new_cap {
                pool.slots.push(PoolSlot {
                    index: i,
                    generation: 0,
                    occupied: false,
                });
                pool.free_list.push(i);
            }
            pool.config.capacity = new_cap;
        } else {
            return None;
        }
    }
    let idx = pool.free_list.remove(0);
    let slot = &mut pool.slots[idx];
    slot.occupied = true;
    pool.alloc_count += 1;
    Some(PoolHandle {
        index: idx,
        generation: slot.generation,
    })
}

#[allow(dead_code)]
pub fn pool_free(pool: &mut ObjectPool, handle: PoolHandle) -> bool {
    if handle.index >= pool.slots.len() {
        return false;
    }
    let slot = &mut pool.slots[handle.index];
    if !slot.occupied || slot.generation != handle.generation {
        return false;
    }
    slot.occupied = false;
    slot.generation += 1;
    pool.free_list.push(handle.index);
    true
}

#[allow(dead_code)]
pub fn pool_is_valid(pool: &ObjectPool, handle: &PoolHandle) -> bool {
    if handle.index >= pool.slots.len() {
        return false;
    }
    let slot = &pool.slots[handle.index];
    slot.occupied && slot.generation == handle.generation
}

#[allow(dead_code)]
pub fn pool_used_count(pool: &ObjectPool) -> usize {
    pool.slots.iter().filter(|s| s.occupied).count()
}

#[allow(dead_code)]
pub fn pool_free_count(pool: &ObjectPool) -> usize {
    pool.free_list.len()
}

#[allow(dead_code)]
pub fn pool_capacity(pool: &ObjectPool) -> usize {
    pool.slots.len()
}

#[allow(dead_code)]
pub fn pool_alloc_count(pool: &ObjectPool) -> u64 {
    pool.alloc_count
}

#[allow(dead_code)]
pub fn pool_to_json(pool: &ObjectPool) -> String {
    format!(
        r#"{{"capacity":{},"used":{},"free":{},"alloc_count":{},"auto_grow":{}}}"#,
        pool_capacity(pool),
        pool_used_count(pool),
        pool_free_count(pool),
        pool.alloc_count,
        pool.config.auto_grow,
    )
}

#[allow(dead_code)]
pub fn pool_reset(pool: &mut ObjectPool) {
    for slot in pool.slots.iter_mut() {
        if slot.occupied {
            slot.generation += 1;
        }
        slot.occupied = false;
    }
    pool.free_list = (0..pool.slots.len()).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pool_config() {
        let cfg = default_pool_config(16);
        assert_eq!(cfg.capacity, 16);
        assert!(!cfg.auto_grow);
        assert_eq!(cfg.grow_factor, 2);
    }

    #[test]
    fn test_new_object_pool() {
        let cfg = default_pool_config(4);
        let pool = new_object_pool(cfg);
        assert_eq!(pool_capacity(&pool), 4);
        assert_eq!(pool_used_count(&pool), 0);
        assert_eq!(pool_free_count(&pool), 4);
    }

    #[test]
    fn test_pool_alloc_and_free() {
        let cfg = default_pool_config(4);
        let mut pool = new_object_pool(cfg);
        let h = pool_alloc(&mut pool).expect("alloc should succeed");
        assert_eq!(pool_used_count(&pool), 1);
        assert!(pool_is_valid(&pool, &h));
        let freed = pool_free(&mut pool, h);
        assert!(freed);
        assert_eq!(pool_used_count(&pool), 0);
    }

    #[test]
    fn test_pool_full_returns_none() {
        let cfg = default_pool_config(2);
        let mut pool = new_object_pool(cfg);
        let _h1 = pool_alloc(&mut pool).expect("should succeed");
        let _h2 = pool_alloc(&mut pool).expect("should succeed");
        let h3 = pool_alloc(&mut pool);
        assert!(h3.is_none());
    }

    #[test]
    fn test_pool_handle_invalidated_after_free() {
        let cfg = default_pool_config(4);
        let mut pool = new_object_pool(cfg);
        let h = pool_alloc(&mut pool).expect("should succeed");
        let h_clone = h.clone();
        pool_free(&mut pool, h);
        assert!(!pool_is_valid(&pool, &h_clone));
    }

    #[test]
    fn test_pool_alloc_count() {
        let cfg = default_pool_config(8);
        let mut pool = new_object_pool(cfg);
        pool_alloc(&mut pool);
        pool_alloc(&mut pool);
        pool_alloc(&mut pool);
        assert_eq!(pool_alloc_count(&pool), 3);
    }

    #[test]
    fn test_pool_reset() {
        let cfg = default_pool_config(4);
        let mut pool = new_object_pool(cfg);
        let h = pool_alloc(&mut pool).expect("should succeed");
        pool_reset(&mut pool);
        assert_eq!(pool_used_count(&pool), 0);
        assert_eq!(pool_free_count(&pool), 4);
        assert!(!pool_is_valid(&pool, &h));
    }

    #[test]
    fn test_pool_to_json() {
        let cfg = default_pool_config(4);
        let pool = new_object_pool(cfg);
        let json = pool_to_json(&pool);
        assert!(json.contains("capacity"));
        assert!(json.contains("alloc_count"));
    }

    #[test]
    fn test_pool_auto_grow() {
        let mut cfg = default_pool_config(2);
        cfg.auto_grow = true;
        let mut pool = new_object_pool(cfg);
        let _h1 = pool_alloc(&mut pool).expect("should succeed");
        let _h2 = pool_alloc(&mut pool).expect("should succeed");
        let h3 = pool_alloc(&mut pool);
        assert!(h3.is_some(), "should grow and allocate");
        assert!(pool_capacity(&pool) > 2);
    }

    #[test]
    fn test_pool_double_free_fails() {
        let cfg = default_pool_config(4);
        let mut pool = new_object_pool(cfg);
        let h = pool_alloc(&mut pool).expect("should succeed");
        let h2 = PoolHandle {
            index: h.index,
            generation: h.generation,
        };
        pool_free(&mut pool, h);
        let second_free = pool_free(&mut pool, h2);
        assert!(!second_free);
    }
}
