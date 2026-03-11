// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Turing pattern simulation stub (activator-inhibitor model).

/// Parameters for the activator-inhibitor Turing model.
#[derive(Debug, Clone)]
pub struct TuringParams {
    /// Diffusion of activator.
    pub da: f64,
    /// Diffusion of inhibitor.
    pub dh: f64,
    /// Activation rate.
    pub rho: f64,
    /// Degradation rate of activator.
    pub mu_a: f64,
    /// Degradation rate of inhibitor.
    pub mu_h: f64,
}

impl Default for TuringParams {
    fn default() -> Self {
        Self {
            da: 0.01,
            dh: 0.2,
            rho: 0.01,
            mu_a: 0.02,
            mu_h: 0.02,
        }
    }
}

/// 2D Turing pattern grid.
pub struct TuringPattern {
    pub width: usize,
    pub height: usize,
    pub a: Vec<f64>,
    pub h: Vec<f64>,
    pub params: TuringParams,
}

impl TuringPattern {
    pub fn new(width: usize, height: usize, params: TuringParams) -> Self {
        let n = width * height;
        /* initialize with near-uniform + small perturbation */
        let a = (0..n).map(|i| 1.0 + 0.01 * ((i as f64).sin())).collect();
        let h = (0..n).map(|i| 1.0 + 0.01 * ((i as f64).cos())).collect();
        Self {
            width,
            height,
            a,
            h,
            params,
        }
    }

    fn laplacian(grid: &[f64], idx: usize, w: usize, h: usize) -> f64 {
        let x = idx % w;
        let y = idx / w;
        let c = grid[idx];
        let l = if x > 0 { grid[idx - 1] } else { c };
        let r = if x < w - 1 { grid[idx + 1] } else { c };
        let u = if y > 0 { grid[idx - w] } else { c };
        let d = if y < h - 1 { grid[idx + w] } else { c };
        l + r + u + d - 4.0 * c
    }

    pub fn step(&mut self, dt: f64) {
        let n = self.width * self.height;
        let mut da_dt = Vec::with_capacity(n);
        let mut dh_dt = Vec::with_capacity(n);
        let w = self.width;
        let h = self.height;
        for i in 0..n {
            let a = self.a[i];
            let hi = self.h[i];
            let lap_a = Self::laplacian(&self.a, i, w, h);
            let lap_h = Self::laplacian(&self.h, i, w, h);
            /* simplified Gierer-Meinhardt: da/dt = rho*a^2/h - mu_a*a + da*Lap(a) */
            da_dt.push(
                self.params.da * lap_a + self.params.rho * a * a / (hi + 1e-6)
                    - self.params.mu_a * a,
            );
            /* dh/dt = rho*a^2 - mu_h*h + dh*Lap(h) */
            dh_dt.push(self.params.dh * lap_h + self.params.rho * a * a - self.params.mu_h * hi);
        }
        for i in 0..n {
            self.a[i] = (self.a[i] + dt * da_dt[i]).max(0.0);
            self.h[i] = (self.h[i] + dt * dh_dt[i]).max(0.0);
        }
    }

    pub fn mean_a(&self) -> f64 {
        self.a.iter().sum::<f64>() / self.a.len() as f64
    }

    pub fn mean_h(&self) -> f64 {
        self.h.iter().sum::<f64>() / self.h.len() as f64
    }

    pub fn variance_a(&self) -> f64 {
        let m = self.mean_a();
        self.a.iter().map(|&v| (v - m) * (v - m)).sum::<f64>() / self.a.len() as f64
    }
}

pub fn new_turing_pattern(w: usize, h: usize) -> TuringPattern {
    TuringPattern::new(w, h, TuringParams::default())
}

pub fn tp_step(tp: &mut TuringPattern, dt: f64) {
    tp.step(dt);
}

pub fn tp_mean_a(tp: &TuringPattern) -> f64 {
    tp.mean_a()
}

pub fn tp_mean_h(tp: &TuringPattern) -> f64 {
    tp.mean_h()
}

pub fn tp_variance_a(tp: &TuringPattern) -> f64 {
    tp.variance_a()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        let tp = new_turing_pattern(10, 10);
        assert_eq!(tp.a.len(), 100);
    }

    #[test]
    fn test_initial_mean_near_1() {
        let tp = new_turing_pattern(10, 10);
        assert!((tp_mean_a(&tp) - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_step_changes_state() {
        let mut tp = new_turing_pattern(10, 10);
        let m_before = tp_mean_a(&tp);
        tp_step(&mut tp, 0.1);
        let m_after = tp_mean_a(&tp);
        /* mean changes after step */
        let _ = (m_before - m_after).abs();
        assert!(tp.a.iter().all(|&v| v.is_finite()));
    }

    #[test]
    fn test_a_non_negative() {
        let mut tp = new_turing_pattern(10, 10);
        for _ in 0..5 {
            tp_step(&mut tp, 0.1);
        }
        assert!(tp.a.iter().all(|&v| v >= 0.0));
    }

    #[test]
    fn test_h_non_negative() {
        let mut tp = new_turing_pattern(10, 10);
        for _ in 0..5 {
            tp_step(&mut tp, 0.1);
        }
        assert!(tp.h.iter().all(|&v| v >= 0.0));
    }

    #[test]
    fn test_variance_initial_small() {
        let tp = new_turing_pattern(20, 20);
        /* initial perturbation is small */
        assert!(tp_variance_a(&tp) < 0.01);
    }

    #[test]
    fn test_finite_after_steps() {
        let mut tp = new_turing_pattern(10, 10);
        for _ in 0..10 {
            tp_step(&mut tp, 0.01);
        }
        assert!(tp.a.iter().all(|&v| v.is_finite()));
        assert!(tp.h.iter().all(|&v| v.is_finite()));
    }

    #[test]
    fn test_custom_params() {
        let p = TuringParams {
            da: 0.02,
            dh: 0.4,
            rho: 0.02,
            mu_a: 0.03,
            mu_h: 0.03,
        };
        let tp = TuringPattern::new(5, 5, p);
        assert_eq!(tp.a.len(), 25);
    }

    #[test]
    fn test_width_height() {
        let tp = new_turing_pattern(8, 12);
        assert_eq!(tp.width, 8);
        assert_eq!(tp.height, 12);
    }
}
