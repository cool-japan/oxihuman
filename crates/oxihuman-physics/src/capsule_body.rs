// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Capsule rigid body: two hemispheres + cylinder.

#![allow(dead_code)]

use std::f32::consts::PI;

/// A capsule rigid body (line segment of half-height + radius).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapsuleBody {
    /// Radius of the two hemispheres.
    pub radius: f32,
    /// Half-height of the cylinder (distance from center to each hemisphere center).
    pub half_height: f32,
    /// Mass of the capsule.
    pub mass: f32,
    /// Center position.
    pub position: [f32; 3],
    /// Orientation as axis-angle (axis, angle in radians).
    pub axis: [f32; 3],
    pub angle: f32,
    /// Linear velocity.
    pub velocity: [f32; 3],
    /// Angular velocity.
    pub angular_velocity: [f32; 3],
}

/// Create a new capsule body.
#[allow(dead_code)]
pub fn new_capsule_body(radius: f32, half_height: f32, mass: f32) -> CapsuleBody {
    CapsuleBody {
        radius,
        half_height,
        mass,
        position: [0.0; 3],
        axis: [0.0, 1.0, 0.0],
        angle: 0.0,
        velocity: [0.0; 3],
        angular_velocity: [0.0; 3],
    }
}

/// Volume of the capsule (cylinder + two hemispheres = one sphere).
#[allow(dead_code)]
pub fn capsule_volume(b: &CapsuleBody) -> f32 {
    let cylinder = PI * b.radius * b.radius * 2.0 * b.half_height;
    let sphere = (4.0 / 3.0) * PI * b.radius * b.radius * b.radius;
    cylinder + sphere
}

/// Total length of the capsule (from tip to tip).
#[allow(dead_code)]
pub fn capsule_total_length(b: &CapsuleBody) -> f32 {
    2.0 * (b.half_height + b.radius)
}

/// Moment of inertia about the cylinder axis (longitudinal).
#[allow(dead_code)]
pub fn capsule_inertia_longitudinal(b: &CapsuleBody) -> f32 {
    // Cylinder: 0.5 * m_cyl * r^2 + hemisphere: 0.4 * m_hemi * r^2 (approximate)
    let v_cyl = PI * b.radius * b.radius * 2.0 * b.half_height;
    let v_sph = (4.0 / 3.0) * PI * b.radius * b.radius * b.radius;
    let v_total = v_cyl + v_sph;
    let rho = if v_total > f32::EPSILON {
        b.mass / v_total
    } else {
        1.0
    };
    let m_cyl = rho * v_cyl;
    let m_sph = rho * v_sph;
    0.5 * m_cyl * b.radius * b.radius + 0.4 * m_sph * b.radius * b.radius
}

/// Moment of inertia about the transverse axis.
#[allow(dead_code)]
pub fn capsule_inertia_transverse(b: &CapsuleBody) -> f32 {
    let v_cyl = PI * b.radius * b.radius * 2.0 * b.half_height;
    let v_sph = (4.0 / 3.0) * PI * b.radius * b.radius * b.radius;
    let v_total = v_cyl + v_sph;
    let rho = if v_total > f32::EPSILON {
        b.mass / v_total
    } else {
        1.0
    };
    let m_cyl = rho * v_cyl;
    let m_sph = rho * v_sph;
    // Cylinder transverse
    let i_cyl = m_cyl * (b.radius * b.radius / 4.0 + b.half_height * b.half_height / 3.0);
    // Hemisphere transverse (approximate)
    let i_sph = m_sph * (0.4 * b.radius * b.radius + b.half_height * b.half_height);
    i_cyl + i_sph
}

