// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Adam's apple (laryngeal prominence) morph control.
//!
//! Models the thyroid cartilage protrusion that varies with sex, age,
//! and body composition. Provides smooth interpolation and asymmetry support.

use std::f32::consts::PI;

/// Configuration for the adam's apple morph.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AdamAppleConfig {
    /// Prominence factor in 0..=1 (0 = flat, 1 = maximal).
    pub prominence: f32,
    /// Vertical position offset from neutral (-1..=1).
    pub vertical_offset: f32,
    /// Lateral asymmetry (-1 = left shift, 1 = right shift).
    pub asymmetry: f32,
    /// Width of the cartilage bulge in normalised units.
    pub width: f32,
    /// Sharpness of the ridge (0 = round, 1 = sharp).
    pub sharpness: f32,
}

impl Default for AdamAppleConfig {
    fn default() -> Self {
        Self {
            prominence: 0.5,
            vertical_offset: 0.0,
            asymmetry: 0.0,
            width: 0.3,
            sharpness: 0.4,
        }
    }
}

/// Result of evaluating the adam's apple morph.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AdamAppleResult {
    /// Per-vertex displacement weights (index, weight).
    pub weights: Vec<(usize, f32)>,
    /// Peak displacement magnitude.
    pub peak_displacement: f32,
    /// Effective volume change estimate.
    pub volume_delta: f32,
}

/// Compute the prominence profile curve.
///
/// Uses a raised-cosine window centred at `centre` with given `width`.
#[allow(dead_code)]
pub fn prominence_profile(x: f32, centre: f32, width: f32, sharpness: f32) -> f32 {
    if width <= 0.0 {
        return 0.0;
    }
    let t = ((x - centre) / width).clamp(-1.0, 1.0);
    let base = 0.5 * (1.0 + (PI * t).cos());
    // Sharpness raises the profile to a power (higher = narrower peak)
    let exponent = 1.0 + sharpness * 3.0;
    base.powf(exponent)
}

/// Apply vertical offset mapping.
#[allow(dead_code)]
pub fn vertical_remap(y: f32, offset: f32) -> f32 {
    y + offset * 0.05
}

/// Compute asymmetry factor for a given lateral position.
#[allow(dead_code)]
pub fn asymmetry_factor(lateral_pos: f32, asymmetry: f32) -> f32 {
    if asymmetry.abs() < 1e-6 {
        return 1.0;
    }
    let shift = asymmetry * 0.02;
    let adjusted = lateral_pos - shift;
    let decay = (-adjusted * adjusted * 50.0).exp();
    decay.clamp(0.0, 1.0)
}

/// Evaluate the adam's apple morph for a set of vertex positions.
///
/// `positions` are `[x, y, z]` in local neck space, where Y is up,
/// Z is forward towards the throat surface.
#[allow(dead_code)]
pub fn evaluate_adam_apple(
    positions: &[[f32; 3]],
    config: &AdamAppleConfig,
) -> AdamAppleResult {
    let mut weights = Vec::new();
    let mut peak = 0.0_f32;

    for (i, pos) in positions.iter().enumerate() {
        let y_mapped = vertical_remap(pos[1], config.vertical_offset);
        let profile = prominence_profile(y_mapped, 0.0, config.width, config.sharpness);
        let asym = asymmetry_factor(pos[0], config.asymmetry);
        let w = config.prominence * profile * asym;
        if w > 1e-6 {
            weights.push((i, w));
            peak = peak.max(w);
        }
    }

    let volume_delta = weights.iter().map(|(_, w)| w).sum::<f32>() * 0.001;

    AdamAppleResult {
        weights,
        peak_displacement: peak,
        volume_delta,
    }
}

