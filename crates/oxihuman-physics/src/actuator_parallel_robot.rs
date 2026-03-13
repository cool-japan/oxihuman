// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Parallel robot (delta) kinematics stub — 3-arm delta robot.

/// Delta robot configuration parameters.
#[derive(Clone, Debug)]
pub struct DeltaRobotParams {
    /// Upper arm length (m).
    pub upper_arm: f32,
    /// Lower arm (forearm) length (m).
    pub lower_arm: f32,
    /// Base radius (m) — attachment points on base.
    pub base_radius: f32,
    /// End-effector radius (m).
    pub ee_radius: f32,
    /// Maximum joint angle (rad) from vertical.
    pub max_joint_angle: f32,
}

impl Default for DeltaRobotParams {
    fn default() -> Self {
        Self {
            upper_arm: 0.3,
            lower_arm: 0.5,
            base_radius: 0.2,
            ee_radius: 0.06,
            max_joint_angle: 1.0,
        }
    }
}

/// End-effector position in 3D.
#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Joint angles for the three arms (radians from vertical).
#[derive(Clone, Copy, Debug, Default)]
pub struct DeltaJointAngles {
    pub theta1: f32,
    pub theta2: f32,
    pub theta3: f32,
}

/// Delta robot state.
#[derive(Clone, Debug, Default)]
pub struct DeltaRobotState {
    pub joints: DeltaJointAngles,
    pub ee_position: Vec3,
    pub velocity: Vec3,
}

/// Computes the end-effector Z position for a single arm given joint angle.
fn arm_z(params: &DeltaRobotParams, theta: f32) -> f32 {
    -(params.upper_arm * theta.cos() + params.lower_arm)
}

/// Stub forward kinematics: computes EE position from joint angles.
pub fn forward_kinematics(params: &DeltaRobotParams, joints: &DeltaJointAngles) -> Vec3 {
    /* Simplified: treat as symmetric delta, project each arm */
    let z1 = arm_z(params, joints.theta1);
    let z2 = arm_z(params, joints.theta2);
    let z3 = arm_z(params, joints.theta3);
    let z_avg = (z1 + z2 + z3) / 3.0;

    /* X and Y offset from theta differences */
    let angles_120 = [
        0.0_f32,
        2.0 * std::f32::consts::PI / 3.0,
        4.0 * std::f32::consts::PI / 3.0,
    ];
    let r = params.base_radius - params.ee_radius;
    let x: f32 = angles_120
        .iter()
        .zip([joints.theta1, joints.theta2, joints.theta3])
        .map(|(&a, t)| a.cos() * params.upper_arm * t.sin())
        .sum::<f32>()
        / 3.0
        * r;
    let y: f32 = angles_120
        .iter()
        .zip([joints.theta1, joints.theta2, joints.theta3])
        .map(|(&a, t)| a.sin() * params.upper_arm * t.sin())
        .sum::<f32>()
        / 3.0
        * r;

    Vec3 { x, y, z: z_avg }
}

/// Sets joint angles (clamped to max_joint_angle).
pub fn set_joint_angles(
    params: &DeltaRobotParams,
    state: &mut DeltaRobotState,
    angles: DeltaJointAngles,
) {
    let clamp = |a: f32| a.clamp(-params.max_joint_angle, params.max_joint_angle);
    state.joints = DeltaJointAngles {
        theta1: clamp(angles.theta1),
        theta2: clamp(angles.theta2),
        theta3: clamp(angles.theta3),
    };
    state.ee_position = forward_kinematics(params, &state.joints);
}

/// Returns the workspace radius at the home position (stub).
pub fn workspace_radius(params: &DeltaRobotParams) -> f32 {
    params.upper_arm + params.lower_arm - params.base_radius - params.ee_radius
}

/// Returns true if the joint angles are within limits.
pub fn joints_in_limits(params: &DeltaRobotParams, joints: &DeltaJointAngles) -> bool {
    joints.theta1.abs() <= params.max_joint_angle
        && joints.theta2.abs() <= params.max_joint_angle
        && joints.theta3.abs() <= params.max_joint_angle
}

/// Returns the end-effector Z height for a symmetric configuration.
pub fn home_z(params: &DeltaRobotParams) -> f32 {
    arm_z(params, 0.0)
}

/// Delta robot stub struct.
pub struct DeltaRobot {
    pub params: DeltaRobotParams,
    pub state: DeltaRobotState,
}

impl DeltaRobot {
    /// Creates a new delta robot with default params.
    pub fn new(params: DeltaRobotParams) -> Self {
        Self {
            state: DeltaRobotState::default(),
            params,
        }
    }

    /// Sets joint angles and updates EE position.
    pub fn set_angles(&mut self, angles: DeltaJointAngles) {
        set_joint_angles(&self.params, &mut self.state, angles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_robot() -> DeltaRobot {
        DeltaRobot::new(DeltaRobotParams::default())
    }

    #[test]
    fn test_forward_kinematics_symmetric_gives_zero_xy() {
        let p = DeltaRobotParams::default();
        let joints = DeltaJointAngles {
            theta1: 0.3,
            theta2: 0.3,
            theta3: 0.3,
        };
        let pos = forward_kinematics(&p, &joints);
        /* symmetric => x and y should be near zero */
        assert!(pos.x.abs() < 1e-4);
        assert!(pos.y.abs() < 1e-4);
    }

    #[test]
    fn test_ee_z_negative_below_base() {
        let p = DeltaRobotParams::default();
        let joints = DeltaJointAngles::default();
        let pos = forward_kinematics(&p, &joints);
        assert!(pos.z < 0.0);
    }

    #[test]
    fn test_set_joint_angles_clamped() {
        let mut r = default_robot();
        r.set_angles(DeltaJointAngles {
            theta1: 10.0,
            theta2: 10.0,
            theta3: 10.0,
        });
        assert!(r.state.joints.theta1 <= r.params.max_joint_angle);
    }

    #[test]
    fn test_joints_in_limits_true_for_small_angles() {
        let p = DeltaRobotParams::default();
        let j = DeltaJointAngles {
            theta1: 0.5,
            theta2: 0.5,
            theta3: 0.5,
        };
        assert!(joints_in_limits(&p, &j));
    }

    #[test]
    fn test_joints_out_of_limits_detected() {
        let p = DeltaRobotParams::default();
        let j = DeltaJointAngles {
            theta1: 2.0,
            theta2: 0.0,
            theta3: 0.0,
        };
        assert!(!joints_in_limits(&p, &j));
    }

    #[test]
    fn test_workspace_radius_positive() {
        let p = DeltaRobotParams::default();
        assert!(workspace_radius(&p) > 0.0);
    }

    #[test]
    fn test_home_z_negative() {
        let p = DeltaRobotParams::default();
        assert!(home_z(&p) < 0.0);
    }

    #[test]
    fn test_ee_position_updated_after_set_angles() {
        let mut r = default_robot();
        r.set_angles(DeltaJointAngles {
            theta1: 0.2,
            theta2: 0.2,
            theta3: 0.2,
        });
        assert!(r.state.ee_position.z < 0.0);
    }

    #[test]
    fn test_different_angles_different_positions() {
        let p = DeltaRobotParams::default();
        let j1 = DeltaJointAngles {
            theta1: 0.2,
            theta2: 0.2,
            theta3: 0.2,
        };
        let j2 = DeltaJointAngles {
            theta1: 0.5,
            theta2: 0.5,
            theta3: 0.5,
        };
        let p1 = forward_kinematics(&p, &j1);
        let p2 = forward_kinematics(&p, &j2);
        assert!((p1.z - p2.z).abs() > 1e-5);
    }
}
