// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct XorShift64 {
    pub state: u64,
}

#[allow(dead_code)]
pub fn new_xorshift(seed: u64) -> XorShift64 {
    XorShift64 { state: if seed == 0 { 1 } else { seed } }
}

#[allow(dead_code)]
pub fn xr_next_u64(rng: &mut XorShift64) -> u64 {
    let mut x = rng.state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    rng.state = x;
    x
}

#[allow(dead_code)]
pub fn xr_next_f64(rng: &mut XorShift64) -> f64 {
    let u = xr_next_u64(rng);
    (u as f64) / (u64::MAX as f64 + 1.0)
}

#[allow(dead_code)]
pub fn xr_next_range(rng: &mut XorShift64, lo: i64, hi: i64) -> i64 {
    if hi <= lo {
        return lo;
    }
    let range = (hi - lo) as u64;
    let u = xr_next_u64(rng);
    lo + (u % range) as i64
}

#[allow(dead_code)]
pub fn xr_shuffle(rng: &mut XorShift64, v: &mut [u64]) {
    let n = v.len();
    #[allow(clippy::needless_range_loop)]
    for i in (1..n).rev() {
        let j = (xr_next_u64(rng) % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}

#[allow(dead_code)]
pub fn xr_fill(rng: &mut XorShift64, buf: &mut [u64]) {
    for item in buf.iter_mut() {
        *item = xr_next_u64(rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_u64_advances() {
        let mut rng = new_xorshift(12345);
        let first = xr_next_u64(&mut rng);
        assert_ne!(first, 12345);
    }

    #[test]
    fn test_next_f64_in_range() {
        let mut rng = new_xorshift(42);
        for _ in 0..100 {
            let f = xr_next_f64(&mut rng);
            assert!((0.0..1.0).contains(&f));
        }
    }

    #[test]
    fn test_next_range() {
        let mut rng = new_xorshift(999);
        for _ in 0..50 {
            let v = xr_next_range(&mut rng, 10, 20);
            assert!((10..20).contains(&v));
        }
    }

    #[test]
    fn test_shuffle_changes_order() {
        let mut rng = new_xorshift(7);
        let mut v: Vec<u64> = (0..10).collect();
        xr_shuffle(&mut rng, &mut v);
        let is_sorted = v.windows(2).all(|w| w[0] <= w[1]);
        assert!(!is_sorted || v.len() <= 1);
    }

    #[test]
    fn test_fill() {
        let mut rng = new_xorshift(1);
        let mut buf = [0u64; 8];
        xr_fill(&mut rng, &mut buf);
        assert!(buf.iter().any(|&x| x != 0));
    }

    #[test]
    fn test_zero_seed_handled() {
        let mut rng = new_xorshift(0);
        let v = xr_next_u64(&mut rng);
        assert_ne!(v, 0);
    }

    #[test]
    fn test_deterministic() {
        let mut rng1 = new_xorshift(123);
        let mut rng2 = new_xorshift(123);
        let v1 = xr_next_u64(&mut rng1);
        let v2 = xr_next_u64(&mut rng2);
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_next_range_lo_eq_hi() {
        let mut rng = new_xorshift(1);
        let v = xr_next_range(&mut rng, 5, 5);
        assert_eq!(v, 5);
    }
}
