// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tendon-driven finger actuator stub — models tendon tension and joint kinematics.

/// Tendon drive parameters for one finger.
#[derive(Clone, Debug)]
pub struct TendonDriveParams {
    /// Number of finger joints.
    pub num_joints: usize,
    /// Moment arm at each joint (m).
    pub moment_arms: Vec<f32>,
    /// Maximum tendon tension (N).
    pub max_tension: f32,
    /// Tendon stiffness (N/m).
    pub tendon_stiffness: f32,
    /// Joint damping (N·m·s/rad).
    pub joint_damping: f32,
}

impl Default for TendonDriveParams {
    fn default() -> Self {
        Self {
            num_joints: 3,
            moment_arms: vec![0.01, 0.008, 0.006],
            max_tension: 20.0,
            tendon_stiffness: 500.0,
            joint_damping: 0.001,
        }
    }
}

/// Tendon drive state for one finger.
#[derive(Clone, Debug)]
pub struct TendonDriveState {
    /// Joint angles (rad), one per joint.
    pub joint_angles: Vec<f32>,
    /// Joint angular velocities (rad/s).
    pub joint_velocities: Vec<f32>,
    /// Tendon tension (N).
    pub tension: f32,
    /// Tendon displacement (m).
    pub tendon_displacement: f32,
}

impl TendonDriveState {
    /// Creates a new state for n joints.
    pub fn new(n: usize) -> Self {
        Self {
            joint_angles: vec![0.0; n],
            joint_velocities: vec![0.0; n],
            tension: 0.0,
            tendon_displacement: 0.0,
        }
    }
}

/// Creates a new tendon drive state.
pub fn new_tendon_state(params: &TendonDriveParams) -> TendonDriveState {
    TendonDriveState::new(params.num_joints)
}

/// Computes the torque at each joint given tendon tension.
pub fn joint_torques(params: &TendonDriveParams, tension: f32) -> Vec<f32> {
    params
        .moment_arms
        .iter()
        .map(|&r| r * tension.min(params.max_tension))
        .collect()
}

/// Computes tendon displacement from joint angles.
pub fn tendon_displacement(params: &TendonDriveParams, state: &TendonDriveState) -> f32 {
    params
        .moment_arms
        .iter()
        .zip(&state.joint_angles)
        .map(|(&r, &a)| r * a)
        .sum()
}

/// Steps the tendon drive by dt seconds under a commanded tension.
pub fn step_tendon_drive(
    params: &TendonDriveParams,
    state: &mut TendonDriveState,
    commanded_tension: f32,
    load_torques: &[f32],
    dt: f32,
) {
    let tension = commanded_tension.clamp(0.0, params.max_tension);
    state.tension = tension;
    let torques = joint_torques(params, tension);

    for i in 0..params.num_joints {
        let load = if i < load_torques.len() {
            load_torques[i]
        } else {
            0.0
        };
        let net_torque = torques[i] - load - params.joint_damping * state.joint_velocities[i];
        /* stub inertia = 0.001 kg·m² per joint */
        state.joint_velocities[i] += (net_torque / 0.001) * dt;
        state.joint_angles[i] += state.joint_velocities[i] * dt;
    }
    state.tendon_displacement = tendon_displacement(params, state);
}

/// Returns total finger closure (sum of joint angles).
pub fn total_closure(state: &TendonDriveState) -> f32 {
    state.joint_angles.iter().sum()
}

/// Returns whether the finger is grasping (all joints above threshold).
pub fn is_grasping(state: &TendonDriveState, threshold: f32) -> bool {
    state.joint_angles.iter().all(|&a| a >= threshold)
}

/// Tendon drive finger stub struct.
pub struct TendonDriveFinger {
    pub params: TendonDriveParams,
    pub state: TendonDriveState,
}

impl TendonDriveFinger {
    /// Creates a new tendon drive finger with default params.
    pub fn new(params: TendonDriveParams) -> Self {
        let state = new_tendon_state(&params);
        Self { state, params }
    }

    /// Commands tension and steps simulation.
    pub fn actuate(&mut self, tension: f32, load_torques: &[f32], dt: f32) {
        step_tendon_drive(&self.params, &mut self.state, tension, load_torques, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_finger() -> TendonDriveFinger {
        TendonDriveFinger::new(TendonDriveParams::default())
    }

    #[test]
    fn test_initial_state_all_zeros() {
        let f = default_finger();
        assert!(f.state.joint_angles.iter().all(|&a| a.abs() < 1e-9));
    }

    #[test]
    fn test_joint_torques_positive_for_positive_tension() {
        let p = TendonDriveParams::default();
        let t = joint_torques(&p, 10.0);
        assert!(t.iter().all(|&v| v > 0.0));
    }

    #[test]
    fn test_joint_torques_clamped_by_max_tension() {
        let p = TendonDriveParams::default();
        let t = joint_torques(&p, 1000.0); /* over max */
        let t_max = joint_torques(&p, p.max_tension);
        for (a, b) in t.iter().zip(t_max.iter()) {
            assert!((a - b).abs() < 1e-5);
        }
    }

    #[test]
    fn test_finger_closes_under_tension() {
        let mut f = default_finger();
        for _ in 0..20 {
            f.actuate(15.0, &[], 0.005);
        }
        assert!(total_closure(&f.state) > 0.0);
    }

    #[test]
    fn test_is_grasping_false_at_rest() {
        let f = default_finger();
        assert!(!is_grasping(&f.state, 0.01));
    }

    #[test]
    fn test_tendon_displacement_increases_with_closure() {
        let mut f = default_finger();
        for _ in 0..20 {
            f.actuate(15.0, &[], 0.005);
        }
        assert!(f.state.tendon_displacement > 0.0);
    }

    #[test]
    fn test_zero_tension_no_motion() {
        let mut f = default_finger();
        f.actuate(0.0, &[], 0.01);
        assert!(total_closure(&f.state).abs() < 1e-6);
    }

    #[test]
    fn test_state_num_joints_correct() {
        let f = default_finger();
        assert_eq!(f.state.joint_angles.len(), f.params.num_joints);
    }

    #[test]
    fn test_new_tendon_state_size() {
        let p = TendonDriveParams::default();
        let s = new_tendon_state(&p);
        assert_eq!(s.joint_angles.len(), p.num_joints);
    }
}
