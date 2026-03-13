// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Screen-space ambient occlusion post-process config.

#![allow(dead_code)]

/// Configuration for SSAO effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoConfig {
    pub radius: f32,
    pub intensity: f32,
    pub samples: u32,
    pub bias: f32,
}

/// Runtime state for SSAO effect.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AoState {
    pub enabled: bool,
    pub config: AoConfig,
}

#[allow(dead_code)]
pub fn default_ao_config() -> AoConfig {
    AoConfig {
        radius: 0.5,
        intensity: 1.0,
        samples: 16,
        bias: 0.025,
    }
}

#[allow(dead_code)]
pub fn new_ao_state() -> AoState {
    AoState {
        enabled: true,
        config: default_ao_config(),
    }
}

#[allow(dead_code)]
pub fn ao_set_radius(state: &mut AoState, radius: f32) {
    state.config.radius = radius.max(0.0);
}

#[allow(dead_code)]
pub fn ao_set_intensity(state: &mut AoState, intensity: f32) {
    state.config.intensity = intensity.clamp(0.0, 10.0);
}

#[allow(dead_code)]
pub fn ao_set_samples(state: &mut AoState, samples: u32) {
    state.config.samples = samples.clamp(1, 64);
}

#[allow(dead_code)]
pub fn ao_set_enabled(state: &mut AoState, enabled: bool) {
    state.enabled = enabled;
}

#[allow(dead_code)]
pub fn ao_is_enabled(state: &AoState) -> bool {
    state.enabled
}

#[allow(dead_code)]
pub fn ao_to_json(state: &AoState) -> String {
    format!(
        r#"{{"enabled":{},"radius":{:.4},"intensity":{:.4},"samples":{}}}"#,
        state.enabled, state.config.radius, state.config.intensity, state.config.samples
    )
}

#[allow(dead_code)]
pub fn ao_reset(state: &mut AoState) {
    *state = new_ao_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ao_config();
        assert!((cfg.radius - 0.5).abs() < 1e-6);
        assert_eq!(cfg.samples, 16);
    }

    #[test]
    fn test_new_state_enabled() {
        let s = new_ao_state();
        assert!(s.enabled);
    }

    #[test]
    fn test_set_radius() {
        let mut s = new_ao_state();
        ao_set_radius(&mut s, 1.5);
        assert!((s.config.radius - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius_clamps_neg() {
        let mut s = new_ao_state();
        ao_set_radius(&mut s, -1.0);
        assert_eq!(s.config.radius, 0.0);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut s = new_ao_state();
        ao_set_intensity(&mut s, 100.0);
        assert!((s.config.intensity - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_samples_clamps() {
        let mut s = new_ao_state();
        ao_set_samples(&mut s, 0);
        assert_eq!(s.config.samples, 1);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_ao_state();
        ao_set_enabled(&mut s, false);
        assert!(!ao_is_enabled(&s));
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_ao_state();
        let j = ao_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("radius"));
        assert!(j.contains("samples"));
    }
}
