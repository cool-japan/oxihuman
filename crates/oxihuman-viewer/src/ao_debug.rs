// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Ambient occlusion debug visualization.

use std::f32::consts::PI;

/// AO debug display mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AoDebugMode {
    Off,
    RawAo,
    BentNormal,
    SampleCount,
}

/// AO debug configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoDebugConfig {
    pub mode: AoDebugMode,
    pub intensity_scale: f32,
    pub show_radius: bool,
    pub sample_display_count: u32,
}

/// AO debug result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoDebugResult {
    pub display_value: f32,
    pub color: [f32; 3],
}

/// Default AO debug config.
#[allow(dead_code)]
pub fn default_ao_debug_config() -> AoDebugConfig {
    AoDebugConfig {
        mode: AoDebugMode::Off,
        intensity_scale: 1.0,
        show_radius: false,
        sample_display_count: 16,
    }
}

/// Compute debug AO value from a normal.
#[allow(dead_code)]
pub fn compute_debug_ao(normal: [f32; 3], config: &AoDebugConfig) -> AoDebugResult {
    let base_ao = (normal[1].abs() * PI / PI).clamp(0.0, 1.0);
    let scaled = (base_ao * config.intensity_scale).clamp(0.0, 1.0);
    let color = match config.mode {
        AoDebugMode::Off => [1.0, 1.0, 1.0],
        AoDebugMode::RawAo => [scaled, scaled, scaled],
        AoDebugMode::BentNormal => [(normal[0] + 1.0) * 0.5, (normal[1] + 1.0) * 0.5, (normal[2] + 1.0) * 0.5],
        AoDebugMode::SampleCount => [0.0, scaled, 1.0 - scaled],
    };
    AoDebugResult {
        display_value: scaled,
        color,
    }
}

/// Check if debug mode is active.
#[allow(dead_code)]
pub fn is_ao_debug_active(config: &AoDebugConfig) -> bool {
    config.mode != AoDebugMode::Off
}

/// Set debug mode.
#[allow(dead_code)]
pub fn set_ao_debug_mode(config: &mut AoDebugConfig, mode: AoDebugMode) {
    config.mode = mode;
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_ao_debug(config: &mut AoDebugConfig) {
    *config = default_ao_debug_config();
}

/// Scale the intensity.
#[allow(dead_code)]
pub fn set_ao_debug_intensity(config: &mut AoDebugConfig, intensity: f32) {
    config.intensity_scale = intensity.clamp(0.1, 10.0);
}

/// Get display label for the mode.
#[allow(dead_code)]
pub fn ao_debug_mode_label(mode: AoDebugMode) -> &'static str {
    match mode {
        AoDebugMode::Off => "Off",
        AoDebugMode::RawAo => "Raw AO",
        AoDebugMode::BentNormal => "Bent Normal",
        AoDebugMode::SampleCount => "Sample Count",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_ao_debug_config();
        assert_eq!(c.mode, AoDebugMode::Off);
    }

    #[test]
    fn test_compute_raw_ao() {
        let mut c = default_ao_debug_config();
        c.mode = AoDebugMode::RawAo;
        let r = compute_debug_ao([0.0, 1.0, 0.0], &c);
        assert!((r.display_value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_bent_normal() {
        let mut c = default_ao_debug_config();
        c.mode = AoDebugMode::BentNormal;
        let r = compute_debug_ao([0.0, 1.0, 0.0], &c);
        assert!((r.color[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_active() {
        let c = default_ao_debug_config();
        assert!(!is_ao_debug_active(&c));
    }

    #[test]
    fn test_set_mode() {
        let mut c = default_ao_debug_config();
        set_ao_debug_mode(&mut c, AoDebugMode::RawAo);
        assert_eq!(c.mode, AoDebugMode::RawAo);
    }

    #[test]
    fn test_reset() {
        let mut c = default_ao_debug_config();
        c.mode = AoDebugMode::RawAo;
        reset_ao_debug(&mut c);
        assert_eq!(c.mode, AoDebugMode::Off);
    }

    #[test]
    fn test_set_intensity() {
        let mut c = default_ao_debug_config();
        set_ao_debug_intensity(&mut c, 5.0);
        assert!((c.intensity_scale - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_mode_label() {
        assert_eq!(ao_debug_mode_label(AoDebugMode::Off), "Off");
        assert_eq!(ao_debug_mode_label(AoDebugMode::RawAo), "Raw AO");
    }

    #[test]
    fn test_off_mode_white() {
        let c = default_ao_debug_config();
        let r = compute_debug_ao([0.0, 1.0, 0.0], &c);
        assert!((r.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_intensity() {
        let mut c = default_ao_debug_config();
        set_ao_debug_intensity(&mut c, 100.0);
        assert!((c.intensity_scale - 10.0).abs() < 1e-6);
    }
}
