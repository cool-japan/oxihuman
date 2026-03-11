#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contact warmstarting for iterative solvers.

use std::collections::HashMap;

/// Stores accumulated lambda values for warm-starting.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ContactWarmstart {
    lambdas: HashMap<u64, f32>,
    hits: u64,
    lookups: u64,
    max_capacity: usize,
}

#[allow(dead_code)]
pub fn new_contact_warmstart(capacity: usize) -> ContactWarmstart {
    ContactWarmstart {
        lambdas: HashMap::with_capacity(capacity),
        hits: 0,
        lookups: 0,
        max_capacity: capacity,
    }
}

#[allow(dead_code)]
pub fn store_contact_lambda(ws: &mut ContactWarmstart, key: u64, lambda: f32) {
    if ws.lambdas.len() < ws.max_capacity || ws.lambdas.contains_key(&key) {
        ws.lambdas.insert(key, lambda);
    }
}

#[allow(dead_code)]
pub fn retrieve_contact_lambda(ws: &mut ContactWarmstart, key: u64) -> f32 {
    ws.lookups += 1;
    if let Some(&v) = ws.lambdas.get(&key) {
        ws.hits += 1;
        v
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn warmstart_count(ws: &ContactWarmstart) -> usize {
    ws.lambdas.len()
}

#[allow(dead_code)]
pub fn clear_warmstart(ws: &mut ContactWarmstart) {
    ws.lambdas.clear();
    ws.hits = 0;
    ws.lookups = 0;
}

#[allow(dead_code)]
pub fn warmstart_hit_rate(ws: &ContactWarmstart) -> f32 {
    if ws.lookups == 0 {
        return 0.0;
    }
    ws.hits as f32 / ws.lookups as f32
}

#[allow(dead_code)]
pub fn warmstart_is_empty(ws: &ContactWarmstart) -> bool {
    ws.lambdas.is_empty()
}

#[allow(dead_code)]
pub fn warmstart_capacity(ws: &ContactWarmstart) -> usize {
    ws.max_capacity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ws = new_contact_warmstart(100);
        assert!(warmstart_is_empty(&ws));
        assert_eq!(warmstart_capacity(&ws), 100);
    }

    #[test]
    fn test_store_retrieve() {
        let mut ws = new_contact_warmstart(100);
        store_contact_lambda(&mut ws, 42, 1.5);
        let v = retrieve_contact_lambda(&mut ws, 42);
        assert!((v - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_retrieve_missing() {
        let mut ws = new_contact_warmstart(100);
        assert_eq!(retrieve_contact_lambda(&mut ws, 99), 0.0);
    }

    #[test]
    fn test_count() {
        let mut ws = new_contact_warmstart(100);
        store_contact_lambda(&mut ws, 1, 0.1);
        store_contact_lambda(&mut ws, 2, 0.2);
        assert_eq!(warmstart_count(&ws), 2);
    }

    #[test]
    fn test_clear() {
        let mut ws = new_contact_warmstart(100);
        store_contact_lambda(&mut ws, 1, 0.1);
        clear_warmstart(&mut ws);
        assert!(warmstart_is_empty(&ws));
    }

    #[test]
    fn test_hit_rate() {
        let mut ws = new_contact_warmstart(100);
        store_contact_lambda(&mut ws, 1, 0.1);
        retrieve_contact_lambda(&mut ws, 1); // hit
        retrieve_contact_lambda(&mut ws, 2); // miss
        assert!((warmstart_hit_rate(&ws) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_hit_rate_zero_lookups() {
        let ws = new_contact_warmstart(100);
        assert_eq!(warmstart_hit_rate(&ws), 0.0);
    }

    #[test]
    fn test_overwrite() {
        let mut ws = new_contact_warmstart(100);
        store_contact_lambda(&mut ws, 1, 0.1);
        store_contact_lambda(&mut ws, 1, 0.9);
        let v = retrieve_contact_lambda(&mut ws, 1);
        assert!((v - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_capacity_limit() {
        let mut ws = new_contact_warmstart(2);
        store_contact_lambda(&mut ws, 1, 0.1);
        store_contact_lambda(&mut ws, 2, 0.2);
        store_contact_lambda(&mut ws, 3, 0.3); // should be ignored
        assert_eq!(warmstart_count(&ws), 2);
    }

    #[test]
    fn test_empty() {
        let ws = new_contact_warmstart(10);
        assert!(warmstart_is_empty(&ws));
    }
}
