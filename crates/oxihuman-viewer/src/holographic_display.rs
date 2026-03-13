// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub struct HologramConfig {
    pub pixel_pitch_um: f32,
    pub wavelength_nm: f32,
    pub viewing_angle_deg: f32,
    pub refresh_rate_hz: f32,
}

pub fn new_hologram_config() -> HologramConfig {
    HologramConfig {
        pixel_pitch_um: 3.74,
        wavelength_nm: 532.0,
        viewing_angle_deg: 30.0,
        refresh_rate_hz: 60.0,
    }
}

/// Diffraction-limited resolution: λ / sin(θ/2) in micrometers.
pub fn hologram_diffraction_limit_um(cfg: &HologramConfig) -> f32 {
    let theta_half = (cfg.viewing_angle_deg * 0.5 * PI / 180.0).sin();
    if theta_half < 1e-9 {
        return f32::MAX;
    }
    (cfg.wavelength_nm * 1e-3) / theta_half
}

pub fn hologram_bandwidth_ghz(cfg: &HologramConfig) -> f32 {
    cfg.refresh_rate_hz * 1e-9
}

pub fn hologram_is_retinal(cfg: &HologramConfig) -> bool {
    cfg.pixel_pitch_um < 1.0
}

pub fn hologram_field_of_view(cfg: &HologramConfig) -> f32 {
    cfg.viewing_angle_deg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        /* refresh rate is 60 Hz */
        let cfg = new_hologram_config();
        assert!((cfg.refresh_rate_hz - 60.0).abs() < 1e-6);
    }

    #[test]
    fn test_diffraction_limit_positive() {
        /* diffraction limit > 0 */
        let cfg = new_hologram_config();
        assert!(hologram_diffraction_limit_um(&cfg) > 0.0);
    }

    #[test]
    fn test_is_retinal_false() {
        /* default pixel_pitch 3.74 um => not retinal */
        let cfg = new_hologram_config();
        assert!(!hologram_is_retinal(&cfg));
    }

    #[test]
    fn test_is_retinal_true() {
        /* pixel_pitch 0.5 um => retinal */
        let mut cfg = new_hologram_config();
        cfg.pixel_pitch_um = 0.5;
        assert!(hologram_is_retinal(&cfg));
    }

    #[test]
    fn test_field_of_view() {
        /* fov returns viewing_angle_deg */
        let cfg = new_hologram_config();
        assert!((hologram_field_of_view(&cfg) - cfg.viewing_angle_deg).abs() < 1e-6);
    }
}
