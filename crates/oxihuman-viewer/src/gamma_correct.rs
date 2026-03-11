// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gamma correction — sRGB encode/decode, arbitrary gamma curves,
//! and tone-curve application.

/// Gamma correction configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GammaConfig {
    /// Gamma exponent (2.2 for sRGB-like).
    pub gamma: f32,
    /// Whether to use the exact sRGB transfer function.
    pub use_srgb: bool,
    /// Brightness offset applied before gamma.
    pub brightness: f32,
    /// Contrast multiplier applied before gamma.
    pub contrast: f32,
}

impl Default for GammaConfig {
    fn default() -> Self {
        Self {
            gamma: 2.2,
            use_srgb: true,
            brightness: 0.0,
            contrast: 1.0,
        }
    }
}

/// Linear to sRGB (exact transfer function).
#[allow(dead_code)]
pub fn linear_to_srgb(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB to linear (exact transfer function).
#[allow(dead_code)]
pub fn srgb_to_linear(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

/// Simple power-law gamma encode (linear -> gamma space).
#[allow(dead_code)]
pub fn gamma_encode(x: f32, gamma: f32) -> f32 {
    if gamma.abs() < 1e-6 {
        return x.clamp(0.0, 1.0);
    }
    x.clamp(0.0, 1.0).powf(1.0 / gamma)
}

/// Simple power-law gamma decode (gamma space -> linear).
#[allow(dead_code)]
pub fn gamma_decode(x: f32, gamma: f32) -> f32 {
    if gamma.abs() < 1e-6 {
        return x.clamp(0.0, 1.0);
    }
    x.clamp(0.0, 1.0).powf(gamma)
}

/// Apply brightness and contrast before gamma.
#[allow(dead_code)]
pub fn brightness_contrast(x: f32, brightness: f32, contrast: f32) -> f32 {
    ((x - 0.5) * contrast + 0.5 + brightness).clamp(0.0, 1.0)
}

/// Apply gamma correction to an RGB colour.
#[allow(dead_code)]
pub fn apply_gamma_rgb(color: [f32; 3], config: &GammaConfig) -> [f32; 3] {
    let mut result = [0.0_f32; 3];
    for i in 0..3 {
        let c = brightness_contrast(color[i], config.brightness, config.contrast);
        result[i] = if config.use_srgb {
            linear_to_srgb(c)
        } else {
            gamma_encode(c, config.gamma)
        };
    }
    result
}

/// Remove gamma from an RGB colour (decode to linear).
#[allow(dead_code)]
pub fn remove_gamma_rgb(color: [f32; 3], config: &GammaConfig) -> [f32; 3] {
    let mut result = [0.0_f32; 3];
    for i in 0..3 {
        result[i] = if config.use_srgb {
            srgb_to_linear(color[i])
        } else {
            gamma_decode(color[i], config.gamma)
        };
    }
    result
}

/// Compute the mid-grey value in gamma space.
#[allow(dead_code)]
pub fn mid_grey(gamma: f32) -> f32 {
    gamma_encode(0.18, gamma)
}

/// Approximate inverse sRGB for fast path (power 2.2).
#[allow(dead_code)]
pub fn fast_srgb_to_linear(x: f32) -> f32 {
    x.clamp(0.0, 1.0).powf(2.2)
}

/// Approximate sRGB for fast path (power 1/2.2).
#[allow(dead_code)]
pub fn fast_linear_to_srgb(x: f32) -> f32 {
    x.clamp(0.0, 1.0).powf(1.0 / 2.2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = GammaConfig::default();
        assert!((c.gamma - 2.2).abs() < 1e-5);
        assert!(c.use_srgb);
    }

    #[test]
    fn test_linear_to_srgb_zero() {
        assert!(linear_to_srgb(0.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_to_srgb_one() {
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_srgb_roundtrip() {
        let values = [0.0, 0.1, 0.18, 0.5, 0.8, 1.0];
        for &v in &values {
            let encoded = linear_to_srgb(v);
            let decoded = srgb_to_linear(encoded);
            assert!((v - decoded).abs() < 1e-5, "Roundtrip failed for {v}");
        }
    }

    #[test]
    fn test_gamma_encode_decode_roundtrip() {
        let gamma = 2.2;
        let values = [0.0, 0.1, 0.5, 1.0];
        for &v in &values {
            let encoded = gamma_encode(v, gamma);
            let decoded = gamma_decode(encoded, gamma);
            assert!((v - decoded).abs() < 1e-5);
        }
    }

    #[test]
    fn test_brightness_contrast_neutral() {
        let v = brightness_contrast(0.5, 0.0, 1.0);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_brightness_increase() {
        let v = brightness_contrast(0.5, 0.1, 1.0);
        assert!(v > 0.5);
    }

    #[test]
    fn test_apply_gamma_rgb_clamped() {
        let r = apply_gamma_rgb([1.5, -0.1, 0.5], &GammaConfig::default());
        for ch in &r {
            assert!((0.0..=1.0).contains(ch));
        }
    }

    #[test]
    fn test_mid_grey() {
        let mg = mid_grey(2.2);
        assert!(mg > 0.4 && mg < 0.5);
    }

    #[test]
    fn test_fast_srgb_approximation() {
        let exact = srgb_to_linear(0.5);
        let fast = fast_srgb_to_linear(0.5);
        assert!((exact - fast).abs() < 0.01);
    }
}
