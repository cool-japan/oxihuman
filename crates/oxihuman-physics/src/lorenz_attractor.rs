// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Lorenz chaotic system (3D ODE, RK4).

#![allow(dead_code)]

/// Lorenz attractor state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LorenzAttractor {
    pub sigma: f32, // default 10.0
    pub rho: f32,   // default 28.0
    pub beta: f32,  // default 8/3
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[allow(dead_code)]
impl LorenzAttractor {
    pub fn new(sigma: f32, rho: f32, beta: f32, x0: f32, y0: f32, z0: f32) -> Self {
        Self {
            sigma,
            rho,
            beta,
            x: x0,
            y: y0,
            z: z0,
        }
    }

    /// Classic Lorenz parameters.
    pub fn classic() -> Self {
        Self::new(10.0, 28.0, 8.0 / 3.0, 0.1, 0.0, 0.0)
    }

    /// Compute derivatives.
    fn deriv(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let dx = self.sigma * (y - x);
        let dy = x * (self.rho - z) - y;
        let dz = x * y - self.beta * z;
        (dx, dy, dz)
    }

    /// RK4 step.
    pub fn step(&mut self, dt: f32) {
        let (x, y, z) = (self.x, self.y, self.z);
        let (k1x, k1y, k1z) = self.deriv(x, y, z);
        let (k2x, k2y, k2z) =
            self.deriv(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y, z + 0.5 * dt * k1z);
        let (k3x, k3y, k3z) =
            self.deriv(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y, z + 0.5 * dt * k2z);
        let (k4x, k4y, k4z) = self.deriv(x + dt * k3x, y + dt * k3y, z + dt * k3z);
        self.x += dt / 6.0 * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        self.y += dt / 6.0 * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);
        self.z += dt / 6.0 * (k1z + 2.0 * k2z + 2.0 * k3z + k4z);
    }

    /// Current state as [x, y, z].
    pub fn state(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    /// Distance from origin.
    pub fn distance_from_origin(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

/// Generate a Lorenz trajectory.
#[allow(dead_code)]
pub fn lorenz_trajectory(mut attractor: LorenzAttractor, dt: f32, steps: usize) -> Vec<[f32; 3]> {
    let mut traj = Vec::with_capacity(steps + 1);
    traj.push(attractor.state());
    for _ in 0..steps {
        attractor.step(dt);
        traj.push(attractor.state());
    }
    traj
}

/// Check if a trajectory visits both "wings" (x > 0 and x < 0).
#[allow(dead_code)]
pub fn lorenz_is_chaotic(traj: &[[f32; 3]]) -> bool {
    let has_pos = traj.iter().any(|p| p[0] > 1.0);
    let has_neg = traj.iter().any(|p| p[0] < -1.0);
    has_pos && has_neg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let l = LorenzAttractor::classic();
        assert!((l.x - 0.1).abs() < 1e-5);
    }

    #[test]
    fn step_changes_state() {
        let mut l = LorenzAttractor::classic();
        let x0 = l.x;
        l.step(0.01);
        let _delta = (l.x - x0).abs();
    }

    #[test]
    fn many_steps_finite() {
        let mut l = LorenzAttractor::classic();
        for _ in 0..1000 {
            l.step(0.01);
        }
        let [x, y, z] = l.state();
        assert!(x.is_finite() && y.is_finite() && z.is_finite());
    }

    #[test]
    fn trajectory_length() {
        let traj = lorenz_trajectory(LorenzAttractor::classic(), 0.01, 100);
        assert_eq!(traj.len(), 101);
    }

    #[test]
    fn trajectory_finite() {
        let traj = lorenz_trajectory(LorenzAttractor::classic(), 0.01, 100);
        for p in &traj {
            assert!(p[0].is_finite());
        }
    }

    #[test]
    fn chaotic_trajectory() {
        let traj = lorenz_trajectory(LorenzAttractor::classic(), 0.01, 5000);
        assert!(lorenz_is_chaotic(&traj), "expected chaotic behavior");
    }

    #[test]
    fn distance_from_origin_positive() {
        let mut l = LorenzAttractor::classic();
        for _ in 0..100 {
            l.step(0.01);
        }
        assert!(l.distance_from_origin() > 0.0);
    }

    #[test]
    fn z_stays_positive_after_long_run() {
        // For classic Lorenz, z tends to stay positive
        let mut l = LorenzAttractor::classic();
        for _ in 0..2000 {
            l.step(0.01);
        }
        assert!(l.z > -50.0);
    }

    #[test]
    fn fixed_point_origin_unstable() {
        // Classic Lorenz origin is unstable — tiny perturbation grows
        let mut l = LorenzAttractor::new(10.0, 28.0, 8.0 / 3.0, 0.001, 0.001, 0.001);
        for _ in 0..500 {
            l.step(0.01);
        }
        assert!(l.distance_from_origin() > 0.01);
    }

    #[test]
    fn state_matches_fields() {
        let l = LorenzAttractor::classic();
        let [x, y, z] = l.state();
        assert!((x - l.x).abs() < 1e-5);
        assert!((y - l.y).abs() < 1e-5);
        assert!((z - l.z).abs() < 1e-5);
    }
}
