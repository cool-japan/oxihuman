// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Acne lesion surface morph stub.

/// Acne lesion type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcneLesionType {
    Comedone,
    Papule,
    Pustule,
    Nodule,
    Cyst,
}

/// An acne lesion entry.
#[derive(Debug, Clone)]
pub struct AcneLesion {
    pub lesion_type: AcneLesionType,
    pub position: [f32; 3],
    pub size: f32,
    pub inflammation: f32,
}

/// Acne morph controller.
#[derive(Debug, Clone)]
pub struct AcneMorph {
    pub lesions: Vec<AcneLesion>,
    pub severity: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl AcneMorph {
    pub fn new(morph_count: usize) -> Self {
        AcneMorph {
            lesions: Vec::new(),
            severity: 0.0,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new acne morph.
pub fn new_acne_morph(morph_count: usize) -> AcneMorph {
    AcneMorph::new(morph_count)
}

/// Add a lesion.
pub fn acm_add_lesion(morph: &mut AcneMorph, lesion: AcneLesion) {
    morph.lesions.push(lesion);
}

/// Set overall severity.
pub fn acm_set_severity(morph: &mut AcneMorph, severity: f32) {
    morph.severity = severity.clamp(0.0, 1.0);
}

/// Clear all lesions.
pub fn acm_clear(morph: &mut AcneMorph) {
    morph.lesions.clear();
}

/// Evaluate morph weights (stub: uniform from severity).
pub fn acm_evaluate(morph: &AcneMorph) -> Vec<f32> {
    /* Stub: uniform weight from severity */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    vec![morph.severity; morph.morph_count]
}

/// Enable or disable.
pub fn acm_set_enabled(morph: &mut AcneMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Return lesion count.
pub fn acm_lesion_count(morph: &AcneMorph) -> usize {
    morph.lesions.len()
}

/// Serialize to JSON-like string.
pub fn acm_to_json(morph: &AcneMorph) -> String {
    format!(
        r#"{{"lesion_count":{},"severity":{},"morph_count":{},"enabled":{}}}"#,
        morph.lesions.len(),
        morph.severity,
        morph.morph_count,
        morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lesion() -> AcneLesion {
        AcneLesion {
            lesion_type: AcneLesionType::Papule,
            position: [0.0, 0.1, 0.0],
            size: 0.01,
            inflammation: 0.7,
        }
    }

    #[test]
    fn test_initial_empty() {
        let m = new_acne_morph(4);
        assert_eq!(acm_lesion_count(&m), 0 /* no lesions initially */);
    }

    #[test]
    fn test_add_lesion() {
        let mut m = new_acne_morph(4);
        acm_add_lesion(&mut m, make_lesion());
        assert_eq!(acm_lesion_count(&m), 1 /* one lesion after add */);
    }

    #[test]
    fn test_clear() {
        let mut m = new_acne_morph(4);
        acm_add_lesion(&mut m, make_lesion());
        acm_clear(&mut m);
        assert_eq!(acm_lesion_count(&m), 0 /* cleared */);
    }

    #[test]
    fn test_severity_clamp() {
        let mut m = new_acne_morph(4);
        acm_set_severity(&mut m, 1.5);
        assert!((m.severity - 1.0).abs() < 1e-6 /* clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let mut m = new_acne_morph(6);
        acm_set_severity(&mut m, 0.5);
        assert_eq!(
            acm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_acne_morph(4);
        acm_set_enabled(&mut m, false);
        assert!(acm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_to_json_has_lesion_count() {
        let m = new_acne_morph(4);
        let j = acm_to_json(&m);
        assert!(j.contains("\"lesion_count\"") /* JSON must have lesion_count */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_acne_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_evaluate_matches_severity() {
        let mut m = new_acne_morph(3);
        acm_set_severity(&mut m, 0.4);
        let out = acm_evaluate(&m);
        assert!((out[0] - 0.4).abs() < 1e-5 /* evaluate must match severity */);
    }

    #[test]
    fn test_lesion_type_variant() {
        let l = make_lesion();
        assert_eq!(
            l.lesion_type,
            AcneLesionType::Papule /* lesion type must be Papule */
        );
    }
}
