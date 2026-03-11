// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Stepper motor step model stub.

/// Stepper motor configuration.
#[derive(Clone, Debug)]
pub struct StepperConfig {
    /// Steps per full revolution.
    pub steps_per_rev: u32,
    /// Step angle in radians.
    pub step_angle: f32,
    /// Maximum step frequency (steps/sec).
    pub max_frequency: f32,
    /// Holding torque (N·m).
    pub holding_torque: f32,
    /// Microstepping factor (1 = full step, 16 = 1/16 microstep).
    pub microstep_factor: u32,
}

impl Default for StepperConfig {
    fn default() -> Self {
        let spr = 200u32;
        Self {
            steps_per_rev: spr,
            step_angle: 2.0 * std::f32::consts::PI / spr as f32,
            max_frequency: 1000.0,
            holding_torque: 0.5,
            microstep_factor: 1,
        }
    }
}

/// Stepper motor state.
#[derive(Clone, Debug, Default)]
pub struct StepperState {
    /// Current position in steps.
    pub position_steps: i64,
    /// Current direction (1 = forward, -1 = backward, 0 = stopped).
    pub direction: i8,
    /// Whether the motor is energized.
    pub energized: bool,
}

/// Creates a new stepper state.
pub fn new_stepper_state() -> StepperState {
    StepperState {
        position_steps: 0,
        direction: 0,
        energized: true,
    }
}

/// Steps the motor by the given number of steps (positive = forward, negative = back).
pub fn step_motor(state: &mut StepperState, steps: i64) {
    if !state.energized {
        return;
    }
    state.position_steps += steps;
    state.direction = if steps > 0 {
        1
    } else if steps < 0 {
        -1
    } else {
        0
    };
}

/// Returns the current angular position in radians.
pub fn current_angle_rad(config: &StepperConfig, state: &StepperState) -> f32 {
    let steps_per_microstep = config.microstep_factor as f32;
    state.position_steps as f32 * config.step_angle / steps_per_microstep
}

/// Returns the number of steps needed to reach the target angle.
pub fn steps_to_angle(config: &StepperConfig, target_rad: f32) -> i64 {
    let microsteps_per_rev = config.steps_per_rev * config.microstep_factor;
    (target_rad / (2.0 * std::f32::consts::PI) * microsteps_per_rev as f32).round() as i64
}

/// Moves the stepper to the target angle from its current position.
pub fn move_to_angle(config: &StepperConfig, state: &mut StepperState, target_rad: f32) {
    let target_steps = steps_to_angle(config, target_rad);
    let delta = target_steps - state.position_steps;
    step_motor(state, delta);
}

/// De-energizes the motor (no holding torque).
pub fn de_energize(state: &mut StepperState) {
    state.energized = false;
}

/// Re-energizes the motor.
pub fn energize(state: &mut StepperState) {
    state.energized = true;
}

/// Returns the effective step angle considering microstepping.
pub fn effective_step_angle(config: &StepperConfig) -> f32 {
    config.step_angle / config.microstep_factor as f32
}

/// Stepper motor stub struct.
pub struct StepperMotor {
    pub config: StepperConfig,
    pub state: StepperState,
}

impl StepperMotor {
    /// Creates a new stepper motor with default config.
    pub fn new(config: StepperConfig) -> Self {
        Self {
            state: new_stepper_state(),
            config,
        }
    }

    /// Steps by given count.
    pub fn step(&mut self, steps: i64) {
        step_motor(&mut self.state, steps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_stepper() -> StepperMotor {
        StepperMotor::new(StepperConfig::default())
    }

    #[test]
    fn test_step_forward_increases_position() {
        let mut m = default_stepper();
        m.step(10);
        assert_eq!(m.state.position_steps, 10);
    }

    #[test]
    fn test_step_backward_decreases_position() {
        let mut m = default_stepper();
        m.step(-5);
        assert_eq!(m.state.position_steps, -5);
    }

    #[test]
    fn test_direction_updated_correctly() {
        let mut m = default_stepper();
        m.step(3);
        assert_eq!(m.state.direction, 1);
        m.step(-3);
        assert_eq!(m.state.direction, -1);
    }

    #[test]
    fn test_de_energize_blocks_stepping() {
        let mut m = default_stepper();
        de_energize(&mut m.state);
        m.step(10);
        assert_eq!(m.state.position_steps, 0);
    }

    #[test]
    fn test_reenergize_allows_stepping() {
        let mut m = default_stepper();
        de_energize(&mut m.state);
        energize(&mut m.state);
        m.step(5);
        assert_eq!(m.state.position_steps, 5);
    }

    #[test]
    fn test_current_angle_zero_at_start() {
        let m = default_stepper();
        let angle = current_angle_rad(&m.config, &m.state);
        assert!((angle).abs() < 1e-6);
    }

    #[test]
    fn test_move_to_angle_reaches_target() {
        let mut m = default_stepper();
        let target = std::f32::consts::PI;
        move_to_angle(&m.config, &mut m.state, target);
        /* should be at 100 steps = half revolution */
        assert_eq!(m.state.position_steps, 100);
    }

    #[test]
    fn test_effective_step_angle_with_microstepping() {
        let spr = 200u32;
        let cfg = StepperConfig {
            steps_per_rev: spr,
            step_angle: 2.0 * std::f32::consts::PI / spr as f32,
            max_frequency: 1000.0,
            holding_torque: 0.5,
            microstep_factor: 16,
        };
        let esa = effective_step_angle(&cfg);
        assert!((esa - cfg.step_angle / 16.0).abs() < 1e-7);
    }

    #[test]
    fn test_steps_to_angle_zero_is_zero() {
        let cfg = StepperConfig::default();
        assert_eq!(steps_to_angle(&cfg, 0.0), 0);
    }
}
