// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Kyphosis (hunchback) spinal curve morph.

/// Kyphosis configuration.
#[derive(Debug, Clone)]
pub struct KyphosisMorphConfig {
    pub curve_degree: f32,
    pub apex_height: f32,
    pub spread: f32,
}

impl Default for KyphosisMorphConfig {
    fn default() -> Self {
        Self {
            curve_degree: 0.0,
            apex_height: 0.5,
            spread: 0.3,
        }
    }
}

/// Kyphosis morph state.
#[derive(Debug, Clone)]
pub struct KyphosisMorph {
    pub config: KyphosisMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl KyphosisMorph {
    pub fn new() -> Self {
        Self {
            config: KyphosisMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for KyphosisMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new KyphosisMorph.
pub fn new_kyphosis_morph() -> KyphosisMorph {
    KyphosisMorph::new()
}

/// Set curve degree (0.0–1.0).
pub fn kyphosis_set_curve(morph: &mut KyphosisMorph, degree: f32) {
    morph.config.curve_degree = degree.clamp(0.0, 1.0);
}

/// Set apex height along the spine (0.0–1.0, 0=base, 1=top).
pub fn kyphosis_set_apex(morph: &mut KyphosisMorph, height: f32) {
    morph.config.apex_height = height.clamp(0.0, 1.0);
}

/// Set curve spread factor.
pub fn kyphosis_set_spread(morph: &mut KyphosisMorph, spread: f32) {
    morph.config.spread = spread.clamp(0.0, 1.0);
}

/// Evaluate Gaussian influence at a normalized spine position.
pub fn kyphosis_evaluate(morph: &KyphosisMorph, spine_t: f32) -> f32 {
    let d = spine_t - morph.config.apex_height;
    let spread = morph.config.spread.max(1e-4);
    let gauss = (-d * d / (2.0 * spread * spread)).exp();
    morph.intensity * morph.config.curve_degree * gauss
}

/// Serialize to JSON.
pub fn kyphosis_to_json(morph: &KyphosisMorph) -> String {
    format!(
        r#"{{"intensity":{},"curve_degree":{},"apex_height":{},"spread":{}}}"#,
        morph.intensity, morph.config.curve_degree, morph.config.apex_height, morph.config.spread,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let m = new_kyphosis_morph();
        assert!((m.config.apex_height - 0.5).abs() < 1e-6 /* apex at midpoint */);
    }

    #[test]
    fn test_set_curve_clamp_high() {
        let mut m = new_kyphosis_morph();
        kyphosis_set_curve(&mut m, 2.0);
        assert!((m.config.curve_degree - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_set_apex() {
        let mut m = new_kyphosis_morph();
        kyphosis_set_apex(&mut m, 0.7);
        assert!((m.config.apex_height - 0.7).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_set_spread() {
        let mut m = new_kyphosis_morph();
        kyphosis_set_spread(&mut m, 0.4);
        assert!((m.config.spread - 0.4).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_evaluate_zero_intensity() {
        let m = new_kyphosis_morph();
        let v = kyphosis_evaluate(&m, 0.5);
        assert!((v - 0.0).abs() < 1e-6 /* zero intensity gives zero */);
    }

    #[test]
    fn test_evaluate_peak_at_apex() {
        let mut m = new_kyphosis_morph();
        kyphosis_set_curve(&mut m, 1.0);
        kyphosis_set_apex(&mut m, 0.5);
        m.intensity = 1.0;
        let peak = kyphosis_evaluate(&m, 0.5);
        let off = kyphosis_evaluate(&m, 0.9);
        assert!(peak > off /* peak at apex */);
    }

    #[test]
    fn test_evaluate_range() {
        let mut m = new_kyphosis_morph();
        kyphosis_set_curve(&mut m, 1.0);
        m.intensity = 1.0;
        let v = kyphosis_evaluate(&m, 0.5);
        assert!((0.0..=1.0).contains(&v) /* within range */);
    }

    #[test]
    fn test_json_keys() {
        let m = new_kyphosis_morph();
        let j = kyphosis_to_json(&m);
        assert!(j.contains("curve_degree") /* key present */);
    }

    #[test]
    fn test_default_trait() {
        let m = KyphosisMorph::default();
        assert!(m.enabled /* enabled by default */);
    }

    #[test]
    fn test_clone_integrity() {
        let m = new_kyphosis_morph();
        let c = m.clone();
        assert!((c.config.spread - m.config.spread).abs() < 1e-6 /* clone equal */);
    }
}
