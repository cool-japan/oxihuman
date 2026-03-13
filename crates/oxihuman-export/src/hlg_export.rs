// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Hybrid Log-Gamma curve export — BBC/NHK HLG OETF/EOTF parameters.

/// HLG OETF constants (ITU-R BT.2100).
pub const HLG_A: f32 = 0.17883_277;
pub const HLG_B: f32 = 1.0 - 4.0 * 0.17883_277;
pub const HLG_C: f32 = 0.559_911;

/// HLG system gamma (nominal display gamma for reference conditions).
pub const HLG_SYSTEM_GAMMA: f32 = 1.2;

/// HLG export configuration.
#[derive(Debug, Clone)]
pub struct HlgConfig {
    pub peak_luminance_nits: f32,
    pub black_level_nits: f32,
    pub system_gamma: f32,
}

impl Default for HlgConfig {
    fn default() -> Self {
        Self {
            peak_luminance_nits: 1000.0,
            black_level_nits: 0.0,
            system_gamma: HLG_SYSTEM_GAMMA,
        }
    }
}

/// Applies the HLG OETF (linear scene light → HLG signal).
pub fn hlg_oetf(e: f32) -> f32 {
    let e = e.clamp(0.0, 1.0);
    if e <= 1.0 / 12.0 {
        (3.0 * e).sqrt()
    } else {
        HLG_A * (12.0 * e - HLG_B).ln() + HLG_C
    }
}

/// Applies the HLG inverse OETF (HLG signal → scene light).
pub fn hlg_inv_oetf(signal: f32) -> f32 {
    let s = signal.clamp(0.0, 1.0);
    if s <= 0.5 {
        s * s / 3.0
    } else {
        ((s - HLG_C) / HLG_A)
            .exp()
            .mul_add(1.0 / 12.0, HLG_B / 12.0)
    }
}

/// Exports HLG config as a descriptor string.
pub fn export_hlg_config(cfg: &HlgConfig) -> String {
    format!(
        "HLG (ITU-R BT.2100)\nPeak: {:.0} nits\nBlack: {:.1} nits\nSystem gamma: {:.2}\n",
        cfg.peak_luminance_nits, cfg.black_level_nits, cfg.system_gamma
    )
}

/// Validates an HLG config.
pub fn validate_hlg_config(cfg: &HlgConfig) -> bool {
    cfg.peak_luminance_nits > 0.0 && cfg.black_level_nits >= 0.0 && cfg.system_gamma > 0.0
}

/// Applies HLG OETF to an RGB triplet.
pub fn hlg_oetf_triplet(rgb: [f32; 3]) -> [f32; 3] {
    [hlg_oetf(rgb[0]), hlg_oetf(rgb[1]), hlg_oetf(rgb[2])]
}

/// Clamps an HLG signal to valid `[0,1]` range.
pub fn clamp_hlg_signal(signal: f32) -> f32 {
    signal.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlg_oetf_zero() {
        /* 0 input → 0 output */
        assert_eq!(hlg_oetf(0.0), 0.0);
    }

    #[test]
    fn test_hlg_oetf_one() {
        /* 1 input → 1 output */
        assert!((hlg_oetf(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hlg_oetf_clamps_negative() {
        /* Negative input should clamp to 0 */
        assert_eq!(hlg_oetf(-1.0), hlg_oetf(0.0));
    }

    #[test]
    fn test_hlg_inv_oetf_zero() {
        /* 0 signal → 0 scene light */
        assert_eq!(hlg_inv_oetf(0.0), 0.0);
    }

    #[test]
    fn test_roundtrip_midrange() {
        /* OETF→inv_OETF roundtrip for midrange value */
        let v = 0.3f32;
        let encoded = hlg_oetf(v);
        let recovered = hlg_inv_oetf(encoded);
        assert!((recovered - v).abs() < 0.01);
    }

    #[test]
    fn test_export_contains_hlg() {
        /* Export should mention HLG */
        assert!(export_hlg_config(&HlgConfig::default()).contains("HLG"));
    }

    #[test]
    fn test_validate_default_config() {
        /* Default config should validate */
        assert!(validate_hlg_config(&HlgConfig::default()));
    }

    #[test]
    fn test_validate_zero_peak_fails() {
        /* Zero peak should fail */
        let cfg = HlgConfig {
            peak_luminance_nits: 0.0,
            ..Default::default()
        };
        assert!(!validate_hlg_config(&cfg));
    }

    #[test]
    fn test_oetf_triplet_length() {
        /* Triplet output should have 3 components */
        assert_eq!(hlg_oetf_triplet([0.1, 0.2, 0.3]).len(), 3);
    }

    #[test]
    fn test_clamp_hlg_signal() {
        /* Signal outside [0,1] should be clamped */
        assert_eq!(clamp_hlg_signal(2.0), 1.0);
        assert_eq!(clamp_hlg_signal(-1.0), 0.0);
    }
}
