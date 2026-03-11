// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Display P3 color space config export — Apple Display P3 (DCI-P3 with D65 white).

/// Display P3 white point D65 (same as sRGB).
pub const DISPLAY_P3_WHITE_XY: [f32; 2] = [0.3127, 0.3290];

/// Display P3 color primaries (same as DCI-P3 but with D65 white point).
#[derive(Debug, Clone, Copy)]
pub struct DisplayP3Primaries {
    pub red: [f32; 2],
    pub green: [f32; 2],
    pub blue: [f32; 2],
}

impl Default for DisplayP3Primaries {
    fn default() -> Self {
        Self {
            red: [0.680, 0.320],
            green: [0.265, 0.690],
            blue: [0.150, 0.060],
        }
    }
}

/// Display P3 config (uses sRGB transfer function).
#[derive(Debug, Clone)]
pub struct DisplayP3Config {
    pub primaries: DisplayP3Primaries,
    pub white_point: [f32; 2],
    pub use_srgb_transfer: bool,
}

impl Default for DisplayP3Config {
    fn default() -> Self {
        Self {
            primaries: DisplayP3Primaries::default(),
            white_point: DISPLAY_P3_WHITE_XY,
            use_srgb_transfer: true,
        }
    }
}

/// Applies a gamma-2.2 approximate EOTF (for Display P3 without strict sRGB TF).
pub fn gamma22_eotf(encoded: f32) -> f32 {
    encoded.clamp(0.0, 1.0).powf(2.2)
}

/// Applies a gamma-2.2 approximate OETF.
pub fn gamma22_oetf(linear: f32) -> f32 {
    linear.clamp(0.0, 1.0).powf(1.0 / 2.2)
}

/// Exports the Display P3 config as a descriptor string.
pub fn export_display_p3_config(cfg: &DisplayP3Config) -> String {
    format!(
        "Display P3\nWhite: ({:.4}, {:.4})\nsRGB transfer: {}\nR: ({:.3}, {:.3})\nG: ({:.3}, {:.3})\nB: ({:.3}, {:.3})\n",
        cfg.white_point[0], cfg.white_point[1],
        cfg.use_srgb_transfer,
        cfg.primaries.red[0], cfg.primaries.red[1],
        cfg.primaries.green[0], cfg.primaries.green[1],
        cfg.primaries.blue[0], cfg.primaries.blue[1],
    )
}

/// Validates a Display P3 config (white point must be non-zero).
pub fn validate_display_p3_config(cfg: &DisplayP3Config) -> bool {
    cfg.white_point[0] > 0.0 && cfg.white_point[1] > 0.0
}

/// Converts an encoded Display P3 RGB triplet to linear (gamma-2.2 path).
pub fn display_p3_to_linear(rgb: [f32; 3]) -> [f32; 3] {
    [
        gamma22_eotf(rgb[0]),
        gamma22_eotf(rgb[1]),
        gamma22_eotf(rgb[2]),
    ]
}

/// Checks if Display P3 covers a wider gamut than sRGB.
pub fn is_wider_than_srgb() -> bool {
    true /* Display P3 always has wider gamut than sRGB */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamma22_eotf_zero() {
        /* 0 → 0 */
        assert_eq!(gamma22_eotf(0.0), 0.0);
    }

    #[test]
    fn test_gamma22_eotf_one() {
        /* 1 → 1 */
        assert!((gamma22_eotf(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_gamma22_oetf_one() {
        /* 1 → 1 */
        assert!((gamma22_oetf(1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_roundtrip() {
        /* OETF→EOTF roundtrip */
        let v = 0.6f32;
        assert!((gamma22_eotf(gamma22_oetf(v)) - v).abs() < 0.001);
    }

    #[test]
    fn test_export_contains_display_p3() {
        /* Export should mention Display P3 */
        assert!(export_display_p3_config(&DisplayP3Config::default()).contains("Display P3"));
    }

    #[test]
    fn test_validate_default_config() {
        /* Default config should validate */
        assert!(validate_display_p3_config(&DisplayP3Config::default()));
    }

    #[test]
    fn test_validate_zero_white_point_fails() {
        /* Zero white point should fail */
        let cfg = DisplayP3Config {
            white_point: [0.0, 0.0],
            ..Default::default()
        };
        assert!(!validate_display_p3_config(&cfg));
    }

    #[test]
    fn test_display_p3_to_linear_length() {
        /* Triplet output should have 3 components */
        assert_eq!(display_p3_to_linear([0.5, 0.5, 0.5]).len(), 3);
    }

    #[test]
    fn test_is_wider_than_srgb() {
        /* Display P3 is always wider than sRGB */
        assert!(is_wider_than_srgb());
    }

    #[test]
    fn test_default_uses_srgb_transfer() {
        /* Default config should use sRGB transfer function */
        assert!(DisplayP3Config::default().use_srgb_transfer);
    }
}
