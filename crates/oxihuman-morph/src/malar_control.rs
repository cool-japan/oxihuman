// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Malar/zygomatic prominence morph controls.

#![allow(dead_code)]

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MalarConfig {
    pub max_prominence: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MalarState {
    pub prominence_l: f32,
    pub prominence_r: f32,
    pub projection_l: f32,
    pub projection_r: f32,
}

#[allow(dead_code)]
pub fn default_malar_config() -> MalarConfig {
    MalarConfig { max_prominence: 1.0 }
}

#[allow(dead_code)]
pub fn new_malar_state() -> MalarState {
    MalarState {
        prominence_l: 0.0,
        prominence_r: 0.0,
        projection_l: 0.0,
        projection_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn malar_set_prominence(state: &mut MalarState, cfg: &MalarConfig, left: f32, right: f32) {
    state.prominence_l = left.clamp(0.0, cfg.max_prominence);
    state.prominence_r = right.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn malar_set_projection(state: &mut MalarState, cfg: &MalarConfig, left: f32, right: f32) {
    state.projection_l = left.clamp(0.0, cfg.max_prominence);
    state.projection_r = right.clamp(0.0, cfg.max_prominence);
}

#[allow(dead_code)]
pub fn malar_mirror(state: &mut MalarState) {
    let avg_prom = (state.prominence_l + state.prominence_r) * 0.5;
    let avg_proj = (state.projection_l + state.projection_r) * 0.5;
    state.prominence_l = avg_prom;
    state.prominence_r = avg_prom;
    state.projection_l = avg_proj;
    state.projection_r = avg_proj;
}

#[allow(dead_code)]
pub fn malar_reset(state: &mut MalarState) {
    *state = new_malar_state();
}

#[allow(dead_code)]
pub fn malar_to_weights(state: &MalarState) -> [f32; 4] {
    [state.prominence_l, state.prominence_r, state.projection_l, state.projection_r]
}

#[allow(dead_code)]
pub fn malar_to_json(state: &MalarState) -> String {
    format!(
        r#"{{"prominence_l":{:.4},"prominence_r":{:.4},"projection_l":{:.4},"projection_r":{:.4}}}"#,
        state.prominence_l, state.prominence_r, state.projection_l, state.projection_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_malar_config();
        assert!((cfg.max_prominence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_malar_state();
        assert!((s.prominence_l - 0.0).abs() < 1e-6);
        assert!((s.prominence_r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence_clamps() {
        let cfg = default_malar_config();
        let mut s = new_malar_state();
        malar_set_prominence(&mut s, &cfg, 5.0, -1.0);
        assert!((s.prominence_l - 1.0).abs() < 1e-6);
        assert!((s.prominence_r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_projection_clamps() {
        let cfg = default_malar_config();
        let mut s = new_malar_state();
        malar_set_projection(&mut s, &cfg, 0.5, 0.3);
        assert!((s.projection_l - 0.5).abs() < 1e-6);
        assert!((s.projection_r - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_mirror_averages() {
        let cfg = default_malar_config();
        let mut s = new_malar_state();
        malar_set_prominence(&mut s, &cfg, 0.2, 0.8);
        malar_mirror(&mut s);
        assert!((s.prominence_l - 0.5).abs() < 1e-6);
        assert!((s.prominence_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_malar_config();
        let mut s = new_malar_state();
        malar_set_prominence(&mut s, &cfg, 0.9, 0.9);
        malar_reset(&mut s);
        assert!((s.prominence_l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_length() {
        let s = new_malar_state();
        let w = malar_to_weights(&s);
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn test_to_json_contains_keys() {
        let s = new_malar_state();
        let j = malar_to_json(&s);
        assert!(j.contains("prominence_l"));
        assert!(j.contains("projection_r"));
    }

    #[test]
    fn test_weights_range() {
        let cfg = default_malar_config();
        let mut s = new_malar_state();
        malar_set_prominence(&mut s, &cfg, 0.7, 0.3);
        let w = malar_to_weights(&s);
        for v in &w {
            assert!(*v >= 0.0 && *v <= 1.0);
        }
    }
}
