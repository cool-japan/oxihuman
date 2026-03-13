// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Environment map rotation v2: supports animated and latlong + cubemap modes.

use std::f32::consts::{PI, TAU};

/// Rotation representation for the environment map.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct EnvRotation {
    /// Azimuth angle in radians [0, 2π).
    pub azimuth: f32,
    /// Elevation angle in radians (−π/2 … +π/2).
    pub elevation: f32,
    /// Roll angle in radians.
    pub roll: f32,
}

impl Default for EnvRotation {
    fn default() -> Self {
        Self {
            azimuth: 0.0,
            elevation: 0.0,
            roll: 0.0,
        }
    }
}

/// Animation mode for environment rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EnvRotMode {
    Static,
    /// Continuously rotates the azimuth at a given speed.
    Spinning,
    /// Follows a pre-baked keyframe track index.
    Keyframed,
}

/// Full v2 configuration for environment rotation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EnvRotationV2Config {
    pub rotation: EnvRotation,
    pub mode: EnvRotMode,
    /// Spin speed in radians per second (used when mode = Spinning).
    pub spin_speed: f32,
    /// Exposure compensation applied after rotation (EV).
    pub exposure_ev: f32,
}

impl Default for EnvRotationV2Config {
    fn default() -> Self {
        Self {
            rotation: EnvRotation::default(),
            mode: EnvRotMode::Static,
            spin_speed: 0.0,
            exposure_ev: 0.0,
        }
    }
}

/// Normalise azimuth into [0, 2π).
#[allow(dead_code)]
pub fn normalise_azimuth(az: f32) -> f32 {
    az.rem_euclid(TAU)
}

/// Clamp elevation to valid range (−π/2 … +π/2).
#[allow(dead_code)]
pub fn clamp_elevation(el: f32) -> f32 {
    el.clamp(-PI / 2.0, PI / 2.0)
}

/// Build a 3×3 rotation matrix (row-major) from an `EnvRotation`.
#[allow(dead_code)]
pub fn rotation_matrix(rot: &EnvRotation) -> [[f32; 3]; 3] {
    let (sa, ca) = rot.azimuth.sin_cos();
    let (se, ce) = rot.elevation.sin_cos();
    let (sr, cr) = rot.roll.sin_cos();
    [
        [ca * ce, ca * se * sr - sa * cr, ca * se * cr + sa * sr],
        [sa * ce, sa * se * sr + ca * cr, sa * se * cr - ca * sr],
        [-se, ce * sr, ce * cr],
    ]
}

/// Advance the spinning azimuth by `dt` seconds.
#[allow(dead_code)]
pub fn advance_spin(cfg: &mut EnvRotationV2Config, dt: f32) {
    if cfg.mode == EnvRotMode::Spinning {
        cfg.rotation.azimuth = normalise_azimuth(cfg.rotation.azimuth + cfg.spin_speed * dt);
    }
}

/// Convert EV offset to linear multiplier.
#[allow(dead_code)]
pub fn ev_to_linear(ev: f32) -> f32 {
    (2.0_f32).powf(ev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn default_rotation_zero() {
        let r = EnvRotation::default();
        assert_eq!(r.azimuth, 0.0);
        assert_eq!(r.elevation, 0.0);
    }

    #[test]
    fn normalise_azimuth_wraps() {
        let az = normalise_azimuth(TAU + 0.1);
        assert!((az - 0.1).abs() < 1e-5);
    }

    #[test]
    fn normalise_azimuth_negative() {
        let az = normalise_azimuth(-0.1);
        assert!(az >= 0.0);
        assert!(az < TAU);
    }

    #[test]
    fn clamp_elevation_high() {
        let el = clamp_elevation(PI);
        assert!((el - FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn clamp_elevation_low() {
        let el = clamp_elevation(-PI);
        assert!((el + FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn rotation_matrix_identity_at_zero() {
        let rot = EnvRotation::default();
        let m = rotation_matrix(&rot);
        assert!((m[0][0] - 1.0).abs() < 1e-5);
        assert!((m[1][1] - 1.0).abs() < 1e-5);
        assert!((m[2][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn advance_spin_changes_azimuth() {
        let mut cfg = EnvRotationV2Config {
            mode: EnvRotMode::Spinning,
            spin_speed: 1.0,
            ..Default::default()
        };
        advance_spin(&mut cfg, 0.5);
        assert!((cfg.rotation.azimuth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn advance_spin_no_effect_static() {
        let mut cfg = EnvRotationV2Config::default();
        advance_spin(&mut cfg, 1.0);
        assert_eq!(cfg.rotation.azimuth, 0.0);
    }

    #[test]
    fn ev_to_linear_zero() {
        assert!((ev_to_linear(0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ev_to_linear_one() {
        assert!((ev_to_linear(1.0) - 2.0).abs() < 1e-5);
    }
}
