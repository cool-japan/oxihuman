// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! BT.2020 HDR color space export — ITU-R BT.2020 parameters and config export.

/// BT.2020 color primaries (CIE xy chromaticity).
pub const BT2020_RED_XY: [f32; 2] = [0.708, 0.292];
pub const BT2020_GREEN_XY: [f32; 2] = [0.170, 0.797];
pub const BT2020_BLUE_XY: [f32; 2] = [0.131, 0.046];
pub const BT2020_WHITE_XY: [f32; 2] = [0.3127, 0.3290];

/// BT.2020 OETF parameters.
pub const BT2020_ALPHA: f32 = 1.099_296_8;
pub const BT2020_BETA: f32 = 0.018_053_97;

/// A BT.2020 export configuration.
#[derive(Debug, Clone)]
pub struct Bt2020Config {
    pub bit_depth: u8,
    pub is_constant_luminance: bool,
    pub reference_peak_luminance: f32,
}

impl Default for Bt2020Config {
    fn default() -> Self {
        Self {
            bit_depth: 10,
            is_constant_luminance: false,
            reference_peak_luminance: 1000.0,
        }
    }
}

/// Applies the BT.2020 OETF (non-linear encoding).
pub fn bt2020_oetf(linear: f32) -> f32 {
    let c = linear.clamp(0.0, 1.0);
    if c < BT2020_BETA {
        4.5 * c
    } else {
        BT2020_ALPHA * c.powf(0.45) - (BT2020_ALPHA - 1.0)
    }
}

/// Applies the BT.2020 EOTF (linearization, approximate inverse).
pub fn bt2020_eotf(encoded: f32) -> f32 {
    let c = encoded.clamp(0.0, 1.0);
    if c < 4.5 * BT2020_BETA {
        c / 4.5
    } else {
        ((c + BT2020_ALPHA - 1.0) / BT2020_ALPHA).powf(1.0 / 0.45)
    }
}

/// Exports the BT.2020 config as a descriptor string.
pub fn export_bt2020_config(cfg: &Bt2020Config) -> String {
    format!(
        "ITU-R BT.2020\nBit depth: {}\nPeak: {:.0} nits\nConstant luminance: {}\n",
        cfg.bit_depth, cfg.reference_peak_luminance, cfg.is_constant_luminance
    )
}

/// Validates a BT.2020 config (bit depth 10 or 12, peak > 0).
pub fn validate_bt2020_config(cfg: &Bt2020Config) -> bool {
    matches!(cfg.bit_depth, 10 | 12) && cfg.reference_peak_luminance > 0.0
}

/// Converts an RGB triplet using BT.2020 OETF.
pub fn bt2020_oetf_triplet(rgb: [f32; 3]) -> [f32; 3] {
    [
        bt2020_oetf(rgb[0]),
        bt2020_oetf(rgb[1]),
        bt2020_oetf(rgb[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oetf_zero() {
        /* 0 linear → 0 encoded */
        assert_eq!(bt2020_oetf(0.0), 0.0);
    }

    #[test]
    fn test_oetf_one() {
        /* 1 linear → close to 1 encoded */
        assert!((bt2020_oetf(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_eotf_zero() {
        /* 0 encoded → 0 linear */
        assert_eq!(bt2020_eotf(0.0), 0.0);
    }

    #[test]
    fn test_eotf_one() {
        /* 1 encoded → close to 1 linear */
        assert!((bt2020_eotf(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_roundtrip() {
        /* OETF→EOTF roundtrip should recover value */
        let v = 0.4f32;
        assert!((bt2020_eotf(bt2020_oetf(v)) - v).abs() < 0.01);
    }

    #[test]
    fn test_export_contains_bt2020() {
        /* Export should mention BT.2020 */
        assert!(export_bt2020_config(&Bt2020Config::default()).contains("BT.2020"));
    }

    #[test]
    fn test_validate_default() {
        /* Default config should validate */
        assert!(validate_bt2020_config(&Bt2020Config::default()));
    }

    #[test]
    fn test_validate_invalid_bit_depth() {
        /* 8-bit is not valid for BT.2020 */
        let cfg = Bt2020Config {
            bit_depth: 8,
            ..Default::default()
        };
        assert!(!validate_bt2020_config(&cfg));
    }

    #[test]
    fn test_oetf_triplet_length() {
        /* Output triplet should have 3 components */
        assert_eq!(bt2020_oetf_triplet([0.5, 0.5, 0.5]).len(), 3);
    }

    #[test]
    fn test_linear_below_beta_uses_linear_segment() {
        /* Values below beta should use the linear segment */
        let v = 0.001f32;
        assert!((bt2020_oetf(v) - 4.5 * v).abs() < 0.0001);
    }
}
