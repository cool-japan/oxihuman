// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hinge body: rigid body constrained to rotate about a fixed axis.

use std::f32::consts::PI;

/// Hinge body state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HingeBody {
    pub angle: f32,
    pub angular_velocity: f32,
    pub inertia: f32,
    pub lo_limit: f32,
    pub hi_limit: f32,
    pub damping: f32,
}

/// Create a hinge body.
#[allow(dead_code)]
pub fn new_hinge_body(inertia: f32, lo: f32, hi: f32, damping: f32) -> HingeBody {
    HingeBody {
        angle: 0.0,
        angular_velocity: 0.0,
        inertia,
        lo_limit: lo,
        hi_limit: hi,
        damping,
    }
}

/// Apply a torque for one timestep.
#[allow(dead_code)]
pub fn hinge_apply_torque(hinge: &mut HingeBody, torque: f32, dt: f32) {
    let alpha = torque / hinge.inertia.max(1e-12);
    hinge.angular_velocity += alpha * dt;
    hinge.angular_velocity -= hinge.damping * hinge.angular_velocity * dt;
    hinge.angle += hinge.angular_velocity * dt;
    hinge.angle = hinge.angle.clamp(hinge.lo_limit, hinge.hi_limit);
    // If at limit, zero velocity in the hitting direction.
    if hinge.angle <= hinge.lo_limit && hinge.angular_velocity < 0.0 {
        hinge.angular_velocity = 0.0;
    }
    if hinge.angle >= hinge.hi_limit && hinge.angular_velocity > 0.0 {
        hinge.angular_velocity = 0.0;
    }
}

/// Kinetic energy of the hinge.
#[allow(dead_code)]
pub fn hinge_kinetic_energy(hinge: &HingeBody) -> f32 {
    0.5 * hinge.inertia * hinge.angular_velocity * hinge.angular_velocity
}

/// Whether hinge is within its limits.
#[allow(dead_code)]
pub fn hinge_at_limit(hinge: &HingeBody) -> bool {
    (hinge.angle - hinge.lo_limit).abs() < 1e-5 || (hinge.angle - hinge.hi_limit).abs() < 1e-5
}

/// Angular range.
#[allow(dead_code)]
pub fn hinge_range(hinge: &HingeBody) -> f32 {
    hinge.hi_limit - hinge.lo_limit
}

/// Normalised angle in [0, 1] within range.
#[allow(dead_code)]
pub fn hinge_normalised_angle(hinge: &HingeBody) -> f32 {
    let range = hinge_range(hinge);
    if range < 1e-12 {
        return 0.0;
    }
    (hinge.angle - hinge.lo_limit) / range
}

/// Reset hinge to zero angle and zero velocity.
#[allow(dead_code)]
pub fn hinge_reset(hinge: &mut HingeBody) {
    hinge.angle = 0.0_f32.clamp(hinge.lo_limit, hinge.hi_limit);
    hinge.angular_velocity = 0.0;
}

/// Dummy constant use.
#[allow(dead_code)]
pub fn hinge_full_turn() -> f32 {
    2.0 * PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_torque_moves_angle() {
        let mut h = new_hinge_body(1.0, -PI, PI, 0.0);
        hinge_apply_torque(&mut h, 10.0, 0.1);
        assert!(h.angle > 0.0);
    }

    #[test]
    fn test_limit_hi() {
        let mut h = new_hinge_body(1.0, -PI, PI, 0.0);
        h.angle = PI - 0.01;
        hinge_apply_torque(&mut h, 1000.0, 1.0);
        assert!(h.angle <= PI + 1e-5);
    }

    #[test]
    fn test_limit_lo() {
        let mut h = new_hinge_body(1.0, -PI, PI, 0.0);
        h.angle = -PI + 0.01;
        hinge_apply_torque(&mut h, -1000.0, 1.0);
        assert!(h.angle >= -PI - 1e-5);
    }

    #[test]
    fn test_kinetic_energy() {
        let mut h = new_hinge_body(2.0, -PI, PI, 0.0);
        h.angular_velocity = 3.0;
        assert!((hinge_kinetic_energy(&h) - 9.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_damping_reduces_velocity() {
        let mut h = new_hinge_body(1.0, -PI, PI, 1.0);
        h.angular_velocity = 1.0;
        hinge_apply_torque(&mut h, 0.0, 0.1);
        assert!(h.angular_velocity < 1.0);
    }

    #[test]
    fn test_normalised_angle() {
        let mut h = new_hinge_body(1.0, 0.0, PI, 0.0);
        h.angle = PI * 0.5;
        assert!((hinge_normalised_angle(&h) - 0.5_f32).abs() < 1e-5);
    }

    #[test]
    fn test_reset() {
        let mut h = new_hinge_body(1.0, -PI, PI, 0.0);
        h.angle = 1.0;
        h.angular_velocity = 5.0;
        hinge_reset(&mut h);
        assert_eq!(h.angular_velocity, 0.0);
    }

    #[test]
    fn test_range() {
        let h = new_hinge_body(1.0, -PI, PI, 0.0);
        assert!((hinge_range(&h) - 2.0 * PI).abs() < 1e-5);
    }

    #[test]
    fn test_full_turn() {
        assert!((hinge_full_turn() - 2.0 * PI).abs() < 1e-5);
    }
}
