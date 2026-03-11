#![allow(dead_code)]

use std::f32::consts::PI;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UniversalJoint {
    angle1: f32,
    angle2: f32,
    limit1: f32,
    limit2: f32,
    torque: f32,
    limited: bool,
}

#[allow(dead_code)]
pub fn new_universal_joint(limit1: f32, limit2: f32) -> UniversalJoint {
    UniversalJoint {
        angle1: 0.0,
        angle2: 0.0,
        limit1: limit1.clamp(0.0, PI),
        limit2: limit2.clamp(0.0, PI),
        torque: 0.0,
        limited: true,
    }
}

#[allow(dead_code)]
pub fn universal_angle_1(joint: &UniversalJoint) -> f32 {
    joint.angle1
}

#[allow(dead_code)]
pub fn universal_angle_2(joint: &UniversalJoint) -> f32 {
    joint.angle2
}

#[allow(dead_code)]
pub fn universal_solve(joint: &mut UniversalJoint, a1: f32, a2: f32, stiffness: f32) {
    let c1 = if joint.limited {
        a1.clamp(-joint.limit1, joint.limit1)
    } else {
        a1
    };
    let c2 = if joint.limited {
        a2.clamp(-joint.limit2, joint.limit2)
    } else {
        a2
    };
    let err1 = a1 - c1;
    let err2 = a2 - c2;
    joint.torque = -(err1 + err2) * stiffness;
    joint.angle1 = c1;
    joint.angle2 = c2;
}

#[allow(dead_code)]
pub fn universal_set_limits(joint: &mut UniversalJoint, l1: f32, l2: f32) {
    joint.limit1 = l1.clamp(0.0, PI);
    joint.limit2 = l2.clamp(0.0, PI);
}

#[allow(dead_code)]
pub fn universal_torque(joint: &UniversalJoint) -> f32 {
    joint.torque
}

#[allow(dead_code)]
pub fn universal_reset(joint: &mut UniversalJoint) {
    joint.angle1 = 0.0;
    joint.angle2 = 0.0;
    joint.torque = 0.0;
}

#[allow(dead_code)]
pub fn universal_is_limited(joint: &UniversalJoint) -> bool {
    joint.limited
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let j = new_universal_joint(PI / 4.0, PI / 4.0);
        assert_eq!(universal_angle_1(&j), 0.0);
        assert_eq!(universal_angle_2(&j), 0.0);
    }

    #[test]
    fn test_solve_within_limits() {
        let mut j = new_universal_joint(1.0, 1.0);
        universal_solve(&mut j, 0.5, 0.5, 10.0);
        assert!((universal_angle_1(&j) - 0.5).abs() < f32::EPSILON);
        assert_eq!(universal_torque(&j), 0.0);
    }

    #[test]
    fn test_solve_clamped() {
        let mut j = new_universal_joint(0.5, 0.5);
        universal_solve(&mut j, 1.0, 1.0, 10.0);
        assert!((universal_angle_1(&j) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_torque_on_violation() {
        let mut j = new_universal_joint(0.5, 0.5);
        universal_solve(&mut j, 1.0, 0.0, 10.0);
        assert!(universal_torque(&j) < 0.0);
    }

    #[test]
    fn test_set_limits() {
        let mut j = new_universal_joint(0.5, 0.5);
        universal_set_limits(&mut j, 1.0, 1.0);
        universal_solve(&mut j, 0.8, 0.8, 10.0);
        assert_eq!(universal_torque(&j), 0.0);
    }

    #[test]
    fn test_reset() {
        let mut j = new_universal_joint(1.0, 1.0);
        universal_solve(&mut j, 0.5, 0.5, 10.0);
        universal_reset(&mut j);
        assert_eq!(universal_angle_1(&j), 0.0);
    }

    #[test]
    fn test_is_limited() {
        let j = new_universal_joint(1.0, 1.0);
        assert!(universal_is_limited(&j));
    }

    #[test]
    fn test_negative_angles() {
        let mut j = new_universal_joint(1.0, 1.0);
        universal_solve(&mut j, -0.5, -0.5, 10.0);
        assert!((universal_angle_1(&j) - (-0.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_limit_clamp_pi() {
        let j = new_universal_joint(10.0, 10.0);
        assert!((j.limit1 - PI).abs() < f32::EPSILON);
    }

    #[test]
    fn test_zero_stiffness() {
        let mut j = new_universal_joint(0.5, 0.5);
        universal_solve(&mut j, 1.0, 1.0, 0.0);
        assert_eq!(universal_torque(&j), 0.0);
    }
}
