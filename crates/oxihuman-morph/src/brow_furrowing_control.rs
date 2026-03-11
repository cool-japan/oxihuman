// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Brow furrowing control — glabellar compression and corrugator activation.

/// Configuration for brow furrowing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowFurrowConfig {
    pub max_furrow: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrowFurrowState {
    pub inner_furrow: f32,
    pub outer_furrow: f32,
    pub vertical_pull: f32,
}

#[allow(dead_code)]
pub fn default_brow_furrow_config() -> BrowFurrowConfig {
    BrowFurrowConfig { max_furrow: 1.0 }
}

#[allow(dead_code)]
pub fn new_brow_furrow_state() -> BrowFurrowState {
    BrowFurrowState {
        inner_furrow: 0.0,
        outer_furrow: 0.0,
        vertical_pull: 0.0,
    }
}

#[allow(dead_code)]
pub fn bfw_set_inner(state: &mut BrowFurrowState, cfg: &BrowFurrowConfig, v: f32) {
    state.inner_furrow = v.clamp(0.0, cfg.max_furrow);
}

#[allow(dead_code)]
pub fn bfw_set_outer(state: &mut BrowFurrowState, cfg: &BrowFurrowConfig, v: f32) {
    state.outer_furrow = v.clamp(0.0, cfg.max_furrow);
}

#[allow(dead_code)]
pub fn bfw_set_vertical(state: &mut BrowFurrowState, v: f32) {
    state.vertical_pull = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn bfw_reset(state: &mut BrowFurrowState) {
    *state = new_brow_furrow_state();
}

#[allow(dead_code)]
pub fn bfw_is_neutral(state: &BrowFurrowState) -> bool {
    let vals = [state.inner_furrow, state.outer_furrow, state.vertical_pull];
    vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn bfw_intensity(state: &BrowFurrowState) -> f32 {
    (state.inner_furrow + state.outer_furrow) * 0.5
}

#[allow(dead_code)]
pub fn bfw_blend(a: &BrowFurrowState, b: &BrowFurrowState, t: f32) -> BrowFurrowState {
    let t = t.clamp(0.0, 1.0);
    BrowFurrowState {
        inner_furrow: a.inner_furrow + (b.inner_furrow - a.inner_furrow) * t,
        outer_furrow: a.outer_furrow + (b.outer_furrow - a.outer_furrow) * t,
        vertical_pull: a.vertical_pull + (b.vertical_pull - a.vertical_pull) * t,
    }
}

#[allow(dead_code)]
pub fn bfw_to_weights(state: &BrowFurrowState) -> Vec<(String, f32)> {
    vec![
        ("brow_furrow_inner".to_string(), state.inner_furrow),
        ("brow_furrow_outer".to_string(), state.outer_furrow),
        ("brow_vertical_pull".to_string(), state.vertical_pull),
    ]
}

#[allow(dead_code)]
pub fn bfw_to_json(state: &BrowFurrowState) -> String {
    format!(
        r#"{{"inner_furrow":{:.4},"outer_furrow":{:.4},"vertical_pull":{:.4}}}"#,
        state.inner_furrow, state.outer_furrow, state.vertical_pull
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_brow_furrow_config();
        assert!((cfg.max_furrow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_brow_furrow_state();
        assert!(bfw_is_neutral(&s));
    }

    #[test]
    fn set_inner_clamps() {
        let cfg = default_brow_furrow_config();
        let mut s = new_brow_furrow_state();
        bfw_set_inner(&mut s, &cfg, 5.0);
        assert!((s.inner_furrow - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_outer() {
        let cfg = default_brow_furrow_config();
        let mut s = new_brow_furrow_state();
        bfw_set_outer(&mut s, &cfg, 0.6);
        assert!((s.outer_furrow - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_vertical_clamps_negative() {
        let mut s = new_brow_furrow_state();
        bfw_set_vertical(&mut s, -3.0);
        assert!((s.vertical_pull + 1.0).abs() < 1e-6);
    }

    #[test]
    fn intensity_average() {
        let cfg = default_brow_furrow_config();
        let mut s = new_brow_furrow_state();
        bfw_set_inner(&mut s, &cfg, 0.4);
        bfw_set_outer(&mut s, &cfg, 0.6);
        assert!((bfw_intensity(&s) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_brow_furrow_config();
        let mut s = new_brow_furrow_state();
        bfw_set_inner(&mut s, &cfg, 0.9);
        bfw_reset(&mut s);
        assert!(bfw_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_brow_furrow_state();
        let cfg = default_brow_furrow_config();
        let mut b = new_brow_furrow_state();
        bfw_set_inner(&mut b, &cfg, 1.0);
        let m = bfw_blend(&a, &b, 0.5);
        assert!((m.inner_furrow - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_brow_furrow_state();
        assert_eq!(bfw_to_weights(&s).len(), 3);
    }

    #[test]
    fn to_json_contains_fields() {
        let s = new_brow_furrow_state();
        let j = bfw_to_json(&s);
        assert!(j.contains("inner_furrow"));
    }
}
