// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ProPhoto RGB color space export — Kodak ProPhoto RGB / ROMM RGB parameters.

/// ProPhoto RGB white point (D50 illuminant, xy chromaticity).
pub const PRO_PHOTO_WHITE_XY: [f32; 2] = [0.3457, 0.3585];

/// ProPhoto RGB gamma value (1.8 power function with linear segment).
pub const PRO_PHOTO_GAMMA: f32 = 1.8;

/// ProPhoto RGB color primaries.
#[derive(Debug, Clone, Copy)]
pub struct ProPhotoPrimaries {
    pub red: [f32; 2],
    pub green: [f32; 2],
    pub blue: [f32; 2],
}

impl Default for ProPhotoPrimaries {
    fn default() -> Self {
        Self {
            red: [0.7347, 0.2653],
            green: [0.1596, 0.8404],
            blue: [0.0366, 0.0001],
        }
    }
}

/// ProPhoto RGB configuration.
#[derive(Debug, Clone)]
pub struct ProPhotoConfig {
    pub primaries: ProPhotoPrimaries,
    pub white_point: [f32; 2],
    pub gamma: f32,
    pub bit_depth: u8,
}

impl Default for ProPhotoConfig {
    fn default() -> Self {
        Self {
            primaries: ProPhotoPrimaries::default(),
            white_point: PRO_PHOTO_WHITE_XY,
            gamma: PRO_PHOTO_GAMMA,
            bit_depth: 16,
        }
    }
}

/// Applies the ProPhoto OETF (power 1/1.8 with linear toe).
pub fn pro_photo_oetf(linear: f32) -> f32 {
    let c = linear.clamp(0.0, 1.0);
    if c < 1.0 / 512.0 {
        c * 16.0
    } else {
        c.powf(1.0 / PRO_PHOTO_GAMMA)
    }
}

/// Applies the ProPhoto EOTF (power 1.8 with linear toe).
pub fn pro_photo_eotf(encoded: f32) -> f32 {
    let c = encoded.clamp(0.0, 1.0);
    if c < 1.0 / 32.0 {
        c / 16.0
    } else {
        c.powf(PRO_PHOTO_GAMMA)
    }
}

/// Exports the ProPhoto RGB config as a descriptor string.
pub fn export_pro_photo_config(cfg: &ProPhotoConfig) -> String {
    format!(
        "ProPhoto RGB (ROMM RGB)\nGamma: {:.1}\nBit depth: {}\nWhite: ({:.4}, {:.4})\n",
        cfg.gamma, cfg.bit_depth, cfg.white_point[0], cfg.white_point[1]
    )
}

/// Validates a ProPhoto config.
pub fn validate_pro_photo_config(cfg: &ProPhotoConfig) -> bool {
    cfg.gamma > 0.0 && cfg.bit_depth >= 8 && cfg.white_point[1] > 0.0
}

/// Converts an encoded ProPhoto RGB triplet to linear.
pub fn pro_photo_to_linear(rgb: [f32; 3]) -> [f32; 3] {
    [
        pro_photo_eotf(rgb[0]),
        pro_photo_eotf(rgb[1]),
        pro_photo_eotf(rgb[2]),
    ]
}

/// Checks if a ProPhoto config uses a wide-gamut bit depth (16 bit or higher).
pub fn is_wide_gamut_bit_depth(cfg: &ProPhotoConfig) -> bool {
    cfg.bit_depth >= 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oetf_zero() {
        /* 0 → 0 */
        assert_eq!(pro_photo_oetf(0.0), 0.0);
    }

    #[test]
    fn test_oetf_one() {
        /* 1 → 1 */
        assert!((pro_photo_oetf(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_eotf_zero() {
        /* 0 → 0 */
        assert_eq!(pro_photo_eotf(0.0), 0.0);
    }

    #[test]
    fn test_eotf_one() {
        /* 1 → 1 */
        assert!((pro_photo_eotf(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_roundtrip() {
        /* OETF→EOTF roundtrip */
        let v = 0.5f32;
        assert!((pro_photo_eotf(pro_photo_oetf(v)) - v).abs() < 0.01);
    }

    #[test]
    fn test_export_contains_pro_photo() {
        /* Export should mention ProPhoto */
        assert!(export_pro_photo_config(&ProPhotoConfig::default()).contains("ProPhoto"));
    }

    #[test]
    fn test_validate_default_config() {
        /* Default config should validate */
        assert!(validate_pro_photo_config(&ProPhotoConfig::default()));
    }

    #[test]
    fn test_validate_zero_gamma_fails() {
        /* Zero gamma should fail */
        let cfg = ProPhotoConfig {
            gamma: 0.0,
            ..Default::default()
        };
        assert!(!validate_pro_photo_config(&cfg));
    }

    #[test]
    fn test_pro_photo_to_linear_length() {
        /* Triplet output should have 3 components */
        assert_eq!(pro_photo_to_linear([0.5, 0.5, 0.5]).len(), 3);
    }

    #[test]
    fn test_is_wide_gamut_bit_depth_16() {
        /* 16-bit should be wide gamut */
        assert!(is_wide_gamut_bit_depth(&ProPhotoConfig::default()));
    }
}
