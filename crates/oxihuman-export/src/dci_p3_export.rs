// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! DCI-P3 color gamut config export — parameters for DCI cinema color space.

use std::f32::consts::E;

/// DCI-P3 white point (DCI spec, xy chromaticity).
pub const DCI_P3_WHITE_XY: [f32; 2] = [0.314, 0.351];

/// DCI-P3 gamma (2.6 power function).
pub const DCI_P3_GAMMA: f32 = 2.6;

/// DCI-P3 color space primaries (CIE xy).
#[derive(Debug, Clone, Copy)]
pub struct DciP3Primaries {
    pub red: [f32; 2],
    pub green: [f32; 2],
    pub blue: [f32; 2],
}

impl Default for DciP3Primaries {
    fn default() -> Self {
        Self {
            red: [0.680, 0.320],
            green: [0.265, 0.690],
            blue: [0.150, 0.060],
        }
    }
}

/// A DCI-P3 export configuration.
#[derive(Debug, Clone)]
pub struct DciP3Config {
    pub primaries: DciP3Primaries,
    pub white_point: [f32; 2],
    pub gamma: f32,
    pub peak_luminance_nits: f32,
}

impl Default for DciP3Config {
    fn default() -> Self {
        Self {
            primaries: DciP3Primaries::default(),
            white_point: DCI_P3_WHITE_XY,
            gamma: DCI_P3_GAMMA,
            peak_luminance_nits: 48.0,
        }
    }
}

/// Applies the DCI-P3 EOTF (power 2.6) to an encoded value.
pub fn dci_p3_eotf(encoded: f32) -> f32 {
    encoded.clamp(0.0, 1.0).powf(DCI_P3_GAMMA)
}

/// Applies the DCI-P3 OETF (inverse, power 1/2.6).
pub fn dci_p3_oetf(linear: f32) -> f32 {
    linear.clamp(0.0, 1.0).powf(1.0 / DCI_P3_GAMMA)
}

/// Exports the DCI-P3 config as a text descriptor.
pub fn export_dci_p3_config(cfg: &DciP3Config) -> String {
    format!(
        "DCI-P3\nGamma: {:.1}\nPeak: {:.0} nits\nWhite: ({:.3}, {:.3})\n",
        cfg.gamma, cfg.peak_luminance_nits, cfg.white_point[0], cfg.white_point[1],
    )
}

/// Validates a DCI-P3 config (peak luminance must be positive, gamma must be positive).
pub fn validate_dci_p3_config(cfg: &DciP3Config) -> bool {
    cfg.peak_luminance_nits > 0.0 && cfg.gamma > 0.0
}

/// Converts a DCI-P3 encoded RGB triplet to linear.
pub fn dci_p3_to_linear(rgb: [f32; 3]) -> [f32; 3] {
    [
        dci_p3_eotf(rgb[0]),
        dci_p3_eotf(rgb[1]),
        dci_p3_eotf(rgb[2]),
    ]
}

/// Computes the natural log of a linear value (utility for tone mapping stubs).
pub fn log_luminance(linear: f32) -> f32 {
    linear.max(f32::EPSILON).ln() / E.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eotf_zero() {
        /* 0 encoded → 0 linear */
        assert_eq!(dci_p3_eotf(0.0), 0.0);
    }

    #[test]
    fn test_eotf_one() {
        /* 1 encoded → 1 linear */
        assert!((dci_p3_eotf(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_oetf_zero() {
        /* 0 linear → 0 encoded */
        assert_eq!(dci_p3_oetf(0.0), 0.0);
    }

    #[test]
    fn test_oetf_one() {
        /* 1 linear → 1 encoded */
        assert!((dci_p3_oetf(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_roundtrip() {
        /* OETF→EOTF roundtrip should recover original value */
        let v = 0.5f32;
        let enc = dci_p3_oetf(v);
        let dec = dci_p3_eotf(enc);
        assert!((dec - v).abs() < 0.001);
    }

    #[test]
    fn test_export_contains_dci() {
        /* Export string should mention DCI-P3 */
        let cfg = DciP3Config::default();
        assert!(export_dci_p3_config(&cfg).contains("DCI-P3"));
    }

    #[test]
    fn test_validate_default() {
        /* Default config should validate */
        assert!(validate_dci_p3_config(&DciP3Config::default()));
    }

    #[test]
    fn test_validate_zero_peak_fails() {
        /* Zero peak luminance should fail */
        let cfg = DciP3Config {
            peak_luminance_nits: 0.0,
            ..Default::default()
        };
        assert!(!validate_dci_p3_config(&cfg));
    }

    #[test]
    fn test_dci_p3_to_linear_length() {
        /* Output should have 3 components */
        assert_eq!(dci_p3_to_linear([0.5, 0.5, 0.5]).len(), 3);
    }

    #[test]
    fn test_log_luminance_one() {
        /* log_luminance(1.0) should be 0 */
        assert!(log_luminance(1.0).abs() < 0.001);
    }
}
