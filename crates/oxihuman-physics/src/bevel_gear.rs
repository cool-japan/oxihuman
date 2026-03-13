// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Bevel gear with angle transmission.

#![allow(dead_code)]

/// A bevel gear pair.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BevelGear {
    /// Number of teeth on input gear.
    pub teeth_in: u32,
    /// Number of teeth on output gear.
    pub teeth_out: u32,
    /// Shaft angle between axes (degrees, typically 90.0).
    pub shaft_angle_deg: f32,
    /// Mechanical efficiency.
    pub efficiency: f32,
    /// Input angular velocity (rad/s).
    pub omega_in: f32,
    /// Output angular velocity (rad/s).
    pub omega_out: f32,
    /// Input torque (Nm).
    pub torque_in: f32,
    /// Output torque (Nm).
    pub torque_out: f32,
}

/// Create a new bevel gear pair.
#[allow(dead_code)]
pub fn new_bevel_gear(
    teeth_in: u32,
    teeth_out: u32,
    shaft_angle_deg: f32,
    efficiency: f32,
) -> BevelGear {
    BevelGear {
        teeth_in,
        teeth_out,
        shaft_angle_deg,
        efficiency: efficiency.clamp(0.0, 1.0),
        omega_in: 0.0,
        omega_out: 0.0,
        torque_in: 0.0,
        torque_out: 0.0,
    }
}

/// Gear ratio: output / input = teeth_in / teeth_out.
#[allow(dead_code)]
pub fn bevel_gear_ratio(bg: &BevelGear) -> f32 {
    if bg.teeth_out == 0 {
        return 0.0;
    }
    bg.teeth_in as f32 / bg.teeth_out as f32
}

/// Set input and compute output velocities and torques.
#[allow(dead_code)]
pub fn bevel_set_input(bg: &mut BevelGear, omega_in: f32, torque_in: f32) {
    bg.omega_in = omega_in;
    bg.torque_in = torque_in;
    let ratio = bevel_gear_ratio(bg);
    bg.omega_out = omega_in * ratio;
    bg.torque_out = if ratio.abs() > 1e-10 {
        torque_in / ratio * bg.efficiency
    } else {
        0.0
    };
}

/// Input power: torque_in * omega_in.
#[allow(dead_code)]
pub fn bevel_input_power(bg: &BevelGear) -> f32 {
    bg.torque_in * bg.omega_in
}

/// Output power: torque_out * omega_out.
#[allow(dead_code)]
pub fn bevel_output_power(bg: &BevelGear) -> f32 {
    bg.torque_out * bg.omega_out
}

/// Power loss.
#[allow(dead_code)]
pub fn bevel_power_loss(bg: &BevelGear) -> f32 {
    (bevel_input_power(bg) - bevel_output_power(bg)).max(0.0)
}

/// Shaft angle in radians.
#[allow(dead_code)]
pub fn bevel_shaft_angle_rad(bg: &BevelGear) -> f32 {
    bg.shaft_angle_deg * std::f32::consts::PI / 180.0
}

/// Is this a miter gear (1:1 ratio at 90 degrees)?
#[allow(dead_code)]
pub fn bevel_is_miter(bg: &BevelGear) -> bool {
    bg.teeth_in == bg.teeth_out && (bg.shaft_angle_deg - 90.0).abs() < 0.5
}

/// Reset to zero.
#[allow(dead_code)]
pub fn bevel_reset(bg: &mut BevelGear) {
    bg.omega_in = 0.0;
    bg.omega_out = 0.0;
    bg.torque_in = 0.0;
    bg.torque_out = 0.0;
}

/// Pitch cone half-angle for input gear (degrees).
#[allow(dead_code)]
pub fn bevel_pitch_cone_angle(bg: &BevelGear) -> f32 {
    if bg.teeth_in == 0 || bg.teeth_out == 0 {
        return 0.0;
    }
    let r = bg.teeth_in as f32 / bg.teeth_out as f32;
    let angle_rad = bg.shaft_angle_deg * std::f32::consts::PI / 180.0;
    (r * angle_rad.sin() / (1.0 + r * angle_rad.cos())).atan() * 180.0 / std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bg() -> BevelGear {
        new_bevel_gear(20, 40, 90.0, 0.95)
    }

    #[test]
    fn test_gear_ratio() {
        let bg = make_bg();
        assert!((bevel_gear_ratio(&bg) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_output_omega() {
        let mut bg = make_bg();
        bevel_set_input(&mut bg, 100.0, 10.0);
        assert!((bg.omega_out - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_output_torque_larger() {
        let mut bg = make_bg();
        bevel_set_input(&mut bg, 100.0, 10.0);
        assert!(bg.torque_out > bg.torque_in);
    }

    #[test]
    fn test_input_power() {
        let mut bg = make_bg();
        bevel_set_input(&mut bg, 100.0, 10.0);
        assert!((bevel_input_power(&bg) - 1000.0).abs() < 1e-4);
    }

    #[test]
    fn test_output_power_less_than_input() {
        let mut bg = make_bg();
        bevel_set_input(&mut bg, 100.0, 10.0);
        assert!(bevel_output_power(&bg) <= bevel_input_power(&bg) + 1e-4);
    }

    #[test]
    fn test_shaft_angle_rad() {
        let bg = make_bg();
        let rad = bevel_shaft_angle_rad(&bg);
        assert!((rad - std::f32::consts::PI / 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_is_miter() {
        let bg = new_bevel_gear(20, 20, 90.0, 0.95);
        assert!(bevel_is_miter(&bg));
    }

    #[test]
    fn test_not_miter() {
        let bg = make_bg();
        assert!(!bevel_is_miter(&bg));
    }

    #[test]
    fn test_reset() {
        let mut bg = make_bg();
        bevel_set_input(&mut bg, 100.0, 10.0);
        bevel_reset(&mut bg);
        assert_eq!(bg.omega_in, 0.0);
        assert_eq!(bg.torque_out, 0.0);
    }

    #[test]
    fn test_efficiency_clamped() {
        let bg = new_bevel_gear(20, 40, 90.0, 1.5);
        assert!(bg.efficiency <= 1.0);
    }
}
