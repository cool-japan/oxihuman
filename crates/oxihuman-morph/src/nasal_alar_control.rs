// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Nasal alar morph control: adjusts the width and flare of the nostrils.

/// Configuration for nasal alar morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalAlarConfig {
    pub min_width: f32,
    pub max_width: f32,
}

/// Runtime state for nasal alar morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalAlarState {
    pub width: f32,
    pub flare: f32,
    pub thickness: f32,
}

#[allow(dead_code)]
pub fn default_nasal_alar_config() -> NasalAlarConfig {
    NasalAlarConfig {
        min_width: 0.0,
        max_width: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_nasal_alar_state() -> NasalAlarState {
    NasalAlarState {
        width: 0.5,
        flare: 0.3,
        thickness: 0.5,
    }
}

#[allow(dead_code)]
pub fn na_set_width(state: &mut NasalAlarState, cfg: &NasalAlarConfig, v: f32) {
    state.width = v.clamp(cfg.min_width, cfg.max_width);
}

#[allow(dead_code)]
pub fn na_set_flare(state: &mut NasalAlarState, v: f32) {
    state.flare = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn na_set_thickness(state: &mut NasalAlarState, v: f32) {
    state.thickness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn na_reset(state: &mut NasalAlarState) {
    *state = new_nasal_alar_state();
}

#[allow(dead_code)]
pub fn na_to_weights(state: &NasalAlarState) -> Vec<(String, f32)> {
    vec![
        ("nasal_alar_width".to_string(), state.width),
        ("nasal_alar_flare".to_string(), state.flare),
        ("nasal_alar_thickness".to_string(), state.thickness),
    ]
}

#[allow(dead_code)]
pub fn na_to_json(state: &NasalAlarState) -> String {
    format!(
        r#"{{"width":{:.4},"flare":{:.4},"thickness":{:.4}}}"#,
        state.width, state.flare, state.thickness
    )
}

#[allow(dead_code)]
pub fn na_blend(a: &NasalAlarState, b: &NasalAlarState, t: f32) -> NasalAlarState {
    let t = t.clamp(0.0, 1.0);
    NasalAlarState {
        width: a.width + (b.width - a.width) * t,
        flare: a.flare + (b.flare - a.flare) * t,
        thickness: a.thickness + (b.thickness - a.thickness) * t,
    }
}

#[allow(dead_code)]
pub fn na_overall_width(state: &NasalAlarState) -> f32 {
    state.width + state.flare * 0.2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_nasal_alar_config();
        assert!(cfg.min_width.abs() < 1e-6);
        assert!((cfg.max_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_nasal_alar_state();
        assert!((s.width - 0.5).abs() < 1e-6);
        assert!((s.flare - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_set_width_clamps() {
        let cfg = default_nasal_alar_config();
        let mut s = new_nasal_alar_state();
        na_set_width(&mut s, &cfg, 5.0);
        assert!((s.width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_flare() {
        let mut s = new_nasal_alar_state();
        na_set_flare(&mut s, 0.8);
        assert!((s.flare - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_thickness() {
        let mut s = new_nasal_alar_state();
        na_set_thickness(&mut s, 0.2);
        assert!((s.thickness - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_nasal_alar_config();
        let mut s = new_nasal_alar_state();
        na_set_width(&mut s, &cfg, 0.9);
        na_reset(&mut s);
        assert!((s.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights() {
        let s = new_nasal_alar_state();
        assert_eq!(na_to_weights(&s).len(), 3);
    }

    #[test]
    fn test_to_json() {
        let s = new_nasal_alar_state();
        let j = na_to_json(&s);
        assert!(j.contains("width"));
        assert!(j.contains("flare"));
    }

    #[test]
    fn test_blend() {
        let a = new_nasal_alar_state();
        let mut b = new_nasal_alar_state();
        b.width = 1.0;
        let mid = na_blend(&a, &b, 0.5);
        assert!((mid.width - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_overall_width() {
        let s = new_nasal_alar_state();
        let w = na_overall_width(&s);
        assert!(w > 0.0);
    }
}
