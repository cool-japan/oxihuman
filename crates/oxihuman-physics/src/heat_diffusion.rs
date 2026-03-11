// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Discrete heat diffusion on a 1D mesh using explicit finite differences.

/// A 1D heat diffusion solver.
#[derive(Debug, Clone)]
pub struct HeatDiffusion1D {
    /// Temperature at each node.
    pub temp: Vec<f64>,
    /// Thermal diffusivity (m²/s).
    pub alpha: f64,
    /// Spatial step (m).
    pub dx: f64,
    /// Time step (s).
    pub dt: f64,
    /// Current time (s).
    pub time: f64,
}

impl HeatDiffusion1D {
    /// Create a new heat diffusion solver (all temps at `init_temp`).
    pub fn new(n: usize, init_temp: f64, alpha: f64, dx: f64, dt: f64) -> Self {
        HeatDiffusion1D {
            temp: vec![init_temp; n],
            alpha,
            dx,
            dt,
            time: 0.0,
        }
    }

    /// Fourier number (stability criterion: should be ≤ 0.5).
    pub fn fourier(&self) -> f64 {
        self.alpha * self.dt / (self.dx * self.dx)
    }

    /// Set temperature at node `i`.
    pub fn set_temp(&mut self, i: usize, t: f64) {
        self.temp[i] = t;
    }

    /// Step the diffusion equation one time step (explicit method).
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self) {
        let n = self.temp.len();
        let fo = self.fourier();
        let mut next = self.temp.clone();
        for i in 1..n.saturating_sub(1) {
            next[i] = self.temp[i]
                + fo * (self.temp[i + 1] - 2.0 * self.temp[i] + self.temp[i - 1]);
        }
        /* Neumann (insulating) boundary conditions */
        next[0] = self.temp[0] + fo * (self.temp[1] - self.temp[0]);
        next[n - 1] = self.temp[n - 1] + fo * (self.temp[n - 2] - self.temp[n - 1]);
        self.temp = next;
        self.time += self.dt;
    }

    /// Mean temperature.
    pub fn mean_temp(&self) -> f64 {
        self.temp.iter().sum::<f64>() / self.temp.len() as f64
    }

    /// Max temperature.
    pub fn max_temp(&self) -> f64 {
        self.temp.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Min temperature.
    pub fn min_temp(&self) -> f64 {
        self.temp.iter().cloned().fold(f64::INFINITY, f64::min)
    }

    /// True if no nodes.
    pub fn is_empty(&self) -> bool {
        self.temp.is_empty()
    }

    /// Number of nodes.
    pub fn len(&self) -> usize {
        self.temp.len()
    }
}

/// Create a new heat diffusion solver.
pub fn new_heat_diffusion(n: usize, init_temp: f64, alpha: f64, dx: f64, dt: f64) -> HeatDiffusion1D {
    HeatDiffusion1D::new(n, init_temp, alpha, dx, dt)
}

/// Step.
pub fn hd_step(s: &mut HeatDiffusion1D) {
    s.step();
}

/// Fourier number.
pub fn hd_fourier(s: &HeatDiffusion1D) -> f64 {
    s.fourier()
}

/// Mean temperature.
pub fn hd_mean_temp(s: &HeatDiffusion1D) -> f64 {
    s.mean_temp()
}

/// Max temperature.
pub fn hd_max_temp(s: &HeatDiffusion1D) -> f64 {
    s.max_temp()
}

/// Set temperature.
pub fn hd_set_temp(s: &mut HeatDiffusion1D, i: usize, t: f64) {
    s.set_temp(i, t);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_uniform_temperature() {
        let s = new_heat_diffusion(10, 25.0, 1e-4, 0.01, 0.001);
        assert!((hd_mean_temp(&s) - 25.0).abs() < 1e-9 /* uniform 25°C */);
    }

    #[test]
    fn test_stable_fourier() {
        let s = new_heat_diffusion(50, 0.0, 1e-4, 0.01, 0.1);
        assert!(hd_fourier(&s) <= 0.5 /* stable criterion */);
    }

    #[test]
    fn test_step_advances_time() {
        let mut s = new_heat_diffusion(10, 0.0, 1e-4, 0.01, 0.001);
        hd_step(&mut s);
        assert!((s.time - 0.001).abs() < 1e-12 /* time advanced */);
    }

    #[test]
    fn test_set_temp() {
        let mut s = new_heat_diffusion(10, 0.0, 1e-4, 0.01, 0.001);
        hd_set_temp(&mut s, 5, 100.0);
        assert!((hd_max_temp(&s) - 100.0).abs() < 1e-9 /* hot spot */);
    }

    #[test]
    fn test_heat_diffuses_outward() {
        let mut s = new_heat_diffusion(10, 0.0, 1e-4, 0.01, 0.05);
        hd_set_temp(&mut s, 5, 100.0);
        let before = s.temp[4];
        hd_step(&mut s);
        assert!(s.temp[4] > before /* heat spread to neighbor */);
    }

    #[test]
    fn test_mean_conserved_interior() {
        /* with Neumann BC, total heat should be conserved */
        let mut s = new_heat_diffusion(20, 10.0, 1e-4, 0.01, 0.05);
        let mean_before = hd_mean_temp(&s);
        hd_step(&mut s);
        let mean_after = hd_mean_temp(&s);
        assert!((mean_before - mean_after).abs() < 1e-3 /* heat conserved */);
    }

    #[test]
    fn test_no_nan_after_steps() {
        let mut s = new_heat_diffusion(20, 0.0, 1e-4, 0.01, 0.05);
        hd_set_temp(&mut s, 10, 200.0);
        for _ in 0..100 {
            hd_step(&mut s);
        }
        assert!(s.temp.iter().all(|&v| !v.is_nan()) /* no NaN */);
    }

    #[test]
    fn test_len() {
        let s = new_heat_diffusion(30, 0.0, 1e-4, 0.01, 0.05);
        assert_eq!(s.len(), 30 /* 30 nodes */);
    }

    #[test]
    fn test_min_temp_initially_uniform() {
        let s = new_heat_diffusion(10, 5.0, 1e-4, 0.01, 0.001);
        assert!((s.min_temp() - 5.0).abs() < 1e-9 /* uniform min */);
    }

    #[test]
    fn test_max_temp_decreases_over_time() {
        let mut s = new_heat_diffusion(20, 0.0, 1e-4, 0.01, 0.05);
        hd_set_temp(&mut s, 10, 100.0);
        let max0 = hd_max_temp(&s);
        for _ in 0..10 {
            hd_step(&mut s);
        }
        assert!(hd_max_temp(&s) < max0 /* peak decreases as heat spreads */);
    }
}
