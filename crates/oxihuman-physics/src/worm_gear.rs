// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Worm gear with reduction ratio and efficiency.

#![allow(dead_code)]

/// A worm gear drive.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WormGear {
    /// Number of starts on the worm (thread count).
    pub worm_starts: u32,
    /// Number of teeth on the worm wheel.
    pub wheel_teeth: u32,
    /// Mechanical efficiency (0..1), accounting for friction.
    pub efficiency: f32,
    /// Input angular velocity (worm, rad/s).
    pub omega_in: f32,
    /// Output angular velocity (wheel, rad/s).
    pub omega_out: f32,
    /// Input torque (Nm).
    pub torque_in: f32,
    /// Output torque (Nm).
    pub torque_out: f32,
}

/// Gear ratio: output_turns / input_turns = starts / teeth.
#[allow(dead_code)]
pub fn worm_gear_ratio(wg: &WormGear) -> f32 {
    if wg.wheel_teeth == 0 {
        return 0.0;
    }
    wg.worm_starts as f32 / wg.wheel_teeth as f32
}

/// Reduction ratio: input_turns / output_turns.
#[allow(dead_code)]
pub fn worm_reduction_ratio(wg: &WormGear) -> f32 {
    if wg.worm_starts == 0 {
        return f32::INFINITY;
    }
    wg.wheel_teeth as f32 / wg.worm_starts as f32
}

/// Create a new worm gear.
#[allow(dead_code)]
pub fn new_worm_gear(worm_starts: u32, wheel_teeth: u32, efficiency: f32) -> WormGear {
    WormGear {
        worm_starts,
        wheel_teeth,
        efficiency: efficiency.clamp(0.0, 1.0),
        omega_in: 0.0,
        omega_out: 0.0,
        torque_in: 0.0,
        torque_out: 0.0,
    }
}

/// Set the input angular velocity and compute output.
#[allow(dead_code)]
pub fn worm_set_input(wg: &mut WormGear, omega_in: f32, torque_in: f32) {
    wg.omega_in = omega_in;
    wg.torque_in = torque_in;
    let ratio = worm_gear_ratio(wg);
    wg.omega_out = omega_in * ratio;
    wg.torque_out = if ratio.abs() > 1e-10 {
        torque_in / ratio * wg.efficiency
    } else {
        0.0
    };
}

/// Output power: torque_out * omega_out.
#[allow(dead_code)]
pub fn worm_output_power(wg: &WormGear) -> f32 {
    wg.torque_out * wg.omega_out
}

/// Input power: torque_in * omega_in.
#[allow(dead_code)]
pub fn worm_input_power(wg: &WormGear) -> f32 {
    wg.torque_in * wg.omega_in
}

/// Power loss due to friction.
#[allow(dead_code)]
pub fn worm_power_loss(wg: &WormGear) -> f32 {
    (worm_input_power(wg) - worm_output_power(wg)).max(0.0)
}

/// Self-locking: worm gears are typically self-locking when efficiency < 0.5.
#[allow(dead_code)]
pub fn worm_is_self_locking(wg: &WormGear) -> bool {
    wg.efficiency < 0.5
}

/// Back-drive ratio (if driven backwards from output).
#[allow(dead_code)]
pub fn worm_backdrive_efficiency(wg: &WormGear) -> f32 {
    1.0 - 2.0 * (1.0 - wg.efficiency)
}

/// Reset to zero state.
#[allow(dead_code)]
pub fn worm_reset(wg: &mut WormGear) {
    wg.omega_in = 0.0;
    wg.omega_out = 0.0;
    wg.torque_in = 0.0;
    wg.torque_out = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wg() -> WormGear {
        new_worm_gear(1, 40, 0.7)
    }

    #[test]
    fn test_reduction_ratio() {
        let wg = make_wg();
        assert!((worm_reduction_ratio(&wg) - 40.0).abs() < 1e-4);
    }

    #[test]
    fn test_gear_ratio() {
        let wg = make_wg();
        assert!((worm_gear_ratio(&wg) - 1.0 / 40.0).abs() < 1e-5);
    }

    #[test]
    fn test_output_omega_reduced() {
        let mut wg = make_wg();
        worm_set_input(&mut wg, 40.0, 10.0);
        assert!((wg.omega_out - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_output_torque_amplified() {
        let mut wg = make_wg();
        worm_set_input(&mut wg, 40.0, 10.0);
        assert!(wg.torque_out > wg.torque_in);
    }

    #[test]
    fn test_power_output_less_than_input() {
        let mut wg = make_wg();
        worm_set_input(&mut wg, 40.0, 10.0);
        assert!(worm_output_power(&wg) <= worm_input_power(&wg) + 1e-4);
    }

    #[test]
    fn test_self_locking_low_efficiency() {
        let wg = new_worm_gear(1, 40, 0.3);
        assert!(worm_is_self_locking(&wg));
    }

    #[test]
    fn test_not_self_locking_high_efficiency() {
        let wg = new_worm_gear(1, 40, 0.8);
        assert!(!worm_is_self_locking(&wg));
    }

    #[test]
    fn test_power_loss() {
        let mut wg = make_wg();
        worm_set_input(&mut wg, 40.0, 10.0);
        assert!(worm_power_loss(&wg) >= 0.0);
    }

    #[test]
    fn test_reset() {
        let mut wg = make_wg();
        worm_set_input(&mut wg, 40.0, 10.0);
        worm_reset(&mut wg);
        assert_eq!(wg.omega_in, 0.0);
        assert_eq!(wg.torque_out, 0.0);
    }

    #[test]
    fn test_efficiency_clamped() {
        let wg = new_worm_gear(1, 40, 1.5);
        assert!(wg.efficiency <= 1.0);
    }
}
