// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! UV stretch/distortion visualization.

#![allow(dead_code)]

/// Stretch type for UV visualization.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum UvStretchType {
    Area,
    Angle,
}

/// Configuration for UV stretch overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvStretchConfig {
    pub stretch_type: UvStretchType,
    pub threshold: f32,
}

/// Runtime state for UV stretch overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvStretchState {
    pub enabled: bool,
    pub config: UvStretchConfig,
    pub max_stretch: f32,
}

#[allow(dead_code)]
pub fn default_uv_stretch_config() -> UvStretchConfig {
    UvStretchConfig {
        stretch_type: UvStretchType::Area,
        threshold: 0.5,
    }
}

#[allow(dead_code)]
pub fn new_uv_stretch_state() -> UvStretchState {
    UvStretchState {
        enabled: false,
        config: default_uv_stretch_config(),
        max_stretch: 0.0,
    }
}

#[allow(dead_code)]
pub fn uvs_set_enabled(state: &mut UvStretchState, v: bool) {
    state.enabled = v;
}

#[allow(dead_code)]
pub fn uvs_set_type(state: &mut UvStretchState, t: UvStretchType) {
    state.config.stretch_type = t;
}

#[allow(dead_code)]
pub fn uvs_compute_stretch(
    state: &UvStretchState,
    uv: [f32; 2],
    ref_uv: [f32; 2],
) -> f32 {
    let du = uv[0] - ref_uv[0];
    let dv = uv[1] - ref_uv[1];
    match state.config.stretch_type {
        UvStretchType::Area => (du * du + dv * dv).sqrt(),
        UvStretchType::Angle => du.abs().max(dv.abs()),
    }
}

#[allow(dead_code)]
pub fn uvs_to_json(state: &UvStretchState) -> String {
    format!(
        r#"{{"enabled":{},"type":"{}","threshold":{:.4},"max_stretch":{:.4}}}"#,
        state.enabled,
        uvs_type_name(state),
        state.config.threshold,
        state.max_stretch
    )
}

#[allow(dead_code)]
pub fn uvs_reset(state: &mut UvStretchState) {
    *state = new_uv_stretch_state();
}

#[allow(dead_code)]
pub fn uvs_type_name(state: &UvStretchState) -> &'static str {
    match state.config.stretch_type {
        UvStretchType::Area => "area",
        UvStretchType::Angle => "angle",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_uv_stretch_config();
        assert_eq!(cfg.stretch_type, UvStretchType::Area);
        assert!((cfg.threshold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_disabled() {
        let s = new_uv_stretch_state();
        assert!(!s.enabled);
        assert_eq!(s.max_stretch, 0.0);
    }

    #[test]
    fn test_set_enabled() {
        let mut s = new_uv_stretch_state();
        uvs_set_enabled(&mut s, true);
        assert!(s.enabled);
    }

    #[test]
    fn test_set_type() {
        let mut s = new_uv_stretch_state();
        uvs_set_type(&mut s, UvStretchType::Angle);
        assert_eq!(s.config.stretch_type, UvStretchType::Angle);
    }

    #[test]
    fn test_compute_stretch_area() {
        let s = new_uv_stretch_state();
        let stretch = uvs_compute_stretch(&s, [1.0, 0.0], [0.0, 0.0]);
        assert!((stretch - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_stretch_angle() {
        let mut s = new_uv_stretch_state();
        uvs_set_type(&mut s, UvStretchType::Angle);
        let stretch = uvs_compute_stretch(&s, [0.3, 0.7], [0.0, 0.0]);
        assert!((stretch - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_type_name() {
        let s = new_uv_stretch_state();
        assert_eq!(uvs_type_name(&s), "area");
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_uv_stretch_state();
        let j = uvs_to_json(&s);
        assert!(j.contains("enabled"));
        assert!(j.contains("type"));
        assert!(j.contains("threshold"));
    }
}
