// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face width v2 control — bizygomatic and bigonial width scaling.

use std::f32::consts::FRAC_1_SQRT_2;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceWidthV2Config {
    /// Reference diagonal factor (informational).
    pub diagonal_factor: f32,
    /// Whether to link bizygomatic and bigonial together.
    pub link_regions: bool,
}

impl Default for FaceWidthV2Config {
    fn default() -> Self {
        FaceWidthV2Config {
            diagonal_factor: FRAC_1_SQRT_2,
            link_regions: false,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceWidthV2State {
    /// Bizygomatic (cheekbone) width in `[0.0, 1.0]`.
    bizygomatic: f32,
    /// Bigonial (jaw) width in `[0.0, 1.0]`.
    bigonial: f32,
    /// Temporal width in `[0.0, 1.0]`.
    temporal: f32,
    config: FaceWidthV2Config,
}

/// Default config.
pub fn default_face_width_v2_config() -> FaceWidthV2Config {
    FaceWidthV2Config::default()
}

/// New neutral state.
pub fn new_face_width_v2_state(config: FaceWidthV2Config) -> FaceWidthV2State {
    FaceWidthV2State {
        bizygomatic: 0.5,
        bigonial: 0.5,
        temporal: 0.5,
        config,
    }
}

/// Set bizygomatic width.
pub fn fw2_set_bizygomatic(state: &mut FaceWidthV2State, v: f32) {
    state.bizygomatic = v.clamp(0.0, 1.0);
    if state.config.link_regions {
        state.bigonial = state.bizygomatic;
    }
}

/// Set bigonial (jaw) width.
pub fn fw2_set_bigonial(state: &mut FaceWidthV2State, v: f32) {
    state.bigonial = v.clamp(0.0, 1.0);
}

/// Set temporal width.
pub fn fw2_set_temporal(state: &mut FaceWidthV2State, v: f32) {
    state.temporal = v.clamp(0.0, 1.0);
}

/// Reset to neutral (0.5).
pub fn fw2_reset(state: &mut FaceWidthV2State) {
    state.bizygomatic = 0.5;
    state.bigonial = 0.5;
    state.temporal = 0.5;
}

/// True when all at 0.5 (neutral).
pub fn fw2_is_neutral(state: &FaceWidthV2State) -> bool {
    (state.bizygomatic - 0.5).abs() < 1e-5
        && (state.bigonial - 0.5).abs() < 1e-5
        && (state.temporal - 0.5).abs() < 1e-5
}

/// Weighted average width across regions.
pub fn fw2_average_width(state: &FaceWidthV2State) -> f32 {
    (state.bizygomatic * 0.5 + state.bigonial * 0.3 + state.temporal * 0.2).clamp(0.0, 1.0)
}

/// Morph weights: `[bizygomatic, bigonial, temporal]`.
pub fn fw2_to_weights(state: &FaceWidthV2State) -> [f32; 3] {
    [state.bizygomatic, state.bigonial, state.temporal]
}

/// Blend.
pub fn fw2_blend(a: &FaceWidthV2State, b: &FaceWidthV2State, t: f32) -> FaceWidthV2State {
    let t = t.clamp(0.0, 1.0);
    FaceWidthV2State {
        bizygomatic: a.bizygomatic + (b.bizygomatic - a.bizygomatic) * t,
        bigonial: a.bigonial + (b.bigonial - a.bigonial) * t,
        temporal: a.temporal + (b.temporal - a.temporal) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn fw2_to_json(state: &FaceWidthV2State) -> String {
    format!(
        r#"{{"bizygomatic":{:.4},"bigonial":{:.4},"temporal":{:.4}}}"#,
        state.bizygomatic, state.bigonial, state.temporal
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> FaceWidthV2State {
        new_face_width_v2_state(default_face_width_v2_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(fw2_is_neutral(&make()));
    }

    #[test]
    fn set_bizygomatic_clamps() {
        let mut s = make();
        fw2_set_bizygomatic(&mut s, 5.0);
        assert!((s.bizygomatic - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        fw2_set_bizygomatic(&mut s, 0.1);
        fw2_reset(&mut s);
        assert!(fw2_is_neutral(&s));
    }

    #[test]
    fn average_in_range() {
        let s = make();
        assert!((0.0..=1.0).contains(&fw2_average_width(&s)));
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in fw2_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_at_one_is_b() {
        let mut b = make();
        fw2_set_bizygomatic(&mut b, 0.9);
        let r = fw2_blend(&make(), &b, 1.0);
        assert!((r.bizygomatic - 0.9).abs() < 1e-5);
    }

    #[test]
    fn blend_midpoint() {
        let mut a = make();
        let mut b = make();
        fw2_set_bizygomatic(&mut a, 0.0);
        fw2_set_bizygomatic(&mut b, 1.0);
        let m = fw2_blend(&a, &b, 0.5);
        assert!((m.bizygomatic - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_bizygomatic() {
        assert!(fw2_to_json(&make()).contains("bizygomatic"));
    }

    #[test]
    fn link_regions_propagates() {
        let mut cfg = default_face_width_v2_config();
        cfg.link_regions = true;
        let mut s = new_face_width_v2_state(cfg);
        fw2_set_bizygomatic(&mut s, 0.3);
        assert!((s.bigonial - 0.3).abs() < 1e-5);
    }
}
