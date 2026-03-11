// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Nasal root morph — controls the bridge height and width at the nasal root.

/// Configuration for nasal root control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalRootConfig {
    pub max_depth: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalRootState {
    pub depth: f32,
    pub width: f32,
    pub height: f32,
    pub squish: f32,
}

#[allow(dead_code)]
pub fn default_nasal_root_config() -> NasalRootConfig {
    NasalRootConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_nasal_root_state() -> NasalRootState {
    NasalRootState {
        depth: 0.0,
        width: 0.0,
        height: 0.0,
        squish: 0.0,
    }
}

#[allow(dead_code)]
pub fn nr_set_depth(state: &mut NasalRootState, cfg: &NasalRootConfig, v: f32) {
    state.depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn nr_set_width(state: &mut NasalRootState, cfg: &NasalRootConfig, v: f32) {
    state.width = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn nr_set_height(state: &mut NasalRootState, v: f32) {
    state.height = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn nr_set_squish(state: &mut NasalRootState, v: f32) {
    state.squish = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn nr_reset(state: &mut NasalRootState) {
    *state = new_nasal_root_state();
}

#[allow(dead_code)]
pub fn nr_is_neutral(state: &NasalRootState) -> bool {
    state.depth.abs() < 1e-6
        && state.width.abs() < 1e-6
        && state.height.abs() < 1e-6
        && state.squish.abs() < 1e-6
}

#[allow(dead_code)]
pub fn nr_bridge_prominence(state: &NasalRootState) -> f32 {
    (state.depth + state.height.max(0.0)) * 0.5
}

#[allow(dead_code)]
pub fn nr_blend(a: &NasalRootState, b: &NasalRootState, t: f32) -> NasalRootState {
    let t = t.clamp(0.0, 1.0);
    NasalRootState {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        height: a.height + (b.height - a.height) * t,
        squish: a.squish + (b.squish - a.squish) * t,
    }
}

#[allow(dead_code)]
pub fn nr_to_weights(state: &NasalRootState) -> Vec<(String, f32)> {
    vec![
        ("nasal_root_depth".to_string(), state.depth),
        ("nasal_root_width".to_string(), state.width),
        ("nasal_root_height".to_string(), state.height),
        ("nasal_root_squish".to_string(), state.squish),
    ]
}

#[allow(dead_code)]
pub fn nr_to_json(state: &NasalRootState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"height":{:.4},"squish":{:.4}}}"#,
        state.depth, state.width, state.height, state.squish
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_nasal_root_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_nasal_root_state();
        assert!(nr_is_neutral(&s));
    }

    #[test]
    fn set_depth_clamps() {
        let cfg = default_nasal_root_config();
        let mut s = new_nasal_root_state();
        nr_set_depth(&mut s, &cfg, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_height_signed() {
        let mut s = new_nasal_root_state();
        nr_set_height(&mut s, -0.5);
        assert!((s.height + 0.5).abs() < 1e-6);
    }

    #[test]
    fn set_squish_clamps() {
        let mut s = new_nasal_root_state();
        nr_set_squish(&mut s, 5.0);
        assert!((s.squish - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bridge_prominence_zero_at_neutral() {
        let s = new_nasal_root_state();
        assert!(nr_bridge_prominence(&s) < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_nasal_root_config();
        let mut s = new_nasal_root_state();
        nr_set_depth(&mut s, &cfg, 0.5);
        nr_reset(&mut s);
        assert!(nr_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_nasal_root_state();
        let cfg = default_nasal_root_config();
        let mut b = new_nasal_root_state();
        nr_set_depth(&mut b, &cfg, 1.0);
        let mid = nr_blend(&a, &b, 0.5);
        assert!((mid.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_nasal_root_state();
        assert_eq!(nr_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_nasal_root_state();
        let j = nr_to_json(&s);
        assert!(j.contains("depth"));
        assert!(j.contains("squish"));
    }
}
