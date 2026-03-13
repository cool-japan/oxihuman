// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Lower lip body shape morph — controls lower vermilion, chin-lip groove, and projection.

/// Lower lip body morph configuration.
#[derive(Debug, Clone)]
pub struct LowerLipBodyMorph {
    pub fullness: f32,
    pub projection: f32,
    pub vermilion_height: f32,
    pub labiomental_groove: f32,
    pub roll: f32,
}

impl LowerLipBodyMorph {
    pub fn new() -> Self {
        Self {
            fullness: 0.5,
            projection: 0.5,
            vermilion_height: 0.5,
            labiomental_groove: 0.5,
            roll: 0.0,
        }
    }
}

impl Default for LowerLipBodyMorph {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new lower lip body morph.
pub fn new_lower_lip_body_morph() -> LowerLipBodyMorph {
    LowerLipBodyMorph::new()
}

/// Set lower lip fullness.
pub fn llb_set_fullness(m: &mut LowerLipBodyMorph, v: f32) {
    m.fullness = v.clamp(0.0, 1.0);
}

/// Set lower lip projection.
pub fn llb_set_projection(m: &mut LowerLipBodyMorph, v: f32) {
    m.projection = v.clamp(0.0, 1.0);
}

/// Set lower vermilion height.
pub fn llb_set_vermilion_height(m: &mut LowerLipBodyMorph, v: f32) {
    m.vermilion_height = v.clamp(0.0, 1.0);
}

/// Set labiomental (chin-lip) groove depth.
pub fn llb_set_labiomental_groove(m: &mut LowerLipBodyMorph, v: f32) {
    m.labiomental_groove = v.clamp(0.0, 1.0);
}

/// Compute volume estimate for lower lip.
pub fn llb_volume_estimate(m: &LowerLipBodyMorph) -> f32 {
    m.fullness * m.projection * m.vermilion_height
}

/// Serialize to JSON-like string.
pub fn lower_lip_body_morph_to_json(m: &LowerLipBodyMorph) -> String {
    format!(
        r#"{{"fullness":{:.4},"projection":{:.4},"vermilion_height":{:.4},"labiomental_groove":{:.4},"roll":{:.4}}}"#,
        m.fullness, m.projection, m.vermilion_height, m.labiomental_groove, m.roll
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let m = new_lower_lip_body_morph();
        assert!((m.fullness - 0.5).abs() < 1e-6);
        assert_eq!(m.roll, 0.0);
    }

    #[test]
    fn test_fullness_clamp() {
        let mut m = new_lower_lip_body_morph();
        llb_set_fullness(&mut m, 2.0);
        assert_eq!(m.fullness, 1.0);
    }

    #[test]
    fn test_projection_set() {
        let mut m = new_lower_lip_body_morph();
        llb_set_projection(&mut m, 0.7);
        assert!((m.projection - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_vermilion_height_set() {
        let mut m = new_lower_lip_body_morph();
        llb_set_vermilion_height(&mut m, 0.6);
        assert!((m.vermilion_height - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_labiomental_groove_set() {
        let mut m = new_lower_lip_body_morph();
        llb_set_labiomental_groove(&mut m, 0.8);
        assert!((m.labiomental_groove - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_labiomental_groove_clamp() {
        let mut m = new_lower_lip_body_morph();
        llb_set_labiomental_groove(&mut m, 3.0);
        assert_eq!(m.labiomental_groove, 1.0);
    }

    #[test]
    fn test_volume_estimate_positive() {
        let m = new_lower_lip_body_morph();
        assert!(llb_volume_estimate(&m) > 0.0);
    }

    #[test]
    fn test_volume_estimate_zero_fullness() {
        let mut m = new_lower_lip_body_morph();
        m.fullness = 0.0;
        assert_eq!(llb_volume_estimate(&m), 0.0);
    }

    #[test]
    fn test_json_keys() {
        let m = new_lower_lip_body_morph();
        let s = lower_lip_body_morph_to_json(&m);
        assert!(s.contains("labiomental_groove"));
    }

    #[test]
    fn test_clone() {
        let m = new_lower_lip_body_morph();
        let m2 = m.clone();
        assert!((m2.projection - m.projection).abs() < 1e-6);
    }
}
