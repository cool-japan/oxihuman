// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 1D wave equation solver using finite differences.
//!
//! u_tt = c² * u_xx  with Dirichlet boundary conditions.

/// A 1D wave equation solver.
#[derive(Debug, Clone)]
pub struct WaveSolver1D {
    /// Grid points.
    pub u: Vec<f64>,
    /// Previous time step.
    pub u_prev: Vec<f64>,
    /// Wave speed (m/s).
    pub wave_speed: f64,
    /// Grid spacing (m).
    pub dx: f64,
    /// Time step (s).
    pub dt: f64,
    /// Current time (s).
    pub time: f64,
}

impl WaveSolver1D {
    /// Create a new wave solver with `n` grid points.
    pub fn new(n: usize, wave_speed: f64, dx: f64, dt: f64) -> Self {
        WaveSolver1D {
            u: vec![0.0; n],
            u_prev: vec![0.0; n],
            wave_speed,
            dx,
            dt,
            time: 0.0,
        }
    }

    /// CFL number (should be ≤ 1 for stability).
    pub fn cfl(&self) -> f64 {
        self.wave_speed * self.dt / self.dx
    }

    /// Set initial displacement profile.
    pub fn set_initial(&mut self, profile: &[f64]) {
        let n = self.u.len().min(profile.len());
        self.u[..n].copy_from_slice(&profile[..n]);
        self.u_prev.copy_from_slice(&self.u);
    }

    /// Apply a point excitation at grid index `i`.
    pub fn excite(&mut self, i: usize, amplitude: f64) {
        if i < self.u.len() {
            self.u[i] += amplitude;
        }
    }

    /// Step the wave equation one time step forward.
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self) {
        let n = self.u.len();
        let r = self.cfl();
        let r2 = r * r;
        let mut u_next = vec![0.0f64; n];
        /* Dirichlet boundaries: u_next[0] = u_next[n-1] = 0 */
        for i in 1..n.saturating_sub(1) {
            u_next[i] = 2.0 * self.u[i] - self.u_prev[i]
                + r2 * (self.u[i + 1] - 2.0 * self.u[i] + self.u[i - 1]);
        }
        self.u_prev.copy_from_slice(&self.u);
        self.u.copy_from_slice(&u_next);
        self.time += self.dt;
    }

    /// Total energy (sum of u²).
    pub fn energy(&self) -> f64 {
        self.u.iter().map(|&v| v * v).sum()
    }

    /// Maximum absolute displacement.
    pub fn max_displacement(&self) -> f64 {
        self.u.iter().map(|&v| v.abs()).fold(0.0_f64, f64::max)
    }

    /// True if the grid has no points.
    pub fn is_empty(&self) -> bool {
        self.u.is_empty()
    }

    /// Grid size.
    pub fn len(&self) -> usize {
        self.u.len()
    }
}

/// Create a new wave solver.
pub fn new_wave_solver_1d(n: usize, wave_speed: f64, dx: f64, dt: f64) -> WaveSolver1D {
    WaveSolver1D::new(n, wave_speed, dx, dt)
}

/// Step the solver.
pub fn ws_step(s: &mut WaveSolver1D) {
    s.step();
}

/// Energy.
pub fn ws_energy(s: &WaveSolver1D) -> f64 {
    s.energy()
}

/// CFL number.
pub fn ws_cfl(s: &WaveSolver1D) -> f64 {
    s.cfl()
}

/// Max displacement.
pub fn ws_max_displacement(s: &WaveSolver1D) -> f64 {
    s.max_displacement()
}

/// Grid size.
pub fn ws_len(s: &WaveSolver1D) -> usize {
    s.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfl() {
        let s = new_wave_solver_1d(100, 1.0, 0.01, 0.005);
        assert!((ws_cfl(&s) - 0.5).abs() < 1e-9 /* CFL = c*dt/dx = 0.5 */);
    }

    #[test]
    fn test_initial_energy_zero() {
        let s = new_wave_solver_1d(50, 1.0, 1.0, 0.5);
        assert_eq!(ws_energy(&s), 0.0 /* starts at rest */);
    }

    #[test]
    fn test_excite_changes_energy() {
        let mut s = new_wave_solver_1d(50, 1.0, 1.0, 0.5);
        s.excite(25, 1.0);
        assert!(ws_energy(&s) > 0.0 /* energy added */);
    }

    #[test]
    fn test_step_advances_time() {
        let mut s = new_wave_solver_1d(50, 1.0, 0.1, 0.05);
        ws_step(&mut s);
        assert!((s.time - 0.05).abs() < 1e-12 /* time advanced */);
    }

    #[test]
    fn test_boundaries_fixed() {
        let mut s = new_wave_solver_1d(50, 1.0, 0.1, 0.05);
        s.excite(25, 1.0);
        for _ in 0..20 {
            ws_step(&mut s);
        }
        assert_eq!(s.u[0], 0.0 /* left boundary */);
        assert_eq!(s.u[49], 0.0 /* right boundary */);
    }

    #[test]
    fn test_len() {
        let s = new_wave_solver_1d(80, 1.0, 1.0, 0.5);
        assert_eq!(ws_len(&s), 80 /* 80 grid points */);
    }

    #[test]
    fn test_max_displacement() {
        let mut s = new_wave_solver_1d(50, 1.0, 0.1, 0.05);
        s.excite(25, 3.0);
        assert!((ws_max_displacement(&s) - 3.0).abs() < 1e-9 /* peak at excited point */);
    }

    #[test]
    fn test_set_initial_profile() {
        let mut s = new_wave_solver_1d(10, 1.0, 1.0, 0.5);
        let profile = vec![0.0, 1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        s.set_initial(&profile);
        assert!((s.u[2] - 2.0).abs() < 1e-9 /* profile set */);
    }

    #[test]
    fn test_no_nan_after_steps() {
        let mut s = new_wave_solver_1d(50, 1.0, 0.1, 0.05);
        s.excite(25, 1.0);
        for _ in 0..100 {
            ws_step(&mut s);
        }
        assert!(s.u.iter().all(|&v| !v.is_nan()) /* no NaN */);
    }

    #[test]
    fn test_stable_cfl_below_one() {
        let s = new_wave_solver_1d(100, 343.0, 1.0, 0.002);
        assert!(ws_cfl(&s) < 1.0 /* stable CFL */);
    }
}
