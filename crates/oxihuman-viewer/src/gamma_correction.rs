// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gamma correction — encode/decode gamma curves for display pipeline.

/// Gamma curve type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GammaMode {
    /// sRGB standard piecewise curve.
    #[default]
    Srgb,
    /// Simple power-law gamma.
    Power(f32),
    /// Linear (no gamma correction).
    Linear,
}

/// State.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GammaCorrectionState {
    pub mode: GammaMode,
    /// Additional lift (black level offset) -0.1..=0.1.
    pub lift: f32,
    /// Additional gain multiplier 0.5..=2.0.
    pub gain: f32,
}

impl Default for GammaCorrectionState {
    fn default() -> Self {
        Self {
            mode: GammaMode::Srgb,
            lift: 0.0,
            gain: 1.0,
        }
    }
}

#[allow(dead_code)]
pub fn new_gamma_correction_state() -> GammaCorrectionState {
    GammaCorrectionState::default()
}

/// Encode a linear value to gamma space.
#[allow(dead_code)]
pub fn gamma_encode(linear: f32, mode: GammaMode) -> f32 {
    let v = linear.clamp(0.0, 1.0);
    match mode {
        GammaMode::Linear => v,
        GammaMode::Power(g) => v.powf(1.0 / g.max(1e-6)),
        GammaMode::Srgb => {
            if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            }
        }
    }
}

/// Decode a gamma-space value to linear.
#[allow(dead_code)]
pub fn gamma_decode(encoded: f32, mode: GammaMode) -> f32 {
    let v = encoded.clamp(0.0, 1.0);
    match mode {
        GammaMode::Linear => v,
        GammaMode::Power(g) => v.powf(g.max(1e-6)),
        GammaMode::Srgb => {
            if v <= 0.040_45 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }
    }
}

/// Apply full gamma correction pipeline to an RGB triplet.
#[allow(dead_code)]
pub fn gc_apply(rgb: [f32; 3], state: &GammaCorrectionState) -> [f32; 3] {
    rgb.map(|c| {
        let c = (c * state.gain + state.lift).clamp(0.0, 1.0);
        gamma_encode(c, state.mode)
    })
}

#[allow(dead_code)]
pub fn gc_set_lift(state: &mut GammaCorrectionState, v: f32) {
    state.lift = v.clamp(-0.1, 0.1);
}

#[allow(dead_code)]
pub fn gc_set_gain(state: &mut GammaCorrectionState, v: f32) {
    state.gain = v.clamp(0.5, 2.0);
}

#[allow(dead_code)]
pub fn gc_reset(state: &mut GammaCorrectionState) {
    *state = GammaCorrectionState::default();
}

#[allow(dead_code)]
pub fn gc_is_identity(state: &GammaCorrectionState) -> bool {
    matches!(state.mode, GammaMode::Linear)
        && state.lift.abs() < 1e-5
        && (state.gain - 1.0).abs() < 1e-5
}

#[allow(dead_code)]
pub fn gc_to_json(state: &GammaCorrectionState) -> String {
    format!("{{\"lift\":{:.4},\"gain\":{:.4}}}", state.lift, state.gain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_srgb_roundtrip() {
        let v = 0.5_f32;
        let enc = gamma_encode(v, GammaMode::Srgb);
        let dec = gamma_decode(enc, GammaMode::Srgb);
        assert!((dec - v).abs() < 1e-4);
    }

    #[test]
    fn linear_mode_identity() {
        let v = 0.7_f32;
        assert!((gamma_encode(v, GammaMode::Linear) - v).abs() < 1e-6);
        assert!((gamma_decode(v, GammaMode::Linear) - v).abs() < 1e-6);
    }

    #[test]
    fn power_gamma_encode_increases() {
        // gamma > 1 encodes lighter
        let a = gamma_encode(0.5, GammaMode::Power(2.2));
        assert!(a > 0.5);
    }

    #[test]
    fn clamps_input_high() {
        assert!((gamma_encode(5.0, GammaMode::Srgb) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn clamps_input_low() {
        assert!(gamma_encode(-1.0, GammaMode::Srgb) < 1e-6);
    }

    #[test]
    fn lift_clamps() {
        let mut s = new_gamma_correction_state();
        gc_set_lift(&mut s, 5.0);
        assert!((s.lift - 0.1).abs() < 1e-6);
    }

    #[test]
    fn gain_clamps() {
        let mut s = new_gamma_correction_state();
        gc_set_gain(&mut s, 0.0);
        assert!((s.gain - 0.5).abs() < 1e-6);
    }

    #[test]
    fn reset_to_default() {
        let mut s = new_gamma_correction_state();
        gc_set_gain(&mut s, 2.0);
        gc_reset(&mut s);
        assert!((s.gain - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_produces_valid_range() {
        let s = new_gamma_correction_state();
        let out = gc_apply([0.5, 0.3, 0.8], &s);
        assert!(out.iter().all(|&v| (0.0..=1.0).contains(&v)));
    }

    #[test]
    fn json_has_keys() {
        let j = gc_to_json(&new_gamma_correction_state());
        assert!(j.contains("lift") && j.contains("gain"));
    }
}
