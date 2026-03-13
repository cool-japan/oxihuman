// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Inner brow (corrugator/procerus) shape morph.

#![allow(dead_code)]

/// Configuration for inner brow morphing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowInnerConfig {
    pub max_raise: f32,
    pub max_furrow: f32,
}

/// Runtime state for inner brow morph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowInnerState {
    pub raise_l: f32,
    pub raise_r: f32,
    pub furrow: f32,
    pub angle_l: f32,
    pub angle_r: f32,
}

#[allow(dead_code)]
pub fn default_brow_inner_config() -> BrowInnerConfig {
    BrowInnerConfig {
        max_raise: 1.0,
        max_furrow: 1.0,
    }
}

#[allow(dead_code)]
pub fn new_brow_inner_state() -> BrowInnerState {
    BrowInnerState {
        raise_l: 0.0,
        raise_r: 0.0,
        furrow: 0.0,
        angle_l: 0.0,
        angle_r: 0.0,
    }
}

#[allow(dead_code)]
pub fn bi_set_raise(state: &mut BrowInnerState, cfg: &BrowInnerConfig, left: f32, right: f32) {
    state.raise_l = left.clamp(0.0, cfg.max_raise);
    state.raise_r = right.clamp(0.0, cfg.max_raise);
}

#[allow(dead_code)]
pub fn bi_set_furrow(state: &mut BrowInnerState, cfg: &BrowInnerConfig, v: f32) {
    state.furrow = v.clamp(0.0, cfg.max_furrow);
}

#[allow(dead_code)]
pub fn bi_set_angle(state: &mut BrowInnerState, left: f32, right: f32) {
    state.angle_l = left.clamp(-1.0, 1.0);
    state.angle_r = right.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn bi_reset(state: &mut BrowInnerState) {
    *state = new_brow_inner_state();
}

#[allow(dead_code)]
pub fn bi_mirror_raise(state: &mut BrowInnerState) {
    let avg = (state.raise_l + state.raise_r) * 0.5;
    state.raise_l = avg;
    state.raise_r = avg;
}

#[allow(dead_code)]
pub fn bi_to_weights(state: &BrowInnerState) -> Vec<(String, f32)> {
    vec![
        ("brow_inner_raise_l".to_string(), state.raise_l),
        ("brow_inner_raise_r".to_string(), state.raise_r),
        ("brow_inner_furrow".to_string(), state.furrow),
        ("brow_inner_angle_l".to_string(), state.angle_l),
        ("brow_inner_angle_r".to_string(), state.angle_r),
    ]
}

#[allow(dead_code)]
pub fn bi_to_json(state: &BrowInnerState) -> String {
    format!(
        r#"{{"raise_l":{:.4},"raise_r":{:.4},"furrow":{:.4},"angle_l":{:.4},"angle_r":{:.4}}}"#,
        state.raise_l, state.raise_r, state.furrow, state.angle_l, state.angle_r
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_brow_inner_config();
        assert!((cfg.max_raise - 1.0).abs() < 1e-6);
        assert!((cfg.max_furrow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_zeros() {
        let s = new_brow_inner_state();
        assert_eq!(s.raise_l, 0.0);
        assert_eq!(s.furrow, 0.0);
        assert_eq!(s.angle_r, 0.0);
    }

    #[test]
    fn test_set_raise_clamps() {
        let cfg = default_brow_inner_config();
        let mut s = new_brow_inner_state();
        bi_set_raise(&mut s, &cfg, 2.0, -0.5);
        assert!((s.raise_l - 1.0).abs() < 1e-6);
        assert_eq!(s.raise_r, 0.0);
    }

    #[test]
    fn test_set_furrow_clamps() {
        let cfg = default_brow_inner_config();
        let mut s = new_brow_inner_state();
        bi_set_furrow(&mut s, &cfg, 3.0);
        assert!((s.furrow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_angle() {
        let mut s = new_brow_inner_state();
        bi_set_angle(&mut s, 0.3, -0.6);
        assert!((s.angle_l - 0.3).abs() < 1e-6);
        assert!((s.angle_r + 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let cfg = default_brow_inner_config();
        let mut s = new_brow_inner_state();
        bi_set_raise(&mut s, &cfg, 0.5, 0.5);
        bi_reset(&mut s);
        assert_eq!(s.raise_l, 0.0);
    }

    #[test]
    fn test_mirror_raise() {
        let cfg = default_brow_inner_config();
        let mut s = new_brow_inner_state();
        bi_set_raise(&mut s, &cfg, 0.2, 0.8);
        bi_mirror_raise(&mut s);
        assert!((s.raise_l - 0.5).abs() < 1e-6);
        assert!((s.raise_r - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_weights_count() {
        let s = new_brow_inner_state();
        assert_eq!(bi_to_weights(&s).len(), 5);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_brow_inner_state();
        let j = bi_to_json(&s);
        assert!(j.contains("raise_l"));
        assert!(j.contains("furrow"));
        assert!(j.contains("angle_r"));
    }
}
