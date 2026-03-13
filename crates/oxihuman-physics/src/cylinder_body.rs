// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cylinder rigid body with rolling constraint.

#![allow(dead_code)]

use std::f32::consts::PI;

/// Cylinder rigid body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CylinderBody {
    pub radius: f32,
    pub height: f32,
    pub mass: f32,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    /// Rotation angle around the cylinder axis (for rolling).
    pub roll_angle: f32,
    /// Angular velocity (roll rate).
    pub roll_rate: f32,
    /// Axis of the cylinder in world space.
    pub axis: [f32; 3],
}

/// Create a new cylinder body.
#[allow(dead_code)]
pub fn new_cylinder_body(radius: f32, height: f32, mass: f32) -> CylinderBody {
    CylinderBody {
        radius,
        height,
        mass,
        position: [0.0; 3],
        velocity: [0.0; 3],
        roll_angle: 0.0,
        roll_rate: 0.0,
        axis: [0.0, 1.0, 0.0],
    }
}

/// Volume of the cylinder.
#[allow(dead_code)]
pub fn cylinder_volume(b: &CylinderBody) -> f32 {
    PI * b.radius * b.radius * b.height
}

/// Moment of inertia about the cylinder axis (axial).
#[allow(dead_code)]
pub fn cylinder_inertia_axial(b: &CylinderBody) -> f32 {
    0.5 * b.mass * b.radius * b.radius
}

/// Moment of inertia about a transverse axis through center.
#[allow(dead_code)]
pub fn cylinder_inertia_transverse(b: &CylinderBody) -> f32 {
    b.mass * (3.0 * b.radius * b.radius + b.height * b.height) / 12.0
}

/// Rolling without slipping: constrain linear velocity from angular velocity.
/// v = omega * r (for rolling on flat surface).
#[allow(dead_code)]
pub fn cylinder_apply_rolling_constraint(b: &mut CylinderBody) {
    // Assume rolling in x direction, axis along z
    let omega = b.roll_rate;
    // v_x = omega * radius (rolling forward)
    let v_rolling = omega * b.radius;
    // Blend constraint: update velocity in rolling direction
    b.velocity[0] = v_rolling;
}

/// Step: integrate position, velocity, and roll.
#[allow(dead_code)]
pub fn cylinder_step(b: &mut CylinderBody, gravity: [f32; 3], dt: f32) {
    b.velocity[0] += gravity[0] * dt;
    b.velocity[1] += gravity[1] * dt;
    b.velocity[2] += gravity[2] * dt;
    b.position[0] += b.velocity[0] * dt;
    b.position[1] += b.velocity[1] * dt;
    b.position[2] += b.velocity[2] * dt;
    // Rolling: angle from velocity
    if b.radius > f32::EPSILON {
        b.roll_rate = b.velocity[0] / b.radius;
    }
    b.roll_angle += b.roll_rate * dt;
}

/// Density of the cylinder.
#[allow(dead_code)]
pub fn cylinder_density(b: &CylinderBody) -> f32 {
    let v = cylinder_volume(b);
    if v > f32::EPSILON {
        b.mass / v
    } else {
        0.0
    }
}

/// Bottom center position of the cylinder.
#[allow(dead_code)]
pub fn cylinder_bottom_center(b: &CylinderBody) -> [f32; 3] {
    [
        b.position[0] - b.axis[0] * b.height * 0.5,
        b.position[1] - b.axis[1] * b.height * 0.5,
        b.position[2] - b.axis[2] * b.height * 0.5,
    ]
}

/// Top center position of the cylinder.
#[allow(dead_code)]
pub fn cylinder_top_center(b: &CylinderBody) -> [f32; 3] {
    [
        b.position[0] + b.axis[0] * b.height * 0.5,
        b.position[1] + b.axis[1] * b.height * 0.5,
        b.position[2] + b.axis[2] * b.height * 0.5,
    ]
}

/// Apply torque to change the roll rate.
#[allow(dead_code)]
pub fn cylinder_apply_torque(b: &mut CylinderBody, torque: f32, dt: f32) {
    let inertia = cylinder_inertia_axial(b);
    if inertia > f32::EPSILON {
        b.roll_rate += (torque / inertia) * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cylinder() -> CylinderBody {
        new_cylinder_body(0.5, 2.0, 1.0)
    }

    #[test]
    fn volume_correct() {
        let b = default_cylinder();
        let expected = PI * 0.25 * 2.0;
        assert!((cylinder_volume(&b) - expected).abs() < 1e-4);
    }

    #[test]
    fn axial_inertia_positive() {
        let b = default_cylinder();
        assert!(cylinder_inertia_axial(&b) > 0.0);
    }

    #[test]
    fn transverse_inertia_positive() {
        let b = default_cylinder();
        assert!(cylinder_inertia_transverse(&b) > 0.0);
    }

    #[test]
    fn step_applies_gravity() {
        let mut b = default_cylinder();
        cylinder_step(&mut b, [0.0, -9.81, 0.0], 0.016);
        assert!(b.position[1] < 0.0);
    }

    #[test]
    fn rolling_constraint_sets_velocity() {
        let mut b = default_cylinder();
        b.roll_rate = 2.0;
        cylinder_apply_rolling_constraint(&mut b);
        assert!((b.velocity[0] - 2.0 * 0.5).abs() < 1e-5);
    }

    #[test]
    fn density_positive() {
        let b = default_cylinder();
        assert!(cylinder_density(&b) > 0.0);
    }

    #[test]
    fn bottom_top_center() {
        let b = default_cylinder();
        let bot = cylinder_bottom_center(&b);
        let top = cylinder_top_center(&b);
        assert!((bot[1] + 1.0).abs() < 1e-5);
        assert!((top[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn torque_changes_roll_rate() {
        let mut b = default_cylinder();
        cylinder_apply_torque(&mut b, 1.0, 0.1);
        assert!(b.roll_rate.abs() > 0.0);
    }

    #[test]
    fn step_updates_roll_angle() {
        let mut b = default_cylinder();
        b.velocity[0] = 1.0;
        cylinder_step(&mut b, [0.0, 0.0, 0.0], 0.1);
        assert!(b.roll_angle.abs() > 0.0);
    }

    #[test]
    fn new_cylinder_at_origin() {
        let b = default_cylinder();
        assert_eq!(b.position, [0.0, 0.0, 0.0]);
    }
}
