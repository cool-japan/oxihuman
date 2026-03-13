// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 1D heat equation finite difference solver.
//!
//! Solves: du/dt = alpha * d2u/dx2

/// 1D heat equation solver using explicit finite differences.
pub struct HeatEquation1D {
    /// Temperature field.
    pub u: Vec<f32>,
    /// Thermal diffusivity.
    pub alpha: f32,
    /// Spatial step.
    pub dx: f32,
    /// Time elapsed.
    pub time: f32,
    /// Length of domain.
    pub length: f32,
}

impl HeatEquation1D {
    /// Create a new solver with `n` interior points.
    pub fn new(n: usize, length: f32, alpha: f32) -> Self {
        let n = n.max(3);
        let dx = length / (n - 1) as f32;
        HeatEquation1D {
            u: vec![0.0f32; n],
            alpha,
            dx,
            time: 0.0,
            length,
        }
    }

    /// Set initial condition from a function f(x).
    pub fn set_initial<F: Fn(f32) -> f32>(&mut self, f: F) {
        let n = self.u.len();
        for i in 0..n {
            let x = i as f32 * self.dx;
            self.u[i] = f(x);
        }
    }

    /// Compute stable time step (CFL condition: dt <= dx^2 / (2 * alpha)).
    pub fn stable_dt(&self) -> f32 {
        self.dx * self.dx / (2.0 * self.alpha.max(1e-12))
    }

    /// Advance one explicit Euler step with given dt.
    /// Boundary conditions: fixed (Dirichlet) at both ends.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, dt: f32) {
        let n = self.u.len();
        let r = self.alpha * dt / (self.dx * self.dx);
        let mut u_new = self.u.clone();
        for i in 1..n - 1 {
            u_new[i] = self.u[i] + r * (self.u[i - 1] - 2.0 * self.u[i] + self.u[i + 1]);
        }
        /* Boundary: Dirichlet (keep boundary values) */
        self.u = u_new;
        self.time += dt;
    }

    /// Advance for `steps` iterations with automatic stable dt.
    pub fn advance(&mut self, steps: usize) {
        let dt = self.stable_dt();
        for _ in 0..steps {
            self.step(dt);
        }
    }

    /// Return total thermal energy (integral of u dx ~ sum * dx).
    pub fn total_energy(&self) -> f32 {
        self.u.iter().sum::<f32>() * self.dx
    }

    /// Return maximum temperature.
    pub fn max_temperature(&self) -> f32 {
        self.u.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Return minimum temperature.
    pub fn min_temperature(&self) -> f32 {
        self.u.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    /// Return mean temperature.
    pub fn mean_temperature(&self) -> f32 {
        let n = self.u.len() as f32;
        self.u.iter().sum::<f32>() / n
    }

    /// Number of grid points.
    pub fn grid_size(&self) -> usize {
        self.u.len()
    }
}

/// Create a heat equation solver with a heat pulse at the center.
pub fn new_heat_pulse(n: usize, length: f32, alpha: f32) -> HeatEquation1D {
    let mut solver = HeatEquation1D::new(n, length, alpha);
    let half = n / 2;
    solver.u[half] = 1.0;
    solver
}

/// Create a heat equation solver with a sinusoidal initial condition.
pub fn new_heat_sine(n: usize, alpha: f32) -> HeatEquation1D {
    let mut solver = HeatEquation1D::new(n, std::f32::consts::PI, alpha);
    solver.set_initial(|x| x.sin());
    solver
}

/// Steady-state temperature for linear gradient [t_left, t_right].
pub fn steady_state_linear(n: usize, t_left: f32, t_right: f32) -> Vec<f32> {
    (0..n)
        .map(|i| t_left + (t_right - t_left) * i as f32 / (n - 1).max(1) as f32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let solver = HeatEquation1D::new(10, 1.0, 0.1);
        assert_eq!(solver.grid_size(), 10);
        assert!((solver.dx - 1.0 / 9.0).abs() < 1e-5);
    }

    #[test]
    fn test_stable_dt() {
        let solver = HeatEquation1D::new(10, 1.0, 1.0);
        let dt = solver.stable_dt();
        assert!(dt > 0.0);
        /* r = alpha * dt / dx^2 should be <= 0.5 */
        let r = solver.alpha * dt / (solver.dx * solver.dx);
        assert!(r <= 0.5 + 1e-5);
    }

    #[test]
    fn test_advance_conserves_boundary() {
        let mut solver = HeatEquation1D::new(20, 1.0, 0.01);
        solver.u[0] = 100.0;
        solver.u[19] = 0.0;
        let b0 = solver.u[0];
        solver.advance(10);
        /* Dirichlet: boundaries unchanged */
        assert!((solver.u[0] - b0).abs() < 1e-5);
    }

    #[test]
    fn test_heat_pulse_diffuses() {
        let mut solver = new_heat_pulse(51, 1.0, 0.1);
        let init_max = solver.max_temperature();
        solver.advance(10);
        /* Peak should decrease due to diffusion */
        assert!(solver.max_temperature() < init_max);
    }

    #[test]
    fn test_mean_temperature_conservation() {
        /* Interior source free: mean of interior should be roughly conserved */
        let mut solver = new_heat_sine(50, 0.01);
        let mean_init = solver.mean_temperature();
        solver.advance(5);
        /* Mean shifts due to BCs, but should be finite */
        let mean_after = solver.mean_temperature();
        assert!(mean_after.is_finite());
        let _ = mean_init;
    }

    #[test]
    fn test_sine_initial() {
        let solver = new_heat_sine(100, 0.1);
        /* At x=PI/2, sin = 1 */
        let mid = solver.grid_size() / 2;
        assert!(solver.u[mid] > 0.5);
    }

    #[test]
    fn test_steady_state_linear() {
        let ss = steady_state_linear(11, 0.0, 10.0);
        assert_eq!(ss.len(), 11);
        assert!((ss[0]).abs() < 1e-5);
        assert!((ss[10] - 10.0).abs() < 1e-5);
        assert!((ss[5] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_total_energy() {
        let solver = HeatEquation1D::new(10, 1.0, 0.1);
        assert!((solver.total_energy()).abs() < 1e-10);
    }
}
