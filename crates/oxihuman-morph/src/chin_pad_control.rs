// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Chin pad control — soft-tissue padding / volume at the chin tip.

use std::f32::consts::FRAC_PI_6;

/// Configuration for chin pad.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinPadConfig {
    /// Reference angle (informational, used in arc approximation).
    pub ref_arc_rad: f32,
    /// Maximum allowed projection.
    pub max_projection: f32,
}

impl Default for ChinPadConfig {
    fn default() -> Self {
        ChinPadConfig {
            ref_arc_rad: FRAC_PI_6,
            max_projection: 1.0,
        }
    }
}

/// Runtime state for chin pad control.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinPadState {
    /// Pad volume in `[0.0, 1.0]`.
    volume: f32,
    /// Downward projection in `[0.0, 1.0]`.
    projection: f32,
    /// Lateral spread in `[0.0, 1.0]`.
    spread: f32,
    config: ChinPadConfig,
}

/// Default config.
pub fn default_chin_pad_config() -> ChinPadConfig {
    ChinPadConfig::default()
}

/// Create a neutral state.
pub fn new_chin_pad_state(config: ChinPadConfig) -> ChinPadState {
    ChinPadState {
        volume: 0.0,
        projection: 0.0,
        spread: 0.0,
        config,
    }
}

/// Set the pad volume.
pub fn cpd_set_volume(state: &mut ChinPadState, v: f32) {
    state.volume = v.clamp(0.0, 1.0);
}

/// Set the downward projection.
pub fn cpd_set_projection(state: &mut ChinPadState, v: f32) {
    state.projection = v.clamp(0.0, 1.0);
}

/// Set the lateral spread.
pub fn cpd_set_spread(state: &mut ChinPadState, v: f32) {
    state.spread = v.clamp(0.0, 1.0);
}

/// Reset all to zero.
pub fn cpd_reset(state: &mut ChinPadState) {
    state.volume = 0.0;
    state.projection = 0.0;
    state.spread = 0.0;
}

/// True when state is effectively zero.
pub fn cpd_is_neutral(state: &ChinPadState) -> bool {
    state.volume < 1e-5 && state.projection < 1e-5 && state.spread < 1e-5
}

/// Combine volume and projection into an overall pad size.
pub fn cpd_pad_size(state: &ChinPadState) -> f32 {
    (state.volume * 0.6 + state.projection * 0.4).clamp(0.0, 1.0)
}

/// Produce morph weights: `[volume, projection, spread]`.
pub fn cpd_to_weights(state: &ChinPadState) -> [f32; 3] {
    [state.volume, state.projection, state.spread]
}

/// Blend between two states.
pub fn cpd_blend(a: &ChinPadState, b: &ChinPadState, t: f32) -> ChinPadState {
    let t = t.clamp(0.0, 1.0);
    ChinPadState {
        volume: a.volume + (b.volume - a.volume) * t,
        projection: a.projection + (b.projection - a.projection) * t,
        spread: a.spread + (b.spread - a.spread) * t,
        config: a.config.clone(),
    }
}

/// Serialise.
pub fn cpd_to_json(state: &ChinPadState) -> String {
    format!(
        r#"{{"volume":{:.4},"projection":{:.4},"spread":{:.4}}}"#,
        state.volume, state.projection, state.spread
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ChinPadState {
        new_chin_pad_state(default_chin_pad_config())
    }

    #[test]
    fn neutral_on_creation() {
        assert!(cpd_is_neutral(&make()));
    }

    #[test]
    fn set_volume_clamps() {
        let mut s = make();
        cpd_set_volume(&mut s, 2.0);
        assert!((s.volume - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reset_zeros_all() {
        let mut s = make();
        cpd_set_volume(&mut s, 0.5);
        cpd_reset(&mut s);
        assert!(cpd_is_neutral(&s));
    }

    #[test]
    fn pad_size_in_range() {
        let mut s = make();
        cpd_set_volume(&mut s, 0.5);
        cpd_set_projection(&mut s, 0.5);
        assert!((0.0..=1.0).contains(&cpd_pad_size(&s)));
    }

    #[test]
    fn weights_all_in_range() {
        let mut s = make();
        cpd_set_volume(&mut s, 0.3);
        cpd_set_projection(&mut s, 0.7);
        for v in cpd_to_weights(&s) {
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn blend_midpoint() {
        let mut b = make();
        cpd_set_volume(&mut b, 1.0);
        let m = cpd_blend(&make(), &b, 0.5);
        assert!((m.volume - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_at_zero_is_a() {
        let a = make();
        let m = cpd_blend(&a, &make(), 0.0);
        assert!((m.volume - a.volume).abs() < 1e-5);
    }

    #[test]
    fn json_has_volume() {
        assert!(cpd_to_json(&make()).contains("volume"));
    }

    #[test]
    fn spread_clamped_negative() {
        let mut s = make();
        cpd_set_spread(&mut s, -5.0);
        assert!(s.spread >= 0.0);
    }
}
