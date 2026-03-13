// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Skin flush/blush morph stub.

/// Flush trigger cause.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlushCause {
    Emotion,
    Heat,
    Exercise,
    Alcohol,
    Rosacea,
}

/// Skin flush morph controller.
#[derive(Debug, Clone)]
pub struct FlushMorph {
    pub cause: FlushCause,
    pub intensity: f32,
    pub spread: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl FlushMorph {
    pub fn new(morph_count: usize) -> Self {
        FlushMorph {
            cause: FlushCause::Emotion,
            intensity: 0.0,
            spread: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new flush morph.
pub fn new_flush_morph(morph_count: usize) -> FlushMorph {
    FlushMorph::new(morph_count)
}

/// Set flush cause.
pub fn flm_set_cause(morph: &mut FlushMorph, cause: FlushCause) {
    morph.cause = cause;
}

/// Set flush intensity.
pub fn flm_set_intensity(morph: &mut FlushMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Set spread factor.
pub fn flm_set_spread(morph: &mut FlushMorph, spread: f32) {
    morph.spread = spread.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn flm_evaluate(morph: &FlushMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn flm_set_enabled(morph: &mut FlushMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn flm_to_json(morph: &FlushMorph) -> String {
    let cause = match morph.cause {
        FlushCause::Emotion => "emotion",
        FlushCause::Heat => "heat",
        FlushCause::Exercise => "exercise",
        FlushCause::Alcohol => "alcohol",
        FlushCause::Rosacea => "rosacea",
    };
    format!(
        r#"{{"cause":"{}","intensity":{},"spread":{},"morph_count":{},"enabled":{}}}"#,
        cause, morph.intensity, morph.spread, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cause() {
        let m = new_flush_morph(4);
        assert_eq!(
            m.cause,
            FlushCause::Emotion /* default cause must be Emotion */
        );
    }

    #[test]
    fn test_set_cause() {
        let mut m = new_flush_morph(4);
        flm_set_cause(&mut m, FlushCause::Exercise);
        assert_eq!(m.cause, FlushCause::Exercise /* cause must be set */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_flush_morph(4);
        flm_set_intensity(&mut m, 1.5);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_spread_clamp() {
        let mut m = new_flush_morph(4);
        flm_set_spread(&mut m, -0.5);
        assert!(m.spread.abs() < 1e-6 /* clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_flush_morph(5);
        flm_set_intensity(&mut m, 0.4);
        assert_eq!(
            flm_evaluate(&m).len(),
            5 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_flush_morph(4);
        flm_set_enabled(&mut m, false);
        assert!(flm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_cause() {
        let m = new_flush_morph(4);
        let j = flm_to_json(&m);
        assert!(j.contains("\"cause\"") /* JSON must have cause */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_flush_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_flush_morph(3);
        flm_set_intensity(&mut m, 0.8);
        let out = flm_evaluate(&m);
        assert!((out[0] - 0.8).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
