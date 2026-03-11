// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cupid's bow (upper lip arch) shape control.

use std::f32::consts::FRAC_PI_2;

/// State.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LipBowState {
    /// Arch height of the Cupid's bow (0..1).
    pub arch_height: f32,
    /// Sharpness of the central dip (0 = smooth, 1 = sharp).
    pub dip_sharpness: f32,
    /// Width spread of the bow peaks (0.5 = narrow, 1.5 = wide).
    pub peak_spread: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct LipBowConfig {
    pub max_arch: f32,
    pub max_spread: f32,
}

impl Default for LipBowConfig {
    fn default() -> Self {
        Self {
            max_arch: 1.0,
            max_spread: 1.5,
        }
    }
}
impl Default for LipBowState {
    fn default() -> Self {
        Self {
            arch_height: 0.0,
            dip_sharpness: 0.3,
            peak_spread: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_lip_bow_state() -> LipBowState {
    LipBowState::default()
}

#[allow(dead_code)]
pub fn default_lip_bow_config() -> LipBowConfig {
    LipBowConfig::default()
}

#[allow(dead_code)]
pub fn lb_set_arch(state: &mut LipBowState, cfg: &LipBowConfig, v: f32) {
    state.arch_height = v.clamp(0.0, cfg.max_arch);
}

#[allow(dead_code)]
pub fn lb_set_dip(state: &mut LipBowState, v: f32) {
    state.dip_sharpness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn lb_set_spread(state: &mut LipBowState, cfg: &LipBowConfig, v: f32) {
    state.peak_spread = v.clamp(0.5, cfg.max_spread);
}

#[allow(dead_code)]
pub fn lb_reset(state: &mut LipBowState) {
    *state = LipBowState::default();
}

#[allow(dead_code)]
pub fn lb_is_flat(state: &LipBowState) -> bool {
    state.arch_height < 1e-4
}

#[allow(dead_code)]
pub fn lb_blend(a: &LipBowState, b: &LipBowState, t: f32) -> LipBowState {
    let t = t.clamp(0.0, 1.0);
    LipBowState {
        arch_height: a.arch_height + (b.arch_height - a.arch_height) * t,
        dip_sharpness: a.dip_sharpness + (b.dip_sharpness - a.dip_sharpness) * t,
        peak_spread: a.peak_spread + (b.peak_spread - a.peak_spread) * t,
    }
}

/// Approximate bow width in normalised units (uses FRAC_PI_2 internally).
#[allow(dead_code)]
pub fn lb_bow_width(state: &LipBowState) -> f32 {
    state.peak_spread * FRAC_PI_2.cos().abs() + 0.1
}

#[allow(dead_code)]
pub fn lb_to_weights(state: &LipBowState) -> [f32; 3] {
    [
        state.arch_height,
        state.dip_sharpness,
        (state.peak_spread - 0.5) / 1.0,
    ]
}

#[allow(dead_code)]
pub fn lb_to_json(state: &LipBowState) -> String {
    format!(
        "{{\"arch\":{:.4},\"dip\":{:.4},\"spread\":{:.4}}}",
        state.arch_height, state.dip_sharpness, state.peak_spread
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flat() {
        assert!(lb_is_flat(&new_lip_bow_state()));
    }

    #[test]
    fn set_arch_clamps_max() {
        let mut s = new_lip_bow_state();
        let cfg = default_lip_bow_config();
        lb_set_arch(&mut s, &cfg, 10.0);
        assert!(s.arch_height <= cfg.max_arch);
    }

    #[test]
    fn arch_not_negative() {
        let mut s = new_lip_bow_state();
        let cfg = default_lip_bow_config();
        lb_set_arch(&mut s, &cfg, -1.0);
        assert!(s.arch_height >= 0.0);
    }

    #[test]
    fn spread_clamps_min() {
        let mut s = new_lip_bow_state();
        let cfg = default_lip_bow_config();
        lb_set_spread(&mut s, &cfg, 0.0);
        assert!(s.peak_spread >= 0.5);
    }

    #[test]
    fn reset_flat() {
        let mut s = new_lip_bow_state();
        let cfg = default_lip_bow_config();
        lb_set_arch(&mut s, &cfg, 0.8);
        lb_reset(&mut s);
        assert!(lb_is_flat(&s));
    }

    #[test]
    fn blend_half() {
        let cfg = default_lip_bow_config();
        let mut a = new_lip_bow_state();
        let mut b = new_lip_bow_state();
        lb_set_arch(&mut a, &cfg, 0.0);
        lb_set_arch(&mut b, &cfg, 1.0);
        let m = lb_blend(&a, &b, 0.5);
        assert!((m.arch_height - 0.5).abs() < 1e-4);
    }

    #[test]
    fn bow_width_positive() {
        assert!(lb_bow_width(&new_lip_bow_state()) > 0.0);
    }

    #[test]
    fn weights_len() {
        assert_eq!(lb_to_weights(&new_lip_bow_state()).len(), 3);
    }

    #[test]
    fn json_has_arch() {
        assert!(lb_to_json(&new_lip_bow_state()).contains("arch"));
    }

    #[test]
    fn dip_clamped() {
        let mut s = new_lip_bow_state();
        lb_set_dip(&mut s, 5.0);
        assert!(s.dip_sharpness <= 1.0);
    }
}
