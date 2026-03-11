// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Scoliosis lateral spinal curve morph.

/// Scoliosis configuration.
#[derive(Debug, Clone)]
pub struct ScoliosisMorphConfig {
    pub lateral_degree: f32,
    pub curve_direction: f32,
    pub rotation_component: f32,
}

impl Default for ScoliosisMorphConfig {
    fn default() -> Self {
        Self {
            lateral_degree: 0.0,
            curve_direction: 1.0,
            rotation_component: 0.0,
        }
    }
}

/// Scoliosis morph state.
#[derive(Debug, Clone)]
pub struct ScoliosisMorph {
    pub config: ScoliosisMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl ScoliosisMorph {
    pub fn new() -> Self {
        Self {
            config: ScoliosisMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for ScoliosisMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new ScoliosisMorph.
pub fn new_scoliosis_morph() -> ScoliosisMorph {
    ScoliosisMorph::new()
}

/// Set lateral curvature degree (0.0–1.0).
pub fn scoliosis_set_lateral(morph: &mut ScoliosisMorph, degree: f32) {
    morph.config.lateral_degree = degree.clamp(0.0, 1.0);
}

/// Set curve direction (-1.0 = left, 1.0 = right).
pub fn scoliosis_set_direction(morph: &mut ScoliosisMorph, dir: f32) {
    morph.config.curve_direction = dir.clamp(-1.0, 1.0);
}

/// Set vertebral rotation component.
pub fn scoliosis_set_rotation(morph: &mut ScoliosisMorph, rot: f32) {
    morph.config.rotation_component = rot.clamp(0.0, 1.0);
}

/// Compute lateral displacement for a given normalized height.
pub fn scoliosis_displacement(morph: &ScoliosisMorph, height_t: f32) -> f32 {
    let sine_curve = (std::f32::consts::PI * height_t).sin();
    morph.intensity * morph.config.lateral_degree * morph.config.curve_direction * sine_curve
}

/// Serialize to JSON.
pub fn scoliosis_to_json(morph: &ScoliosisMorph) -> String {
    format!(
        r#"{{"intensity":{},"lateral_degree":{},"curve_direction":{},"rotation":{}}}"#,
        morph.intensity,
        morph.config.lateral_degree,
        morph.config.curve_direction,
        morph.config.rotation_component,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_scoliosis_morph();
        assert!((m.config.curve_direction - 1.0).abs() < 1e-6 /* right by default */);
    }

    #[test]
    fn test_lateral_clamp() {
        let mut m = new_scoliosis_morph();
        scoliosis_set_lateral(&mut m, 5.0);
        assert!((m.config.lateral_degree - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_direction_left() {
        let mut m = new_scoliosis_morph();
        scoliosis_set_direction(&mut m, -1.0);
        assert!((m.config.curve_direction - (-1.0)).abs() < 1e-6 /* left */);
    }

    #[test]
    fn test_direction_clamp() {
        let mut m = new_scoliosis_morph();
        scoliosis_set_direction(&mut m, 3.0);
        assert!((m.config.curve_direction - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_rotation() {
        let mut m = new_scoliosis_morph();
        scoliosis_set_rotation(&mut m, 0.4);
        assert!((m.config.rotation_component - 0.4).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_displacement_zero_intensity() {
        let m = new_scoliosis_morph();
        assert!((scoliosis_displacement(&m, 0.5) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn test_displacement_peak() {
        let mut m = new_scoliosis_morph();
        scoliosis_set_lateral(&mut m, 1.0);
        m.intensity = 1.0;
        let peak = scoliosis_displacement(&m, 0.5).abs();
        let base = scoliosis_displacement(&m, 0.0).abs();
        assert!(peak > base /* peak at mid height */);
    }

    #[test]
    fn test_json_keys() {
        let m = new_scoliosis_morph();
        let j = scoliosis_to_json(&m);
        assert!(j.contains("lateral_degree") /* key present */);
    }

    #[test]
    fn test_default_enabled() {
        let m = ScoliosisMorph::default();
        assert!(m.enabled /* enabled */);
    }
}
