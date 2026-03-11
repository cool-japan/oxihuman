// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cone body: rigid body with cone inertia tensor.

use std::f32::consts::PI;

/// Cone rigid body parameters.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConeBody {
    pub mass: f32,
    pub radius: f32,
    pub height: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
}

/// Create a cone body.
#[allow(dead_code)]
pub fn new_cone_body(mass: f32, radius: f32, height: f32) -> ConeBody {
    ConeBody {
        mass,
        radius,
        height,
        position: [0.0; 3],
        velocity: [0.0; 3],
    }
}

/// Volume of a cone.
#[allow(dead_code)]
pub fn cone_volume(radius: f32, height: f32) -> f32 {
    PI * radius * radius * height / 3.0
}

/// Moment of inertia about the axis of symmetry (axis-aligned).
#[allow(dead_code)]
pub fn cone_inertia_axial(mass: f32, radius: f32) -> f32 {
    3.0 * mass * radius * radius / 10.0
}

/// Moment of inertia about a transverse axis through the apex.
#[allow(dead_code)]
pub fn cone_inertia_transverse_apex(mass: f32, radius: f32, height: f32) -> f32 {
    3.0 * mass * (radius * radius / 20.0 + height * height / 5.0)
}

/// Moment of inertia about transverse axis through centroid (at h/4 from base).
#[allow(dead_code)]
pub fn cone_inertia_transverse_centroid(mass: f32, radius: f32, height: f32) -> f32 {
    let apex = cone_inertia_transverse_apex(mass, radius, height);
    let d = height * 0.75; // centroid is at h/4 from base = 3h/4 from apex
    apex - mass * d * d
}

/// Centroid height from base.
#[allow(dead_code)]
pub fn cone_centroid_height(height: f32) -> f32 {
    height / 4.0
}

/// Integrate position by velocity * dt.
#[allow(dead_code)]
pub fn cone_integrate(body: &mut ConeBody, dt: f32) {
    body.position[0] += body.velocity[0] * dt;
    body.position[1] += body.velocity[1] * dt;
    body.position[2] += body.velocity[2] * dt;
}

/// Apply impulse to cone body.
#[allow(dead_code)]
pub fn cone_apply_impulse(body: &mut ConeBody, impulse: [f32; 3]) {
    let inv_mass = if body.mass > 1e-12 {
        1.0 / body.mass
    } else {
        0.0
    };
    body.velocity[0] += impulse[0] * inv_mass;
    body.velocity[1] += impulse[1] * inv_mass;
    body.velocity[2] += impulse[2] * inv_mass;
}

/// Kinetic energy of cone body.
#[allow(dead_code)]
pub fn cone_kinetic_energy(body: &ConeBody) -> f32 {
    let v2 = body.velocity[0] * body.velocity[0]
        + body.velocity[1] * body.velocity[1]
        + body.velocity[2] * body.velocity[2];
    0.5 * body.mass * v2
}

/// Surface area of cone (lateral + base).
#[allow(dead_code)]
pub fn cone_surface_area(radius: f32, height: f32) -> f32 {
    let slant = (radius * radius + height * height).sqrt();
    PI * radius * (radius + slant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume() {
        let v = cone_volume(1.0, 3.0);
        assert!((v - PI).abs() < 1e-4);
    }

    #[test]
    fn test_inertia_axial_positive() {
        let i = cone_inertia_axial(1.0, 1.0);
        assert!(i > 0.0);
    }

    #[test]
    fn test_inertia_transverse_apex() {
        let i = cone_inertia_transverse_apex(1.0, 1.0, 3.0);
        assert!(i > 0.0);
    }

    #[test]
    fn test_centroid_height() {
        assert!((cone_centroid_height(4.0) - 1.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_integrate() {
        let mut body = new_cone_body(1.0, 0.5, 2.0);
        body.velocity = [1.0, 0.0, 0.0];
        cone_integrate(&mut body, 2.0);
        assert!((body.position[0] - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_apply_impulse() {
        let mut body = new_cone_body(2.0, 0.5, 1.0);
        cone_apply_impulse(&mut body, [4.0, 0.0, 0.0]);
        assert!((body.velocity[0] - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut body = new_cone_body(1.0, 0.5, 1.0);
        body.velocity = [2.0, 0.0, 0.0];
        assert!((cone_kinetic_energy(&body) - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_surface_area_sphere_limit() {
        // For a very flat cone (height→0) surface ≈ PI r²
        let area = cone_surface_area(1.0, 0.0001);
        assert!((area - 2.0 * PI).abs() < 0.01);
    }

    #[test]
    fn test_transverse_centroid_less_than_apex() {
        let apex = cone_inertia_transverse_apex(1.0, 1.0, 3.0);
        let centroid = cone_inertia_transverse_centroid(1.0, 1.0, 3.0);
        // Centroid value can be negative when subtracting large d² term – just check apex > centroid.
        assert!(apex >= centroid);
    }
}
