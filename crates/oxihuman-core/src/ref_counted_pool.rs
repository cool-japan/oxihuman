#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// A pool of reference-counted items.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RefCountedPool {
    items: Vec<String>,
    ref_counts: Vec<u32>,
}

#[allow(dead_code)]
pub fn new_rc_pool() -> RefCountedPool {
    RefCountedPool {
        items: Vec::new(),
        ref_counts: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn rc_pool_alloc(pool: &mut RefCountedPool, item: &str) -> usize {
    let idx = pool.items.len();
    pool.items.push(item.to_string());
    pool.ref_counts.push(1);
    idx
}

#[allow(dead_code)]
pub fn rc_pool_retain(pool: &mut RefCountedPool, idx: usize) -> bool {
    if idx < pool.ref_counts.len() && pool.ref_counts[idx] > 0 {
        pool.ref_counts[idx] += 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn rc_pool_release(pool: &mut RefCountedPool, idx: usize) -> bool {
    if idx < pool.ref_counts.len() && pool.ref_counts[idx] > 0 {
        pool.ref_counts[idx] -= 1;
        true
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn rc_pool_ref_count(pool: &RefCountedPool, idx: usize) -> u32 {
    if idx < pool.ref_counts.len() {
        pool.ref_counts[idx]
    } else {
        0
    }
}

#[allow(dead_code)]
pub fn rc_pool_count(pool: &RefCountedPool) -> usize {
    pool.ref_counts.iter().filter(|&&c| c > 0).count()
}

#[allow(dead_code)]
pub fn rc_pool_clear(pool: &mut RefCountedPool) {
    pool.items.clear();
    pool.ref_counts.clear();
}

#[allow(dead_code)]
pub fn rc_pool_is_empty(pool: &RefCountedPool) -> bool {
    pool.ref_counts.iter().all(|&c| c == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rc_pool() {
        let p = new_rc_pool();
        assert!(rc_pool_is_empty(&p));
    }

    #[test]
    fn test_alloc() {
        let mut p = new_rc_pool();
        let idx = rc_pool_alloc(&mut p, "item");
        assert_eq!(idx, 0);
        assert_eq!(rc_pool_ref_count(&p, idx), 1);
    }

    #[test]
    fn test_retain() {
        let mut p = new_rc_pool();
        let idx = rc_pool_alloc(&mut p, "item");
        assert!(rc_pool_retain(&mut p, idx));
        assert_eq!(rc_pool_ref_count(&p, idx), 2);
    }

    #[test]
    fn test_release() {
        let mut p = new_rc_pool();
        let idx = rc_pool_alloc(&mut p, "item");
        assert!(rc_pool_release(&mut p, idx));
        assert_eq!(rc_pool_ref_count(&p, idx), 0);
    }

    #[test]
    fn test_count() {
        let mut p = new_rc_pool();
        rc_pool_alloc(&mut p, "a");
        rc_pool_alloc(&mut p, "b");
        assert_eq!(rc_pool_count(&p), 2);
    }

    #[test]
    fn test_clear() {
        let mut p = new_rc_pool();
        rc_pool_alloc(&mut p, "a");
        rc_pool_clear(&mut p);
        assert_eq!(rc_pool_count(&p), 0);
    }

    #[test]
    fn test_is_empty() {
        let mut p = new_rc_pool();
        let idx = rc_pool_alloc(&mut p, "item");
        assert!(!rc_pool_is_empty(&p));
        rc_pool_release(&mut p, idx);
        assert!(rc_pool_is_empty(&p));
    }

    #[test]
    fn test_retain_invalid() {
        let mut p = new_rc_pool();
        assert!(!rc_pool_retain(&mut p, 999));
    }

    #[test]
    fn test_release_invalid() {
        let mut p = new_rc_pool();
        assert!(!rc_pool_release(&mut p, 999));
    }

    #[test]
    fn test_ref_count_invalid() {
        let p = new_rc_pool();
        assert_eq!(rc_pool_ref_count(&p, 999), 0);
    }
}
