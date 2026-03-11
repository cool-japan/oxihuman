// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Harmonic drive gear stub — high-ratio, zero-backlash gear.

/// Harmonic drive parameters.
#[derive(Clone, Debug)]
pub struct HarmonicDriveParams {
    /// Gear ratio (typically 50:1 to 320:1).
    pub ratio: f32,
    /// Mechanical efficiency (0–1).
    pub efficiency: f32,
    /// Maximum input speed (rad/s).
    pub max_input_speed: f32,
    /// Rated output torque (N·m).
    pub rated_torque: f32,
    /// Torsional stiffness (N·m/rad).
    pub torsional_stiffness: f32,
}

impl Default for HarmonicDriveParams {
    fn default() -> Self {
        Self {
            ratio: 100.0,
            efficiency: 0.85,
            max_input_speed: 6000.0,
            rated_torque: 50.0,
            torsional_stiffness: 20_000.0,
        }
    }
}

/// Harmonic drive state.
#[derive(Clone, Debug, Default)]
pub struct HarmonicDriveState {
    /// Input (motor) speed (rad/s).
    pub input_omega: f32,
    /// Output speed (rad/s).
    pub output_omega: f32,
    /// Input torque (N·m).
    pub input_torque: f32,
    /// Output torque (N·m).
    pub output_torque: f32,
    /// Torsional deflection angle at output (rad).
    pub torsional_deflection: f32,
}

/// Computes the output angular velocity from input.
pub fn output_speed(params: &HarmonicDriveParams, input_omega: f32) -> f32 {
    input_omega / params.ratio
}

/// Computes the output torque (amplified by ratio, reduced by efficiency).
pub fn output_torque(params: &HarmonicDriveParams, input_torque: f32) -> f32 {
    (input_torque * params.ratio * params.efficiency).min(params.rated_torque)
}

/// Updates state from current input.
pub fn update_harmonic_drive(params: &HarmonicDriveParams, state: &mut HarmonicDriveState) {
    let clamped_input = state
        .input_omega
        .clamp(-params.max_input_speed, params.max_input_speed);
    state.output_omega = output_speed(params, clamped_input);
    state.output_torque = output_torque(params, state.input_torque);
    /* torsional deflection approximation under load */
    state.torsional_deflection = state.output_torque / params.torsional_stiffness;
}

/// Returns the back-drive torque needed to drive output from input side.
pub fn back_drive_torque(params: &HarmonicDriveParams, output_torque_applied: f32) -> f32 {
    output_torque_applied / (params.ratio * params.efficiency)
}

/// Returns true if the input speed is within the rated range.
pub fn input_speed_valid(params: &HarmonicDriveParams, omega: f32) -> bool {
    omega.abs() <= params.max_input_speed
}

/// Returns the reflected inertia at the input (output inertia / ratio²).
pub fn reflected_inertia(params: &HarmonicDriveParams, output_inertia: f32) -> f32 {
    output_inertia / (params.ratio * params.ratio)
}

/// Harmonic drive actuator stub struct.
pub struct HarmonicDriveActuator {
    pub params: HarmonicDriveParams,
    pub state: HarmonicDriveState,
}

impl HarmonicDriveActuator {
    /// Creates a new harmonic drive actuator with default params.
    pub fn new(params: HarmonicDriveParams) -> Self {
        Self {
            state: HarmonicDriveState::default(),
            params,
        }
    }

    /// Applies motor input and updates output state.
    pub fn apply_input(&mut self, omega: f32, torque: f32) {
        self.state.input_omega = omega;
        self.state.input_torque = torque;
        update_harmonic_drive(&self.params, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_hd() -> HarmonicDriveActuator {
        HarmonicDriveActuator::new(HarmonicDriveParams::default())
    }

    #[test]
    fn test_output_speed_reduced_by_ratio() {
        let p = HarmonicDriveParams::default();
        let s = output_speed(&p, 1000.0);
        assert!((s - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_output_torque_amplified() {
        let p = HarmonicDriveParams::default();
        let t = output_torque(&p, 0.1);
        assert!(t > 0.1);
    }

    #[test]
    fn test_output_torque_clamped_to_rated() {
        let p = HarmonicDriveParams::default();
        let t = output_torque(&p, 1000.0);
        assert!(t <= p.rated_torque);
    }

    #[test]
    fn test_input_speed_valid() {
        let p = HarmonicDriveParams::default();
        assert!(input_speed_valid(&p, 5000.0));
        assert!(!input_speed_valid(&p, 7000.0));
    }

    #[test]
    fn test_reflected_inertia_much_smaller() {
        let p = HarmonicDriveParams::default();
        let ri = reflected_inertia(&p, 1.0);
        assert!(ri < 0.001); /* 1/100² = 0.0001 */
    }

    #[test]
    fn test_back_drive_torque_small() {
        let p = HarmonicDriveParams::default();
        let bdt = back_drive_torque(&p, 50.0);
        assert!(bdt < 1.0);
    }

    #[test]
    fn test_update_sets_output_fields() {
        let mut hd = default_hd();
        hd.apply_input(500.0, 0.3);
        assert!(hd.state.output_omega > 0.0);
        assert!(hd.state.output_torque > 0.0);
    }

    #[test]
    fn test_torsional_deflection_positive_under_load() {
        let mut hd = default_hd();
        hd.apply_input(500.0, 1.0);
        assert!(hd.state.torsional_deflection > 0.0);
    }

    #[test]
    fn test_excessive_input_speed_clamped() {
        let mut hd = default_hd();
        hd.apply_input(100_000.0, 0.1);
        /* output should correspond to clamped input */
        let max_out = hd.params.max_input_speed / hd.params.ratio;
        assert!(hd.state.output_omega <= max_out + 1e-4);
    }
}
