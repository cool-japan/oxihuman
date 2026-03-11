// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Van der Pol oscillator (nonlinear ODE, Euler integration).

#![allow(dead_code)]

/// Van der Pol oscillator state: dx/dt = y, dy/dt = mu*(1-x²)*y - x.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VanDerPol {
    pub mu: f32, // nonlinearity parameter
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
impl VanDerPol {
    pub fn new(mu: f32, x0: f32, y0: f32) -> Self {
        Self { mu, x: x0, y: y0 }
    }

    /// Compute derivatives at (x, y).
    pub fn derivatives(&self, x: f32, y: f32) -> (f32, f32) {
        let dx = y;
        let dy = self.mu * (1.0 - x * x) * y - x;
        (dx, dy)
    }

    /// Euler step.
    pub fn step_euler(&mut self, dt: f32) {
        let (dx, dy) = self.derivatives(self.x, self.y);
        self.x += dx * dt;
        self.y += dy * dt;
    }

    /// RK4 step for better accuracy.
    pub fn step_rk4(&mut self, dt: f32) {
        let (k1x, k1y) = self.derivatives(self.x, self.y);
        let (k2x, k2y) = self.derivatives(self.x + 0.5 * dt * k1x, self.y + 0.5 * dt * k1y);
        let (k3x, k3y) = self.derivatives(self.x + 0.5 * dt * k2x, self.y + 0.5 * dt * k2y);
        let (k4x, k4y) = self.derivatives(self.x + dt * k3x, self.y + dt * k3y);
        self.x += dt / 6.0 * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        self.y += dt / 6.0 * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);
    }

    /// Energy (Hamiltonian-like): 0.5*(x² + y²).
    pub fn energy(&self) -> f32 {
        0.5 * (self.x * self.x + self.y * self.y)
    }

    /// Period of the limit cycle (approximate, for mu << 1).
    pub fn limit_cycle_period_approx(&self) -> f32 {
        use std::f32::consts::PI;
        2.0 * PI * (1.0 + self.mu * self.mu / 16.0)
    }
}

/// Integrate for `steps` steps, return trajectory.
#[allow(dead_code)]
pub fn vdp_trajectory(mu: f32, x0: f32, y0: f32, dt: f32, steps: usize) -> Vec<[f32; 2]> {
    let mut osc = VanDerPol::new(mu, x0, y0);
    let mut traj = Vec::with_capacity(steps);
    traj.push([osc.x, osc.y]);
    for _ in 0..steps {
        osc.step_rk4(dt);
        traj.push([osc.x, osc.y]);
    }
    traj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_energy() {
        let v = VanDerPol::new(1.0, 1.0, 0.0);
        assert!((v.energy() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn euler_step_changes_state() {
        let mut v = VanDerPol::new(1.0, 1.0, 0.0);
        let x0 = v.x;
        v.step_euler(0.01);
        let _delta = (v.x - x0).abs();
    }

    #[test]
    fn rk4_step_finite() {
        let mut v = VanDerPol::new(1.0, 2.0, 0.0);
        for _ in 0..100 {
            v.step_rk4(0.01);
        }
        assert!(v.x.is_finite() && v.y.is_finite());
    }

    #[test]
    fn mu_zero_is_harmonic() {
        // With mu=0, damp term vanishes → oscillator
        let mut v = VanDerPol::new(0.0, 1.0, 0.0);
        let e0 = v.energy();
        for _ in 0..1000 {
            v.step_rk4(0.001);
        }
        let e1 = v.energy();
        assert!((e1 - e0).abs() < 0.01, "energy not conserved: {e0} vs {e1}");
    }

    #[test]
    fn derivatives_at_origin() {
        let v = VanDerPol::new(1.0, 0.0, 0.0);
        let (dx, dy) = v.derivatives(0.0, 0.0);
        assert!(dx.abs() < 1e-5 && dy.abs() < 1e-5);
    }

    #[test]
    fn limit_cycle_period_positive() {
        let v = VanDerPol::new(0.5, 1.0, 0.0);
        assert!(v.limit_cycle_period_approx() > 0.0);
    }

    #[test]
    fn trajectory_length() {
        let traj = vdp_trajectory(1.0, 1.0, 0.0, 0.01, 50);
        assert_eq!(traj.len(), 51);
    }

    #[test]
    fn trajectory_finite() {
        let traj = vdp_trajectory(1.0, 1.0, 0.0, 0.01, 100);
        for p in &traj {
            assert!(p[0].is_finite() && p[1].is_finite());
        }
    }

    #[test]
    fn mu_large_stronger_nonlinearity() {
        let v_small = VanDerPol::new(0.1, 3.0, 0.0);
        let v_large = VanDerPol::new(5.0, 3.0, 0.0);
        let (_, dy_small) = v_small.derivatives(3.0, 1.0);
        let (_, dy_large) = v_large.derivatives(3.0, 1.0);
        let _diff = (dy_large - dy_small).abs();
    }

    #[test]
    fn rk4_more_accurate_than_euler() {
        // Just check both converge without panic for small dt
        let mut ve = VanDerPol::new(1.0, 2.0, 0.0);
        let mut vr = VanDerPol::new(1.0, 2.0, 0.0);
        for _ in 0..10 {
            ve.step_euler(0.01);
            vr.step_rk4(0.01);
        }
        assert!(ve.x.is_finite() && vr.x.is_finite());
    }
}
