// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct Wave1d {
    pub u: Vec<f32>,
    pub u_prev: Vec<f32>,
    pub c: f32,
    pub dx: f32,
    pub n: usize,
}

pub fn new_wave_1d(n: usize, c: f32, dx: f32) -> Wave1d {
    Wave1d {
        u: vec![0.0; n],
        u_prev: vec![0.0; n],
        c,
        dx,
        n,
    }
}

#[allow(clippy::needless_range_loop)]
pub fn wave_1d_step(w: &mut Wave1d, dt: f32) {
    let r = (w.c * dt / w.dx).powi(2);
    let mut u_next = w.u.clone();
    let n = w.n;
    for i in 1..n - 1 {
        u_next[i] = 2.0 * w.u[i] - w.u_prev[i] + r * (w.u[i + 1] - 2.0 * w.u[i] + w.u[i - 1]);
    }
    /* Dirichlet boundary conditions (fixed ends) */
    u_next[0] = 0.0;
    u_next[n - 1] = 0.0;
    w.u_prev = w.u.clone();
    w.u = u_next;
}

pub fn wave_1d_get(w: &Wave1d, i: usize) -> f32 {
    w.u[i]
}

pub fn wave_1d_set(w: &mut Wave1d, i: usize, val: f32) {
    w.u[i] = val;
    w.u_prev[i] = val;
}

pub fn wave_1d_energy(w: &Wave1d) -> f32 {
    w.u.iter().map(|&v| v * v).sum()
}

pub fn wave_1d_max(w: &Wave1d) -> f32 {
    w.u.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wave_1d() {
        /* initialized to zeros */
        let w = new_wave_1d(10, 1.0, 0.1);
        assert_eq!(w.n, 10);
        assert_eq!(wave_1d_get(&w, 5), 0.0);
    }

    #[test]
    fn test_wave_1d_set_get() {
        /* set and get values */
        let mut w = new_wave_1d(10, 1.0, 0.1);
        wave_1d_set(&mut w, 5, 1.0);
        assert!((wave_1d_get(&w, 5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_wave_1d_energy() {
        /* energy is sum of squares */
        let mut w = new_wave_1d(10, 1.0, 0.1);
        wave_1d_set(&mut w, 3, 2.0);
        wave_1d_set(&mut w, 4, 1.0);
        let e = wave_1d_energy(&w);
        assert!((e - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_wave_1d_max() {
        /* max returns largest value */
        let mut w = new_wave_1d(10, 1.0, 0.1);
        wave_1d_set(&mut w, 5, 3.0);
        wave_1d_set(&mut w, 7, 1.0);
        assert!((wave_1d_max(&w) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_wave_1d_step_boundary() {
        /* boundaries remain zero after step */
        let mut w = new_wave_1d(10, 0.5, 0.1);
        wave_1d_set(&mut w, 5, 1.0);
        wave_1d_step(&mut w, 0.1);
        assert_eq!(wave_1d_get(&w, 0), 0.0);
        assert_eq!(wave_1d_get(&w, 9), 0.0);
    }

    #[test]
    fn test_wave_1d_step_propagates() {
        /* disturbance propagates to neighbors */
        let mut w = new_wave_1d(20, 1.0, 1.0);
        wave_1d_set(&mut w, 10, 1.0);
        wave_1d_step(&mut w, 0.5);
        /* neighbors should be nonzero now */
        let left = wave_1d_get(&w, 9);
        let right = wave_1d_get(&w, 11);
        assert!(left.abs() > 0.0 || right.abs() > 0.0);
    }

    #[test]
    fn test_wave_1d_step_no_panic() {
        /* step runs without panic on minimal grid */
        let mut w = new_wave_1d(5, 1.0, 0.5);
        wave_1d_set(&mut w, 2, 0.5);
        wave_1d_step(&mut w, 0.4);
        assert!(wave_1d_get(&w, 2).is_finite());
    }
}
