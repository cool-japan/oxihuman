#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Joint angle constraint and limits.

/// Limits for a joint angle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AngleLimits {
    pub min: f32,
    pub max: f32,
}

/// A joint with a current angle and limits.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JointAngle {
    pub angle: f32,
    pub limits: AngleLimits,
    pub stiffness: f32,
}

/// Create a new `JointAngle`.
#[allow(dead_code)]
pub fn new_joint_angle(angle: f32, min: f32, max: f32, stiffness: f32) -> JointAngle {
    JointAngle { angle, limits: AngleLimits { min, max }, stiffness }
}

/// Clamp an angle to the joint limits.
#[allow(dead_code)]
pub fn clamp_angle(joint: &JointAngle, angle: f32) -> f32 {
    angle.clamp(joint.limits.min, joint.limits.max)
}

/// Compute the angular error (how far beyond the limits).
#[allow(dead_code)]
pub fn angle_error(joint: &JointAngle) -> f32 {
    if joint.angle < joint.limits.min {
        joint.angle - joint.limits.min
    } else if joint.angle > joint.limits.max {
        joint.angle - joint.limits.max
    } else {
        0.0
    }
}

/// Compute a restoring torque to push the joint within limits.
#[allow(dead_code)]
pub fn joint_angle_torque(joint: &JointAngle) -> f32 {
    -joint.stiffness * angle_error(joint)
}

/// Return true if the current angle is within limits.
#[allow(dead_code)]
pub fn angle_in_limits(joint: &JointAngle) -> bool {
    (joint.limits.min..=joint.limits.max).contains(&joint.angle)
}

/// Map the current angle to a normalized parameter [0, 1] within limits.
#[allow(dead_code)]
pub fn angle_to_param(joint: &JointAngle) -> f32 {
    let range = joint.limits.max - joint.limits.min;
    if range < 1e-9 {
        return 0.0;
    }
    ((joint.angle - joint.limits.min) / range).clamp(0.0, 1.0)
}

/// Set new angle limits on a joint.
#[allow(dead_code)]
pub fn set_angle_limits(joint: &mut JointAngle, min: f32, max: f32) {
    joint.limits.min = min;
    joint.limits.max = max;
}

/// Compute the delta between current angle and the midpoint of the limits.
#[allow(dead_code)]
pub fn joint_angle_delta(joint: &JointAngle) -> f32 {
    let mid = (joint.limits.min + joint.limits.max) * 0.5;
    joint.angle - mid
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_joint_angle() {
        let j = new_joint_angle(0.0, -PI, PI, 10.0);
        assert!((j.angle).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_angle_within() {
        let j = new_joint_angle(0.5, -1.0, 1.0, 1.0);
        assert!((clamp_angle(&j, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_angle_exceeded() {
        let j = new_joint_angle(2.0, -1.0, 1.0, 1.0);
        assert!((clamp_angle(&j, 2.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_angle_in_limits() {
        let j = new_joint_angle(0.5, -1.0, 1.0, 1.0);
        assert!(angle_in_limits(&j));
        let j2 = new_joint_angle(2.0, -1.0, 1.0, 1.0);
        assert!(!angle_in_limits(&j2));
    }

    #[test]
    fn test_angle_error_within() {
        let j = new_joint_angle(0.0, -1.0, 1.0, 1.0);
        assert!(angle_error(&j).abs() < 1e-6);
    }

    #[test]
    fn test_angle_error_exceeded() {
        let j = new_joint_angle(2.0, -1.0, 1.0, 1.0);
        assert!((angle_error(&j) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_joint_angle_torque() {
        let j = new_joint_angle(2.0, -1.0, 1.0, 5.0);
        assert!(joint_angle_torque(&j) < 0.0); // restoring
    }

    #[test]
    fn test_angle_to_param() {
        let j = new_joint_angle(0.0, -1.0, 1.0, 1.0);
        assert!((angle_to_param(&j) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_limits() {
        let mut j = new_joint_angle(0.0, -1.0, 1.0, 1.0);
        set_angle_limits(&mut j, -2.0, 2.0);
        assert!((j.limits.min + 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_joint_angle_delta() {
        let j = new_joint_angle(1.0, 0.0, 2.0, 1.0);
        assert!(joint_angle_delta(&j).abs() < 1e-6); // at midpoint
    }
}
