// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Torus rigid body with donut geometry.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Torus rigid body (major radius R, minor/tube radius r).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TorusBody {
    /// Major radius (center of tube from center of torus).
    pub major_radius: f32,
    /// Minor radius (radius of the tube).
    pub minor_radius: f32,
    pub mass: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    /// Normal axis of the torus plane.
    pub normal: [f32; 3],
    /// Angular velocity around normal.
    pub angular_velocity: f32,
    /// Rotation angle.
    pub angle: f32,
}

/// Create a new torus body.
#[allow(dead_code)]
pub fn new_torus_body(major_radius: f32, minor_radius: f32, mass: f32) -> TorusBody {
    TorusBody {
        major_radius,
        minor_radius,
        mass,
        position: [0.0; 3],
        velocity: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        angular_velocity: 0.0,
        angle: 0.0,
    }
}

/// Volume of the torus: 2 * pi^2 * R * r^2.
#[allow(dead_code)]
pub fn torus_volume(b: &TorusBody) -> f32 {
    2.0 * PI * PI * b.major_radius * b.minor_radius * b.minor_radius
}

/// Surface area of the torus: 4 * pi^2 * R * r.
#[allow(dead_code)]
pub fn torus_surface_area(b: &TorusBody) -> f32 {
    4.0 * PI * PI * b.major_radius * b.minor_radius
}

/// Moment of inertia about the axis through the center (symmetric axis).
/// Approximation: I_z ≈ m * (R^2 + (3/4) * r^2)
#[allow(dead_code)]
pub fn torus_inertia_axial(b: &TorusBody) -> f32 {
    b.mass * (b.major_radius * b.major_radius + 0.75 * b.minor_radius * b.minor_radius)
}

/// Moment of inertia about a diameter axis (transverse).
/// Approximation: I_x ≈ m * ((5/8) * r^2 + (1/2) * R^2)
#[allow(dead_code)]
pub fn torus_inertia_transverse(b: &TorusBody) -> f32 {
    b.mass * (0.625 * b.minor_radius * b.minor_radius + 0.5 * b.major_radius * b.major_radius)
}

/// Outer radius of the torus (major + minor).
#[allow(dead_code)]
pub fn torus_outer_radius(b: &TorusBody) -> f32 {
    b.major_radius + b.minor_radius
}

/// Inner radius of the torus (major - minor).
#[allow(dead_code)]
pub fn torus_inner_radius(b: &TorusBody) -> f32 {
    (b.major_radius - b.minor_radius).max(0.0)
}

/// Check if a 2D point (in the torus plane) is inside the torus cross-section.
#[allow(dead_code)]
pub fn torus_contains_point_2d(b: &TorusBody, x: f32, y: f32) -> bool {
    let dist_center = (x * x + y * y).sqrt();
    let dist_tube = (dist_center - b.major_radius).abs();
    dist_tube <= b.minor_radius
}

/// Step: apply gravity, integrate position and spin.
#[allow(dead_code)]
pub fn torus_step(b: &mut TorusBody, gravity: [f32; 3], dt: f32) {
    b.velocity[0] += gravity[0] * dt;
    b.velocity[1] += gravity[1] * dt;
    b.velocity[2] += gravity[2] * dt;
    b.position[0] += b.velocity[0] * dt;
    b.position[1] += b.velocity[1] * dt;
    b.position[2] += b.velocity[2] * dt;
    b.angle += b.angular_velocity * dt;
}

/// Apply a spin impulse around the normal axis.
#[allow(dead_code)]
pub fn torus_apply_spin(b: &mut TorusBody, impulse: f32) {
    let i = torus_inertia_axial(b);
    if i > f32::EPSILON {
        b.angular_velocity += impulse / i;
    }
}

/// Density of the torus.
#[allow(dead_code)]
pub fn torus_density(b: &TorusBody) -> f32 {
    let v = torus_volume(b);
    if v > f32::EPSILON {
        b.mass / v
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_torus() -> TorusBody {
        new_torus_body(2.0, 0.5, 1.0)
    }

    #[test]
    fn volume_positive() {
        let b = default_torus();
        assert!(torus_volume(&b) > 0.0);
    }

    #[test]
    fn surface_area_positive() {
        let b = default_torus();
        assert!(torus_surface_area(&b) > 0.0);
    }

    #[test]
    fn inertia_axial_positive() {
        let b = default_torus();
        assert!(torus_inertia_axial(&b) > 0.0);
    }

    #[test]
    fn inertia_transverse_positive() {
        let b = default_torus();
        assert!(torus_inertia_transverse(&b) > 0.0);
    }

    #[test]
    fn outer_inner_radius() {
        let b = default_torus();
        assert!((torus_outer_radius(&b) - 2.5).abs() < 1e-5);
        assert!((torus_inner_radius(&b) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn point_inside_torus() {
        let b = default_torus();
        // Point at (2, 0): distance from center = 2 = major_radius, tube dist = 0 < 0.5
        assert!(torus_contains_point_2d(&b, 2.0, 0.0));
    }

    #[test]
    fn point_outside_torus() {
        let b = default_torus();
        // Point at origin: distance from center = 0, |0-2| = 2 > 0.5
        assert!(!torus_contains_point_2d(&b, 0.0, 0.0));
    }

    #[test]
    fn step_applies_gravity() {
        let mut b = default_torus();
        torus_step(&mut b, [0.0, -9.81, 0.0], 0.016);
        assert!(b.position[1] < 0.0);
    }

    #[test]
    fn spin_changes_angular_velocity() {
        let mut b = default_torus();
        torus_apply_spin(&mut b, 1.0);
        assert!(b.angular_velocity.abs() > 0.0);
    }

    #[test]
    fn density_positive() {
        let b = default_torus();
        assert!(torus_density(&b) > 0.0);
    }
}
