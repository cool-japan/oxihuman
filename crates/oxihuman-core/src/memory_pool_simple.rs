#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Fixed-size block memory pool (pre-allocated).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MemBlock {
    pub data: Vec<u8>,
    pub in_use: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MemPoolSimple {
    pub blocks: Vec<MemBlock>,
    pub block_size: usize,
}

#[allow(dead_code)]
pub fn new_mem_pool_simple(block_size: usize, count: usize) -> MemPoolSimple {
    let blocks = (0..count)
        .map(|_| MemBlock {
            data: vec![0u8; block_size],
            in_use: false,
        })
        .collect();
    MemPoolSimple { blocks, block_size }
}

#[allow(dead_code)]
pub fn pool_alloc(pool: &mut MemPoolSimple) -> Option<usize> {
    pool.blocks.iter().position(|b| !b.in_use).inspect(|&idx| {
        pool.blocks[idx].in_use = true;
    })
}

#[allow(dead_code)]
pub fn pool_free(pool: &mut MemPoolSimple, idx: usize) {
    if idx < pool.blocks.len() {
        pool.blocks[idx].in_use = false;
        pool.blocks[idx].data.iter_mut().for_each(|b| *b = 0);
    }
}

#[allow(dead_code)]
pub fn pool_used_count(pool: &MemPoolSimple) -> usize {
    pool.blocks.iter().filter(|b| b.in_use).count()
}

#[allow(dead_code)]
pub fn pool_free_count(pool: &MemPoolSimple) -> usize {
    pool.blocks.iter().filter(|b| !b.in_use).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_all_free() {
        let pool = new_mem_pool_simple(64, 8);
        assert_eq!(pool_free_count(&pool), 8);
        assert_eq!(pool_used_count(&pool), 0);
    }

    #[test]
    fn alloc_returns_index() {
        let mut pool = new_mem_pool_simple(16, 4);
        let idx = pool_alloc(&mut pool);
        assert!(idx.is_some());
        assert_eq!(pool_used_count(&pool), 1);
    }

    #[test]
    fn alloc_sequential() {
        let mut pool = new_mem_pool_simple(16, 4);
        let a = pool_alloc(&mut pool).expect("should succeed");
        let b = pool_alloc(&mut pool).expect("should succeed");
        assert_ne!(a, b);
        assert_eq!(pool_used_count(&pool), 2);
    }

    #[test]
    fn free_releases_block() {
        let mut pool = new_mem_pool_simple(16, 4);
        let idx = pool_alloc(&mut pool).expect("should succeed");
        pool_free(&mut pool, idx);
        assert_eq!(pool_used_count(&pool), 0);
        assert_eq!(pool_free_count(&pool), 4);
    }

    #[test]
    fn alloc_all_then_fail() {
        let mut pool = new_mem_pool_simple(8, 3);
        pool_alloc(&mut pool);
        pool_alloc(&mut pool);
        pool_alloc(&mut pool);
        assert!(pool_alloc(&mut pool).is_none());
    }

    #[test]
    fn reuse_after_free() {
        let mut pool = new_mem_pool_simple(8, 2);
        let idx = pool_alloc(&mut pool).expect("should succeed");
        pool_free(&mut pool, idx);
        let reused = pool_alloc(&mut pool);
        assert!(reused.is_some());
    }

    #[test]
    fn free_out_of_bounds_no_panic() {
        let mut pool = new_mem_pool_simple(8, 2);
        pool_free(&mut pool, 99); // should not panic
    }

    #[test]
    fn block_size_stored() {
        let pool = new_mem_pool_simple(128, 2);
        assert_eq!(pool.block_size, 128);
        assert_eq!(pool.blocks[0].data.len(), 128);
    }

    #[test]
    fn free_zeroes_data() {
        let mut pool = new_mem_pool_simple(4, 2);
        let idx = pool_alloc(&mut pool).expect("should succeed");
        pool.blocks[idx].data[0] = 42;
        pool_free(&mut pool, idx);
        assert_eq!(pool.blocks[idx].data[0], 0);
    }

    #[test]
    fn used_plus_free_equals_total() {
        let mut pool = new_mem_pool_simple(8, 5);
        pool_alloc(&mut pool);
        pool_alloc(&mut pool);
        assert_eq!(pool_used_count(&pool) + pool_free_count(&pool), 5);
    }
}
