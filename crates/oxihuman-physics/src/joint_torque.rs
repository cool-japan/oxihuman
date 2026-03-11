// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Joint torque and moment arm model for musculoskeletal simulation.

/// A joint torque actuator.
#[derive(Debug, Clone)]
pub struct JointTorque {
    /// Joint angle (radians).
    pub angle: f32,
    /// Angular velocity (rad/s).
    pub angular_velocity: f32,
    /// Applied torque (N·m).
    pub torque: f32,
    /// Moment of inertia (kg·m²).
    pub inertia: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Joint limits [min, max] radians.
    pub limits: [f32; 2],
}

impl JointTorque {
    pub fn new(inertia: f32, limits: [f32; 2]) -> Self {
        JointTorque {
            angle: 0.0,
            angular_velocity: 0.0,
            torque: 0.0,
            inertia,
            damping: 0.1,
            limits,
        }
    }
}

/// Create a new joint torque model.
pub fn new_joint_torque(inertia: f32, limits: [f32; 2]) -> JointTorque {
    JointTorque::new(inertia, limits)
}

/// Set the applied torque.
pub fn jt_set_torque(j: &mut JointTorque, torque: f32) {
    j.torque = torque;
}

/// Integrate joint state by `dt`.
pub fn jt_step(j: &mut JointTorque, dt: f32) {
    let net = j.torque - j.damping * j.angular_velocity;
    let alpha = net / j.inertia.max(1e-10);
    j.angular_velocity += alpha * dt;
    j.angle += j.angular_velocity * dt;
    /* enforce limits */
    if j.angle < j.limits[0] {
        j.angle = j.limits[0];
        j.angular_velocity = j.angular_velocity.max(0.0);
    } else if j.angle > j.limits[1] {
        j.angle = j.limits[1];
        j.angular_velocity = j.angular_velocity.min(0.0);
    }
}

/// Return the moment arm for a muscle at this joint given `moment_arm_len` (m).
pub fn jt_moment_arm(moment_arm_len: f32) -> f32 {
    moment_arm_len.abs()
}

/// Compute the torque produced by a muscle force `f` (N) and moment arm `r` (m).
pub fn jt_torque_from_force(f: f32, r: f32) -> f32 {
    f * r
}

/// Return `true` if the joint is within its angular limits.
pub fn jt_in_limits(j: &JointTorque) -> bool {
    j.angle >= j.limits[0] && j.angle <= j.limits[1]
}

/// Return joint stiffness torque (restoring force toward 0).
pub fn jt_stiffness_torque(j: &JointTorque, stiffness: f32) -> f32 {
    -stiffness * j.angle
}

/// Compute rotational kinetic energy.
pub fn jt_kinetic_energy(j: &JointTorque) -> f32 {
    0.5 * j.inertia * j.angular_velocity * j.angular_velocity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_joint_at_rest() {
        let j = new_joint_torque(1.0, [-1.57, 1.57]);
        assert_eq!(j.angle, 0.0);
        assert_eq!(j.angular_velocity, 0.0);
    }

    #[test]
    fn test_torque_accelerates_joint() {
        let mut j = new_joint_torque(1.0, [-10.0, 10.0]);
        jt_set_torque(&mut j, 10.0);
        jt_step(&mut j, 0.1);
        assert!(j.angular_velocity > 0.0);
    }

    #[test]
    fn test_angle_changes_with_velocity() {
        let mut j = new_joint_torque(1.0, [-10.0, 10.0]);
        jt_set_torque(&mut j, 10.0);
        jt_step(&mut j, 0.1);
        jt_step(&mut j, 0.1);
        assert!(j.angle > 0.0);
    }

    #[test]
    fn test_limits_enforced() {
        let mut j = new_joint_torque(1.0, [-0.1, 0.1]);
        jt_set_torque(&mut j, 1000.0);
        for _ in 0..100 {
            jt_step(&mut j, 0.1);
        }
        assert!(jt_in_limits(&j));
    }

    #[test]
    fn test_torque_from_force() {
        let t = jt_torque_from_force(100.0, 0.05);
        assert!((t - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_stiffness_torque_sign() {
        let mut j = new_joint_torque(1.0, [-10.0, 10.0]);
        j.angle = 0.5;
        assert!(jt_stiffness_torque(&j, 10.0) < 0.0);
    }

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let j = new_joint_torque(2.0, [-1.57, 1.57]);
        assert!((jt_kinetic_energy(&j) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_kinetic_energy_positive_when_spinning() {
        let mut j = new_joint_torque(2.0, [-100.0, 100.0]);
        j.angular_velocity = 3.0;
        assert!(jt_kinetic_energy(&j) > 0.0);
    }

    #[test]
    fn test_moment_arm_absolute() {
        assert!((jt_moment_arm(-0.05) - 0.05).abs() < 1e-6);
    }
}
