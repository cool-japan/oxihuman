// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Erythema (redness) morph stub.

/// Erythema pattern type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErythemaPattern {
    Diffuse,
    Macular,
    Papular,
    Reticular,
}

/// Erythema morph controller.
#[derive(Debug, Clone)]
pub struct ErythemaMorph {
    pub pattern: ErythemaPattern,
    pub intensity: f32,
    pub affected_area: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl ErythemaMorph {
    pub fn new(morph_count: usize) -> Self {
        ErythemaMorph {
            pattern: ErythemaPattern::Diffuse,
            intensity: 0.0,
            affected_area: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new erythema morph.
pub fn new_erythema_morph(morph_count: usize) -> ErythemaMorph {
    ErythemaMorph::new(morph_count)
}

/// Set erythema pattern.
pub fn erm_set_pattern(morph: &mut ErythemaMorph, pattern: ErythemaPattern) {
    morph.pattern = pattern;
}

/// Set redness intensity.
pub fn erm_set_intensity(morph: &mut ErythemaMorph, intensity: f32) {
    morph.intensity = intensity.clamp(0.0, 1.0);
}

/// Set affected area fraction.
pub fn erm_set_affected_area(morph: &mut ErythemaMorph, area: f32) {
    morph.affected_area = area.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: uniform from intensity).
pub fn erm_evaluate(morph: &ErythemaMorph) -> Vec<f32> {
    /* Stub: uniform weight from intensity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.intensity; morph.morph_count]
}

/// Enable or disable.
pub fn erm_set_enabled(morph: &mut ErythemaMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn erm_to_json(morph: &ErythemaMorph) -> String {
    let pat = match morph.pattern {
        ErythemaPattern::Diffuse => "diffuse",
        ErythemaPattern::Macular => "macular",
        ErythemaPattern::Papular => "papular",
        ErythemaPattern::Reticular => "reticular",
    };
    format!(
        r#"{{"pattern":"{}","intensity":{},"affected_area":{},"morph_count":{},"enabled":{}}}"#,
        pat, morph.intensity, morph.affected_area, morph.morph_count, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pattern() {
        let m = new_erythema_morph(4);
        assert_eq!(
            m.pattern,
            ErythemaPattern::Diffuse /* default pattern must be Diffuse */
        );
    }

    #[test]
    fn test_set_pattern() {
        let mut m = new_erythema_morph(4);
        erm_set_pattern(&mut m, ErythemaPattern::Macular);
        assert_eq!(
            m.pattern,
            ErythemaPattern::Macular /* pattern must be set */
        );
    }

    #[test]
    fn test_intensity_clamp() {
        let mut m = new_erythema_morph(4);
        erm_set_intensity(&mut m, 1.5);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_area_clamp() {
        let mut m = new_erythema_morph(4);
        erm_set_affected_area(&mut m, -0.2);
        assert!(m.affected_area.abs() < 1e-6 /* clamped to 0.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_erythema_morph(7);
        erm_set_intensity(&mut m, 0.5);
        assert_eq!(
            erm_evaluate(&m).len(),
            7 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_erythema_morph(4);
        erm_set_enabled(&mut m, false);
        assert!(erm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_pattern() {
        let m = new_erythema_morph(4);
        let j = erm_to_json(&m);
        assert!(j.contains("\"pattern\"") /* JSON must have pattern */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_erythema_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_intensity() {
        let mut m = new_erythema_morph(4);
        erm_set_intensity(&mut m, 0.7);
        let out = erm_evaluate(&m);
        assert!((out[0] - 0.7).abs() < 1e-5 /* evaluate must match intensity */);
    }
}
