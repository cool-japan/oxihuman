// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lip volume/plumpness morph stub.

/// Lip area to control.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LipArea {
    Upper,
    Lower,
    Both,
    Cupid,
    Corners,
}

/// Lip volume morph controller.
#[derive(Debug, Clone)]
pub struct LipVolumeMorph {
    pub area: LipArea,
    pub volume: f32,
    pub projection: f32,
    pub definition: f32,
    pub morph_count: usize,
    pub enabled: bool,
}

impl LipVolumeMorph {
    pub fn new(morph_count: usize) -> Self {
        LipVolumeMorph {
            area: LipArea::Both,
            volume: 0.5,
            projection: 0.5,
            definition: 0.5,
            morph_count,
            enabled: true,
        }
    }
}

/// Create a new lip volume morph controller.
pub fn new_lip_volume_morph(morph_count: usize) -> LipVolumeMorph {
    LipVolumeMorph::new(morph_count)
}

/// Set target lip area.
pub fn lvm_set_area(morph: &mut LipVolumeMorph, area: LipArea) {
    morph.area = area;
}

/// Set lip volume.
pub fn lvm_set_volume(morph: &mut LipVolumeMorph, volume: f32) {
    morph.volume = volume.clamp(0.0, 1.0);
}

/// Set lip projection (forward push).
pub fn lvm_set_projection(morph: &mut LipVolumeMorph, projection: f32) {
    morph.projection = projection.clamp(0.0, 1.0);
}

/// Set lip edge definition.
pub fn lvm_set_definition(morph: &mut LipVolumeMorph, definition: f32) {
    morph.definition = definition.clamp(0.0, 1.0);
}

/// Evaluate morph weights (stub: combined volume and projection).
pub fn lvm_evaluate(morph: &LipVolumeMorph) -> Vec<f32> {
    /* Stub: weight from volume and projection average */
    if !morph.enabled || morph.morph_count == 0 {
        return vec![];
    }
    let w = (morph.volume + morph.projection) * 0.5;
    vec![w.clamp(0.0, 1.0); morph.morph_count]
}

/// Enable or disable.
pub fn lvm_set_enabled(morph: &mut LipVolumeMorph, enabled: bool) {
    morph.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn lvm_to_json(morph: &LipVolumeMorph) -> String {
    let area = match morph.area {
        LipArea::Upper => "upper",
        LipArea::Lower => "lower",
        LipArea::Both => "both",
        LipArea::Cupid => "cupid",
        LipArea::Corners => "corners",
    };
    format!(
        r#"{{"area":"{}","volume":{},"projection":{},"definition":{},"enabled":{}}}"#,
        area, morph.volume, morph.projection, morph.definition, morph.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_area() {
        let m = new_lip_volume_morph(4);
        assert_eq!(m.area, LipArea::Both /* default area must be Both */);
    }

    #[test]
    fn test_set_area() {
        let mut m = new_lip_volume_morph(4);
        lvm_set_area(&mut m, LipArea::Upper);
        assert_eq!(m.area, LipArea::Upper /* area must be set */);
    }

    #[test]
    fn test_volume_clamped() {
        let mut m = new_lip_volume_morph(4);
        lvm_set_volume(&mut m, 2.0);
        assert!((m.volume - 1.0).abs() < 1e-6 /* volume clamped to 1.0 */);
    }

    #[test]
    fn test_projection_clamped() {
        let mut m = new_lip_volume_morph(4);
        lvm_set_projection(&mut m, -1.0);
        assert!((m.projection).abs() < 1e-6 /* projection clamped to 0.0 */);
    }

    #[test]
    fn test_definition_clamped() {
        let mut m = new_lip_volume_morph(4);
        lvm_set_definition(&mut m, 1.5);
        assert!((m.definition - 1.0).abs() < 1e-6 /* definition clamped to 1.0 */);
    }

    #[test]
    fn test_evaluate_length() {
        let m = new_lip_volume_morph(6);
        assert_eq!(
            lvm_evaluate(&m).len(),
            6 /* output must match morph_count */
        );
    }

    #[test]
    fn test_evaluate_disabled() {
        let mut m = new_lip_volume_morph(4);
        lvm_set_enabled(&mut m, false);
        assert!(lvm_evaluate(&m).is_empty() /* disabled must return empty */);
    }

    #[test]
    fn test_evaluate_avg() {
        let mut m = new_lip_volume_morph(2);
        lvm_set_volume(&mut m, 0.6);
        lvm_set_projection(&mut m, 0.4);
        let out = lvm_evaluate(&m);
        assert!((out[0] - 0.5).abs() < 1e-5 /* weight must be average of volume and projection */);
    }

    #[test]
    fn test_to_json_has_area() {
        let m = new_lip_volume_morph(4);
        let j = lvm_to_json(&m);
        assert!(j.contains("\"area\"") /* JSON must have area */);
    }

    #[test]
    fn test_enabled_default() {
        let m = new_lip_volume_morph(4);
        assert!(m.enabled /* must be enabled by default */);
    }
}
