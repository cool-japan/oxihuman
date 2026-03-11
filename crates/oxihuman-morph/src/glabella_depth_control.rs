// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Glabella depth control — the inter-brow hollow / depth.

use std::f32::consts::FRAC_PI_4;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlabellaDepthConfig {
    pub max_depth: f32,
    /// Reference angle used in tangent approximation.
    pub slope_ref_rad: f32,
}

impl Default for GlabellaDepthConfig {
    fn default() -> Self {
        GlabellaDepthConfig {
            max_depth: 1.0,
            slope_ref_rad: FRAC_PI_4,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlabellaDepthState {
    /// Depth in `[0.0, 1.0]`.
    depth: f32,
    /// Width of the depression in `[0.0, 1.0]`.
    width: f32,
    /// Vertical position shift in `[-1.0, 1.0]`.
    v_shift: f32,
    config: GlabellaDepthConfig,
}

/// Default config.
pub fn default_glabella_depth_config() -> GlabellaDepthConfig {
    GlabellaDepthConfig::default()
}

/// New neutral state.
pub fn new_glabella_depth_state(config: GlabellaDepthConfig) -> GlabellaDepthState {
    GlabellaDepthState {
        depth: 0.0,
        width: 0.5,
        v_shift: 0.0,
        config,
    }
}

/// Set depth.
pub fn gd_set_depth(state: &mut GlabellaDepthState, v: f32) {
    state.depth = v.clamp(0.0, 1.0);
}

/// Set width.
pub fn gd_set_width(state: &mut GlabellaDepthState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

/// Set vertical shift.
pub fn gd_set_v_shift(state: &mut GlabellaDepthState, v: f32) {
    state.v_shift = v.clamp(-1.0, 1.0);
}

/// Reset.
pub fn gd_reset(state: &mut GlabellaDepthState) {
    state.depth = 0.0;
    state.width = 0.5;
    state.v_shift = 0.0;
}

/// True when neutral.
pub fn gd_is_neutral(state: &GlabellaDepthState) -> bool {
    state.depth < 1e-5 && state.v_shift.abs() < 1e-5
}

/// Compute the slope angle (radians) from depth and width.
pub fn gd_slope_angle_rad(state: &GlabellaDepthState) -> f32 {
    if state.width < 1e-5 {
        return 0.0;
    }
    (state.depth / state.width)
        .atan()
        .min(state.config.slope_ref_rad)
}

/// Morph weights: `[depth, width, v_shift_normalised]`.
pub fn gd_to_weights(state: &GlabellaDepthState) -> [f32; 3] {
    [
        state.depth,
        state.width,
        (state.v_shift * 0.5 + 0.5).clamp(0.0, 1.0),
    ]
}

/// Blend.
pub fn gd_blend(a: &GlabellaDepthState, b: &GlabellaDepthState, t: f32) -> GlabellaDepthState {
    let t = t.clamp(0.0, 1.0);
    GlabellaDepthState {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        v_shift: a.v_shift + (b.v_shift - a.v_shift) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn gd_to_json(state: &GlabellaDepthState) -> String {
    format!(
        r#"{{"depth":{:.4},"width":{:.4},"v_shift":{:.4}}}"#,
        state.depth, state.width, state.v_shift
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> GlabellaDepthState {
        new_glabella_depth_state(default_glabella_depth_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(gd_is_neutral(&make()));
    }

    #[test]
    fn set_depth_clamps_high() {
        let mut s = make();
        gd_set_depth(&mut s, 5.0);
        assert!((s.depth - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_clears_depth() {
        let mut s = make();
        gd_set_depth(&mut s, 0.8);
        gd_reset(&mut s);
        assert!(gd_is_neutral(&s));
    }

    #[test]
    fn slope_angle_positive_when_depth_nonzero() {
        let mut s = make();
        gd_set_depth(&mut s, 0.5);
        assert!(gd_slope_angle_rad(&s) > 0.0);
    }

    #[test]
    fn slope_angle_zero_neutral() {
        assert!(gd_slope_angle_rad(&make()) < 1e-5);
    }

    #[test]
    fn weights_in_range() {
        let mut s = make();
        gd_set_depth(&mut s, 0.6);
        for v in gd_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        gd_set_depth(&mut b, 1.0);
        let m = gd_blend(&make(), &b, 0.5);
        assert!((m.depth - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_depth() {
        assert!(gd_to_json(&make()).contains("depth"));
    }

    #[test]
    fn v_shift_clamped() {
        let mut s = make();
        gd_set_v_shift(&mut s, 5.0);
        assert!((s.v_shift - 1.0).abs() < 1e-5);
    }
}
