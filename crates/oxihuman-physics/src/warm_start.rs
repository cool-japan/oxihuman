#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Warm starting for iterative constraint solvers.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WarmCache {
    pub lambda_n: f32,
    pub lambda_t: [f32; 2],
    pub age: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct WarmStartData {
    pub caches: Vec<WarmCache>,
}

#[allow(dead_code)]
pub fn new_warm_start_data() -> WarmStartData {
    WarmStartData { caches: Vec::new() }
}

#[allow(dead_code)]
pub fn add_warm_cache(ws: &mut WarmStartData, ln: f32, lt: [f32; 2]) {
    ws.caches.push(WarmCache {
        lambda_n: ln,
        lambda_t: lt,
        age: 0,
    });
}

#[allow(dead_code)]
pub fn get_warm_lambda(ws: &WarmStartData, idx: usize) -> Option<f32> {
    ws.caches.get(idx).map(|c| c.lambda_n)
}

#[allow(dead_code)]
pub fn age_caches(ws: &mut WarmStartData) {
    for c in &mut ws.caches {
        c.age += 1;
    }
}

#[allow(dead_code)]
pub fn evict_old_caches(ws: &mut WarmStartData, max_age: u32) {
    ws.caches.retain(|c| c.age <= max_age);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let ws = new_warm_start_data();
        assert!(ws.caches.is_empty());
    }

    #[test]
    fn add_cache() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 1.5, [0.1, -0.1]);
        assert_eq!(ws.caches.len(), 1);
        assert!((ws.caches[0].lambda_n - 1.5).abs() < 1e-6);
    }

    #[test]
    fn get_warm_lambda_valid() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 3.0, [0.0, 0.0]);
        assert_eq!(get_warm_lambda(&ws, 0), Some(3.0));
    }

    #[test]
    fn get_warm_lambda_out_of_bounds() {
        let ws = new_warm_start_data();
        assert!(get_warm_lambda(&ws, 0).is_none());
    }

    #[test]
    fn age_caches_increments() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 1.0, [0.0, 0.0]);
        age_caches(&mut ws);
        assert_eq!(ws.caches[0].age, 1);
    }

    #[test]
    fn age_twice() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 1.0, [0.0, 0.0]);
        age_caches(&mut ws);
        age_caches(&mut ws);
        assert_eq!(ws.caches[0].age, 2);
    }

    #[test]
    fn evict_old_removes_stale() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 1.0, [0.0, 0.0]);
        age_caches(&mut ws);
        age_caches(&mut ws);
        evict_old_caches(&mut ws, 1);
        assert!(ws.caches.is_empty());
    }

    #[test]
    fn evict_keeps_young() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 1.0, [0.0, 0.0]);
        age_caches(&mut ws);
        evict_old_caches(&mut ws, 2);
        assert_eq!(ws.caches.len(), 1);
    }

    #[test]
    fn lambda_t_stored() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 0.0, [0.5, -0.5]);
        assert_eq!(ws.caches[0].lambda_t, [0.5, -0.5]);
    }

    #[test]
    fn multiple_caches() {
        let mut ws = new_warm_start_data();
        add_warm_cache(&mut ws, 1.0, [0.0, 0.0]);
        add_warm_cache(&mut ws, 2.0, [0.0, 0.0]);
        add_warm_cache(&mut ws, 3.0, [0.0, 0.0]);
        assert_eq!(ws.caches.len(), 3);
        assert_eq!(get_warm_lambda(&ws, 2), Some(3.0));
    }
}
