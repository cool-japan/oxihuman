// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ratchet mechanism allowing rotation in only one direction.

#![allow(dead_code)]

/// Direction of free rotation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatchetDir {
    /// Ratchet allows positive (CCW) rotation.
    Positive,
    /// Ratchet allows negative (CW) rotation.
    Negative,
}

/// A ratchet mechanism.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RatchetBody {
    /// Allowed rotation direction.
    pub free_dir: RatchetDir,
    /// Number of teeth on the ratchet wheel.
    pub teeth: u32,
    /// Current angle (radians).
    pub angle: f32,
    /// Angular velocity (rad/s).
    pub omega: f32,
    /// Moment of inertia (kg·m²).
    pub inertia: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Count of clicks (tooth engagements).
    pub click_count: u32,
}

/// Angle per tooth (radians).
#[allow(dead_code)]
pub fn ratchet_tooth_angle(rb: &RatchetBody) -> f32 {
    if rb.teeth == 0 {
        return 2.0 * std::f32::consts::PI;
    }
    2.0 * std::f32::consts::PI / rb.teeth as f32
}

/// Create a new ratchet body.
#[allow(dead_code)]
pub fn new_ratchet(teeth: u32, free_dir: RatchetDir, inertia: f32, damping: f32) -> RatchetBody {
    RatchetBody {
        free_dir,
        teeth,
        angle: 0.0,
        omega: 0.0,
        inertia: inertia.max(1e-6),
        damping,
        click_count: 0,
    }
}

/// Apply a torque and step the ratchet. Blocked direction results in zero velocity.
#[allow(dead_code)]
pub fn ratchet_step(rb: &mut RatchetBody, torque: f32, dt: f32) {
    let drag = -rb.damping * rb.omega;
    let alpha = (torque + drag) / rb.inertia;
    let new_omega = rb.omega + alpha * dt;

    let allowed = match rb.free_dir {
        RatchetDir::Positive => new_omega >= 0.0,
        RatchetDir::Negative => new_omega <= 0.0,
    };

    if allowed {
        rb.omega = new_omega;
    } else {
        rb.omega = 0.0;
    }

    let prev_tooth = (rb.angle / ratchet_tooth_angle(rb)).floor() as i64;
    rb.angle += rb.omega * dt;
    let new_tooth = (rb.angle / ratchet_tooth_angle(rb)).floor() as i64;
    if (new_tooth - prev_tooth).abs() > 0 {
        rb.click_count += 1;
    }
}

/// RPM.
#[allow(dead_code)]
pub fn ratchet_rpm(rb: &RatchetBody) -> f32 {
    rb.omega * 60.0 / (2.0 * std::f32::consts::PI)
}

/// Kinetic energy.
#[allow(dead_code)]
pub fn ratchet_energy(rb: &RatchetBody) -> f32 {
    0.5 * rb.inertia * rb.omega * rb.omega
}

/// Reset the ratchet.
#[allow(dead_code)]
pub fn ratchet_reset(rb: &mut RatchetBody) {
    rb.angle = 0.0;
    rb.omega = 0.0;
    rb.click_count = 0;
}

/// Check if motion is currently blocked.
#[allow(dead_code)]
pub fn ratchet_is_blocked(rb: &RatchetBody, torque: f32) -> bool {
    match rb.free_dir {
        RatchetDir::Positive => torque < 0.0,
        RatchetDir::Negative => torque > 0.0,
    }
}

/// Number of full rotations made.
#[allow(dead_code)]
pub fn ratchet_full_rotations(rb: &RatchetBody) -> f32 {
    rb.angle / (2.0 * std::f32::consts::PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ratchet() -> RatchetBody {
        new_ratchet(12, RatchetDir::Positive, 0.1, 0.05)
    }

    #[test]
    fn test_initial_state() {
        let rb = make_ratchet();
        assert_eq!(rb.omega, 0.0);
        assert_eq!(rb.click_count, 0);
    }

    #[test]
    fn test_positive_torque_allowed() {
        let mut rb = make_ratchet();
        ratchet_step(&mut rb, 1.0, 0.1);
        assert!(rb.omega > 0.0);
    }

    #[test]
    fn test_negative_torque_blocked() {
        let mut rb = make_ratchet();
        ratchet_step(&mut rb, -1.0, 0.1);
        assert_eq!(rb.omega, 0.0);
    }

    #[test]
    fn test_angle_advances() {
        let mut rb = make_ratchet();
        ratchet_step(&mut rb, 1.0, 0.5);
        assert!(rb.angle >= 0.0);
    }

    #[test]
    fn test_tooth_angle() {
        let rb = make_ratchet();
        let ta = ratchet_tooth_angle(&rb);
        assert!((ta - std::f32::consts::PI / 6.0).abs() < 1e-4);
    }

    #[test]
    fn test_energy() {
        let mut rb = make_ratchet();
        ratchet_step(&mut rb, 1.0, 0.1);
        assert!(ratchet_energy(&rb) >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut rb = make_ratchet();
        ratchet_step(&mut rb, 1.0, 1.0);
        ratchet_reset(&mut rb);
        assert_eq!(rb.omega, 0.0);
        assert_eq!(rb.click_count, 0);
    }

    #[test]
    fn test_is_blocked() {
        let rb = make_ratchet();
        assert!(ratchet_is_blocked(&rb, -1.0));
        assert!(!ratchet_is_blocked(&rb, 1.0));
    }

    #[test]
    fn test_negative_dir_allows_cw() {
        let mut rb = new_ratchet(12, RatchetDir::Negative, 0.1, 0.0);
        ratchet_step(&mut rb, -1.0, 0.1);
        assert!(rb.omega < 0.0);
    }

    #[test]
    fn test_full_rotations() {
        let mut rb = make_ratchet();
        rb.angle = 2.0 * std::f32::consts::PI;
        assert!((ratchet_full_rotations(&rb) - 1.0).abs() < 1e-5);
    }
}
