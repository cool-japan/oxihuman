// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! 1D advection equation solver using upwind scheme.
//!
//! Solves: du/dt + c * du/dx = 0

/// 1D advection solver.
pub struct Advection1D {
    pub u: Vec<f32>,
    pub velocity: f32,
    pub dx: f32,
    pub time: f32,
    pub length: f32,
}

impl Advection1D {
    /// Create solver with `n` points over [0, length] with advection speed `c`.
    pub fn new(n: usize, length: f32, velocity: f32) -> Self {
        let n = n.max(3);
        Advection1D {
            u: vec![0.0f32; n],
            velocity,
            dx: length / n as f32,
            time: 0.0,
            length,
        }
    }

    /// CFL stable dt: dt = CFL * dx / |c|.
    pub fn stable_dt(&self, cfl: f32) -> f32 {
        cfl * self.dx / self.velocity.abs().max(1e-12)
    }

    /// Advance one step using first-order upwind scheme with periodic BCs.
    #[allow(clippy::needless_range_loop)]
    pub fn step_periodic(&mut self, dt: f32) {
        let n = self.u.len();
        let r = self.velocity * dt / self.dx;
        let mut u_new = self.u.clone();
        for i in 0..n {
            if self.velocity >= 0.0 {
                let im1 = if i == 0 { n - 1 } else { i - 1 };
                u_new[i] = self.u[i] - r * (self.u[i] - self.u[im1]);
            } else {
                let ip1 = (i + 1) % n;
                u_new[i] = self.u[i] - r * (self.u[ip1] - self.u[i]);
            }
        }
        self.u = u_new;
        self.time += dt;
    }

    /// Advance one step with inflow BC (left = 0 for positive velocity).
    #[allow(clippy::needless_range_loop)]
    pub fn step_inflow(&mut self, dt: f32) {
        let n = self.u.len();
        let r = self.velocity * dt / self.dx;
        let mut u_new = self.u.clone();
        if self.velocity >= 0.0 {
            for i in 1..n {
                u_new[i] = self.u[i] - r * (self.u[i] - self.u[i - 1]);
            }
            u_new[0] = 0.0; /* inflow = 0 */
        } else {
            for i in 0..n - 1 {
                u_new[i] = self.u[i] - r * (self.u[i + 1] - self.u[i]);
            }
            u_new[n - 1] = 0.0;
        }
        self.u = u_new;
        self.time += dt;
    }

    /// Advance `steps` using periodic BCs with automatic CFL=0.5 dt.
    pub fn advance_periodic(&mut self, steps: usize) {
        let dt = self.stable_dt(0.5);
        for _ in 0..steps {
            self.step_periodic(dt);
        }
    }

    /// Return L1 norm of the solution.
    pub fn l1_norm(&self) -> f32 {
        self.u.iter().map(|&v| v.abs()).sum::<f32>() * self.dx
    }

    /// Return max value.
    pub fn max_value(&self) -> f32 {
        self.u.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
    }

    /// Return min value.
    pub fn min_value(&self) -> f32 {
        self.u.iter().cloned().fold(f32::INFINITY, f32::min)
    }

    /// Number of grid points.
    pub fn grid_size(&self) -> usize {
        self.u.len()
    }

    /// Set a rectangular pulse initial condition.
    pub fn set_pulse(&mut self, x_lo: f32, x_hi: f32, height: f32) {
        let n = self.u.len();
        for i in 0..n {
            let x = i as f32 * self.dx;
            if x >= x_lo && x <= x_hi {
                self.u[i] = height;
            }
        }
    }
}

/// CFL number for given velocity, dt, dx.
pub fn cfl_number(velocity: f32, dt: f32, dx: f32) -> f32 {
    velocity.abs() * dt / dx
}

/// Lax-Wendroff step (second order in space and time) with periodic BCs.
pub fn lax_wendroff_step(u: &[f32], velocity: f32, dt: f32, dx: f32) -> Vec<f32> {
    let n = u.len();
    let r = velocity * dt / dx;
    let mut u_new = vec![0.0f32; n];
    for i in 0..n {
        let im1 = if i == 0 { n - 1 } else { i - 1 };
        let ip1 = (i + 1) % n;
        u_new[i] =
            u[i] - 0.5 * r * (u[ip1] - u[im1]) + 0.5 * r * r * (u[ip1] - 2.0 * u[i] + u[im1]);
    }
    u_new
}

pub fn new_advection_1d(n: usize, length: f32, velocity: f32) -> Advection1D {
    Advection1D::new(n, length, velocity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construction() {
        let a = new_advection_1d(50, 1.0, 1.0);
        assert_eq!(a.grid_size(), 50);
    }

    #[test]
    fn test_stable_dt_cfl() {
        let a = Advection1D::new(100, 1.0, 2.0);
        let dt = a.stable_dt(0.5);
        assert!(cfl_number(a.velocity, dt, a.dx) <= 0.5 + 1e-5);
    }

    #[test]
    fn test_pulse_advances() {
        let mut a = Advection1D::new(100, 1.0, 1.0);
        a.set_pulse(0.1, 0.2, 1.0);
        let init_max_pos =
            a.u.iter()
                .enumerate()
                .max_by(|x, y| x.1.partial_cmp(y.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();
        a.advance_periodic(20);
        let after_max_pos =
            a.u.iter()
                .enumerate()
                .max_by(|x, y| x.1.partial_cmp(y.1).unwrap())
                .map(|(i, _)| i)
                .unwrap();
        /* Pulse should move to the right */
        assert!(after_max_pos >= init_max_pos || after_max_pos < 5 /* wrapped around */);
    }

    #[test]
    fn test_l1_norm_positive() {
        let mut a = Advection1D::new(50, 1.0, 1.0);
        a.set_pulse(0.2, 0.4, 1.0);
        assert!(a.l1_norm() > 0.0);
    }

    #[test]
    fn test_cfl_number() {
        let cfl = cfl_number(2.0, 0.1, 1.0);
        assert!((cfl - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_lax_wendroff() {
        let u = vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let u_new = lax_wendroff_step(&u, 1.0, 0.1, 1.0);
        assert_eq!(u_new.len(), 8);
    }

    #[test]
    fn test_min_max() {
        let mut a = Advection1D::new(20, 1.0, 1.0);
        a.u[5] = 3.0;
        a.u[10] = -1.0;
        assert!((a.max_value() - 3.0).abs() < 1e-5);
        assert!((a.min_value() - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn test_inflow_step() {
        let mut a = Advection1D::new(20, 1.0, 1.0);
        a.u[5] = 1.0;
        let dt = a.stable_dt(0.5);
        a.step_inflow(dt);
        /* Left boundary should be zero */
        assert!((a.u[0]).abs() < 1e-10);
    }
}
