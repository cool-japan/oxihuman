// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Jaw protrusion (prognathism / retrognathism) control.

use std::f32::consts::FRAC_PI_8;

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct JawProtrusionState {
    /// Protrusion amount (-1 = receded, 0 = neutral, 1 = protruded).
    pub protrusion: f32,
    /// Vertical inclination of mandibular plane, degrees.
    pub mandibular_plane_deg: f32,
    /// Asymmetric shift (-1 left, 0 neutral, 1 right).
    pub lateral_shift: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct JawProtrusionConfig {
    pub max_protrusion: f32,
    pub max_plane_deg: f32,
}

impl Default for JawProtrusionConfig {
    fn default() -> Self {
        Self {
            max_protrusion: 1.0,
            max_plane_deg: 15.0,
        }
    }
}
impl Default for JawProtrusionState {
    fn default() -> Self {
        Self {
            protrusion: 0.0,
            mandibular_plane_deg: 0.0,
            lateral_shift: 0.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_jaw_protrusion_state() -> JawProtrusionState {
    JawProtrusionState::default()
}

#[allow(dead_code)]
pub fn default_jaw_protrusion_config() -> JawProtrusionConfig {
    JawProtrusionConfig::default()
}

#[allow(dead_code)]
pub fn jp_set_protrusion(state: &mut JawProtrusionState, cfg: &JawProtrusionConfig, v: f32) {
    state.protrusion = v.clamp(-cfg.max_protrusion, cfg.max_protrusion);
}

#[allow(dead_code)]
pub fn jp_set_plane(state: &mut JawProtrusionState, cfg: &JawProtrusionConfig, deg: f32) {
    state.mandibular_plane_deg = deg.clamp(-cfg.max_plane_deg, cfg.max_plane_deg);
}

#[allow(dead_code)]
pub fn jp_set_lateral(state: &mut JawProtrusionState, v: f32) {
    state.lateral_shift = v.clamp(-1.0, 1.0);
}

#[allow(dead_code)]
pub fn jp_reset(state: &mut JawProtrusionState) {
    *state = JawProtrusionState::default();
}

#[allow(dead_code)]
pub fn jp_is_neutral(state: &JawProtrusionState) -> bool {
    state.protrusion.abs() < 1e-4
        && state.mandibular_plane_deg.abs() < 1e-4
        && state.lateral_shift.abs() < 1e-4
}

#[allow(dead_code)]
pub fn jp_blend(a: &JawProtrusionState, b: &JawProtrusionState, t: f32) -> JawProtrusionState {
    let t = t.clamp(0.0, 1.0);
    JawProtrusionState {
        protrusion: a.protrusion + (b.protrusion - a.protrusion) * t,
        mandibular_plane_deg: a.mandibular_plane_deg
            + (b.mandibular_plane_deg - a.mandibular_plane_deg) * t,
        lateral_shift: a.lateral_shift + (b.lateral_shift - a.lateral_shift) * t,
    }
}

/// Approximate horizontal jaw offset in normalised units.
#[allow(dead_code)]
pub fn jp_horizontal_offset(state: &JawProtrusionState) -> f32 {
    state.protrusion * (state.mandibular_plane_deg.to_radians() + FRAC_PI_8).cos()
}

#[allow(dead_code)]
pub fn jp_to_weights(state: &JawProtrusionState) -> [f32; 3] {
    [
        state.protrusion,
        state.mandibular_plane_deg / 15.0,
        state.lateral_shift,
    ]
}

#[allow(dead_code)]
pub fn jp_to_json(state: &JawProtrusionState) -> String {
    format!(
        "{{\"protrusion\":{:.4},\"plane_deg\":{:.4},\"lateral\":{:.4}}}",
        state.protrusion, state.mandibular_plane_deg, state.lateral_shift
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_neutral() {
        assert!(jp_is_neutral(&new_jaw_protrusion_state()));
    }

    #[test]
    fn protrusion_clamp_max() {
        let mut s = new_jaw_protrusion_state();
        let cfg = default_jaw_protrusion_config();
        jp_set_protrusion(&mut s, &cfg, 5.0);
        assert!(s.protrusion <= cfg.max_protrusion);
    }

    #[test]
    fn protrusion_clamp_min() {
        let mut s = new_jaw_protrusion_state();
        let cfg = default_jaw_protrusion_config();
        jp_set_protrusion(&mut s, &cfg, -5.0);
        assert!(s.protrusion >= -cfg.max_protrusion);
    }

    #[test]
    fn plane_clamp() {
        let mut s = new_jaw_protrusion_state();
        let cfg = default_jaw_protrusion_config();
        jp_set_plane(&mut s, &cfg, 999.0);
        assert!(s.mandibular_plane_deg <= cfg.max_plane_deg);
    }

    #[test]
    fn lateral_clamp() {
        let mut s = new_jaw_protrusion_state();
        jp_set_lateral(&mut s, 5.0);
        assert!(s.lateral_shift <= 1.0);
    }

    #[test]
    fn reset_neutral() {
        let mut s = new_jaw_protrusion_state();
        let cfg = default_jaw_protrusion_config();
        jp_set_protrusion(&mut s, &cfg, 0.5);
        jp_reset(&mut s);
        assert!(jp_is_neutral(&s));
    }

    #[test]
    fn blend_half() {
        let cfg = default_jaw_protrusion_config();
        let mut a = new_jaw_protrusion_state();
        let mut b = new_jaw_protrusion_state();
        jp_set_protrusion(&mut a, &cfg, 0.0);
        jp_set_protrusion(&mut b, &cfg, 1.0);
        let m = jp_blend(&a, &b, 0.5);
        assert!((m.protrusion - 0.5).abs() < 1e-4);
    }

    #[test]
    fn horizontal_offset_finite() {
        let s = new_jaw_protrusion_state();
        assert!(jp_horizontal_offset(&s).is_finite());
    }

    #[test]
    fn weights_len() {
        assert_eq!(jp_to_weights(&new_jaw_protrusion_state()).len(), 3);
    }

    #[test]
    fn json_has_protrusion() {
        assert!(jp_to_json(&new_jaw_protrusion_state()).contains("protrusion"));
    }
}
