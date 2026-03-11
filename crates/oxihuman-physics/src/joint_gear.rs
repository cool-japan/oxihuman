#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A gear joint linking rotations of two bodies by a ratio.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GearJoint {
    angle_a: f32,
    angle_b: f32,
    ratio: f32,
    stiffness: f32,
}

#[allow(dead_code)]
pub fn new_gear_joint(ratio: f32) -> GearJoint {
    GearJoint {
        angle_a: 0.0,
        angle_b: 0.0,
        ratio,
        stiffness: 1.0,
    }
}

#[allow(dead_code)]
pub fn gear_ratio(joint: &GearJoint) -> f32 {
    joint.ratio
}

#[allow(dead_code)]
pub fn gear_angle_a(joint: &GearJoint) -> f32 {
    joint.angle_a
}

#[allow(dead_code)]
pub fn gear_angle_b(joint: &GearJoint) -> f32 {
    joint.angle_b
}

#[allow(dead_code)]
pub fn gear_solve(joint: &mut GearJoint, angle_a: f32, angle_b: f32) -> f32 {
    joint.angle_a = angle_a;
    joint.angle_b = angle_b;
    let expected_b = angle_a * joint.ratio;
    (angle_b - expected_b) * joint.stiffness
}

#[allow(dead_code)]
pub fn gear_torque(joint: &GearJoint) -> f32 {
    let expected_b = joint.angle_a * joint.ratio;
    (joint.angle_b - expected_b) * joint.stiffness
}

#[allow(dead_code)]
pub fn gear_reset(joint: &mut GearJoint) {
    joint.angle_a = 0.0;
    joint.angle_b = 0.0;
}

#[allow(dead_code)]
pub fn gear_error(joint: &GearJoint) -> f32 {
    let expected_b = joint.angle_a * joint.ratio;
    (joint.angle_b - expected_b).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_gear_joint() {
        let j = new_gear_joint(2.0);
        assert!((gear_ratio(&j) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_ratio() {
        let j = new_gear_joint(3.0);
        assert!((gear_ratio(&j) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_angle_a() {
        let j = new_gear_joint(1.0);
        assert!((gear_angle_a(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_gear_angle_b() {
        let j = new_gear_joint(1.0);
        assert!((gear_angle_b(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_gear_solve_no_error() {
        let mut j = new_gear_joint(2.0);
        let err = gear_solve(&mut j, 1.0, 2.0);
        assert!(err.abs() < 1e-6);
    }

    #[test]
    fn test_gear_solve_with_error() {
        let mut j = new_gear_joint(2.0);
        let err = gear_solve(&mut j, 1.0, 3.0);
        assert!((err - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_torque() {
        let mut j = new_gear_joint(2.0);
        gear_solve(&mut j, 1.0, 3.0);
        assert!((gear_torque(&j) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_reset() {
        let mut j = new_gear_joint(2.0);
        gear_solve(&mut j, 1.0, 2.0);
        gear_reset(&mut j);
        assert!((gear_angle_a(&j)).abs() < 1e-6);
    }

    #[test]
    fn test_gear_error() {
        let mut j = new_gear_joint(2.0);
        gear_solve(&mut j, 1.0, 3.0);
        assert!((gear_error(&j) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gear_error_zero() {
        let mut j = new_gear_joint(2.0);
        gear_solve(&mut j, 1.0, 2.0);
        assert!(gear_error(&j) < 1e-6);
    }
}
