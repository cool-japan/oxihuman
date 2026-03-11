// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Pear body shape morph — wider hips/thighs, narrower shoulders.

/// Configuration for the pear shape morph.
#[derive(Debug, Clone)]
pub struct PearShapeConfig {
    pub hip_width: f32,
    pub thigh_fullness: f32,
    pub shoulder_slim: f32,
}

impl Default for PearShapeConfig {
    fn default() -> Self {
        PearShapeConfig {
            hip_width: 0.8,
            thigh_fullness: 0.75,
            shoulder_slim: 0.6,
        }
    }
}

/// State for the pear shape morph.
#[derive(Debug, Clone)]
pub struct PearShapeMorph {
    pub intensity: f32,
    pub config: PearShapeConfig,
    pub enabled: bool,
}

/// Create a new pear shape morph.
pub fn new_pear_shape_morph() -> PearShapeMorph {
    PearShapeMorph {
        intensity: 0.0,
        config: PearShapeConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn pear_set_intensity(m: &mut PearShapeMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Hip width weight.
pub fn pear_hip_width(m: &PearShapeMorph) -> f32 {
    m.intensity * m.config.hip_width
}

/// Thigh fullness weight.
pub fn pear_thigh_fullness(m: &PearShapeMorph) -> f32 {
    m.intensity * m.config.thigh_fullness
}

/// Shoulder slimming weight.
pub fn pear_shoulder_slim(m: &PearShapeMorph) -> f32 {
    m.intensity * m.config.shoulder_slim
}

/// Hip-to-shoulder ratio estimate.
pub fn pear_hip_shoulder_ratio(m: &PearShapeMorph) -> f32 {
    1.0 + 0.35 * m.intensity
}

/// Serialise to JSON.
pub fn pear_to_json(m: &PearShapeMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"hip_width":{:.3},"enabled":{}}}"#,
        m.intensity,
        pear_hip_width(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_pear_shape_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp() {
        let mut m = new_pear_shape_morph();
        pear_set_intensity(&mut m, 2.0);
        assert!((m.intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn hip_width_at_max() {
        let mut m = new_pear_shape_morph();
        pear_set_intensity(&mut m, 1.0);
        assert!((pear_hip_width(&m) - m.config.hip_width).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn thigh_fullness_zero_at_zero() {
        let m = new_pear_shape_morph();
        assert!((pear_thigh_fullness(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn hip_shoulder_ratio_increases() {
        let mut m = new_pear_shape_morph();
        pear_set_intensity(&mut m, 0.0);
        let r0 = pear_hip_shoulder_ratio(&m);
        pear_set_intensity(&mut m, 1.0);
        let r1 = pear_hip_shoulder_ratio(&m);
        assert!(r1 > r0 /* wider hips relative to shoulders */);
    }

    #[test]
    fn json_has_hip_width() {
        let m = new_pear_shape_morph();
        assert!(pear_to_json(&m).contains("hip_width") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let m = new_pear_shape_morph();
        assert!(m.enabled /* enabled */);
    }

    #[test]
    fn config_thigh_positive() {
        let m = new_pear_shape_morph();
        assert!(m.config.thigh_fullness > 0.0 /* valid */);
    }
}
