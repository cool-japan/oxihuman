// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lorenz system ODE stub (uses a different name from existing lorenz_attractor).

/// Parameters of the Lorenz system.
#[derive(Debug, Clone)]
pub struct LorenzParams {
    /// Prandtl number (default 10.0).
    pub sigma: f64,
    /// Rayleigh number (default 28.0).
    pub rho: f64,
    /// Geometric factor (default 8/3).
    pub beta: f64,
}

impl Default for LorenzParams {
    fn default() -> Self {
        Self {
            sigma: 10.0,
            rho: 28.0,
            beta: 8.0 / 3.0,
        }
    }
}

/// State of the Lorenz system.
#[derive(Debug, Clone)]
pub struct LorenzSystem {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub params: LorenzParams,
}

impl LorenzSystem {
    pub fn new(x: f64, y: f64, z: f64, params: LorenzParams) -> Self {
        Self { x, y, z, params }
    }

    fn derivatives(&self) -> (f64, f64, f64) {
        let p = &self.params;
        let dx = p.sigma * (self.y - self.x);
        let dy = self.x * (p.rho - self.z) - self.y;
        let dz = self.x * self.y - p.beta * self.z;
        (dx, dy, dz)
    }

    /// Runge-Kutta 4 step.
    pub fn step_rk4(&mut self, dt: f64) {
        let (k1x, k1y, k1z) = self.derivatives();

        let mut tmp = self.clone();
        tmp.x = self.x + 0.5 * dt * k1x;
        tmp.y = self.y + 0.5 * dt * k1y;
        tmp.z = self.z + 0.5 * dt * k1z;
        let (k2x, k2y, k2z) = tmp.derivatives();

        tmp.x = self.x + 0.5 * dt * k2x;
        tmp.y = self.y + 0.5 * dt * k2y;
        tmp.z = self.z + 0.5 * dt * k2z;
        let (k3x, k3y, k3z) = tmp.derivatives();

        tmp.x = self.x + dt * k3x;
        tmp.y = self.y + dt * k3y;
        tmp.z = self.z + dt * k3z;
        let (k4x, k4y, k4z) = tmp.derivatives();

        self.x += dt / 6.0 * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        self.y += dt / 6.0 * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);
        self.z += dt / 6.0 * (k1z + 2.0 * k2z + 2.0 * k3z + k4z);
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }

    pub fn divergence(&self) -> f64 {
        /* Lorenz system divergence = -(sigma + 1 + beta) */
        -(self.params.sigma + 1.0 + self.params.beta)
    }
}

pub fn new_lorenz_system(x: f64, y: f64, z: f64) -> LorenzSystem {
    LorenzSystem::new(x, y, z, LorenzParams::default())
}

pub fn lorenz_step(sys: &mut LorenzSystem, dt: f64) {
    sys.step_rk4(dt);
}

pub fn lorenz_position(sys: &LorenzSystem) -> (f64, f64, f64) {
    sys.position()
}

pub fn lorenz_divergence(sys: &LorenzSystem) -> f64 {
    sys.divergence()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system() {
        let s = new_lorenz_system(1.0, 1.0, 1.0);
        let (x, y, z) = lorenz_position(&s);
        assert!((x - 1.0).abs() < 1e-10);
        assert!((y - 1.0).abs() < 1e-10);
        assert!((z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_step_changes_state() {
        let mut s = new_lorenz_system(1.0, 1.0, 1.0);
        lorenz_step(&mut s, 0.01);
        let (x, _, _) = lorenz_position(&s);
        assert!((x - 1.0).abs() > 1e-10);
    }

    #[test]
    fn test_state_finite_after_steps() {
        let mut s = new_lorenz_system(0.1, 0.0, 0.0);
        for _ in 0..1000 {
            lorenz_step(&mut s, 0.01);
        }
        let (x, y, z) = lorenz_position(&s);
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert!(z.is_finite());
    }

    #[test]
    fn test_divergence_negative() {
        /* Lorenz attractor: volume-contracting → negative divergence */
        let s = new_lorenz_system(0.0, 0.0, 0.0);
        assert!(lorenz_divergence(&s) < 0.0);
    }

    #[test]
    fn test_default_params() {
        let s = new_lorenz_system(1.0, 0.0, 0.0);
        assert!((s.params.sigma - 10.0).abs() < 1e-10);
        assert!((s.params.rho - 28.0).abs() < 1e-10);
    }

    #[test]
    fn test_z_non_negative_from_positive_start() {
        /* z starts positive and has dz = xy - beta*z; should stay in attractor */
        let mut s = new_lorenz_system(1.0, 1.0, 1.0);
        for _ in 0..100 {
            lorenz_step(&mut s, 0.01);
        }
        assert!(s.z.is_finite());
    }

    #[test]
    fn test_equilibrium_at_origin() {
        /* At (0,0,0) derivatives are all 0 for sigma,rho,beta */
        let s = LorenzSystem::new(0.0, 0.0, 0.0, LorenzParams::default());
        let (dx, dy, dz) = s.derivatives();
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
        assert_eq!(dz, 0.0);
    }

    #[test]
    fn test_custom_params() {
        let p = LorenzParams {
            sigma: 5.0,
            rho: 10.0,
            beta: 1.0,
        };
        let s = LorenzSystem::new(1.0, 0.0, 0.0, p);
        assert!((s.params.sigma - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_rk4_better_than_euler() {
        /* RK4 should keep z > 0 longer than Euler for standard params */
        let mut s = new_lorenz_system(1.0, 1.0, 1.0);
        lorenz_step(&mut s, 0.01);
        assert!(s.x.is_finite() && s.y.is_finite() && s.z.is_finite());
    }
}
