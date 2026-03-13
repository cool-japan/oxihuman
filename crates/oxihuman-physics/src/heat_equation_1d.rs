// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Heat1d {
    pub u: Vec<f32>,
    pub alpha: f32,
    pub dx: f32,
    pub n: usize,
}

pub fn new_heat_1d(n: usize, alpha: f32, dx: f32) -> Heat1d {
    Heat1d {
        u: vec![0.0; n],
        alpha,
        dx,
        n,
    }
}

#[allow(clippy::needless_range_loop)]
pub fn heat_1d_step(h: &mut Heat1d, dt: f32) {
    let r = h.alpha * dt / (h.dx * h.dx);
    let mut u_next = h.u.clone();
    let n = h.n;
    for i in 1..n - 1 {
        u_next[i] = h.u[i] + r * (h.u[i + 1] - 2.0 * h.u[i] + h.u[i - 1]);
    }
    h.u = u_next;
}

pub fn heat_1d_get(h: &Heat1d, i: usize) -> f32 {
    h.u[i]
}

pub fn heat_1d_set(h: &mut Heat1d, i: usize, val: f32) {
    h.u[i] = val;
}

pub fn heat_1d_mean(h: &Heat1d) -> f32 {
    if h.u.is_empty() {
        return 0.0;
    }
    h.u.iter().sum::<f32>() / h.u.len() as f32
}

pub fn heat_1d_max(h: &Heat1d) -> f32 {
    h.u.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_heat_1d() {
        /* initialized to zeros */
        let h = new_heat_1d(10, 0.1, 0.5);
        assert_eq!(h.n, 10);
        assert_eq!(heat_1d_get(&h, 5), 0.0);
    }

    #[test]
    fn test_heat_1d_set_get() {
        /* set and get values */
        let mut h = new_heat_1d(10, 0.1, 0.5);
        heat_1d_set(&mut h, 4, 100.0);
        assert!((heat_1d_get(&h, 4) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_heat_1d_mean() {
        /* mean computed correctly */
        let mut h = new_heat_1d(4, 0.1, 0.1);
        heat_1d_set(&mut h, 0, 4.0);
        heat_1d_set(&mut h, 1, 8.0);
        heat_1d_set(&mut h, 2, 0.0);
        heat_1d_set(&mut h, 3, 0.0);
        assert!((heat_1d_mean(&h) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_heat_1d_max() {
        /* max returns largest */
        let mut h = new_heat_1d(5, 0.1, 0.1);
        heat_1d_set(&mut h, 2, 50.0);
        assert!((heat_1d_max(&h) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_heat_1d_step_diffuses() {
        /* heat diffuses to neighbors */
        let mut h = new_heat_1d(10, 0.1, 1.0);
        heat_1d_set(&mut h, 5, 100.0);
        /* stability: dt <= dx^2 / (2*alpha) = 1/(0.2) = 5; use dt=1 */
        heat_1d_step(&mut h, 1.0);
        /* neighbors should now be warmer */
        assert!(heat_1d_get(&h, 4) > 0.0);
        assert!(heat_1d_get(&h, 6) > 0.0);
        /* center should be cooler */
        assert!(heat_1d_get(&h, 5) < 100.0);
    }

    #[test]
    fn test_heat_1d_boundaries_unchanged() {
        /* endpoints unchanged (no flux) */
        let mut h = new_heat_1d(8, 0.1, 1.0);
        heat_1d_set(&mut h, 4, 10.0);
        heat_1d_step(&mut h, 1.0);
        assert_eq!(heat_1d_get(&h, 0), 0.0);
        assert_eq!(heat_1d_get(&h, 7), 0.0);
    }

    #[test]
    fn test_heat_1d_step_no_panic() {
        /* step runs without panic */
        let mut h = new_heat_1d(5, 0.1, 0.5);
        heat_1d_set(&mut h, 2, 1.0);
        heat_1d_step(&mut h, 0.1);
        assert!(heat_1d_get(&h, 2).is_finite());
    }
}