/// Step: apply gravity, integrate position and angle.
#[allow(dead_code)]
pub fn capsule_step(b: &mut CapsuleBody, gravity: [f32; 3], dt: f32) {
    // Apply gravity
    b.velocity[0] += gravity[0] * dt;
    b.velocity[1] += gravity[1] * dt;
    b.velocity[2] += gravity[2] * dt;
    // Integrate position
    b.position[0] += b.velocity[0] * dt;
    b.position[1] += b.velocity[1] * dt;
    b.position[2] += b.velocity[2] * dt;
    // Integrate angle (around axis)
    let omega = (b.angular_velocity[0] * b.angular_velocity[0]
        + b.angular_velocity[1] * b.angular_velocity[1]
        + b.angular_velocity[2] * b.angular_velocity[2])
        .sqrt();
    b.angle += omega * dt;
}

/// Closest points on a capsule's segment to a world-space point.
#[allow(dead_code)]
pub fn capsule_closest_point(b: &CapsuleBody, point: [f32; 3]) -> [f32; 3] {
    // The capsule axis goes from position - axis*half_height to position + axis*half_height
    let p0 = [
        b.position[0] - b.axis[0] * b.half_height,
        b.position[1] - b.axis[1] * b.half_height,
        b.position[2] - b.axis[2] * b.half_height,
    ];
    let p1 = [
        b.position[0] + b.axis[0] * b.half_height,
        b.position[1] + b.axis[1] * b.half_height,
        b.position[2] + b.axis[2] * b.half_height,
    ];
    let seg = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let to_point = [point[0] - p0[0], point[1] - p0[1], point[2] - p0[2]];
    let seg_len_sq = seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2];
    let t = if seg_len_sq > f32::EPSILON {
        (to_point[0] * seg[0] + to_point[1] * seg[1] + to_point[2] * seg[2]) / seg_len_sq
    } else {
        0.0
    };
    let t = t.clamp(0.0, 1.0);
    [p0[0] + seg[0] * t, p0[1] + seg[1] * t, p0[2] + seg[2] * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_capsule() -> CapsuleBody {
        new_capsule_body(0.5, 1.0, 1.0)
    }

    #[test]
    fn volume_positive() {
        let b = default_capsule();
        assert!(capsule_volume(&b) > 0.0);
    }

    #[test]
    fn total_length_correct() {
        let b = default_capsule();
        // 2*(1.0 + 0.5) = 3.0
        assert!((capsule_total_length(&b) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn inertia_longitudinal_positive() {
        let b = default_capsule();
        assert!(capsule_inertia_longitudinal(&b) > 0.0);
    }

    #[test]
    fn inertia_transverse_positive() {
        let b = default_capsule();
        assert!(capsule_inertia_transverse(&b) > 0.0);
    }

    #[test]
    fn step_applies_gravity() {
        let mut b = default_capsule();
        let before_y = b.position[1];
        capsule_step(&mut b, [0.0, -9.81, 0.0], 0.016);
        assert!(b.position[1] < before_y);
    }

    #[test]
    fn closest_point_on_axis() {
        let b = default_capsule();
        // Point directly above the center
        let cp = capsule_closest_point(&b, [0.0, 5.0, 0.0]);
        // Closest point on capsule axis (from -1 to 1) should be [0,1,0]
        assert!((cp[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn closest_point_at_middle() {
        let b = default_capsule();
        let cp = capsule_closest_point(&b, [0.0, 0.0, 0.0]);
        assert!((cp[1]).abs() < 1e-5);
    }

    #[test]
    fn volume_sphere_limit() {
        // When half_height = 0, capsule is just a sphere
        let b = new_capsule_body(1.0, 0.0, 1.0);
        let v = capsule_volume(&b);
        let sphere_v = (4.0 / 3.0) * PI * 1.0;
        assert!((v - sphere_v).abs() < 1e-5);
    }

    #[test]
    fn step_integrates_velocity() {
        let mut b = default_capsule();
        b.velocity = [1.0, 0.0, 0.0];
        capsule_step(&mut b, [0.0, 0.0, 0.0], 1.0);
        assert!((b.position[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn new_capsule_at_origin() {
        let b = default_capsule();
        assert_eq!(b.position, [0.0, 0.0, 0.0]);
    }
}
