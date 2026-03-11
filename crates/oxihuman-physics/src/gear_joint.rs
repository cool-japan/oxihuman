// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A gear joint that couples the rotation of two bodies with a gear ratio.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GearJoint {
    pub body_a: u32,
    pub body_b: u32,
    pub ratio: f32,
    pub compliance: f32,
    pub accumulated_error: f32,
}

#[allow(dead_code)]
impl GearJoint {
    pub fn new(body_a: u32, body_b: u32, ratio: f32) -> Self {
        Self {
            body_a,
            body_b,
            ratio,
            compliance: 0.0,
            accumulated_error: 0.0,
        }
    }

    pub fn from_teeth(body_a: u32, body_b: u32, teeth_a: u32, teeth_b: u32) -> Self {
        let ratio = if teeth_b > 0 {
            teeth_a as f32 / teeth_b as f32
        } else {
            1.0
        };
        Self::new(body_a, body_b, ratio)
    }

    pub fn with_compliance(mut self, c: f32) -> Self {
        self.compliance = c.max(0.0);
        self
    }

    /// Given angle_a, compute expected angle_b.
    pub fn expected_angle_b(&self, angle_a: f32) -> f32 {
        -angle_a * self.ratio
    }

    /// Constraint error: angle_a * ratio + angle_b should equal 0.
    pub fn constraint_error(&self, angle_a: f32, angle_b: f32) -> f32 {
        angle_a * self.ratio + angle_b
    }

    /// Angular velocity relationship: omega_b = -ratio * omega_a.
    pub fn expected_omega_b(&self, omega_a: f32) -> f32 {
        -omega_a * self.ratio
    }

    /// Torque transmitted from A to B.
    pub fn transmitted_torque(&self, torque_a: f32) -> f32 {
        -torque_a * self.ratio
    }

    /// Inverse ratio (B->A).
    pub fn inverse_ratio(&self) -> f32 {
        if self.ratio.abs() < 1e-10 {
            0.0
        } else {
            1.0 / self.ratio
        }
    }

    pub fn is_reversing(&self) -> bool {
        self.ratio < 0.0
    }

    /// Speed ratio: if |ratio| > 1, body B rotates faster than A.
    pub fn speed_multiplier(&self) -> f32 {
        self.ratio.abs()
    }

    /// Torque multiplier: inverse of speed ratio.
    pub fn torque_multiplier(&self) -> f32 {
        let sr = self.speed_multiplier();
        if sr < 1e-10 {
            0.0
        } else {
            1.0 / sr
        }
    }

    /// Full rotation of A corresponds to how many degrees on B.
    pub fn degrees_per_revolution_b(&self) -> f32 {
        360.0 * self.ratio.abs()
    }

    /// How many radians B rotates per radian of A.
    pub fn radians_per_radian_b(&self) -> f32 {
        self.ratio.abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new() {
        let j = GearJoint::new(0, 1, 2.0);
        assert!((j.ratio - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_from_teeth() {
        let j = GearJoint::from_teeth(0, 1, 20, 10);
        assert!((j.ratio - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_expected_angle_b() {
        let j = GearJoint::new(0, 1, 2.0);
        assert!((j.expected_angle_b(PI) - (-2.0 * PI)).abs() < 1e-5);
    }

    #[test]
    fn test_constraint_error_zero() {
        let j = GearJoint::new(0, 1, 2.0);
        let err = j.constraint_error(1.0, -2.0);
        assert!(err.abs() < 1e-6);
    }

    #[test]
    fn test_constraint_error_nonzero() {
        let j = GearJoint::new(0, 1, 2.0);
        let err = j.constraint_error(1.0, -1.0);
        assert!((err - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_transmitted_torque() {
        let j = GearJoint::new(0, 1, 3.0);
        assert!((j.transmitted_torque(10.0) - (-30.0)).abs() < 1e-5);
    }

    #[test]
    fn test_inverse_ratio() {
        let j = GearJoint::new(0, 1, 4.0);
        assert!((j.inverse_ratio() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_is_reversing() {
        assert!(GearJoint::new(0, 1, -1.0).is_reversing());
        assert!(!GearJoint::new(0, 1, 1.0).is_reversing());
    }

    #[test]
    fn test_speed_multiplier() {
        let j = GearJoint::new(0, 1, -3.0);
        assert!((j.speed_multiplier() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_degrees_per_revolution() {
        let j = GearJoint::new(0, 1, 2.0);
        assert!((j.degrees_per_revolution_b() - 720.0).abs() < 1e-4);
    }
}
