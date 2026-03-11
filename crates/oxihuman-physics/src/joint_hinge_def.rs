#![allow(dead_code)]

use std::f32::consts::PI;

/// A hinge joint that constrains rotation to a single axis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HingeJointDef {
    pub anchor: [f32; 3],
    pub axis: [f32; 3],
    pub angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub limited: bool,
}

/// Creates a new hinge joint at the given anchor along the given axis.
#[allow(dead_code)]
pub fn new_hinge_joint_def(anchor: [f32; 3], axis: [f32; 3]) -> HingeJointDef {
    HingeJointDef {
        anchor,
        axis,
        angle: 0.0,
        angular_velocity: 0.0,
        min_angle: -PI,
        max_angle: PI,
        limited: false,
    }
}

/// Returns the current angle.
#[allow(dead_code)]
pub fn hinge_angle_def(joint: &HingeJointDef) -> f32 {
    joint.angle
}

/// Returns the current angular velocity.
#[allow(dead_code)]
pub fn hinge_angular_velocity_def(joint: &HingeJointDef) -> f32 {
    joint.angular_velocity
}

/// Computes torque needed to maintain constraints.
#[allow(dead_code)]
pub fn hinge_torque_def(joint: &HingeJointDef) -> f32 {
    if joint.limited {
        if joint.angle < joint.min_angle {
            (joint.min_angle - joint.angle) * 100.0
        } else if joint.angle > joint.max_angle {
            (joint.max_angle - joint.angle) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Sets angle limits and enables limiting.
#[allow(dead_code)]
pub fn hinge_set_limits_def(joint: &mut HingeJointDef, min_angle: f32, max_angle: f32) {
    joint.min_angle = min_angle;
    joint.max_angle = max_angle;
    joint.limited = true;
}

/// Solves one step of the hinge constraint.
#[allow(dead_code)]
pub fn hinge_solve_def(joint: &mut HingeJointDef, dt: f32) {
    joint.angle += joint.angular_velocity * dt;
    if joint.limited {
        joint.angle = joint.angle.clamp(joint.min_angle, joint.max_angle);
    }
}

/// Returns whether limits are active.
#[allow(dead_code)]
pub fn hinge_is_limited_def(joint: &HingeJointDef) -> bool {
    joint.limited
}

/// Resets the joint to its initial state.
#[allow(dead_code)]
pub fn hinge_reset_def(joint: &mut HingeJointDef) {
    joint.angle = 0.0;
    joint.angular_velocity = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hinge() {
        let j = new_hinge_joint_def([0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((hinge_angle_def(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_angular_velocity() {
        let j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        assert!((hinge_angular_velocity_def(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_limits() {
        let mut j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        hinge_set_limits_def(&mut j, -0.5, 0.5);
        assert!(hinge_is_limited_def(&j));
    }

    #[test]
    fn test_solve() {
        let mut j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        j.angular_velocity = 1.0;
        hinge_solve_def(&mut j, 0.1);
        assert!((hinge_angle_def(&j) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_solve_clamped() {
        let mut j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        hinge_set_limits_def(&mut j, -0.5, 0.5);
        j.angular_velocity = 100.0;
        hinge_solve_def(&mut j, 1.0);
        assert!((hinge_angle_def(&j) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_torque_no_limits() {
        let j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        assert!((hinge_torque_def(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_torque_at_limit() {
        let mut j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        hinge_set_limits_def(&mut j, -0.5, 0.5);
        j.angle = 1.0;
        let t = hinge_torque_def(&j);
        assert!(t < 0.0);
    }

    #[test]
    fn test_reset() {
        let mut j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        j.angle = 1.0;
        j.angular_velocity = 2.0;
        hinge_reset_def(&mut j);
        assert!((hinge_angle_def(&j)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_not_limited_default() {
        let j = new_hinge_joint_def([0.0; 3], [0.0, 1.0, 0.0]);
        assert!(!hinge_is_limited_def(&j));
    }

    #[test]
    fn test_anchor() {
        let j = new_hinge_joint_def([1.0, 2.0, 3.0], [0.0, 1.0, 0.0]);
        assert!((j.anchor[0] - 1.0).abs() < f32::EPSILON);
    }
}
