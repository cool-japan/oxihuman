// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rabin-Karp style rolling hash for sliding window operations.

use std::collections::VecDeque;

pub struct RollingHashNew {
    pub window: VecDeque<u8>,
    pub hash: u64,
    pub base: u64,
    pub modulus: u64,
    pub window_size: usize,
}

pub fn new_rolling_hash_new(window_size: usize) -> RollingHashNew {
    RollingHashNew {
        window: VecDeque::new(),
        hash: 0,
        base: 257,
        modulus: 1_000_000_007,
        window_size,
    }
}

pub fn rh_new_push(r: &mut RollingHashNew, byte: u8) {
    if r.window_size == 0 {
        return;
    }
    if r.window.len() == r.window_size {
        r.window.pop_front();
    }
    r.window.push_back(byte);
    // Recompute hash over current window
    let mut h: u64 = 0;
    for &b in &r.window {
        h = (h.wrapping_mul(r.base).wrapping_add(b as u64)) % r.modulus;
    }
    r.hash = h;
}

pub fn rh_new_hash(r: &RollingHashNew) -> u64 {
    r.hash
}

pub fn rh_new_window_full(r: &RollingHashNew) -> bool {
    r.window.len() == r.window_size
}

pub fn rh_new_window_size(r: &RollingHashNew) -> usize {
    r.window_size
}

pub fn rh_new_reset(r: &mut RollingHashNew) {
    r.window.clear();
    r.hash = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rolling_hash() {
        /* new hash starts empty */
        let r = new_rolling_hash_new(4);
        assert_eq!(rh_new_hash(&r), 0);
        assert!(!rh_new_window_full(&r));
        assert_eq!(rh_new_window_size(&r), 4);
    }

    #[test]
    fn test_push_fills_window() {
        /* pushing window_size bytes fills the window */
        let mut r = new_rolling_hash_new(3);
        rh_new_push(&mut r, b'a');
        rh_new_push(&mut r, b'b');
        rh_new_push(&mut r, b'c');
        assert!(rh_new_window_full(&r));
    }

    #[test]
    fn test_push_beyond_window() {
        /* window size stays fixed after overflow */
        let mut r = new_rolling_hash_new(3);
        for b in b"abcdef" {
            rh_new_push(&mut r, *b);
        }
        assert!(rh_new_window_full(&r));
        assert_eq!(r.window.len(), 3);
    }

    #[test]
    fn test_hash_changes_on_push() {
        /* hash changes as bytes are pushed */
        let mut r = new_rolling_hash_new(4);
        rh_new_push(&mut r, b'x');
        let h1 = rh_new_hash(&r);
        rh_new_push(&mut r, b'y');
        let h2 = rh_new_hash(&r);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_reset_clears_state() {
        /* reset clears window and hash */
        let mut r = new_rolling_hash_new(4);
        rh_new_push(&mut r, b'a');
        rh_new_reset(&mut r);
        assert_eq!(rh_new_hash(&r), 0);
        assert!(!rh_new_window_full(&r));
    }

    #[test]
    fn test_same_sequence_same_hash() {
        /* same byte sequence produces same hash */
        let mut r1 = new_rolling_hash_new(3);
        let mut r2 = new_rolling_hash_new(3);
        for b in b"abc" {
            rh_new_push(&mut r1, *b);
            rh_new_push(&mut r2, *b);
        }
        assert_eq!(rh_new_hash(&r1), rh_new_hash(&r2));
    }

    #[test]
    fn test_window_size_zero() {
        /* zero-size window does nothing */
        let mut r = new_rolling_hash_new(0);
        rh_new_push(&mut r, b'a');
        assert_eq!(rh_new_hash(&r), 0);
    }
}
