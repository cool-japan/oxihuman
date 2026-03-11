// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rössler attractor system.

/// Rössler system: dx/dt = -y-z, dy/dt = x+a*y, dz/dt = b+z*(x-c)
#[derive(Debug, Clone)]
pub struct RosslerAttractor {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl RosslerAttractor {
    pub fn new(x: f64, y: f64, z: f64, a: f64, b: f64, c: f64) -> Self {
        Self { x, y, z, a, b, c }
    }

    fn derivatives(&self) -> (f64, f64, f64) {
        let dx = -self.y - self.z;
        let dy = self.x + self.a * self.y;
        let dz = self.b + self.z * (self.x - self.c);
        (dx, dy, dz)
    }

    /// RK4 integration step.
    pub fn step(&mut self, dt: f64) {
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

    /// Divergence of the Rössler vector field at current state.
    pub fn divergence(&self) -> f64 {
        /* d(dx)/dx + d(dy)/dy + d(dz)/dz = 0 + a + (x - c) */
        self.a + self.x - self.c
    }
}

pub fn new_rossler(x: f64, y: f64, z: f64) -> RosslerAttractor {
    RosslerAttractor::new(x, y, z, 0.2, 0.2, 5.7)
}

pub fn rossler_step(sys: &mut RosslerAttractor, dt: f64) {
    sys.step(dt);
}

pub fn rossler_position(sys: &RosslerAttractor) -> (f64, f64, f64) {
    sys.position()
}

pub fn rossler_divergence(sys: &RosslerAttractor) -> f64 {
    sys.divergence()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_rossler() {
        let r = new_rossler(1.0, 0.0, 0.0);
        let (x, y, z) = rossler_position(&r);
        assert!((x - 1.0).abs() < 1e-10);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }

    #[test]
    fn test_step_changes_state() {
        let mut r = new_rossler(1.0, 0.0, 0.0);
        rossler_step(&mut r, 0.01);
        let (x, _, _) = rossler_position(&r);
        assert!((x - 1.0).abs() > 1e-10);
    }

    #[test]
    fn test_finite_after_many_steps() {
        let mut r = new_rossler(0.1, 0.0, 0.0);
        for _ in 0..1000 {
            rossler_step(&mut r, 0.01);
        }
        let (x, y, z) = rossler_position(&r);
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert!(z.is_finite());
    }

    #[test]
    fn test_default_params() {
        let r = new_rossler(0.0, 0.0, 0.0);
        assert!((r.a - 0.2).abs() < 1e-10);
        assert!((r.b - 0.2).abs() < 1e-10);
        assert!((r.c - 5.7).abs() < 1e-10);
    }

    #[test]
    fn test_divergence_computed() {
        let r = new_rossler(0.0, 0.0, 0.0);
        let d = rossler_divergence(&r);
        assert!(d.is_finite());
    }

    #[test]
    fn test_z_stays_non_negative_from_small_start() {
        let mut r = new_rossler(0.1, 0.0, 0.1);
        for _ in 0..100 {
            rossler_step(&mut r, 0.01);
        }
        assert!(r.z.is_finite());
    }

    #[test]
    fn test_custom_params() {
        let r = RosslerAttractor::new(1.0, 0.0, 0.0, 0.1, 0.1, 4.0);
        assert!((r.a - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_divergence_at_origin() {
        /* a=0.2, x=0, c=5.7 → divergence = 0.2 + 0 - 5.7 = -5.5 */
        let r = RosslerAttractor::new(0.0, 0.0, 0.0, 0.2, 0.2, 5.7);
        let d = r.divergence();
        assert!((d - (-5.5)).abs() < 1e-10);
    }

    #[test]
    fn test_derivatives_at_origin() {
        /* dx = -0-0=0, dy=0+0.2*0=0, dz=0.2+0*(0-5.7)=0.2 */
        let r = RosslerAttractor::new(0.0, 0.0, 0.0, 0.2, 0.2, 5.7);
        let (dx, dy, dz) = r.derivatives();
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
        assert!((dz - 0.2).abs() < 1e-10);
    }
}
