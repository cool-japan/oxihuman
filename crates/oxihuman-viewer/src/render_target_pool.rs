// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A pooled render target descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PoolTarget {
    width: u32,
    height: u32,
    in_use: bool,
}

/// A pool of reusable render targets.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderTargetPool {
    targets: Vec<PoolTarget>,
}

/// Create a new empty render target pool.
#[allow(dead_code)]
pub fn new_render_target_pool() -> RenderTargetPool {
    RenderTargetPool {
        targets: Vec::new(),
    }
}

/// Acquire a render target with the given dimensions. Returns the index.
#[allow(dead_code)]
pub fn acquire_target(pool: &mut RenderTargetPool, width: u32, height: u32) -> usize {
    // Try to find a free target with matching dimensions
    for (i, t) in pool.targets.iter_mut().enumerate() {
        if !t.in_use && t.width == width && t.height == height {
            t.in_use = true;
            return i;
        }
    }
    // Create a new one
    pool.targets.push(PoolTarget {
        width,
        height,
        in_use: true,
    });
    pool.targets.len() - 1
}

/// Release a render target back to the pool.
#[allow(dead_code)]
pub fn release_target(pool: &mut RenderTargetPool, index: usize) {
    if let Some(t) = pool.targets.get_mut(index) {
        t.in_use = false;
    }
}

/// Return the total number of targets in the pool.
#[allow(dead_code)]
pub fn pool_target_count(pool: &RenderTargetPool) -> usize {
    pool.targets.len()
}

/// Return the number of free targets.
#[allow(dead_code)]
pub fn pool_free_count(pool: &RenderTargetPool) -> usize {
    pool.targets.iter().filter(|t| !t.in_use).count()
}

/// Return the number of targets currently in use.
#[allow(dead_code)]
pub fn pool_used_count(pool: &RenderTargetPool) -> usize {
    pool.targets.iter().filter(|t| t.in_use).count()
}

/// Return the dimensions of a target at the given index.
#[allow(dead_code)]
pub fn target_dimensions(pool: &RenderTargetPool, index: usize) -> (u32, u32) {
    pool.targets
        .get(index)
        .map_or((0, 0), |t| (t.width, t.height))
}

/// Clear all targets from the pool.
#[allow(dead_code)]
pub fn pool_clear(pool: &mut RenderTargetPool) {
    pool.targets.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_empty() {
        let p = new_render_target_pool();
        assert_eq!(pool_target_count(&p), 0);
    }

    #[test]
    fn acquire_creates_target() {
        let mut p = new_render_target_pool();
        acquire_target(&mut p, 1920, 1080);
        assert_eq!(pool_target_count(&p), 1);
    }

    #[test]
    fn release_makes_free() {
        let mut p = new_render_target_pool();
        let idx = acquire_target(&mut p, 1920, 1080);
        release_target(&mut p, idx);
        assert_eq!(pool_free_count(&p), 1);
    }

    #[test]
    fn reuse_released_target() {
        let mut p = new_render_target_pool();
        let idx = acquire_target(&mut p, 1920, 1080);
        release_target(&mut p, idx);
        let idx2 = acquire_target(&mut p, 1920, 1080);
        assert_eq!(idx, idx2);
        assert_eq!(pool_target_count(&p), 1);
    }

    #[test]
    fn used_count() {
        let mut p = new_render_target_pool();
        acquire_target(&mut p, 1920, 1080);
        assert_eq!(pool_used_count(&p), 1);
    }

    #[test]
    fn dimensions() {
        let mut p = new_render_target_pool();
        let idx = acquire_target(&mut p, 800, 600);
        let (w, h) = target_dimensions(&p, idx);
        assert_eq!(w, 800);
        assert_eq!(h, 600);
    }

    #[test]
    fn clear_pool() {
        let mut p = new_render_target_pool();
        acquire_target(&mut p, 100, 100);
        pool_clear(&mut p);
        assert_eq!(pool_target_count(&p), 0);
    }

    #[test]
    fn multiple_targets() {
        let mut p = new_render_target_pool();
        acquire_target(&mut p, 100, 100);
        acquire_target(&mut p, 200, 200);
        assert_eq!(pool_target_count(&p), 2);
    }

    #[test]
    fn dimensions_invalid_index() {
        let p = new_render_target_pool();
        let (w, h) = target_dimensions(&p, 999);
        assert_eq!(w, 0);
        assert_eq!(h, 0);
    }

    #[test]
    fn free_after_multiple() {
        let mut p = new_render_target_pool();
        let a = acquire_target(&mut p, 100, 100);
        acquire_target(&mut p, 200, 200);
        release_target(&mut p, a);
        assert_eq!(pool_free_count(&p), 1);
        assert_eq!(pool_used_count(&p), 1);
    }
}
