// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ACES color space config stub — exports Academy Color Encoding System parameters.

/// ACES color primaries (ITU-R BT.2020 gamut approximation).
pub const ACES_AP0_WHITE: [f32; 2] = [0.32168, 0.33767];
pub const ACES_AP1_WHITE: [f32; 2] = [0.32168, 0.33767];

/// ACES color space variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcesVariant {
    Ap0,
    Ap1,
    AcesCct,
    AcesProxy,
}

/// ACES export configuration.
#[derive(Debug, Clone)]
pub struct AcesExportConfig {
    pub variant: AcesVariant,
    pub exposure_offset: f32,
    pub include_lut: bool,
}

impl Default for AcesExportConfig {
    fn default() -> Self {
        Self {
            variant: AcesVariant::Ap1,
            exposure_offset: 0.0,
            include_lut: false,
        }
    }
}

/// Stub ACES export result.
#[derive(Debug, Clone)]
pub struct AcesExportResult {
    pub variant_name: String,
    pub config_blob: Vec<u8>,
}

/// Exports an ACES color config as a byte blob stub.
pub fn export_aces_config(cfg: &AcesExportConfig) -> AcesExportResult {
    let name = match cfg.variant {
        AcesVariant::Ap0 => "ACES-AP0",
        AcesVariant::Ap1 => "ACES-AP1",
        AcesVariant::AcesCct => "ACEScct",
        AcesVariant::AcesProxy => "ACESproxy",
    };
    let blob = format!("aces_variant={name},exposure={}", cfg.exposure_offset).into_bytes();
    AcesExportResult {
        variant_name: name.to_string(),
        config_blob: blob,
    }
}

/// Applies an exposure offset (in stops) to a linear value.
pub fn apply_exposure(value: f32, stops: f32) -> f32 {
    value * 2.0f32.powf(stops)
}

/// Converts ACEScg to ACES AP0 (simplified matrix stub — identity for testing).
pub fn acescg_to_ap0(rgb: [f32; 3]) -> [f32; 3] {
    rgb /* stub: real conversion would apply a 3x3 matrix */
}

/// Checks if a variant is a log-encoded variant.
pub fn is_log_variant(variant: AcesVariant) -> bool {
    matches!(variant, AcesVariant::AcesCct | AcesVariant::AcesProxy)
}

/// Returns the nominal bit depth for a given ACES variant.
pub fn variant_bit_depth(variant: AcesVariant) -> u32 {
    match variant {
        AcesVariant::AcesProxy => 10,
        AcesVariant::AcesCct => 16,
        _ => 32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        /* Default variant should be Ap1 */
        assert_eq!(AcesExportConfig::default().variant, AcesVariant::Ap1);
    }

    #[test]
    fn test_export_aces_ap0_name() {
        /* Export result should contain AP0 name */
        let cfg = AcesExportConfig {
            variant: AcesVariant::Ap0,
            ..Default::default()
        };
        assert_eq!(export_aces_config(&cfg).variant_name, "ACES-AP0");
    }

    #[test]
    fn test_export_config_blob_nonempty() {
        /* Config blob should not be empty */
        let result = export_aces_config(&AcesExportConfig::default());
        assert!(!result.config_blob.is_empty());
    }

    #[test]
    fn test_apply_exposure_zero() {
        /* Zero stops should not change the value */
        assert!((apply_exposure(1.0, 0.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_exposure_one_stop() {
        /* One stop should double the value */
        assert!((apply_exposure(1.0, 1.0) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_exposure_negative() {
        /* Negative stop should halve the value */
        assert!((apply_exposure(1.0, -1.0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_acescg_to_ap0_stub() {
        /* Stub should return input unchanged */
        let rgb = [0.5f32, 0.3, 0.1];
        assert_eq!(acescg_to_ap0(rgb), rgb);
    }

    #[test]
    fn test_is_log_variant_cct() {
        /* ACEScct should be a log variant */
        assert!(is_log_variant(AcesVariant::AcesCct));
    }

    #[test]
    fn test_is_log_variant_ap0() {
        /* AP0 should not be a log variant */
        assert!(!is_log_variant(AcesVariant::Ap0));
    }

    #[test]
    fn test_variant_bit_depth_proxy() {
        /* ACESproxy should have 10-bit depth */
        assert_eq!(variant_bit_depth(AcesVariant::AcesProxy), 10);
    }
}
