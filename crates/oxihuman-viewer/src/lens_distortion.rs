// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Lens distortion post-process (barrel/pincushion).

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LensDistortionConfig {
    pub k1: f32,
    pub k2: f32,
    pub center: [f32; 2],
    pub strength: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LensDistortionState {
    pub enabled: bool,
    pub config: LensDistortionConfig,
}

#[allow(dead_code)]
pub fn default_lens_distortion_config() -> LensDistortionConfig {
    LensDistortionConfig {
        k1: 0.0,
        k2: 0.0,
        center: [0.5, 0.5],
        strength: 0.0,
    }
}

#[allow(dead_code)]
pub fn new_lens_distortion_state() -> LensDistortionState {
    LensDistortionState {
        enabled: false,
        config: default_lens_distortion_config(),
    }
}

#[allow(dead_code)]
pub fn ld_distort_uv(state: &LensDistortionState, uv: [f32; 2]) -> [f32; 2] {
    if !state.enabled {
        return uv;
    }
    let cx = state.config.center[0];
    let cy = state.config.center[1];
    let dx = uv[0] - cx;
    let dy = uv[1] - cy;
    let r2 = dx * dx + dy * dy;
    let factor = 1.0 + state.config.k1 * r2 + state.config.k2 * r2 * r2;
    [cx + dx * factor, cy + dy * factor]
}

#[allow(dead_code)]
pub fn ld_set_strength(state: &mut LensDistortionState, strength: f32) {
    state.config.strength = strength;
    state.config.k1 = strength;
}

#[allow(dead_code)]
pub fn ld_set_enabled(state: &mut LensDistortionState, enabled: bool) {
    state.enabled = enabled;
}

#[allow(dead_code)]
pub fn ld_is_barrel(config: &LensDistortionConfig) -> bool {
    config.k1 < 0.0
}

#[allow(dead_code)]
pub fn ld_to_json(state: &LensDistortionState) -> String {
    format!(
        r#"{{"enabled":{},"k1":{:.4},"k2":{:.4},"strength":{:.4}}}"#,
        state.enabled, state.config.k1, state.config.k2, state.config.strength
    )
}

#[allow(dead_code)]
pub fn ld_reset(state: &mut LensDistortionState) {
    *state = new_lens_distortion_state();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_lens_distortion_config();
        assert!((cfg.k1 - 0.0).abs() < 1e-6);
        assert!((cfg.center[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_disabled() {
        let s = new_lens_distortion_state();
        assert!(!s.enabled);
    }

    #[test]
    fn test_distort_uv_disabled_passthrough() {
        let s = new_lens_distortion_state();
        let uv = [0.3, 0.7];
        let out = ld_distort_uv(&s, uv);
        assert!((out[0] - 0.3).abs() < 1e-6);
        assert!((out[1] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_distort_uv_enabled_center_unchanged() {
        let mut s = new_lens_distortion_state();
        ld_set_enabled(&mut s, true);
        s.config.k1 = 0.5;
        // Center point should remain at center
        let uv = [0.5, 0.5];
        let out = ld_distort_uv(&s, uv);
        assert!((out[0] - 0.5).abs() < 1e-6);
        assert!((out[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_strength() {
        let mut s = new_lens_distortion_state();
        ld_set_strength(&mut s, 0.5);
        assert!((s.config.strength - 0.5).abs() < 1e-6);
        assert!((s.config.k1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_lens_distortion_state();
        ld_set_enabled(&mut s, true);
        assert!(s.enabled);
    }

    #[test]
    fn test_is_barrel_negative_k1() {
        let mut cfg = default_lens_distortion_config();
        cfg.k1 = -0.5;
        assert!(ld_is_barrel(&cfg));
    }

    #[test]
    fn test_is_not_barrel_positive_k1() {
        let mut cfg = default_lens_distortion_config();
        cfg.k1 = 0.5;
        assert!(!ld_is_barrel(&cfg));
    }

    #[test]
    fn test_to_json() {
        let s = new_lens_distortion_state();
        let j = ld_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("strength"));
    }

    #[test]
    fn test_reset() {
        let mut s = new_lens_distortion_state();
        ld_set_enabled(&mut s, true);
        ld_set_strength(&mut s, 0.9);
        ld_reset(&mut s);
        assert!(!s.enabled);
        assert!((s.config.k1 - 0.0).abs() < 1e-6);
    }
}
