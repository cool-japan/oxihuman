// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gravity sources: uniform gravity, point gravity attractors, and orbital mechanics.

use std::f32::consts::PI;

/// Gravitational constant (SI, not game-scale).
pub const G: f32 = 6.674e-11;

/// A uniform (global) gravity field.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct UniformGravity {
    pub acceleration: [f32; 3],
}

#[allow(dead_code)]
impl UniformGravity {
    pub fn earth() -> Self {
        Self {
            acceleration: [0.0, -9.81, 0.0],
        }
    }

    pub fn zero() -> Self {
        Self {
            acceleration: [0.0; 3],
        }
    }

    pub fn force_on(&self, mass: f32) -> [f32; 3] {
        [
            self.acceleration[0] * mass,
            self.acceleration[1] * mass,
            self.acceleration[2] * mass,
        ]
    }
}

/// A point mass gravity attractor.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GravitySource {
    pub position: [f32; 3],
    pub mass: f32,
}

#[allow(dead_code)]
impl GravitySource {
    pub fn new(position: [f32; 3], mass: f32) -> Self {
        Self { position, mass }
    }

    /// Gravitational force on a body at `pos` with mass `body_mass`.
    pub fn force_on(&self, pos: [f32; 3], body_mass: f32) -> [f32; 3] {
        let dx = self.position[0] - pos[0];
        let dy = self.position[1] - pos[1];
        let dz = self.position[2] - pos[2];
        let r2 = dx * dx + dy * dy + dz * dz;
        if r2 < 1e-12 {
            return [0.0; 3];
        }
        let r = r2.sqrt();
        let f = G * self.mass * body_mass / r2;
        [dx / r * f, dy / r * f, dz / r * f]
    }

    /// Gravitational acceleration at distance `r`.
    pub fn acceleration_at_radius(&self, r: f32) -> f32 {
        if r < 1e-9 {
            return 0.0;
        }
        G * self.mass / (r * r)
    }

    /// Orbital velocity for a circular orbit at distance `r`.
    pub fn orbital_velocity(&self, r: f32) -> f32 {
        if r < 1e-9 {
            return 0.0;
        }
        (G * self.mass / r).sqrt()
    }

    /// Escape velocity from distance `r`.
    pub fn escape_velocity(&self, r: f32) -> f32 {
        if r < 1e-9 {
            return 0.0;
        }
        (2.0 * G * self.mass / r).sqrt()
    }
}

/// Combine multiple gravity sources at a point.
#[allow(dead_code)]
pub fn combined_gravity(sources: &[GravitySource], pos: [f32; 3], body_mass: f32) -> [f32; 3] {
    let mut fx = 0.0f32;
    let mut fy = 0.0f32;
    let mut fz = 0.0f32;
    for s in sources {
        let f = s.force_on(pos, body_mass);
        fx += f[0];
        fy += f[1];
        fz += f[2];
    }
    [fx, fy, fz]
}

/// Gravitational potential energy between two masses at distance `r`.
#[allow(dead_code)]
pub fn gravitational_potential(mass1: f32, mass2: f32, r: f32) -> f32 {
    if r < 1e-9 {
        return f32::NEG_INFINITY;
    }
    -G * mass1 * mass2 / r
}

/// Orbital period for circular orbit at radius `r` around mass `M`.
#[allow(dead_code)]
pub fn orbital_period(r: f32, big_m: f32) -> f32 {
    if big_m < 1e-9 || r < 1e-9 {
        return 0.0;
    }
    2.0 * PI * (r * r * r / (G * big_m)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_gravity_earth_force() {
        let g = UniformGravity::earth();
        let f = g.force_on(1.0);
        assert!((f[1] - (-9.81)).abs() < 1e-4);
    }

    #[test]
    fn zero_gravity_no_force() {
        let g = UniformGravity::zero();
        assert_eq!(g.force_on(100.0), [0.0; 3]);
    }

    #[test]
    fn point_gravity_direction_toward_source() {
        let src = GravitySource::new([0.0, 0.0, 0.0], 1e12);
        let f = src.force_on([1.0, 0.0, 0.0], 1.0);
        // Force should point from body toward source (-x direction)
        assert!(f[0] < 0.0);
    }

    #[test]
    fn point_gravity_zero_at_coincident() {
        let src = GravitySource::new([1.0, 0.0, 0.0], 1e12);
        let f = src.force_on([1.0, 0.0, 0.0], 1.0);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn escape_velocity_gt_orbital_velocity() {
        let src = GravitySource::new([0.0; 3], 5.972e24);
        let v_orb = src.orbital_velocity(6.371e6);
        let v_esc = src.escape_velocity(6.371e6);
        assert!(v_esc > v_orb);
    }

    #[test]
    fn orbital_period_formula() {
        let period = orbital_period(1.496e11, 1.989e30);
        // Earth's period ~3.16e7 s; within 5%
        assert!((period - 3.156e7).abs() / 3.156e7 < 0.05);
    }

    #[test]
    fn combined_gravity_sums_forces() {
        let s1 = GravitySource::new([1.0, 0.0, 0.0], 1e15);
        let s2 = GravitySource::new([-1.0, 0.0, 0.0], 1e15);
        // Symmetric sources: net force at origin ≈ 0
        let f = combined_gravity(&[s1, s2], [0.0, 0.0, 0.0], 1.0);
        assert!(f[0].abs() < 1e-3);
    }

    #[test]
    fn gravitational_potential_negative() {
        let u = gravitational_potential(1e12, 1.0, 1.0);
        assert!(u < 0.0);
    }

    #[test]
    fn acceleration_at_radius_decreases_with_distance() {
        let src = GravitySource::new([0.0; 3], 1e15);
        let a1 = src.acceleration_at_radius(1.0);
        let a2 = src.acceleration_at_radius(2.0);
        assert!(a1 > a2);
    }
}
