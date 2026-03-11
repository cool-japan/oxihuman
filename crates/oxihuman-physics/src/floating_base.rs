// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Floating-base dynamics stub.

/// State of a floating-base body (6-DOF: position + orientation).
#[derive(Debug, Clone)]
pub struct FloatingBaseState {
    pub position: [f32; 3],
    pub orientation_quat: [f32; 4],
    pub linear_vel: [f32; 3],
    pub angular_vel: [f32; 3],
    pub mass: f32,
}

impl Default for FloatingBaseState {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            orientation_quat: [0.0, 0.0, 0.0, 1.0],
            linear_vel: [0.0; 3],
            angular_vel: [0.0; 3],
            mass: 1.0,
        }
    }
}

impl FloatingBaseState {
    pub fn new(mass: f32) -> Self {
        Self {
            mass: mass.max(1e-6),
            ..Default::default()
        }
    }
}

/// Apply a wrench (force + torque) to the floating base for one time step.
pub fn apply_wrench(state: &mut FloatingBaseState, force: [f32; 3], torque: [f32; 3], dt: f32) {
    /* stub: F = ma → a = F/m */
    let inv_mass = 1.0 / state.mass.max(1e-6);
    for i in 0..3 {
        state.linear_vel[i] += force[i] * inv_mass * dt;
        state.angular_vel[i] += torque[i] * inv_mass * dt;
        state.position[i] += state.linear_vel[i] * dt;
    }
}

/// Normalize the orientation quaternion (stub).
pub fn normalize_quat(q: &mut [f32; 4]) {
    let norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if norm > 1e-8 {
        for v in q.iter_mut() {
            *v /= norm;
        }
    }
}

/// Compute the linear kinetic energy (stub).
pub fn linear_kinetic_energy(state: &FloatingBaseState) -> f32 {
    let v2: f32 = state.linear_vel.iter().map(|v| v * v).sum();
    0.5 * state.mass * v2
}

/// Return whether the body is nearly at rest.
pub fn is_at_rest(state: &FloatingBaseState, tol: f32) -> bool {
    state.linear_vel.iter().all(|v| v.abs() < tol)
        && state.angular_vel.iter().all(|v| v.abs() < tol)
}

/// Reset the body velocity to zero.
pub fn reset_velocity(state: &mut FloatingBaseState) {
    state.linear_vel = [0.0; 3];
    state.angular_vel = [0.0; 3];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_position_is_origin() {
        /* default position is [0,0,0] */
        let s = FloatingBaseState::default();
        assert_eq!(s.position, [0.0; 3]);
    }

    #[test]
    fn test_default_quat_is_identity() {
        /* identity quaternion has w=1 */
        let s = FloatingBaseState::default();
        assert_eq!(s.orientation_quat[3], 1.0);
    }

    #[test]
    fn test_apply_wrench_moves_body() {
        /* force changes position */
        let mut s = FloatingBaseState::new(1.0);
        apply_wrench(&mut s, [1.0, 0.0, 0.0], [0.0; 3], 0.1);
        assert!(s.position[0] > 0.0);
    }

    #[test]
    fn test_kinetic_energy_zero_initially() {
        /* zero velocity → zero energy */
        let s = FloatingBaseState::default();
        assert_eq!(linear_kinetic_energy(&s), 0.0);
    }

    #[test]
    fn test_kinetic_energy_nonzero() {
        /* moving body has energy */
        let mut s = FloatingBaseState::new(2.0);
        apply_wrench(&mut s, [1.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!(linear_kinetic_energy(&s) > 0.0);
    }

    #[test]
    fn test_is_at_rest_true() {
        /* zero velocity is at rest */
        let s = FloatingBaseState::default();
        assert!(is_at_rest(&s, 1e-4));
    }

    #[test]
    fn test_is_at_rest_false() {
        /* moving body is not at rest */
        let mut s = FloatingBaseState::new(1.0);
        s.linear_vel = [1.0, 0.0, 0.0];
        assert!(!is_at_rest(&s, 1e-4));
    }

    #[test]
    fn test_reset_velocity() {
        /* velocity zeroed after reset */
        let mut s = FloatingBaseState::new(1.0);
        s.linear_vel = [5.0; 3];
        reset_velocity(&mut s);
        assert!(is_at_rest(&s, 1e-6));
    }

    #[test]
    fn test_normalize_quat() {
        /* quaternion is normalized */
        let mut q = [2.0f32, 0.0, 0.0, 0.0];
        normalize_quat(&mut q);
        let len2: f32 = q.iter().map(|v| v * v).sum();
        assert!((len2 - 1.0).abs() < 1e-5);
    }
}
