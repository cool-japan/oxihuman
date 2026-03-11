#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pressure-volume constraint for soft body simulation.

/// A pressure-volume body.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PressureVolume {
    pub rest_volume: f32,
    pub current_volume: f32,
    pub pressure_coeff: f32,
}

/// Create a new `PressureVolume`.
#[allow(dead_code)]
pub fn new_pressure_volume(rest_volume: f32, pressure_coeff: f32) -> PressureVolume {
    PressureVolume { rest_volume, current_volume: rest_volume, pressure_coeff }
}

/// Compute the net pressure force magnitude.
#[allow(dead_code)]
pub fn volume_pressure_force(pv: &PressureVolume) -> f32 {
    pv.pressure_coeff * (pv.rest_volume - pv.current_volume)
}

/// Return the current volume.
#[allow(dead_code)]
pub fn current_volume(pv: &PressureVolume) -> f32 {
    pv.current_volume
}

/// Return the rest volume.
#[allow(dead_code)]
pub fn rest_volume(pv: &PressureVolume) -> f32 {
    pv.rest_volume
}

/// Return the pressure coefficient.
#[allow(dead_code)]
pub fn pressure_coefficient(pv: &PressureVolume) -> f32 {
    pv.pressure_coeff
}

/// Return the volume error (current - rest).
#[allow(dead_code)]
pub fn volume_error(pv: &PressureVolume) -> f32 {
    pv.current_volume - pv.rest_volume
}

/// Apply pressure by adjusting current volume toward rest volume.
#[allow(dead_code)]
pub fn apply_pressure(pv: &mut PressureVolume, delta_volume: f32) {
    pv.current_volume += delta_volume;
}

/// Step the pressure simulation (move current volume toward rest).
#[allow(dead_code)]
pub fn pressure_step(pv: &mut PressureVolume, dt: f32) {
    let error = pv.rest_volume - pv.current_volume;
    pv.current_volume += error * pv.pressure_coeff * dt;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pressure_volume() {
        let pv = new_pressure_volume(10.0, 1.0);
        assert!((rest_volume(&pv) - 10.0).abs() < 1e-6);
        assert!((current_volume(&pv) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_force_at_rest() {
        let pv = new_pressure_volume(5.0, 2.0);
        assert!(volume_pressure_force(&pv).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_force_compressed() {
        let mut pv = new_pressure_volume(5.0, 2.0);
        apply_pressure(&mut pv, -1.0); // volume decreases
        assert!(volume_pressure_force(&pv) > 0.0);
    }

    #[test]
    fn test_volume_error() {
        let mut pv = new_pressure_volume(5.0, 1.0);
        apply_pressure(&mut pv, 2.0);
        assert!((volume_error(&pv) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_pressure() {
        let mut pv = new_pressure_volume(10.0, 1.0);
        apply_pressure(&mut pv, -3.0);
        assert!((current_volume(&pv) - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_pressure_step_restores() {
        let mut pv = new_pressure_volume(10.0, 1.0);
        pv.current_volume = 8.0;
        let err_before = volume_error(&pv).abs();
        pressure_step(&mut pv, 0.1);
        let err_after = volume_error(&pv).abs();
        assert!(err_after < err_before);
    }

    #[test]
    fn test_pressure_coefficient() {
        let pv = new_pressure_volume(1.0, 3.5);
        assert!((pressure_coefficient(&pv) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_rest_volume() {
        let pv = new_pressure_volume(7.0, 1.0);
        assert!((rest_volume(&pv) - 7.0).abs() < 1e-6);
    }
}
