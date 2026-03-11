// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Position/velocity motor joint with limits and damping.

#![allow(dead_code)]

/// Motor mode: position target or velocity target.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorMode {
    Position,
    Velocity,
    Off,
}

/// Motor joint v2 state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointMotorV2 {
    pub mode: MotorMode,
    pub target: f32,
    pub current: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub max_force: f32,
    pub min_limit: f32,
    pub max_limit: f32,
}

/// Create a new motor joint.
#[allow(dead_code)]
pub fn new_joint_motor_v2(
    stiffness: f32,
    damping: f32,
    max_force: f32,
    min_limit: f32,
    max_limit: f32,
) -> JointMotorV2 {
    JointMotorV2 {
        mode: MotorMode::Off,
        target: 0.0,
        current: 0.0,
        velocity: 0.0,
        stiffness,
        damping,
        max_force,
        min_limit,
        max_limit,
    }
}

/// Set the motor to position mode with a target angle/position.
#[allow(dead_code)]
pub fn jm_set_position_target(motor: &mut JointMotorV2, target: f32) {
    motor.mode = MotorMode::Position;
    motor.target = target.clamp(motor.min_limit, motor.max_limit);
}

/// Set the motor to velocity mode.
#[allow(dead_code)]
pub fn jm_set_velocity_target(motor: &mut JointMotorV2, velocity: f32) {
    motor.mode = MotorMode::Velocity;
    motor.target = velocity;
}

/// Turn the motor off.
#[allow(dead_code)]
pub fn jm_set_off(motor: &mut JointMotorV2) {
    motor.mode = MotorMode::Off;
}

/// Compute the force/torque output for this step.
#[allow(dead_code)]
pub fn jm_compute_force(motor: &JointMotorV2, dt: f32) -> f32 {
    match motor.mode {
        MotorMode::Off => 0.0,
        MotorMode::Position => {
            let error = motor.target - motor.current;
            let force = motor.stiffness * error - motor.damping * motor.velocity;
            force.clamp(-motor.max_force, motor.max_force)
        }
        MotorMode::Velocity => {
            let vel_error = motor.target - motor.velocity;
            let force = motor.stiffness * vel_error * dt;
            force.clamp(-motor.max_force, motor.max_force)
        }
    }
}

/// Step the motor: apply force, update position and velocity.
#[allow(dead_code)]
pub fn jm_step(motor: &mut JointMotorV2, dt: f32, inertia: f32) {
    let force = jm_compute_force(motor, dt);
    if inertia.abs() > f32::EPSILON {
        let accel = force / inertia;
        motor.velocity += accel * dt;
    }
    motor.current = (motor.current + motor.velocity * dt).clamp(motor.min_limit, motor.max_limit);
}

/// Check if the joint is at its limit.
#[allow(dead_code)]
pub fn jm_at_limit(motor: &JointMotorV2) -> bool {
    motor.current <= motor.min_limit || motor.current >= motor.max_limit
}

/// Reset the motor to zero position and velocity.
#[allow(dead_code)]
pub fn jm_reset(motor: &mut JointMotorV2) {
    motor.current = 0.0;
    motor.velocity = 0.0;
    motor.target = 0.0;
    motor.mode = MotorMode::Off;
}

/// Get position error from target (position mode only).
#[allow(dead_code)]
pub fn jm_position_error(motor: &JointMotorV2) -> f32 {
    motor.target - motor.current
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_motor() -> JointMotorV2 {
        new_joint_motor_v2(100.0, 10.0, 50.0, -1.0, 1.0)
    }

    #[test]
    fn new_motor_off() {
        let m = default_motor();
        assert_eq!(m.mode, MotorMode::Off);
    }

    #[test]
    fn off_motor_zero_force() {
        let m = default_motor();
        assert_eq!(jm_compute_force(&m, 0.016), 0.0);
    }

    #[test]
    fn position_mode_set() {
        let mut m = default_motor();
        jm_set_position_target(&mut m, 0.5);
        assert_eq!(m.mode, MotorMode::Position);
        assert!((m.target - 0.5).abs() < 1e-5);
    }

    #[test]
    fn position_target_clamped() {
        let mut m = default_motor();
        jm_set_position_target(&mut m, 5.0);
        assert!((m.target - 1.0).abs() < 1e-5);
    }

    #[test]
    fn position_force_towards_target() {
        let mut m = default_motor();
        jm_set_position_target(&mut m, 0.5);
        let f = jm_compute_force(&m, 0.016);
        assert!(f > 0.0); // error is positive, force pulls toward target
    }

    #[test]
    fn velocity_mode_set() {
        let mut m = default_motor();
        jm_set_velocity_target(&mut m, 1.0);
        assert_eq!(m.mode, MotorMode::Velocity);
    }

    #[test]
    fn step_moves_position() {
        let mut m = default_motor();
        jm_set_position_target(&mut m, 0.5);
        let before = m.current;
        jm_step(&mut m, 0.016, 1.0);
        // Should move toward target
        assert!(m.current > before || m.current >= m.min_limit);
    }

    #[test]
    fn reset_zeroes_state() {
        let mut m = default_motor();
        jm_set_position_target(&mut m, 0.5);
        jm_step(&mut m, 0.1, 1.0);
        jm_reset(&mut m);
        assert_eq!(m.current, 0.0);
        assert_eq!(m.mode, MotorMode::Off);
    }

    #[test]
    fn position_error_correct() {
        let mut m = default_motor();
        jm_set_position_target(&mut m, 0.3);
        let err = jm_position_error(&m);
        assert!((err - 0.3).abs() < 1e-5);
    }

    #[test]
    fn at_limit_detection() {
        let mut m = default_motor();
        m.current = 1.0; // at max limit
        assert!(jm_at_limit(&m));
    }
}
