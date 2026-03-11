// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! RC servo position controller stub.

/// Servo configuration.
#[derive(Clone, Debug)]
pub struct ServoConfig {
    /// Minimum angle (radians).
    pub min_angle: f32,
    /// Maximum angle (radians).
    pub max_angle: f32,
    /// Maximum angular velocity (rad/s).
    pub max_speed: f32,
    /// Proportional gain for position control.
    pub kp: f32,
    /// Maximum torque output (N·m).
    pub max_torque: f32,
}

impl Default for ServoConfig {
    fn default() -> Self {
        Self {
            min_angle: -std::f32::consts::FRAC_PI_2,
            max_angle: std::f32::consts::FRAC_PI_2,
            max_speed: 5.0,
            kp: 10.0,
            max_torque: 2.0,
        }
    }
}

/// Servo state.
#[derive(Clone, Debug, Default)]
pub struct ServoState {
    pub current_angle: f32,
    pub target_angle: f32,
    pub velocity: f32,
}

/// Creates a servo state at a given initial angle.
pub fn new_servo_state(initial_angle: f32) -> ServoState {
    ServoState {
        current_angle: initial_angle,
        target_angle: initial_angle,
        velocity: 0.0,
    }
}

/// Sets the servo target angle (clamped to min/max).
pub fn set_target(config: &ServoConfig, state: &mut ServoState, angle: f32) {
    state.target_angle = angle.clamp(config.min_angle, config.max_angle);
}

/// Computes the torque needed to move toward the target angle.
pub fn compute_servo_torque(config: &ServoConfig, state: &ServoState) -> f32 {
    let error = state.target_angle - state.current_angle;
    let torque = config.kp * error;
    torque.clamp(-config.max_torque, config.max_torque)
}

/// Steps the servo simulation by dt seconds.
pub fn step_servo(config: &ServoConfig, state: &mut ServoState, dt: f32) {
    let torque = compute_servo_torque(config, state);
    /* simple first-order velocity response */
    state.velocity = (torque).clamp(-config.max_speed, config.max_speed);
    state.current_angle += state.velocity * dt;
    state.current_angle = state
        .current_angle
        .clamp(config.min_angle, config.max_angle);
}

/// Returns the angular error between target and current.
pub fn angle_error(state: &ServoState) -> f32 {
    state.target_angle - state.current_angle
}

/// Returns true if the servo has reached the target within tolerance.
pub fn at_target(state: &ServoState, tolerance: f32) -> bool {
    angle_error(state).abs() <= tolerance
}

/// RC servo stub struct.
pub struct RcServo {
    pub config: ServoConfig,
    pub state: ServoState,
}

impl RcServo {
    /// Creates a new servo with default config.
    pub fn new(config: ServoConfig) -> Self {
        Self {
            state: new_servo_state(0.0),
            config,
        }
    }

    /// Moves toward the target by one dt step.
    pub fn step(&mut self, dt: f32) {
        step_servo(&self.config, &mut self.state, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_servo() -> RcServo {
        RcServo::new(ServoConfig::default())
    }

    #[test]
    fn test_set_target_clamped_to_max() {
        let mut s = default_servo();
        set_target(&s.config, &mut s.state, 10.0); /* beyond max_angle */
        assert!((s.state.target_angle - s.config.max_angle).abs() < 1e-5);
    }

    #[test]
    fn test_set_target_clamped_to_min() {
        let mut s = default_servo();
        set_target(&s.config, &mut s.state, -10.0);
        assert!((s.state.target_angle - s.config.min_angle).abs() < 1e-5);
    }

    #[test]
    fn test_compute_torque_positive_when_target_greater() {
        let mut s = default_servo();
        s.state.current_angle = 0.0;
        s.state.target_angle = 1.0;
        let t = compute_servo_torque(&s.config, &s.state);
        assert!(t > 0.0);
    }

    #[test]
    fn test_torque_clamped_to_max() {
        let mut s = default_servo();
        s.state.current_angle = -1.5;
        s.state.target_angle = 1.5;
        let t = compute_servo_torque(&s.config, &s.state);
        assert!(t <= s.config.max_torque);
    }

    #[test]
    fn test_servo_moves_toward_target() {
        let mut s = default_servo();
        set_target(&s.config, &mut s.state, 1.0);
        step_servo(&s.config, &mut s.state, 0.1);
        assert!(s.state.current_angle > 0.0);
    }

    #[test]
    fn test_at_target_initially() {
        let s = default_servo();
        assert!(at_target(&s.state, 0.001));
    }

    #[test]
    fn test_angle_error_zero_at_target() {
        let s = default_servo();
        assert!((angle_error(&s.state)).abs() < 1e-6);
    }

    #[test]
    fn test_angle_stays_within_limits() {
        let mut s = default_servo();
        s.state.target_angle = s.config.max_angle;
        for _ in 0..100 {
            step_servo(&s.config, &mut s.state, 0.1);
        }
        assert!(s.state.current_angle <= s.config.max_angle + 1e-5);
    }

    #[test]
    fn test_new_servo_state_angle() {
        let state = new_servo_state(0.5);
        assert!((state.current_angle - 0.5).abs() < 1e-6);
    }
}
