// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pelvis tilt control — sagittal (anterior/posterior) and frontal (lateral) tilt morphs.

use std::f32::consts::PI;

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct PelvisTiltConfig {
    /// Max anterior tilt angle in radians.
    pub max_anterior_rad: f32,
    /// Max posterior tilt angle in radians.
    pub max_posterior_rad: f32,
    /// Max lateral tilt angle in radians.
    pub max_lateral_rad: f32,
}

impl Default for PelvisTiltConfig {
    fn default() -> Self {
        Self {
            max_anterior_rad: PI / 8.0,
            max_posterior_rad: PI / 10.0,
            max_lateral_rad: PI / 12.0,
        }
    }
}

/// Pelvis tilt state, all values -1..=1.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PelvisTiltState {
    /// Sagittal tilt: positive = anterior tilt (front down), negative = posterior (back down).
    pub sagittal: f32,
    /// Frontal tilt: positive = right hip up (left lateral tilt), negative = left hip up.
    pub frontal: f32,
}

#[allow(dead_code)]
pub fn new_pelvis_tilt_state() -> PelvisTiltState {
    PelvisTiltState::default()
}

#[allow(dead_code)]
pub fn default_pelvis_tilt_config() -> PelvisTiltConfig {
    PelvisTiltConfig::default()
}

#[allow(dead_code)]
pub fn pt_set_sagittal(state: &mut PelvisTiltState, v: f32) {
    state.sagittal = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn pt_set_frontal(state: &mut PelvisTiltState, v: f32) {
    state.frontal = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn pt_reset(state: &mut PelvisTiltState) {
    *state = PelvisTiltState::default();
}

#[allow(dead_code)]
pub fn pt_is_neutral(state: &PelvisTiltState) -> bool {
    state.sagittal.abs() < 1e-4 && state.frontal.abs() < 1e-4
}

/// Sagittal angle in radians (positive = anterior).
#[allow(dead_code)]
pub fn pt_sagittal_angle_rad(state: &PelvisTiltState, cfg: &PelvisTiltConfig) -> f32 {
    if state.sagittal >= 0.0 {
        state.sagittal * cfg.max_anterior_rad
    } else {
        state.sagittal * cfg.max_posterior_rad
    }
}

/// Frontal angle in radians.
#[allow(dead_code)]
pub fn pt_frontal_angle_rad(state: &PelvisTiltState, cfg: &PelvisTiltConfig) -> f32 {
    state.frontal * cfg.max_lateral_rad
}

/// Combined tilt magnitude.
#[allow(dead_code)]
pub fn pt_magnitude(state: &PelvisTiltState) -> f32 {
    (state.sagittal * state.sagittal + state.frontal * state.frontal).sqrt()
}

/// Returns morph weights \[anterior, posterior, lateral_right, lateral_left\].
#[allow(dead_code)]
pub fn pt_to_weights(state: &PelvisTiltState) -> [f32; 4] {
    [
        state.sagittal.max(0.0),
        (-state.sagittal).max(0.0),
        state.frontal.max(0.0),
        (-state.frontal).max(0.0),
    ]
}

#[allow(dead_code)]
pub fn pt_blend(a: &PelvisTiltState, b: &PelvisTiltState, t: f32) -> PelvisTiltState {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    PelvisTiltState {
        sagittal: a.sagittal * inv + b.sagittal * t,
        frontal: a.frontal * inv + b.frontal * t,
    }
}

#[allow(dead_code)]
pub fn pt_to_json(state: &PelvisTiltState) -> String {
    format!(
        "{{\"sagittal\":{:.4},\"frontal\":{:.4}}}",
        state.sagittal, state.frontal
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn default_is_neutral() {
        assert!(pt_is_neutral(&new_pelvis_tilt_state()));
    }

    #[test]
    fn sagittal_clamps_high() {
        let mut s = new_pelvis_tilt_state();
        pt_set_sagittal(&mut s, 5.0);
        assert!((s.sagittal - 1.0).abs() < 1e-6);
    }

    #[test]
    fn frontal_clamps_low() {
        let mut s = new_pelvis_tilt_state();
        pt_set_frontal(&mut s, -5.0);
        assert!((s.frontal + 1.0).abs() < 1e-6);
    }

    #[test]
    fn reset_works() {
        let mut s = new_pelvis_tilt_state();
        pt_set_sagittal(&mut s, 0.7);
        pt_reset(&mut s);
        assert!(pt_is_neutral(&s));
    }

    #[test]
    fn sagittal_anterior_angle_positive() {
        let cfg = default_pelvis_tilt_config();
        let mut s = new_pelvis_tilt_state();
        pt_set_sagittal(&mut s, 1.0);
        let a = pt_sagittal_angle_rad(&s, &cfg);
        assert!(a > 0.0 && a <= PI / 8.0 + 1e-5);
    }

    #[test]
    fn sagittal_posterior_angle_negative() {
        let cfg = default_pelvis_tilt_config();
        let mut s = new_pelvis_tilt_state();
        pt_set_sagittal(&mut s, -1.0);
        let a = pt_sagittal_angle_rad(&s, &cfg);
        assert!(a < 0.0);
    }

    #[test]
    fn magnitude_diagonal() {
        let mut s = new_pelvis_tilt_state();
        pt_set_sagittal(&mut s, 1.0);
        pt_set_frontal(&mut s, 1.0);
        let m = pt_magnitude(&s);
        assert!((m - 2.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn weights_four_elements() {
        let w = pt_to_weights(&new_pelvis_tilt_state());
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn blend_midpoint() {
        let mut b = new_pelvis_tilt_state();
        pt_set_sagittal(&mut b, 1.0);
        let r = pt_blend(&new_pelvis_tilt_state(), &b, 0.5);
        assert!((r.sagittal - 0.5).abs() < 1e-5);
    }

    #[test]
    fn json_has_keys() {
        let j = pt_to_json(&new_pelvis_tilt_state());
        assert!(j.contains("sagittal") && j.contains("frontal"));
    }
}
