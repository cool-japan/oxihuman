#![allow(dead_code)]

use std::f32::consts::PI;

/// A ball (spherical) joint allowing rotation in all axes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BallJointDef {
    pub anchor_a: [f32; 3],
    pub anchor_b: [f32; 3],
    pub max_angle: f32,
    pub limited: bool,
}

/// Creates a new ball joint between two anchor points.
#[allow(dead_code)]
pub fn new_ball_joint(anchor_a: [f32; 3], anchor_b: [f32; 3]) -> BallJointDef {
    BallJointDef {
        anchor_a,
        anchor_b,
        max_angle: PI,
        limited: false,
    }
}

/// Computes the positional error (distance between anchors).
#[allow(dead_code)]
pub fn ball_error(joint: &BallJointDef) -> f32 {
    let dx = joint.anchor_b[0] - joint.anchor_a[0];
    let dy = joint.anchor_b[1] - joint.anchor_a[1];
    let dz = joint.anchor_b[2] - joint.anchor_a[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Solves the ball joint constraint by moving anchors closer.
#[allow(dead_code)]
pub fn ball_solve(joint: &mut BallJointDef, stiffness: f32) {
    let dx = joint.anchor_b[0] - joint.anchor_a[0];
    let dy = joint.anchor_b[1] - joint.anchor_a[1];
    let dz = joint.anchor_b[2] - joint.anchor_a[2];
    let correction = stiffness * 0.5;
    joint.anchor_a[0] += dx * correction;
    joint.anchor_a[1] += dy * correction;
    joint.anchor_a[2] += dz * correction;
    joint.anchor_b[0] -= dx * correction;
    joint.anchor_b[1] -= dy * correction;
    joint.anchor_b[2] -= dz * correction;
}

/// Returns the maximum allowed angle.
#[allow(dead_code)]
pub fn ball_max_angle(joint: &BallJointDef) -> f32 {
    joint.max_angle
}

/// Sets the maximum cone angle.
#[allow(dead_code)]
pub fn ball_set_max_angle(joint: &mut BallJointDef, angle: f32) {
    joint.max_angle = angle;
    joint.limited = true;
}

/// Returns whether the joint is angle-limited.
#[allow(dead_code)]
pub fn ball_is_limited(joint: &BallJointDef) -> bool {
    joint.limited
}

/// Resets the joint anchors to the origin.
#[allow(dead_code)]
pub fn ball_reset(joint: &mut BallJointDef) {
    joint.anchor_a = [0.0; 3];
    joint.anchor_b = [0.0; 3];
}

/// Returns an approximation of the constraint torque.
#[allow(dead_code)]
pub fn ball_torque(joint: &BallJointDef) -> f32 {
    ball_error(joint) * 10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ball_joint() {
        let j = new_ball_joint([0.0; 3], [1.0, 0.0, 0.0]);
        assert!((ball_error(&j) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ball_solve() {
        let mut j = new_ball_joint([0.0; 3], [2.0, 0.0, 0.0]);
        ball_solve(&mut j, 0.5);
        assert!(ball_error(&j) < 2.0);
    }

    #[test]
    fn test_ball_max_angle() {
        let j = new_ball_joint([0.0; 3], [0.0; 3]);
        assert!((ball_max_angle(&j) - PI).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ball_set_max_angle() {
        let mut j = new_ball_joint([0.0; 3], [0.0; 3]);
        ball_set_max_angle(&mut j, 1.0);
        assert!(ball_is_limited(&j));
        assert!((ball_max_angle(&j) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ball_not_limited() {
        let j = new_ball_joint([0.0; 3], [0.0; 3]);
        assert!(!ball_is_limited(&j));
    }

    #[test]
    fn test_ball_reset() {
        let mut j = new_ball_joint([1.0; 3], [2.0; 3]);
        ball_reset(&mut j);
        assert!(ball_error(&j).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ball_torque() {
        let j = new_ball_joint([0.0; 3], [1.0, 0.0, 0.0]);
        assert!(ball_torque(&j) > 0.0);
    }

    #[test]
    fn test_ball_error_zero() {
        let j = new_ball_joint([5.0; 3], [5.0; 3]);
        assert!(ball_error(&j).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ball_solve_convergence() {
        let mut j = new_ball_joint([0.0; 3], [1.0, 0.0, 0.0]);
        for _ in 0..100 {
            ball_solve(&mut j, 0.1);
        }
        assert!(ball_error(&j) < 0.01);
    }

    #[test]
    fn test_ball_torque_zero() {
        let j = new_ball_joint([0.0; 3], [0.0; 3]);
        assert!(ball_torque(&j).abs() < f32::EPSILON);
    }
}
