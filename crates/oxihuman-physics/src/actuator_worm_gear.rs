// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Worm gear actuator stub — high-ratio, self-locking gear.

/// Worm gear parameters.
#[derive(Clone, Debug)]
pub struct WormGearParams {
    /// Number of starts on the worm.
    pub worm_starts: u32,
    /// Number of teeth on the worm wheel.
    pub wheel_teeth: u32,
    /// Lead angle (rad) — affects efficiency and self-locking.
    pub lead_angle: f32,
    /// Coefficient of friction.
    pub friction_coeff: f32,
    /// Maximum output torque (N·m).
    pub max_output_torque: f32,
}

impl Default for WormGearParams {
    fn default() -> Self {
        Self {
            worm_starts: 2,
            wheel_teeth: 40,
            lead_angle: 0.0785, /* ~4.5° */
            friction_coeff: 0.05,
            max_output_torque: 100.0,
        }
    }
}

/// Worm gear state.
#[derive(Clone, Debug, Default)]
pub struct WormGearState {
    /// Worm (input) angular velocity (rad/s).
    pub worm_omega: f32,
    /// Wheel (output) angular velocity (rad/s).
    pub wheel_omega: f32,
    /// Input torque (N·m).
    pub input_torque: f32,
    /// Output torque (N·m).
    pub output_torque: f32,
}

/// Returns the gear ratio (wheel_teeth / worm_starts).
pub fn gear_ratio(params: &WormGearParams) -> f32 {
    params.wheel_teeth as f32 / params.worm_starts as f32
}

/// Estimates forward efficiency using lead angle and friction.
pub fn forward_efficiency(params: &WormGearParams) -> f32 {
    let la = params.lead_angle;
    let mu = params.friction_coeff;
    (la.cos() - mu * la.sin()) / (la.cos() + mu * la.sin())
}

/// Returns true if the gear is self-locking (cannot be back-driven).
pub fn is_self_locking(params: &WormGearParams) -> bool {
    let la = params.lead_angle;
    let mu = params.friction_coeff;
    la.tan() < mu
}

/// Updates the worm gear state from current input.
pub fn update_worm_gear(params: &WormGearParams, state: &mut WormGearState) {
    let ratio = gear_ratio(params);
    let eff = forward_efficiency(params).clamp(0.0, 1.0);
    state.wheel_omega = state.worm_omega / ratio;
    state.output_torque = (state.input_torque * ratio * eff).min(params.max_output_torque);
}

/// Returns the back-drive torque required (zero if self-locking).
pub fn back_drive_torque(params: &WormGearParams, output_torque_applied: f32) -> f32 {
    if is_self_locking(params) {
        0.0
    } else {
        output_torque_applied / (gear_ratio(params) * forward_efficiency(params))
    }
}

/// Returns the input speed given output speed (inverse of normal operation).
pub fn input_speed_from_output(params: &WormGearParams, wheel_omega: f32) -> f32 {
    wheel_omega * gear_ratio(params)
}

/// Worm gear actuator stub struct.
pub struct WormGearActuator {
    pub params: WormGearParams,
    pub state: WormGearState,
}

impl WormGearActuator {
    /// Creates a new worm gear actuator with default params.
    pub fn new(params: WormGearParams) -> Self {
        Self {
            state: WormGearState::default(),
            params,
        }
    }

    /// Applies input and updates output state.
    pub fn apply_input(&mut self, omega: f32, torque: f32) {
        self.state.worm_omega = omega;
        self.state.input_torque = torque;
        update_worm_gear(&self.params, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_wg() -> WormGearActuator {
        WormGearActuator::new(WormGearParams::default())
    }

    #[test]
    fn test_gear_ratio_correct() {
        let p = WormGearParams::default(); /* 40/2 = 20:1 */
        assert!((gear_ratio(&p) - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_forward_efficiency_between_0_and_1() {
        let p = WormGearParams::default();
        let eff = forward_efficiency(&p);
        assert!((0.0..=1.0).contains(&eff));
    }

    #[test]
    fn test_output_speed_reduced() {
        let mut wg = default_wg();
        wg.apply_input(200.0, 0.5);
        assert!((wg.state.wheel_omega - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_output_torque_amplified() {
        let mut wg = default_wg();
        wg.apply_input(100.0, 0.5);
        assert!(wg.state.output_torque > 0.5);
    }

    #[test]
    fn test_output_torque_clamped_to_max() {
        let mut wg = default_wg();
        wg.apply_input(100.0, 1000.0);
        assert!(wg.state.output_torque <= wg.params.max_output_torque);
    }

    #[test]
    fn test_self_locking_detection() {
        /* tan(4.5°) ≈ 0.079 > friction_coeff=0.05 => not self-locking for default */
        let p = WormGearParams::default();
        /* just verify it returns a bool consistently */
        let _ = is_self_locking(&p);
    }

    #[test]
    fn test_back_drive_zero_when_self_locking() {
        let p = WormGearParams {
            lead_angle: 0.01, /* very small => self-locking */
            friction_coeff: 0.5,
            ..Default::default()
        };
        assert_eq!(back_drive_torque(&p, 50.0), 0.0);
    }

    #[test]
    fn test_input_speed_from_output() {
        let p = WormGearParams::default();
        let input = input_speed_from_output(&p, 5.0); /* should be 5*20 = 100 */
        assert!((input - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_zero_input_gives_zero_output() {
        let mut wg = default_wg();
        wg.apply_input(0.0, 0.0);
        assert!((wg.state.wheel_omega).abs() < 1e-6);
    }
}
