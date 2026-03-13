// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Standing posture and sway morph.

/// Configuration for posture morph blending.
#[derive(Debug, Clone)]
pub struct PostureMorphConfig {
    pub sway_amplitude: f32,
    pub forward_lean: f32,
    pub lateral_lean: f32,
}

impl Default for PostureMorphConfig {
    fn default() -> Self {
        Self {
            sway_amplitude: 0.0,
            forward_lean: 0.0,
            lateral_lean: 0.0,
        }
    }
}

/// Posture morph state.
#[derive(Debug, Clone)]
pub struct PostureMorph {
    pub config: PostureMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl PostureMorph {
    pub fn new() -> Self {
        Self {
            config: PostureMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for PostureMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new PostureMorph with default config.
pub fn new_posture_morph() -> PostureMorph {
    PostureMorph::new()
}

/// Set overall sway intensity (0.0–1.0).
pub fn posture_set_sway(morph: &mut PostureMorph, amplitude: f32) {
    morph.config.sway_amplitude = amplitude.clamp(0.0, 1.0);
}

/// Set forward lean offset.
pub fn posture_set_forward_lean(morph: &mut PostureMorph, lean: f32) {
    morph.config.forward_lean = lean.clamp(-1.0, 1.0);
}

/// Set lateral lean offset.
pub fn posture_set_lateral_lean(morph: &mut PostureMorph, lean: f32) {
    morph.config.lateral_lean = lean.clamp(-1.0, 1.0);
}

/// Apply morph weights to a vertex buffer.
#[allow(clippy::needless_range_loop)]
pub fn posture_apply_weights(morph: &PostureMorph, weights: &mut [f32]) {
    let scale = morph.intensity * morph.config.sway_amplitude;
    for i in 0..weights.len() {
        weights[i] *= scale;
    }
}

/// Serialize to JSON string.
pub fn posture_to_json(morph: &PostureMorph) -> String {
    format!(
        r#"{{"intensity":{},"sway":{},"forward_lean":{},"lateral_lean":{}}}"#,
        morph.intensity,
        morph.config.sway_amplitude,
        morph.config.forward_lean,
        morph.config.lateral_lean,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_posture_morph() {
        let m = new_posture_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* default intensity zero */);
    }

    #[test]
    fn test_set_sway_clamp() {
        let mut m = new_posture_morph();
        posture_set_sway(&mut m, 2.5);
        assert!((m.config.sway_amplitude - 1.0).abs() < 1e-6 /* clamped to 1 */);
    }

    #[test]
    fn test_set_sway_negative() {
        let mut m = new_posture_morph();
        posture_set_sway(&mut m, -0.5);
        assert!((m.config.sway_amplitude - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn test_forward_lean() {
        let mut m = new_posture_morph();
        posture_set_forward_lean(&mut m, 0.3);
        assert!((m.config.forward_lean - 0.3).abs() < 1e-6 /* value stored */);
    }

    #[test]
    fn test_lateral_lean_clamp() {
        let mut m = new_posture_morph();
        posture_set_lateral_lean(&mut m, 5.0);
        assert!((m.config.lateral_lean - 1.0).abs() < 1e-6 /* clamped upper */);
    }

    #[test]
    fn test_apply_weights_empty() {
        let m = new_posture_morph();
        let mut w: Vec<f32> = vec![];
        posture_apply_weights(&m, &mut w);
        assert!(w.is_empty() /* no panic on empty */);
    }

    #[test]
    fn test_apply_weights_zero_intensity() {
        let mut m = new_posture_morph();
        posture_set_sway(&mut m, 1.0);
        m.intensity = 0.0;
        let mut w = vec![1.0f32, 1.0];
        posture_apply_weights(&m, &mut w);
        assert!((w[0] - 0.0).abs() < 1e-6 /* scaled to zero */);
    }

    #[test]
    fn test_json_output() {
        let m = new_posture_morph();
        let j = posture_to_json(&m);
        assert!(j.contains("intensity") /* json key present */);
    }

    #[test]
    fn test_default_enabled() {
        let m = PostureMorph::default();
        assert!(m.enabled /* default enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_posture_morph();
        let m2 = m.clone();
        assert!((m2.intensity - m.intensity).abs() < 1e-6 /* clone equal */);
    }
}