/// Blend two adam's apple configurations by factor `t` in 0..=1.
#[allow(dead_code)]
pub fn blend_configs(a: &AdamAppleConfig, b: &AdamAppleConfig, t: f32) -> AdamAppleConfig {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    AdamAppleConfig {
        prominence: a.prominence * inv + b.prominence * t,
        vertical_offset: a.vertical_offset * inv + b.vertical_offset * t,
        asymmetry: a.asymmetry * inv + b.asymmetry * t,
        width: a.width * inv + b.width * t,
        sharpness: a.sharpness * inv + b.sharpness * t,
    }
}

/// Sex-based default prominence.
#[allow(dead_code)]
pub fn default_prominence_for_sex(male_factor: f32) -> f32 {
    let male_factor = male_factor.clamp(0.0, 1.0);
    0.1 + 0.7 * male_factor
}

/// Age-based scaling: prominence increases through puberty then stabilises.
#[allow(dead_code)]
pub fn age_prominence_scale(age_param: f32) -> f32 {
    let age_param = age_param.clamp(0.0, 1.0);
    if age_param < 0.15 {
        // Pre-puberty: minimal
        0.1 * (age_param / 0.15)
    } else if age_param < 0.25 {
        // Puberty: rapid growth
        let t = (age_param - 0.15) / 0.1;
        0.1 + 0.9 * t
    } else {
        // Post-puberty: full
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let c = AdamAppleConfig::default();
        assert!((0.0..=1.0).contains(&c.prominence));
        assert!((0.0..=1.0).contains(&c.sharpness));
    }

    #[test]
    fn test_prominence_profile_centre() {
        let v = prominence_profile(0.0, 0.0, 0.3, 0.0);
        assert!((v - 1.0).abs() < 1e-5, "Centre should be 1.0, got {v}");
    }

    #[test]
    fn test_prominence_profile_edge() {
        let v = prominence_profile(0.3, 0.0, 0.3, 0.0);
        assert!(v < 0.01, "Edge should be near 0, got {v}");
    }

    #[test]
    fn test_prominence_profile_zero_width() {
        assert_eq!(prominence_profile(0.0, 0.0, 0.0, 0.5), 0.0);
    }

    #[test]
    fn test_asymmetry_factor_zero() {
        let f = asymmetry_factor(0.0, 0.0);
        assert!((f - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let config = AdamAppleConfig::default();
        let result = evaluate_adam_apple(&[], &config);
        assert!(result.weights.is_empty());
        assert_eq!(result.peak_displacement, 0.0);
    }

    #[test]
    fn test_evaluate_single_vertex() {
        let config = AdamAppleConfig {
            prominence: 1.0,
            vertical_offset: 0.0,
            asymmetry: 0.0,
            width: 0.5,
            sharpness: 0.0,
        };
        let positions = [[0.0, 0.0, 0.1]];
        let result = evaluate_adam_apple(&positions, &config);
        assert!(!result.weights.is_empty());
        assert!(result.peak_displacement > 0.0);
    }

    #[test]
    fn test_blend_configs_endpoints() {
        let a = AdamAppleConfig::default();
        let b = AdamAppleConfig {
            prominence: 1.0,
            vertical_offset: 0.5,
            asymmetry: -0.3,
            width: 0.6,
            sharpness: 0.8,
        };
        let at0 = blend_configs(&a, &b, 0.0);
        assert!((at0.prominence - a.prominence).abs() < 1e-6);
        let at1 = blend_configs(&a, &b, 1.0);
        assert!((at1.prominence - b.prominence).abs() < 1e-6);
    }

    #[test]
    fn test_default_prominence_for_sex() {
        let male = default_prominence_for_sex(1.0);
        let female = default_prominence_for_sex(0.0);
        assert!(male > female);
        assert!((0.0..=1.0).contains(&male));
    }

    #[test]
    fn test_age_prominence_scale() {
        let child = age_prominence_scale(0.05);
        let adult = age_prominence_scale(0.5);
        assert!(adult > child);
        assert!((adult - 1.0).abs() < 1e-6);
    }
}
