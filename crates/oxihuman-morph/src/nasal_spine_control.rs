// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nasal spine control — anterior nasal spine projection and base angle.

use std::f32::consts::FRAC_PI_6;

/// Configuration for nasal spine.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalSpineConfig {
    pub max_projection: f32,
    pub max_angle_rad: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalSpineState {
    pub projection: f32,
    pub base_angle_rad: f32,
    pub width: f32,
}

#[allow(dead_code)]
pub fn default_nasal_spine_config() -> NasalSpineConfig {
    NasalSpineConfig {
        max_projection: 1.0,
        max_angle_rad: FRAC_PI_6,
    }
}

#[allow(dead_code)]
pub fn new_nasal_spine_state() -> NasalSpineState {
    NasalSpineState {
        projection: 0.0,
        base_angle_rad: 0.0,
        width: 0.0,
    }
}

#[allow(dead_code)]
pub fn ns_set_projection(state: &mut NasalSpineState, cfg: &NasalSpineConfig, v: f32) {
    state.projection = v.clamp(-cfg.max_projection, cfg.max_projection);
}

#[allow(dead_code)]
pub fn ns_set_angle(state: &mut NasalSpineState, cfg: &NasalSpineConfig, v: f32) {
    state.base_angle_rad = v.clamp(-cfg.max_angle_rad, cfg.max_angle_rad);
}

#[allow(dead_code)]
pub fn ns_set_width(state: &mut NasalSpineState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn ns_reset(state: &mut NasalSpineState) {
    *state = new_nasal_spine_state();
}

#[allow(dead_code)]
pub fn ns_is_neutral(state: &NasalSpineState) -> bool {
    state.projection.abs() < 1e-6 && state.base_angle_rad.abs() < 1e-6 && state.width.abs() < 1e-6
}

#[allow(dead_code)]
pub fn ns_prominence(state: &NasalSpineState) -> f32 {
    state.projection.abs()
}

#[allow(dead_code)]
pub fn ns_blend(a: &NasalSpineState, b: &NasalSpineState, t: f32) -> NasalSpineState {
    let t = t.clamp(0.0, 1.0);
    NasalSpineState {
        projection: a.projection + (b.projection - a.projection) * t,
        base_angle_rad: a.base_angle_rad + (b.base_angle_rad - a.base_angle_rad) * t,
        width: a.width + (b.width - a.width) * t,
    }
}

#[allow(dead_code)]
pub fn ns_to_weights(state: &NasalSpineState) -> Vec<(String, f32)> {
    let anorm = 1.0 / FRAC_PI_6;
    vec![
        ("nasal_spine_projection".to_string(), state.projection),
        (
            "nasal_spine_angle".to_string(),
            state.base_angle_rad * anorm,
        ),
        ("nasal_spine_width".to_string(), state.width),
    ]
}

#[allow(dead_code)]
pub fn ns_to_json(state: &NasalSpineState) -> String {
    format!(
        r#"{{"projection":{:.4},"base_angle_rad":{:.4},"width":{:.4}}}"#,
        state.projection, state.base_angle_rad, state.width
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_nasal_spine_config();
        assert!((cfg.max_projection - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_nasal_spine_state();
        assert!(ns_is_neutral(&s));
    }

    #[test]
    fn set_projection_clamps() {
        let cfg = default_nasal_spine_config();
        let mut s = new_nasal_spine_state();
        ns_set_projection(&mut s, &cfg, 5.0);
        assert!((s.projection - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_projection_negative() {
        let cfg = default_nasal_spine_config();
        let mut s = new_nasal_spine_state();
        ns_set_projection(&mut s, &cfg, -0.5);
        assert!((s.projection + 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_angle_clamps() {
        let cfg = default_nasal_spine_config();
        let mut s = new_nasal_spine_state();
        ns_set_angle(&mut s, &cfg, 10.0);
        assert!((s.base_angle_rad - FRAC_PI_6).abs() < 1e-6);
    }

    #[test]
    fn set_width() {
        let mut s = new_nasal_spine_state();
        ns_set_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_nasal_spine_config();
        let mut s = new_nasal_spine_state();
        ns_set_projection(&mut s, &cfg, 0.8);
        ns_reset(&mut s);
        assert!(ns_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_nasal_spine_state();
        let cfg = default_nasal_spine_config();
        let mut b = new_nasal_spine_state();
        ns_set_projection(&mut b, &cfg, 1.0);
        let m = ns_blend(&a, &b, 0.5);
        assert!((m.projection - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_nasal_spine_state();
        assert_eq!(ns_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_fields() {
        let s = new_nasal_spine_state();
        let j = ns_to_json(&s);
        assert!(j.contains("projection"));
    }
}
