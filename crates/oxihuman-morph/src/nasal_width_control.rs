// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Nasal width control — overall nasal width including alar base.

use std::f32::consts::FRAC_PI_4;

/// Configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalWidthConfig {
    /// Reference alar angle (informational).
    pub alar_ref_rad: f32,
    /// Scale factor applied to alar output.
    pub scale: f32,
}

impl Default for NasalWidthConfig {
    fn default() -> Self {
        NasalWidthConfig {
            alar_ref_rad: FRAC_PI_4,
            scale: 1.0,
        }
    }
}

/// Runtime state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NasalWidthState {
    /// Overall width in `[0.0, 1.0]`.
    width: f32,
    /// Alar base flare in `[0.0, 1.0]`.
    alar_flare: f32,
    /// Bridge width in `[0.0, 1.0]`.
    bridge: f32,
    config: NasalWidthConfig,
}

/// Default config.
pub fn default_nasal_width_config() -> NasalWidthConfig {
    NasalWidthConfig::default()
}

/// New neutral state.
pub fn new_nasal_width_state(config: NasalWidthConfig) -> NasalWidthState {
    NasalWidthState {
        width: 0.5,
        alar_flare: 0.0,
        bridge: 0.5,
        config,
    }
}

/// Set overall width.
pub fn nwc_set_width(state: &mut NasalWidthState, v: f32) {
    state.width = v.clamp(0.0, 1.0);
}

/// Set alar flare.
pub fn nwc_set_alar_flare(state: &mut NasalWidthState, v: f32) {
    state.alar_flare = v.clamp(0.0, 1.0);
}

/// Set bridge width.
pub fn nwc_set_bridge(state: &mut NasalWidthState, v: f32) {
    state.bridge = v.clamp(0.0, 1.0);
}

/// Reset.
pub fn nwc_reset(state: &mut NasalWidthState) {
    state.width = 0.5;
    state.alar_flare = 0.0;
    state.bridge = 0.5;
}

/// True when neutral.
pub fn nwc_is_neutral(state: &NasalWidthState) -> bool {
    (state.width - 0.5).abs() < 1e-5 && state.alar_flare < 1e-5 && (state.bridge - 0.5).abs() < 1e-5
}

/// Effective total width including alar flare.
pub fn nwc_effective_width(state: &NasalWidthState) -> f32 {
    (state.width + state.alar_flare * 0.3).clamp(0.0, 1.0)
}

/// Morph weights: `[width, alar_flare, bridge]`.
pub fn nwc_to_weights(state: &NasalWidthState) -> [f32; 3] {
    let s = state.config.scale;
    [
        (state.width * s).clamp(0.0, 1.0),
        (state.alar_flare * s).clamp(0.0, 1.0),
        (state.bridge * s).clamp(0.0, 1.0),
    ]
}

/// Blend.
pub fn nwc_blend(a: &NasalWidthState, b: &NasalWidthState, t: f32) -> NasalWidthState {
    let t = t.clamp(0.0, 1.0);
    NasalWidthState {
        width: a.width + (b.width - a.width) * t,
        alar_flare: a.alar_flare + (b.alar_flare - a.alar_flare) * t,
        bridge: a.bridge + (b.bridge - a.bridge) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn nwc_to_json(state: &NasalWidthState) -> String {
    format!(
        r#"{{"width":{:.4},"alar_flare":{:.4},"bridge":{:.4}}}"#,
        state.width, state.alar_flare, state.bridge
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> NasalWidthState {
        new_nasal_width_state(default_nasal_width_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(nwc_is_neutral(&make()));
    }

    #[test]
    fn set_width_clamps() {
        let mut s = make();
        nwc_set_width(&mut s, 5.0);
        assert!((s.width - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_neutral() {
        let mut s = make();
        nwc_set_width(&mut s, 0.1);
        nwc_reset(&mut s);
        assert!(nwc_is_neutral(&s));
    }

    #[test]
    fn effective_width_in_range() {
        let s = make();
        assert!((0.0..=1.0).contains(&nwc_effective_width(&s)));
    }

    #[test]
    fn weights_in_range() {
        let s = make();
        for v in nwc_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut a = make();
        let mut b = make();
        nwc_set_width(&mut a, 0.0);
        nwc_set_width(&mut b, 1.0);
        let m = nwc_blend(&a, &b, 0.5);
        assert!((m.width - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_at_one_is_b() {
        let mut b = make();
        nwc_set_alar_flare(&mut b, 0.9);
        let r = nwc_blend(&make(), &b, 1.0);
        assert!((r.alar_flare - 0.9).abs() < 1e-5);
    }

    #[test]
    fn json_has_width() {
        assert!(nwc_to_json(&make()).contains("width"));
    }

    #[test]
    fn alar_flare_clamped_negative() {
        let mut s = make();
        nwc_set_alar_flare(&mut s, -3.0);
        assert!(s.alar_flare >= 0.0);
    }
}
