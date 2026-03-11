// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! IEC 61966 (sRGB standard) stub — encodes/decodes sRGB gamma and exports config.

/// sRGB transfer function parameters per IEC 61966-2-1.
pub const SRGB_GAMMA: f32 = 2.4;
pub const SRGB_LINEAR_THRESHOLD: f32 = 0.04045;
pub const SRGB_LINEAR_SLOPE: f32 = 12.92;
pub const SRGB_ALPHA: f32 = 0.055;

/// An sRGB color space configuration.
#[derive(Debug, Clone)]
pub struct SrgbConfig {
    pub white_point: [f32; 2],
    pub primaries_r: [f32; 2],
    pub primaries_g: [f32; 2],
    pub primaries_b: [f32; 2],
}

impl Default for SrgbConfig {
    fn default() -> Self {
        Self {
            white_point: [0.3127, 0.3290],
            primaries_r: [0.64, 0.33],
            primaries_g: [0.30, 0.60],
            primaries_b: [0.15, 0.06],
        }
    }
}

/// Applies the sRGB EOTF (linearization): converts encoded to linear.
pub fn srgb_to_linear(encoded: f32) -> f32 {
    let c = encoded.clamp(0.0, 1.0);
    if c <= SRGB_LINEAR_THRESHOLD / SRGB_LINEAR_SLOPE {
        c / SRGB_LINEAR_SLOPE
    } else {
        ((c + SRGB_ALPHA) / (1.0 + SRGB_ALPHA)).powf(SRGB_GAMMA)
    }
}

/// Applies the sRGB OETF (encoding): converts linear to sRGB-encoded.
pub fn linear_to_srgb(linear: f32) -> f32 {
    let c = linear.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * SRGB_LINEAR_SLOPE
    } else {
        (1.0 + SRGB_ALPHA) * c.powf(1.0 / SRGB_GAMMA) - SRGB_ALPHA
    }
}

/// Exports the sRGB config as a text descriptor.
pub fn export_srgb_config(cfg: &SrgbConfig) -> String {
    format!(
        "sRGB IEC 61966-2-1\nWhite: ({:.4}, {:.4})\nR: ({:.2}, {:.2})\nG: ({:.2}, {:.2})\nB: ({:.2}, {:.2})\n",
        cfg.white_point[0], cfg.white_point[1],
        cfg.primaries_r[0], cfg.primaries_r[1],
        cfg.primaries_g[0], cfg.primaries_g[1],
        cfg.primaries_b[0], cfg.primaries_b[1],
    )
}

/// Converts a full sRGB triplet to linear.
pub fn srgb_triplet_to_linear(rgb: [f32; 3]) -> [f32; 3] {
    [
        srgb_to_linear(rgb[0]),
        srgb_to_linear(rgb[1]),
        srgb_to_linear(rgb[2]),
    ]
}

/// Validates that an sRGB config has a non-zero white point.
pub fn validate_srgb_config(cfg: &SrgbConfig) -> bool {
    cfg.white_point[0] > 0.0 && cfg.white_point[1] > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_linear_zero() {
        /* 0 encodes to 0 linear */
        assert_eq!(srgb_to_linear(0.0), 0.0);
    }

    #[test]
    fn test_srgb_to_linear_one() {
        /* 1 encodes to 1 linear */
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_to_srgb_zero() {
        /* 0 linear → 0 sRGB */
        assert_eq!(linear_to_srgb(0.0), 0.0);
    }

    #[test]
    fn test_linear_to_srgb_one() {
        /* 1 linear → 1 sRGB */
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip() {
        /* Linear → sRGB → linear roundtrip should be close */
        let v = 0.5f32;
        let encoded = linear_to_srgb(v);
        let decoded = srgb_to_linear(encoded);
        assert!((decoded - v).abs() < 0.001);
    }

    #[test]
    fn test_srgb_triplet_length() {
        /* Output should have 3 components */
        let r = srgb_triplet_to_linear([0.5, 0.5, 0.5]);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_export_contains_srgb() {
        /* Export should mention sRGB */
        let cfg = SrgbConfig::default();
        assert!(export_srgb_config(&cfg).contains("sRGB"));
    }

    #[test]
    fn test_validate_default_config() {
        /* Default config should validate */
        assert!(validate_srgb_config(&SrgbConfig::default()));
    }

    #[test]
    fn test_validate_zero_white_point_fails() {
        /* Zero white point should fail */
        let cfg = SrgbConfig {
            white_point: [0.0, 0.0],
            ..Default::default()
        };
        assert!(!validate_srgb_config(&cfg));
    }

    #[test]
    fn test_srgb_midgray() {
        /* sRGB midgray (0.5) should linearize to roughly 0.214 */
        let linear = srgb_to_linear(0.5);
        assert!(linear > 0.18 && linear < 0.24);
    }
}
