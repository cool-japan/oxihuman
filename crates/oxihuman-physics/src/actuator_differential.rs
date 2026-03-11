// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Differential drive model stub — open differential with two outputs.

/// Differential drive parameters.
#[derive(Clone, Debug)]
pub struct DifferentialParams {
    /// Overall gear ratio from input to differential ring.
    pub input_ratio: f32,
    /// Drive efficiency (0–1).
    pub efficiency: f32,
    /// Maximum torque per output shaft (N·m).
    pub max_output_torque: f32,
    /// Torque bias between left and right (1.0 = equal split).
    pub torque_bias: f32,
}

impl Default for DifferentialParams {
    fn default() -> Self {
        Self {
            input_ratio: 4.0,
            efficiency: 0.95,
            max_output_torque: 200.0,
            torque_bias: 1.0,
        }
    }
}

/// Differential state.
#[derive(Clone, Debug, Default)]
pub struct DifferentialState {
    /// Input (ring gear) angular velocity (rad/s).
    pub input_omega: f32,
    /// Left output angular velocity (rad/s).
    pub left_omega: f32,
    /// Right output angular velocity (rad/s).
    pub right_omega: f32,
    /// Input torque (N·m).
    pub input_torque: f32,
    /// Left output torque (N·m).
    pub left_torque: f32,
    /// Right output torque (N·m).
    pub right_torque: f32,
}

/// Updates the differential state from input, applying steering offset.
pub fn update_differential(
    params: &DifferentialParams,
    state: &mut DifferentialState,
    steering_offset: f32, /* rad/s difference between left and right */
) {
    let ring_omega = state.input_omega / params.input_ratio;
    let ring_torque = state.input_torque * params.input_ratio * params.efficiency;

    /* split torque based on bias */
    let total_bias = 1.0 + params.torque_bias;
    let left_frac = 1.0 / total_bias;
    let right_frac = params.torque_bias / total_bias;

    state.left_omega = ring_omega - steering_offset * 0.5;
    state.right_omega = ring_omega + steering_offset * 0.5;
    state.left_torque = (ring_torque * left_frac).min(params.max_output_torque);
    state.right_torque = (ring_torque * right_frac).min(params.max_output_torque);
}

/// Returns the average output angular velocity.
pub fn average_output_omega(state: &DifferentialState) -> f32 {
    (state.left_omega + state.right_omega) * 0.5
}

/// Returns the total output torque delivered to both shafts.
pub fn total_output_torque(state: &DifferentialState) -> f32 {
    state.left_torque + state.right_torque
}

/// Returns the speed difference between left and right outputs.
pub fn speed_difference(state: &DifferentialState) -> f32 {
    (state.right_omega - state.left_omega).abs()
}

/// Returns the yaw rate implied by the speed difference and wheel base.
pub fn yaw_rate(state: &DifferentialState, track_width: f32) -> f32 {
    if track_width.abs() < 1e-9 {
        0.0
    } else {
        (state.right_omega - state.left_omega) / track_width
    }
}

/// Differential drive stub struct.
pub struct DifferentialDrive {
    pub params: DifferentialParams,
    pub state: DifferentialState,
}

impl DifferentialDrive {
    /// Creates a new differential drive with default params.
    pub fn new(params: DifferentialParams) -> Self {
        Self {
            state: DifferentialState::default(),
            params,
        }
    }

    /// Applies input and updates drive state.
    pub fn apply_input(&mut self, omega: f32, torque: f32, steering_offset: f32) {
        self.state.input_omega = omega;
        self.state.input_torque = torque;
        update_differential(&self.params, &mut self.state, steering_offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_diff() -> DifferentialDrive {
        DifferentialDrive::new(DifferentialParams::default())
    }

    #[test]
    fn test_straight_driving_equal_output_speeds() {
        let mut d = default_diff();
        d.apply_input(100.0, 10.0, 0.0);
        assert!((d.state.left_omega - d.state.right_omega).abs() < 1e-5);
    }

    #[test]
    fn test_steering_offset_creates_speed_difference() {
        let mut d = default_diff();
        d.apply_input(100.0, 10.0, 5.0);
        assert!(speed_difference(&d.state) > 0.0);
    }

    #[test]
    fn test_output_speed_reduced_by_ratio() {
        let mut d = default_diff();
        d.apply_input(100.0, 10.0, 0.0);
        let expected_ring = 100.0 / d.params.input_ratio;
        assert!((average_output_omega(&d.state) - expected_ring).abs() < 1e-4);
    }

    #[test]
    fn test_output_torque_positive() {
        let mut d = default_diff();
        d.apply_input(100.0, 5.0, 0.0);
        assert!(total_output_torque(&d.state) > 0.0);
    }

    #[test]
    fn test_torque_clamped_to_max() {
        let mut d = default_diff();
        d.apply_input(100.0, 10_000.0, 0.0);
        assert!(d.state.left_torque <= d.params.max_output_torque);
        assert!(d.state.right_torque <= d.params.max_output_torque);
    }

    #[test]
    fn test_yaw_rate_zero_for_straight() {
        let mut d = default_diff();
        d.apply_input(100.0, 5.0, 0.0);
        let yr = yaw_rate(&d.state, 1.5);
        assert!(yr.abs() < 1e-5);
    }

    #[test]
    fn test_yaw_rate_nonzero_when_turning() {
        let mut d = default_diff();
        d.apply_input(100.0, 5.0, 3.0);
        let yr = yaw_rate(&d.state, 1.5);
        assert!(yr.abs() > 0.0);
    }

    #[test]
    fn test_yaw_rate_zero_track_width_returns_zero() {
        let d = default_diff();
        assert_eq!(yaw_rate(&d.state, 0.0), 0.0);
    }

    #[test]
    fn test_average_omega_equals_ring_omega_straight() {
        let mut d = default_diff();
        d.apply_input(80.0, 5.0, 0.0);
        let expected = 80.0 / d.params.input_ratio;
        assert!((average_output_omega(&d.state) - expected).abs() < 1e-4);
    }
}
