#![allow(dead_code)]

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConeJoint {
    half_angle: f32,
    current_angle: f32,
    limited: bool,
    torque: f32,
}

#[allow(dead_code)]
pub fn new_cone_joint(half_angle: f32) -> ConeJoint {
    ConeJoint {
        half_angle: half_angle.clamp(0.0, PI),
        current_angle: 0.0,
        limited: true,
        torque: 0.0,
    }
}

#[allow(dead_code)]
pub fn cone_half_angle_joint(joint: &ConeJoint) -> f32 {
    joint.half_angle
}

#[allow(dead_code)]
pub fn cone_angle_error(joint: &ConeJoint) -> f32 {
    if joint.limited && joint.current_angle > joint.half_angle {
        joint.current_angle - joint.half_angle
    } else {
        0.0
    }
}

#[allow(dead_code)]
pub fn cone_solve(joint: &mut ConeJoint, current_angle: f32, stiffness: f32) {
    joint.current_angle = current_angle;
    let error = cone_angle_error(joint);
    joint.torque = -error * stiffness;
}

#[allow(dead_code)]
pub fn cone_set_limit(joint: &mut ConeJoint, half_angle: f32) {
    joint.half_angle = half_angle.clamp(0.0, PI);
}

#[allow(dead_code)]
pub fn cone_is_limited(joint: &ConeJoint) -> bool {
    joint.limited
}

#[allow(dead_code)]
pub fn cone_reset(joint: &mut ConeJoint) {
    joint.current_angle = 0.0;
    joint.torque = 0.0;
}

#[allow(dead_code)]
pub fn cone_torque(joint: &ConeJoint) -> f32 {
    joint.torque
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = new_cone_joint(PI / 4.0);
        assert!((cone_half_angle_joint(&j) - PI / 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_no_error_within_limit() {
        let j = new_cone_joint(PI / 2.0);
        assert_eq!(cone_angle_error(&j), 0.0);
    }

    #[test]
    fn test_error_beyond_limit() {
        let mut j = new_cone_joint(0.5);
        j.current_angle = 0.8;
        let err = cone_angle_error(&j);
        assert!((err - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_solve() {
        let mut j = new_cone_joint(0.5);
        cone_solve(&mut j, 0.8, 10.0);
        assert!(cone_torque(&j) < 0.0);
    }

    #[test]
    fn test_solve_within_limit() {
        let mut j = new_cone_joint(1.0);
        cone_solve(&mut j, 0.5, 10.0);
        assert_eq!(cone_torque(&j), 0.0);
    }

    #[test]
    fn test_set_limit() {
        let mut j = new_cone_joint(0.5);
        cone_set_limit(&mut j, 1.0);
        assert!((cone_half_angle_joint(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_limited() {
        let j = new_cone_joint(0.5);
        assert!(cone_is_limited(&j));
    }

    #[test]
    fn test_reset() {
        let mut j = new_cone_joint(0.5);
        cone_solve(&mut j, 1.0, 5.0);
        cone_reset(&mut j);
        assert_eq!(cone_torque(&j), 0.0);
    }

    #[test]
    fn test_clamp_limit() {
        let j = new_cone_joint(10.0);
        assert!((cone_half_angle_joint(&j) - PI).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zero_limit() {
        let j = new_cone_joint(0.0);
        assert_eq!(cone_half_angle_joint(&j), 0.0);
    }
}
