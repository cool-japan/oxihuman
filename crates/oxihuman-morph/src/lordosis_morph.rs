// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Lordosis (swayback) lumbar spinal curve morph.

/// Lordosis configuration.
#[derive(Debug, Clone)]
pub struct LordosisMorphConfig {
    pub curve_degree: f32,
    pub lumbar_apex: f32,
    pub anterior_tilt: f32,
}

impl Default for LordosisMorphConfig {
    fn default() -> Self {
        Self {
            curve_degree: 0.0,
            lumbar_apex: 0.3,
            anterior_tilt: 0.0,
        }
    }
}

/// Lordosis morph state.
#[derive(Debug, Clone)]
pub struct LordosisMorph {
    pub config: LordosisMorphConfig,
    pub intensity: f32,
    pub enabled: bool,
}

impl LordosisMorph {
    pub fn new() -> Self {
        Self {
            config: LordosisMorphConfig::default(),
            intensity: 0.0,
            enabled: true,
        }
    }
}

impl Default for LordosisMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new LordosisMorph.
pub fn new_lordosis_morph() -> LordosisMorph {
    LordosisMorph::new()
}

/// Set lordotic curve degree.
pub fn lordosis_set_curve(morph: &mut LordosisMorph, degree: f32) {
    morph.config.curve_degree = degree.clamp(0.0, 1.0);
}

/// Set lumbar apex position along the spine (0.0–1.0).
pub fn lordosis_set_lumbar_apex(morph: &mut LordosisMorph, apex: f32) {
    morph.config.lumbar_apex = apex.clamp(0.0, 1.0);
}

/// Set anterior pelvic tilt factor.
pub fn lordosis_set_anterior_tilt(morph: &mut LordosisMorph, tilt: f32) {
    morph.config.anterior_tilt = tilt.clamp(-1.0, 1.0);
}

/// Evaluate morph influence at a normalized spine position.
pub fn lordosis_evaluate(morph: &LordosisMorph, spine_t: f32) -> f32 {
    let d = spine_t - morph.config.lumbar_apex;
    let gauss = (-d * d * 20.0).exp();
    morph.intensity * morph.config.curve_degree * gauss
}

/// Serialize to JSON.
pub fn lordosis_to_json(morph: &LordosisMorph) -> String {
    format!(
        r#"{{"intensity":{},"curve_degree":{},"lumbar_apex":{},"anterior_tilt":{}}}"#,
        morph.intensity,
        morph.config.curve_degree,
        morph.config.lumbar_apex,
        morph.config.anterior_tilt,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_lordosis_morph();
        assert!((m.config.lumbar_apex - 0.3).abs() < 1e-6 /* default apex */);
    }

    #[test]
    fn test_set_curve() {
        let mut m = new_lordosis_morph();
        lordosis_set_curve(&mut m, 0.8);
        assert!((m.config.curve_degree - 0.8).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_clamp_high() {
        let mut m = new_lordosis_morph();
        lordosis_set_curve(&mut m, 10.0);
        assert!((m.config.curve_degree - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_lumbar_apex() {
        let mut m = new_lordosis_morph();
        lordosis_set_lumbar_apex(&mut m, 0.25);
        assert!((m.config.lumbar_apex - 0.25).abs() < 1e-6 /* stored */);
    }

    #[test]
    fn test_anterior_tilt_negative() {
        let mut m = new_lordosis_morph();
        lordosis_set_anterior_tilt(&mut m, -2.0);
        assert!((m.config.anterior_tilt - (-1.0)).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_evaluate_zero() {
        let m = new_lordosis_morph();
        assert!((lordosis_evaluate(&m, 0.3) - 0.0).abs() < 1e-6 /* zero intensity */);
    }

    #[test]
    fn test_evaluate_peak() {
        let mut m = new_lordosis_morph();
        lordosis_set_curve(&mut m, 1.0);
        lordosis_set_lumbar_apex(&mut m, 0.3);
        m.intensity = 1.0;
        let peak = lordosis_evaluate(&m, 0.3);
        let off = lordosis_evaluate(&m, 0.9);
        assert!(peak > off /* peak at apex */);
    }

    #[test]
    fn test_json_contains() {
        let m = new_lordosis_morph();
        let j = lordosis_to_json(&m);
        assert!(j.contains("lumbar_apex") /* key present */);
    }

    #[test]
    fn test_default() {
        let m = LordosisMorph::default();
        assert!(m.enabled /* enabled */);
    }
}
