// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Clavicle (collarbone) shape and position morph controls.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClavicleConfig {
    pub prominence: f32,
    pub length_ratio: f32,
    pub angle_range: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClavicleState {
    pub prominence: f32,
    pub angle: f32,
    pub length: f32,
    pub left_offset: f32,
    pub right_offset: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClavicleMorphWeights {
    pub prominent: f32,
    pub flat: f32,
    pub angled_up: f32,
    pub angled_down: f32,
    pub wide: f32,
}

#[allow(dead_code)]
pub fn default_clavicle_config() -> ClavicleConfig {
    ClavicleConfig {
        prominence: 0.5,
        length_ratio: 0.5,
        angle_range: 0.3,
    }
}

#[allow(dead_code)]
pub fn new_clavicle_state() -> ClavicleState {
    ClavicleState {
        prominence: 0.5,
        angle: 0.5,
        length: 0.5,
        left_offset: 0.0,
        right_offset: 0.0,
    }
}

#[allow(dead_code)]
pub fn set_clavicle_prominence(state: &mut ClavicleState, value: f32) {
    state.prominence = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_clavicle_angle(state: &mut ClavicleState, value: f32) {
    state.angle = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_clavicle_length(state: &mut ClavicleState, value: f32) {
    state.length = value.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_clavicle_offset(state: &mut ClavicleState, left: f32, right: f32) {
    state.left_offset = left.clamp(-1.0, 1.0);
    state.right_offset = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_clavicle_weights(
    state: &ClavicleState,
    cfg: &ClavicleConfig,
) -> ClavicleMorphWeights {
    let p = state.prominence * cfg.prominence;
    let prominent = p.clamp(0.0, 1.0);
    let flat = (1.0 - p).clamp(0.0, 1.0);
    let angle_bias = (state.angle - 0.5) * 2.0 * cfg.angle_range;
    let angled_up = angle_bias.max(0.0).clamp(0.0, 1.0);
    let angled_down = (-angle_bias).max(0.0).clamp(0.0, 1.0);
    let wide = (state.length * cfg.length_ratio).clamp(0.0, 1.0);
    ClavicleMorphWeights {
        prominent,
        flat,
        angled_up,
        angled_down,
        wide,
    }
}

#[allow(dead_code)]
pub fn clavicle_to_json(state: &ClavicleState) -> String {
    format!(
        r#"{{"prominence":{},"angle":{},"length":{},"left_offset":{},"right_offset":{}}}"#,
        state.prominence, state.angle, state.length, state.left_offset, state.right_offset
    )
}

#[allow(dead_code)]
pub fn blend_clavicle_states(a: &ClavicleState, b: &ClavicleState, t: f32) -> ClavicleState {
    let t = t.clamp(0.0, 1.0);
    ClavicleState {
        prominence: a.prominence + (b.prominence - a.prominence) * t,
        angle: a.angle + (b.angle - a.angle) * t,
        length: a.length + (b.length - a.length) * t,
        left_offset: a.left_offset + (b.left_offset - a.left_offset) * t,
        right_offset: a.right_offset + (b.right_offset - a.right_offset) * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_clavicle_config();
        assert!((0.0..=1.0).contains(&c.prominence));
    }

    #[test]
    fn test_new_state() {
        let s = new_clavicle_state();
        assert!((s.prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_prominence() {
        let mut s = new_clavicle_state();
        set_clavicle_prominence(&mut s, 0.9);
        assert!((s.prominence - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle_clamp() {
        let mut s = new_clavicle_state();
        set_clavicle_angle(&mut s, 1.5);
        assert!((s.angle - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_length() {
        let mut s = new_clavicle_state();
        set_clavicle_length(&mut s, 0.7);
        assert!((s.length - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_offset() {
        let mut s = new_clavicle_state();
        set_clavicle_offset(&mut s, 0.3, -0.2);
        assert!((s.left_offset - 0.3).abs() < 1e-6);
        assert!((s.right_offset - (-0.2)).abs() < 1e-6);
    }

    #[test]
    fn test_compute_weights() {
        let s = new_clavicle_state();
        let cfg = default_clavicle_config();
        let w = compute_clavicle_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.prominent));
        assert!((0.0..=1.0).contains(&w.flat));
    }

    #[test]
    fn test_to_json() {
        let s = new_clavicle_state();
        let j = clavicle_to_json(&s);
        assert!(j.contains("prominence"));
    }

    #[test]
    fn test_blend() {
        let a = new_clavicle_state();
        let mut b = new_clavicle_state();
        b.prominence = 1.0;
        let mid = blend_clavicle_states(&a, &b, 0.5);
        assert!((mid.prominence - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = new_clavicle_state();
        let r = blend_clavicle_states(&a, &a, 0.3);
        assert!((r.angle - a.angle).abs() < 1e-6);
    }
}
