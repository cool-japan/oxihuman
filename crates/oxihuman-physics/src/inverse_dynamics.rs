// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Inverse dynamics model for a kinematic chain.
#[derive(Debug, Clone)]
pub struct InverseDynamicsModel {
    pub num_joints: usize,
    pub joint_angles: Vec<f32>,
    pub joint_torques: Vec<f32>,
    pub masses: Vec<f32>,
}

/// Create a new InverseDynamicsModel with n joints (all zeros).
pub fn new_inverse_dynamics(n: usize) -> InverseDynamicsModel {
    InverseDynamicsModel {
        num_joints: n,
        joint_angles: vec![0.0; n],
        joint_torques: vec![0.0; n],
        masses: vec![1.0; n],
    }
}

/// Set the angle (radians) for joint i.
pub fn set_joint_angle(m: &mut InverseDynamicsModel, i: usize, angle: f32) {
    if i < m.num_joints {
        m.joint_angles[i] = angle;
    }
}

/// Compute joint torques using simplified static inverse dynamics:
/// tau_i = mass_i * gravity * sin(theta_i).
pub fn compute_torque(m: &mut InverseDynamicsModel, gravity: f32) {
    for i in 0..m.num_joints {
        m.joint_torques[i] = m.masses[i] * gravity * m.joint_angles[i].sin();
    }
}

/// Total joint work for a set of angular displacements: W = sum(tau_i * delta_i).
pub fn total_joint_work(m: &InverseDynamicsModel, delta_angles: &[f32]) -> f32 {
    let n = m.num_joints.min(delta_angles.len());
    (0..n).map(|i| m.joint_torques[i] * delta_angles[i]).sum()
}

/// Maximum absolute joint torque.
pub fn max_joint_torque(m: &InverseDynamicsModel) -> f32 {
    m.joint_torques
        .iter()
        .copied()
        .map(f32::abs)
        .fold(0.0f32, f32::max)
}

/// Set mass for joint link i.
pub fn set_link_mass(m: &mut InverseDynamicsModel, i: usize, mass: f32) {
    if i < m.num_joints {
        m.masses[i] = mass;
    }
}

/// Returns all joint torques as a slice.
pub fn joint_torques_slice(m: &InverseDynamicsModel) -> &[f32] {
    &m.joint_torques
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_new_inverse_dynamics() {
        /* constructor */
        let m = new_inverse_dynamics(3);
        assert_eq!(m.num_joints, 3);
        assert_eq!(m.joint_angles.len(), 3);
        assert_eq!(m.joint_torques.len(), 3);
    }

    #[test]
    fn test_set_joint_angle() {
        let mut m = new_inverse_dynamics(3);
        set_joint_angle(&mut m, 1, FRAC_PI_2);
        assert!((m.joint_angles[1] - FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_set_joint_angle_out_of_bounds() {
        /* no panic for out of bounds */
        let mut m = new_inverse_dynamics(2);
        set_joint_angle(&mut m, 5, 1.0);
    }

    #[test]
    fn test_compute_torque_at_pi_half() {
        /* sin(pi/2) = 1 -> torque = mass * g */
        let mut m = new_inverse_dynamics(1);
        set_joint_angle(&mut m, 0, FRAC_PI_2);
        compute_torque(&mut m, 9.81);
        assert!((m.joint_torques[0] - 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_compute_torque_at_zero() {
        /* sin(0) = 0 -> torque = 0 */
        let mut m = new_inverse_dynamics(1);
        compute_torque(&mut m, 9.81);
        assert!(m.joint_torques[0].abs() < 1e-9);
    }

    #[test]
    fn test_total_joint_work() {
        let mut m = new_inverse_dynamics(2);
        m.joint_torques = vec![10.0, 5.0];
        let w = total_joint_work(&m, &[0.1, 0.2]);
        assert!((w - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_joint_torque() {
        let mut m = new_inverse_dynamics(3);
        m.joint_torques = vec![1.0, -5.0, 3.0];
        let mx = max_joint_torque(&m);
        assert!((mx - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_max_joint_torque_empty() {
        let m = new_inverse_dynamics(0);
        assert!(max_joint_torque(&m).abs() < 1e-9);
    }

    #[test]
    fn test_set_link_mass() {
        let mut m = new_inverse_dynamics(2);
        set_link_mass(&mut m, 0, 5.0);
        assert!((m.masses[0] - 5.0).abs() < 1e-9);
    }
}
