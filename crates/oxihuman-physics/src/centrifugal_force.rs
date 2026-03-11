// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rotating frame pseudo-forces: centrifugal acceleration.

/// A rotating reference frame.
#[derive(Debug, Clone)]
pub struct RotatingFrame {
    /// Angular velocity vector ω (rad/s).
    pub omega: [f64; 3],
}

impl RotatingFrame {
    /// Create a new rotating frame.
    pub fn new(omega: [f64; 3]) -> Self {
        RotatingFrame { omega }
    }

    /// Centrifugal acceleration: a_cf = -ω × (ω × r).
    pub fn centrifugal_accel(&self, r: [f64; 3]) -> [f64; 3] {
        let omega_cross_r = cross(self.omega, r);
        let neg = cross(self.omega, omega_cross_r);
        [-neg[0], -neg[1], -neg[2]]
    }

    /// Centrifugal force on a mass.
    pub fn centrifugal_force(&self, r: [f64; 3], mass: f64) -> [f64; 3] {
        let a = self.centrifugal_accel(r);
        [a[0] * mass, a[1] * mass, a[2] * mass]
    }

    /// Angular speed (magnitude of omega).
    pub fn angular_speed(&self) -> f64 {
        mag3(self.omega)
    }

    /// Centrifugal potential energy: U = -0.5 * m * |ω × r|².
    pub fn centrifugal_potential(&self, r: [f64; 3], mass: f64) -> f64 {
        let omega_cross_r = cross(self.omega, r);
        let mag2: f64 = omega_cross_r.iter().map(|&x| x * x).sum();
        -0.5 * mass * mag2
    }

    /// Effective gravity (true gravity + centrifugal).
    pub fn effective_gravity(&self, r: [f64; 3], g: f64) -> [f64; 3] {
        let cf = self.centrifugal_accel(r);
        [cf[0], cf[1] - g, cf[2]]
    }
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn mag3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Create a new rotating frame.
pub fn new_rotating_frame(omega: [f64; 3]) -> RotatingFrame {
    RotatingFrame::new(omega)
}

/// Centrifugal acceleration.
pub fn cf_centrifugal_accel(f: &RotatingFrame, r: [f64; 3]) -> [f64; 3] {
    f.centrifugal_accel(r)
}

/// Centrifugal force.
pub fn cf_centrifugal_force(f: &RotatingFrame, r: [f64; 3], mass: f64) -> [f64; 3] {
    f.centrifugal_force(r, mass)
}

/// Angular speed.
pub fn cf_angular_speed(f: &RotatingFrame) -> f64 {
    f.angular_speed()
}

/// Centrifugal potential.
pub fn cf_centrifugal_potential(f: &RotatingFrame, r: [f64; 3], mass: f64) -> f64 {
    f.centrifugal_potential(r, mass)
}

/// Effective gravity.
pub fn cf_effective_gravity(f: &RotatingFrame, r: [f64; 3], g: f64) -> [f64; 3] {
    f.effective_gravity(r, g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_rotation_zero_force() {
        let f = new_rotating_frame([0.0; 3]);
        let a = cf_centrifugal_accel(&f, [1.0, 0.0, 0.0]);
        assert!(a.iter().all(|&x| x.abs() < 1e-12) /* no rotation → no centrifugal */);
    }

    #[test]
    fn test_rotation_about_z_radial_outward() {
        /* ω = [0,0,1], r = [1,0,0] → a_cf = [1,0,0] */
        let f = new_rotating_frame([0.0, 0.0, 1.0]);
        let a = cf_centrifugal_accel(&f, [1.0, 0.0, 0.0]);
        assert!((a[0] - 1.0).abs() < 1e-9 /* outward in x */);
        assert!(a[1].abs() < 1e-9);
    }

    #[test]
    fn test_angular_speed() {
        let f = new_rotating_frame([0.0, 0.0, 3.0]);
        assert!((cf_angular_speed(&f) - 3.0).abs() < 1e-9 /* ω = 3 rad/s */);
    }

    #[test]
    fn test_centrifugal_force_scaled_by_mass() {
        let f = new_rotating_frame([0.0, 0.0, 1.0]);
        let a = cf_centrifugal_accel(&f, [2.0, 0.0, 0.0]);
        let force = cf_centrifugal_force(&f, [2.0, 0.0, 0.0], 3.0);
        assert!((force[0] - a[0] * 3.0).abs() < 1e-9 /* F = m*a */);
    }

    #[test]
    fn test_centrifugal_potential_negative() {
        let f = new_rotating_frame([0.0, 0.0, 2.0]);
        let u = cf_centrifugal_potential(&f, [1.0, 0.0, 0.0], 1.0);
        assert!(u < 0.0 /* potential is negative */);
    }

    #[test]
    fn test_potential_zero_at_axis() {
        let f = new_rotating_frame([0.0, 0.0, 5.0]);
        /* r along rotation axis → ω × r = 0 */
        let u = cf_centrifugal_potential(&f, [0.0, 0.0, 1.0], 1.0);
        assert!(u.abs() < 1e-9 /* on axis: no centrifugal */);
    }

    #[test]
    fn test_effective_gravity_points_down() {
        let f = new_rotating_frame([0.0; 3]);
        let eg = cf_effective_gravity(&f, [0.0; 3], 9.81);
        assert!(eg[1] < 0.0 /* gravity pulls down */);
    }

    #[test]
    fn test_force_increases_with_radius() {
        let f = new_rotating_frame([0.0, 0.0, 1.0]);
        let f1 = cf_centrifugal_accel(&f, [1.0, 0.0, 0.0]);
        let f2 = cf_centrifugal_accel(&f, [2.0, 0.0, 0.0]);
        assert!(f2[0] > f1[0] /* larger radius → larger centrifugal */);
    }

    #[test]
    fn test_force_increases_with_omega() {
        let f1 = new_rotating_frame([0.0, 0.0, 1.0]);
        let f2 = new_rotating_frame([0.0, 0.0, 2.0]);
        let a1 = cf_centrifugal_accel(&f1, [1.0, 0.0, 0.0]);
        let a2 = cf_centrifugal_accel(&f2, [1.0, 0.0, 0.0]);
        assert!(a2[0] > a1[0] /* higher ω → larger centrifugal */);
    }

    #[test]
    fn test_omega_stored() {
        let f = new_rotating_frame([1.0, 2.0, 3.0]);
        assert_eq!(f.omega, [1.0, 2.0, 3.0] /* omega stored correctly */);
    }
}
