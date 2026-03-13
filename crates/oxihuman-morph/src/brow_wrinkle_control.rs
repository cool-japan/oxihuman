// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brow wrinkle morph — controls horizontal and vertical furrow lines on the brow.

/// Configuration for brow wrinkle control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowWrinkleConfig {
    pub max_depth: f32,
}

/// Brow wrinkle runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowWrinkleState {
    pub horizontal_depth: f32,
    pub vertical_furrow: f32,
    pub left_arch_wrinkle: f32,
    pub right_arch_wrinkle: f32,
}

#[allow(dead_code)]
pub fn default_brow_wrinkle_config() -> BrowWrinkleConfig {
    BrowWrinkleConfig { max_depth: 1.0 }
}

#[allow(dead_code)]
pub fn new_brow_wrinkle_state() -> BrowWrinkleState {
    BrowWrinkleState {
        horizontal_depth: 0.0,
        vertical_furrow: 0.0,
        left_arch_wrinkle: 0.0,
        right_arch_wrinkle: 0.0,
    }
}

#[allow(dead_code)]
pub fn bw_set_horizontal(state: &mut BrowWrinkleState, cfg: &BrowWrinkleConfig, v: f32) {
    state.horizontal_depth = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn bw_set_vertical(state: &mut BrowWrinkleState, cfg: &BrowWrinkleConfig, v: f32) {
    state.vertical_furrow = v.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn bw_set_arch(state: &mut BrowWrinkleState, cfg: &BrowWrinkleConfig, left: f32, right: f32) {
    state.left_arch_wrinkle = left.clamp(0.0, cfg.max_depth);
    state.right_arch_wrinkle = right.clamp(0.0, cfg.max_depth);
}

#[allow(dead_code)]
pub fn bw_reset(state: &mut BrowWrinkleState) {
    *state = new_brow_wrinkle_state();
}

#[allow(dead_code)]
pub fn bw_is_neutral(state: &BrowWrinkleState) -> bool {
    let vals = [
        state.horizontal_depth,
        state.vertical_furrow,
        state.left_arch_wrinkle,
        state.right_arch_wrinkle,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn bw_intensity(state: &BrowWrinkleState) -> f32 {
    let vals = [
        state.horizontal_depth,
        state.vertical_furrow,
        state.left_arch_wrinkle,
        state.right_arch_wrinkle,
    ];
    vals.iter().cloned().fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn bw_blend(a: &BrowWrinkleState, b: &BrowWrinkleState, t: f32) -> BrowWrinkleState {
    let t = t.clamp(0.0, 1.0);
    BrowWrinkleState {
        horizontal_depth: a.horizontal_depth + (b.horizontal_depth - a.horizontal_depth) * t,
        vertical_furrow: a.vertical_furrow + (b.vertical_furrow - a.vertical_furrow) * t,
        left_arch_wrinkle: a.left_arch_wrinkle + (b.left_arch_wrinkle - a.left_arch_wrinkle) * t,
        right_arch_wrinkle: a.right_arch_wrinkle
            + (b.right_arch_wrinkle - a.right_arch_wrinkle) * t,
    }
}

#[allow(dead_code)]
pub fn bw_symmetry(state: &BrowWrinkleState) -> f32 {
    (state.left_arch_wrinkle - state.right_arch_wrinkle).abs()
}

#[allow(dead_code)]
pub fn bw_to_weights(state: &BrowWrinkleState) -> Vec<(String, f32)> {
    vec![
        (
            "brow_horizontal_wrinkle".to_string(),
            state.horizontal_depth,
        ),
        ("brow_vertical_furrow".to_string(), state.vertical_furrow),
        ("brow_arch_wrinkle_l".to_string(), state.left_arch_wrinkle),
        ("brow_arch_wrinkle_r".to_string(), state.right_arch_wrinkle),
    ]
}

#[allow(dead_code)]
pub fn bw_to_json(state: &BrowWrinkleState) -> String {
    format!(
        r#"{{"horizontal_depth":{:.4},"vertical_furrow":{:.4},"left_arch":{:.4},"right_arch":{:.4}}}"#,
        state.horizontal_depth,
        state.vertical_furrow,
        state.left_arch_wrinkle,
        state.right_arch_wrinkle
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_max() {
        let cfg = default_brow_wrinkle_config();
        assert!((cfg.max_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_brow_wrinkle_state();
        assert!(bw_is_neutral(&s));
    }

    #[test]
    fn set_horizontal_clamps() {
        let cfg = default_brow_wrinkle_config();
        let mut s = new_brow_wrinkle_state();
        bw_set_horizontal(&mut s, &cfg, 5.0);
        assert!((s.horizontal_depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_vertical_negative_clamped() {
        let cfg = default_brow_wrinkle_config();
        let mut s = new_brow_wrinkle_state();
        bw_set_vertical(&mut s, &cfg, -1.0);
        assert_eq!(s.vertical_furrow, 0.0);
    }

    #[test]
    fn set_arch_both_sides() {
        let cfg = default_brow_wrinkle_config();
        let mut s = new_brow_wrinkle_state();
        bw_set_arch(&mut s, &cfg, 0.3, 0.7);
        assert!((s.left_arch_wrinkle - 0.3).abs() < 1e-6);
        assert!((s.right_arch_wrinkle - 0.7).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_brow_wrinkle_config();
        let mut s = new_brow_wrinkle_state();
        bw_set_horizontal(&mut s, &cfg, 0.5);
        bw_reset(&mut s);
        assert!(bw_is_neutral(&s));
    }

    #[test]
    fn intensity_max() {
        let cfg = default_brow_wrinkle_config();
        let mut s = new_brow_wrinkle_state();
        bw_set_horizontal(&mut s, &cfg, 0.6);
        bw_set_vertical(&mut s, &cfg, 0.9);
        let i = bw_intensity(&s);
        assert!((i - 0.9).abs() < 1e-6);
    }

    #[test]
    fn blend_midpoint() {
        let a = new_brow_wrinkle_state();
        let cfg = default_brow_wrinkle_config();
        let mut b = new_brow_wrinkle_state();
        bw_set_horizontal(&mut b, &cfg, 1.0);
        let mid = bw_blend(&a, &b, 0.5);
        assert!((mid.horizontal_depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn symmetry_zero_when_equal() {
        let cfg = default_brow_wrinkle_config();
        let mut s = new_brow_wrinkle_state();
        bw_set_arch(&mut s, &cfg, 0.4, 0.4);
        assert!(bw_symmetry(&s) < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_brow_wrinkle_state();
        assert_eq!(bw_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_contains_keys() {
        let s = new_brow_wrinkle_state();
        let j = bw_to_json(&s);
        assert!(j.contains("horizontal_depth"));
        assert!(j.contains("right_arch"));
    }
}
