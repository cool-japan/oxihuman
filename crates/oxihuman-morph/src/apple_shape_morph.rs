// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Apple body shape morph — weight centred around mid-section/abdomen.

/// Configuration for the apple shape morph.
#[derive(Debug, Clone)]
pub struct AppleShapeConfig {
    pub abdomen_fullness: f32,
    pub chest_roundness: f32,
    pub limb_slenderness: f32,
}

impl Default for AppleShapeConfig {
    fn default() -> Self {
        AppleShapeConfig {
            abdomen_fullness: 0.85,
            chest_roundness: 0.6,
            limb_slenderness: 0.7,
        }
    }
}

/// State for the apple shape morph.
#[derive(Debug, Clone)]
pub struct AppleShapeMorph {
    pub intensity: f32,
    pub config: AppleShapeConfig,
    pub enabled: bool,
}

/// Create a new apple shape morph.
pub fn new_apple_shape_morph() -> AppleShapeMorph {
    AppleShapeMorph {
        intensity: 0.0,
        config: AppleShapeConfig::default(),
        enabled: true,
    }
}

/// Set intensity [0, 1].
pub fn apple_set_intensity(m: &mut AppleShapeMorph, v: f32) {
    m.intensity = v.clamp(0.0, 1.0);
}

/// Abdomen fullness weight.
pub fn apple_abdomen(m: &AppleShapeMorph) -> f32 {
    m.intensity * m.config.abdomen_fullness
}

/// Chest roundness weight.
pub fn apple_chest(m: &AppleShapeMorph) -> f32 {
    m.intensity * m.config.chest_roundness
}

/// Limb slenderness weight (apple shapes tend to have slimmer limbs).
pub fn apple_limb_slim(m: &AppleShapeMorph) -> f32 {
    m.intensity * m.config.limb_slenderness
}

/// Waist circumference estimate (relative to neutral).
pub fn apple_waist_scale(m: &AppleShapeMorph) -> f32 {
    1.0 + 0.3 * apple_abdomen(m)
}

/// Serialise to JSON.
pub fn apple_to_json(m: &AppleShapeMorph) -> String {
    format!(
        r#"{{"intensity":{:.3},"abdomen":{:.3},"enabled":{}}}"#,
        m.intensity,
        apple_abdomen(m),
        m.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_zero() {
        let m = new_apple_shape_morph();
        assert!((m.intensity - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn clamp() {
        let mut m = new_apple_shape_morph();
        apple_set_intensity(&mut m, -5.0);
        assert!((m.intensity - 0.0).abs() < 1e-6 /* clamped to 0 */);
    }

    #[test]
    fn abdomen_at_max() {
        let mut m = new_apple_shape_morph();
        apple_set_intensity(&mut m, 1.0);
        assert!((apple_abdomen(&m) - m.config.abdomen_fullness).abs() < 1e-6 /* correct */);
    }

    #[test]
    fn chest_zero_at_zero() {
        let m = new_apple_shape_morph();
        assert!((apple_chest(&m) - 0.0).abs() < 1e-6 /* zero */);
    }

    #[test]
    fn waist_scale_increases() {
        let mut m = new_apple_shape_morph();
        apple_set_intensity(&mut m, 0.0);
        let w0 = apple_waist_scale(&m);
        apple_set_intensity(&mut m, 1.0);
        let w1 = apple_waist_scale(&m);
        assert!(w1 > w0 /* larger waist with intensity */);
    }

    #[test]
    fn limb_slim_positive_at_half() {
        let mut m = new_apple_shape_morph();
        apple_set_intensity(&mut m, 0.5);
        assert!(apple_limb_slim(&m) > 0.0 /* positive slim */);
    }

    #[test]
    fn json_has_abdomen() {
        let m = new_apple_shape_morph();
        assert!(apple_to_json(&m).contains("abdomen") /* json has field */);
    }

    #[test]
    fn enabled_default() {
        let m = new_apple_shape_morph();
        assert!(m.enabled /* enabled */);
    }
}
