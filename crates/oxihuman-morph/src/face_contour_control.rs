// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face contour morph — controls the overall outline/silhouette of the face.

/// Configuration for face contour control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceContourConfig {
    pub max_scale: f32,
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceContourState {
    pub temporal_width: f32,
    pub zygomatic_projection: f32,
    pub mandible_flare: f32,
    pub overall_taper: f32,
}

#[allow(dead_code)]
pub fn default_face_contour_config() -> FaceContourConfig {
    FaceContourConfig { max_scale: 1.0 }
}

#[allow(dead_code)]
pub fn new_face_contour_state() -> FaceContourState {
    FaceContourState {
        temporal_width: 0.0,
        zygomatic_projection: 0.0,
        mandible_flare: 0.0,
        overall_taper: 0.0,
    }
}

#[allow(dead_code)]
pub fn fc_set_temporal(state: &mut FaceContourState, cfg: &FaceContourConfig, v: f32) {
    state.temporal_width = v.clamp(0.0, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fc_set_zygomatic(state: &mut FaceContourState, cfg: &FaceContourConfig, v: f32) {
    state.zygomatic_projection = v.clamp(0.0, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fc_set_mandible(state: &mut FaceContourState, cfg: &FaceContourConfig, v: f32) {
    state.mandible_flare = v.clamp(0.0, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fc_set_taper(state: &mut FaceContourState, cfg: &FaceContourConfig, v: f32) {
    state.overall_taper = v.clamp(0.0, cfg.max_scale);
}

#[allow(dead_code)]
pub fn fc_reset(state: &mut FaceContourState) {
    *state = new_face_contour_state();
}

#[allow(dead_code)]
pub fn fc_is_neutral(state: &FaceContourState) -> bool {
    let vals = [
        state.temporal_width,
        state.zygomatic_projection,
        state.mandible_flare,
        state.overall_taper,
    ];
    !vals.is_empty() && vals.iter().all(|v| v.abs() < 1e-6)
}

#[allow(dead_code)]
pub fn fc_contour_intensity(state: &FaceContourState) -> f32 {
    let vals = [
        state.temporal_width,
        state.zygomatic_projection,
        state.mandible_flare,
        state.overall_taper,
    ];
    vals.iter().cloned().fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn fc_blend(a: &FaceContourState, b: &FaceContourState, t: f32) -> FaceContourState {
    let t = t.clamp(0.0, 1.0);
    FaceContourState {
        temporal_width: a.temporal_width + (b.temporal_width - a.temporal_width) * t,
        zygomatic_projection: a.zygomatic_projection
            + (b.zygomatic_projection - a.zygomatic_projection) * t,
        mandible_flare: a.mandible_flare + (b.mandible_flare - a.mandible_flare) * t,
        overall_taper: a.overall_taper + (b.overall_taper - a.overall_taper) * t,
    }
}

#[allow(dead_code)]
pub fn fc_to_weights(state: &FaceContourState) -> Vec<(String, f32)> {
    vec![
        ("face_temporal_width".to_string(), state.temporal_width),
        (
            "face_zygomatic_proj".to_string(),
            state.zygomatic_projection,
        ),
        ("face_mandible_flare".to_string(), state.mandible_flare),
        ("face_overall_taper".to_string(), state.overall_taper),
    ]
}

#[allow(dead_code)]
pub fn fc_to_json(state: &FaceContourState) -> String {
    format!(
        r#"{{"temporal_width":{:.4},"zygomatic_projection":{:.4},"mandible_flare":{:.4},"overall_taper":{:.4}}}"#,
        state.temporal_width, state.zygomatic_projection, state.mandible_flare, state.overall_taper
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_face_contour_config();
        assert!((cfg.max_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn new_state_neutral() {
        let s = new_face_contour_state();
        assert!(fc_is_neutral(&s));
    }

    #[test]
    fn set_temporal_clamps() {
        let cfg = default_face_contour_config();
        let mut s = new_face_contour_state();
        fc_set_temporal(&mut s, &cfg, 5.0);
        assert!((s.temporal_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_zygomatic() {
        let cfg = default_face_contour_config();
        let mut s = new_face_contour_state();
        fc_set_zygomatic(&mut s, &cfg, 0.6);
        assert!((s.zygomatic_projection - 0.6).abs() < 1e-6);
    }

    #[test]
    fn set_mandible() {
        let cfg = default_face_contour_config();
        let mut s = new_face_contour_state();
        fc_set_mandible(&mut s, &cfg, 0.4);
        assert!((s.mandible_flare - 0.4).abs() < 1e-6);
    }

    #[test]
    fn contour_intensity_max() {
        let cfg = default_face_contour_config();
        let mut s = new_face_contour_state();
        fc_set_zygomatic(&mut s, &cfg, 0.9);
        fc_set_temporal(&mut s, &cfg, 0.3);
        assert!((fc_contour_intensity(&s) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn reset_clears() {
        let cfg = default_face_contour_config();
        let mut s = new_face_contour_state();
        fc_set_taper(&mut s, &cfg, 0.5);
        fc_reset(&mut s);
        assert!(fc_is_neutral(&s));
    }

    #[test]
    fn blend_midpoint() {
        let a = new_face_contour_state();
        let cfg = default_face_contour_config();
        let mut b = new_face_contour_state();
        fc_set_temporal(&mut b, &cfg, 1.0);
        let mid = fc_blend(&a, &b, 0.5);
        assert!((mid.temporal_width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn to_weights_count() {
        let s = new_face_contour_state();
        assert_eq!(fc_to_weights(&s).len(), 4);
    }

    #[test]
    fn to_json_fields() {
        let s = new_face_contour_state();
        let j = fc_to_json(&s);
        assert!(j.contains("temporal_width"));
        assert!(j.contains("overall_taper"));
    }
}
