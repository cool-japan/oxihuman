// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Slouch and forward head posture morph.

/// Configuration for slouch morph.
#[derive(Debug, Clone)]
pub struct SlouchMorphConfig {
    pub slouch_degree: f32,
    pub head_forward: f32,
    pub shoulder_round: f32,
}

impl Default for SlouchMorphConfig {
    fn default() -> Self {
        Self {
            slouch_degree: 0.0,
            head_forward: 0.0,
            shoulder_round: 0.0,
        }
    }
}

/// Slouch morph state.
#[derive(Debug, Clone)]
pub struct SlouchMorph {
    pub config: SlouchMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl SlouchMorph {
    pub fn new() -> Self {
        Self {
            config: SlouchMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for SlouchMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new SlouchMorph.
pub fn new_slouch_morph() -> SlouchMorph {
    SlouchMorph::new()
}

/// Set slouch degree (0.0–1.0).
pub fn slouch_set_degree(morph: &mut SlouchMorph, degree: f32) {
    morph.config.slouch_degree = degree.clamp(0.0, 1.0);
}

/// Set forward head translation factor.
pub fn slouch_set_head_forward(morph: &mut SlouchMorph, v: f32) {
    morph.config.head_forward = v.clamp(0.0, 1.0);
}

/// Set shoulder rounding factor.
pub fn slouch_set_shoulder_round(morph: &mut SlouchMorph, v: f32) {
    morph.config.shoulder_round = v.clamp(0.0, 1.0);
}

/// Apply morph to a weight buffer.
#[allow(clippy::needless_range_loop)]
pub fn slouch_apply(morph: &SlouchMorph, weights: &mut [f32]) {
    let scale = morph.intensity * morph.config.slouch_degree;
    for i in 0..weights.len() {
        weights[i] = (weights[i] + scale).min(1.0);
    }
}

/// Serialize to JSON.
pub fn slouch_to_json(morph: &SlouchMorph) -> String {
    format!(
        r#"{{"intensity":{},"slouch_degree":{},"head_forward":{},"shoulder_round":{}}}"#,
        morph.intensity,
        morph.config.slouch_degree,
        morph.config.head_forward,
        morph.config.shoulder_round,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_slouch_morph();
        assert!((m.config.slouch_degree - 0.0).abs() < 1e-6 /* defaults zero */);
    }

    #[test]
    fn test_degree_clamp_high() {
        let mut m = new_slouch_morph();
        slouch_set_degree(&mut m, 3.0);
        assert!((m.config.slouch_degree - 1.0).abs() < 1e-6 /* clamped to 1 */);
    }

    #[test]
    fn test_degree_clamp_low() {
        let mut m = new_slouch_morph();
        slouch_set_degree(&mut m, -1.0);
        assert!((m.config.slouch_degree - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn test_head_forward() {
        let mut m = new_slouch_morph();
        slouch_set_head_forward(&mut m, 0.7);
        assert!((m.config.head_forward - 0.7).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_shoulder_round() {
        let mut m = new_slouch_morph();
        slouch_set_shoulder_round(&mut m, 0.5);
        assert!((m.config.shoulder_round - 0.5).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_apply_increases_weights() {
        let mut m = new_slouch_morph();
        slouch_set_degree(&mut m, 0.5);
        m.intensity = 1.0;
        let mut w = vec![0.0f32, 0.0f32];
        slouch_apply(&m, &mut w);
        assert!(w[0] > 0.0 /* increased */);
    }

    #[test]
    fn test_apply_clamps_to_one() {
        let mut m = new_slouch_morph();
        slouch_set_degree(&mut m, 1.0);
        m.intensity = 1.0;
        let mut w = vec![1.0f32];
        slouch_apply(&m, &mut w);
        assert!((w[0] - 1.0).abs() < 1e-6 /* capped at 1 */);
    }

    #[test]
    fn test_json_contains_key() {
        let m = new_slouch_morph();
        let j = slouch_to_json(&m);
        assert!(j.contains("slouch_degree") /* key present */);
    }

    #[test]
    fn test_default() {
        let m = SlouchMorph::default();
        assert!(m.enabled /* default enabled */);
    }

    #[test]
    fn test_clone() {
        let m = new_slouch_morph();
        let c = m.clone();
        assert!((c.intensity - m.intensity).abs() < 1e-6 /* equal */);
    }
}
