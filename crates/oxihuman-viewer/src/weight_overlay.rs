// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Skin weight heat-map visualization.

#![allow(dead_code)]

/// Configuration for weight overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightOverlayConfig {
    pub bone_index: u32,
    pub zero_color: [f32; 4],
    pub full_color: [f32; 4],
}

/// Runtime state for weight overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightOverlayState {
    pub enabled: bool,
    pub config: WeightOverlayConfig,
}

#[allow(dead_code)]
pub fn default_weight_overlay_config() -> WeightOverlayConfig {
    WeightOverlayConfig {
        bone_index: 0,
        zero_color: [0.0, 0.0, 1.0, 1.0],
        full_color: [1.0, 0.0, 0.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn new_weight_overlay_state() -> WeightOverlayState {
    WeightOverlayState {
        enabled: false,
        config: default_weight_overlay_config(),
    }
}

#[allow(dead_code)]
pub fn wov_set_enabled(state: &mut WeightOverlayState, v: bool) {
    state.enabled = v;
}

#[allow(dead_code)]
pub fn wov_set_bone(state: &mut WeightOverlayState, bone_index: u32) {
    state.config.bone_index = bone_index;
}

#[allow(dead_code)]
pub fn wov_color_for_weight(state: &WeightOverlayState, w: f32) -> [f32; 4] {
    let t = w.clamp(0.0, 1.0);
    let z = &state.config.zero_color;
    let f = &state.config.full_color;
    [
        z[0] + (f[0] - z[0]) * t,
        z[1] + (f[1] - z[1]) * t,
        z[2] + (f[2] - z[2]) * t,
        z[3] + (f[3] - z[3]) * t,
    ]
}

#[allow(dead_code)]
pub fn wov_to_json(state: &WeightOverlayState) -> String {
    let zc = &state.config.zero_color;
    let fc = &state.config.full_color;
    format!(
        r#"{{"enabled":{},"bone_index":{},"zero_color":[{:.4},{:.4},{:.4},{:.4}],"full_color":[{:.4},{:.4},{:.4},{:.4}]}}"#,
        state.enabled,
        state.config.bone_index,
        zc[0], zc[1], zc[2], zc[3],
        fc[0], fc[1], fc[2], fc[3]
    )
}

#[allow(dead_code)]
pub fn wov_reset(state: &mut WeightOverlayState) {
    *state = new_weight_overlay_state();
}

#[allow(dead_code)]
pub fn wov_is_enabled(state: &WeightOverlayState) -> bool {
    state.enabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_weight_overlay_config();
        assert_eq!(cfg.bone_index, 0);
        assert!((cfg.zero_color[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_disabled() {
        let s = new_weight_overlay_state();
        assert!(!wov_is_enabled(&s));
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_weight_overlay_state();
        wov_set_enabled(&mut s, true);
        assert!(wov_is_enabled(&s));
    }

    #[test]
    fn test_set_bone() {
        let mut s = new_weight_overlay_state();
        wov_set_bone(&mut s, 5);
        assert_eq!(s.config.bone_index, 5);
    }

    #[test]
    fn test_color_for_weight_zero() {
        let s = new_weight_overlay_state();
        let c = wov_color_for_weight(&s, 0.0);
        assert!((c[2] - 1.0).abs() < 1e-6); // blue component from zero_color
    }

    #[test]
    fn test_color_for_weight_full() {
        let s = new_weight_overlay_state();
        let c = wov_color_for_weight(&s, 1.0);
        assert!((c[0] - 1.0).abs() < 1e-6); // red component from full_color
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_weight_overlay_state();
        let j = wov_to_json(&s);
        assert!(j.contains("bone_index"));
        assert!(j.contains("zero_color"));
        assert!(j.contains("full_color"));
    }

    #[test]
    fn test_reset() {
        let mut s = new_weight_overlay_state();
        wov_set_enabled(&mut s, true);
        wov_set_bone(&mut s, 10);
        wov_reset(&mut s);
        assert!(!s.enabled);
        assert_eq!(s.config.bone_index, 0);
    }
}
