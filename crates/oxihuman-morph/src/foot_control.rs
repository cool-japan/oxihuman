// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Foot and ankle morphology controls for character customization.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootConfig {
    pub foot_length: f32,
    pub foot_width: f32,
    pub arch_height: f32,
    pub toe_length: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootState {
    pub length: f32,
    pub width: f32,
    pub arch: f32,
    pub toe_spread: f32,
    pub heel_size: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FootMorphWeights {
    pub long: f32,
    pub short: f32,
    pub wide: f32,
    pub narrow: f32,
    pub high_arch: f32,
    pub flat: f32,
}

#[allow(dead_code)]
pub fn default_foot_config() -> FootConfig {
    FootConfig {
        foot_length: 26.0,
        foot_width: 9.5,
        arch_height: 2.5,
        toe_length: 7.0,
    }
}

#[allow(dead_code)]
pub fn new_foot_state() -> FootState {
    FootState {
        length: 0.5,
        width: 0.5,
        arch: 0.5,
        toe_spread: 0.0,
        heel_size: 0.5,
    }
}

#[allow(dead_code)]
pub fn set_foot_length(state: &mut FootState, length: f32) {
    state.length = length.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_foot_width(state: &mut FootState, width: f32) {
    state.width = width.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_arch_height(state: &mut FootState, arch: f32) {
    state.arch = arch.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_toe_spread(state: &mut FootState, spread: f32) {
    state.toe_spread = spread.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn compute_foot_weights(state: &FootState, cfg: &FootConfig) -> FootMorphWeights {
    let len_norm = (state.length - 0.5) * 2.0;
    let wid_norm = (state.width - 0.5) * 2.0;
    let arch_norm = (state.arch - 0.5) * 2.0;

    let long_w = (len_norm * cfg.foot_length / 30.0).clamp(0.0, 1.0);
    let short_w = ((-len_norm) * cfg.foot_length / 30.0).clamp(0.0, 1.0);
    let wide_w = (wid_norm * cfg.foot_width / 12.0).clamp(0.0, 1.0);
    let narrow_w = ((-wid_norm) * cfg.foot_width / 12.0).clamp(0.0, 1.0);
    let high_arch_w = (arch_norm * cfg.arch_height / 4.0).clamp(0.0, 1.0);
    let flat_w = ((-arch_norm) * cfg.arch_height / 4.0).clamp(0.0, 1.0);

    FootMorphWeights {
        long: long_w,
        short: short_w,
        wide: wide_w,
        narrow: narrow_w,
        high_arch: high_arch_w,
        flat: flat_w,
    }
}

#[allow(dead_code)]
pub fn blend_feet(a: &FootState, b: &FootState, t: f32) -> FootState {
    let t = t.clamp(0.0, 1.0);
    let s = 1.0 - t;
    FootState {
        length: a.length * s + b.length * t,
        width: a.width * s + b.width * t,
        arch: a.arch * s + b.arch * t,
        toe_spread: a.toe_spread * s + b.toe_spread * t,
        heel_size: a.heel_size * s + b.heel_size * t,
    }
}

#[allow(dead_code)]
pub fn reset_foot(state: &mut FootState) {
    *state = new_foot_state();
}

#[allow(dead_code)]
pub fn foot_state_to_json(state: &FootState) -> String {
    format!(
        r#"{{"length":{:.4},"width":{:.4},"arch":{:.4},"toe_spread":{:.4},"heel_size":{:.4}}}"#,
        state.length, state.width, state.arch, state.toe_spread, state.heel_size
    )
}

#[allow(dead_code)]
pub fn foot_size_index(state: &FootState) -> f32 {
    state.length * state.width / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_foot_config() {
        let cfg = default_foot_config();
        assert!(cfg.foot_length > 0.0);
        assert!(cfg.foot_width > 0.0);
        assert!(cfg.arch_height > 0.0);
        assert!(cfg.toe_length > 0.0);
    }

    #[test]
    fn test_new_foot_state() {
        let s = new_foot_state();
        assert_eq!(s.length, 0.5);
        assert_eq!(s.width, 0.5);
    }

    #[test]
    fn test_set_foot_length_clamp() {
        let mut s = new_foot_state();
        set_foot_length(&mut s, 2.0);
        assert_eq!(s.length, 1.0);
        set_foot_length(&mut s, -1.0);
        assert_eq!(s.length, 0.0);
    }

    #[test]
    fn test_set_foot_width_clamp() {
        let mut s = new_foot_state();
        set_foot_width(&mut s, 0.7);
        assert!((s.width - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_set_arch_height() {
        let mut s = new_foot_state();
        set_arch_height(&mut s, 0.9);
        assert!((s.arch - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_compute_foot_weights_range() {
        let cfg = default_foot_config();
        let mut s = new_foot_state();
        set_foot_length(&mut s, 0.9);
        let w = compute_foot_weights(&s, &cfg);
        assert!((0.0..=1.0).contains(&w.long));
        assert!((0.0..=1.0).contains(&w.short));
        assert!((0.0..=1.0).contains(&w.wide));
        assert!((0.0..=1.0).contains(&w.narrow));
        assert!((0.0..=1.0).contains(&w.high_arch));
        assert!((0.0..=1.0).contains(&w.flat));
    }

    #[test]
    fn test_blend_feet() {
        let a = new_foot_state();
        let mut b = new_foot_state();
        b.length = 1.0;
        let blended = blend_feet(&a, &b, 0.5);
        assert!((blended.length - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_reset_foot() {
        let mut s = new_foot_state();
        set_foot_length(&mut s, 1.0);
        reset_foot(&mut s);
        assert_eq!(s.length, 0.5);
    }

    #[test]
    fn test_foot_state_to_json() {
        let s = new_foot_state();
        let json = foot_state_to_json(&s);
        assert!(json.contains("length"));
        assert!(json.contains("width"));
    }

    #[test]
    fn test_foot_size_index() {
        let mut s = new_foot_state();
        s.length = 0.5;
        s.width = 0.5;
        let idx = foot_size_index(&s);
        assert!((idx - 0.0025).abs() < 1e-6);
    }

    #[test]
    fn test_set_toe_spread() {
        let mut s = new_foot_state();
        set_toe_spread(&mut s, 0.4);
        assert!((s.toe_spread - 0.4).abs() < 1e-5);
    }
}
