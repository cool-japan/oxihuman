// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! PQ (ST 2084) EOTF export — Perceptual Quantizer HDR transfer function.

/// PQ EOTF constants (SMPTE ST 2084).
pub const PQ_M1: f32 = 0.1593_0175;
pub const PQ_M2: f32 = 78.843_75;
pub const PQ_C1: f32 = 0.8359_375;
pub const PQ_C2: f32 = 18.851_563;
pub const PQ_C3: f32 = 18.6875;

/// Maximum absolute luminance for PQ (nits).
pub const PQ_MAX_LUMINANCE: f32 = 10_000.0;

/// PQ export configuration.
#[derive(Debug, Clone)]
pub struct PqConfig {
    pub peak_luminance_nits: f32,
    pub black_level_nits: f32,
}

impl Default for PqConfig {
    fn default() -> Self {
        Self {
            peak_luminance_nits: 10_000.0,
            black_level_nits: 0.0,
        }
    }
}

/// Applies the PQ EOTF: normalized signal `[0,1]` → absolute luminance [0, 10000 nits].
pub fn pq_eotf(signal: f32) -> f32 {
    let s = signal.clamp(0.0, 1.0);
    let sp = s.powf(1.0 / PQ_M2);
    let num = (sp - PQ_C1).max(0.0);
    let den = PQ_C2 - PQ_C3 * sp;
    let y = (num / den.max(f32::EPSILON)).powf(1.0 / PQ_M1);
    y * PQ_MAX_LUMINANCE
}

/// Applies the PQ OETF: absolute luminance [0, 10000 nits] → normalized signal `[0,1]`.
pub fn pq_oetf(luminance_nits: f32) -> f32 {
    let y = (luminance_nits / PQ_MAX_LUMINANCE).clamp(0.0, 1.0);
    let yp = y.powf(PQ_M1);
    ((PQ_C1 + PQ_C2 * yp) / (1.0 + PQ_C3 * yp)).powf(PQ_M2)
}

/// Exports PQ config as a descriptor string.
pub fn export_pq_config(cfg: &PqConfig) -> String {
    format!(
        "SMPTE ST 2084 (PQ)\nPeak: {:.0} nits\nBlack: {:.2} nits\n",
        cfg.peak_luminance_nits, cfg.black_level_nits
    )
}

/// Validates a PQ config (peak must be in (0, 10000]).
pub fn validate_pq_config(cfg: &PqConfig) -> bool {
    cfg.peak_luminance_nits > 0.0
        && cfg.peak_luminance_nits <= PQ_MAX_LUMINANCE
        && cfg.black_level_nits >= 0.0
}

/// Converts a normalized PQ signal triplet to absolute luminance.
pub fn pq_eotf_triplet(rgb: [f32; 3]) -> [f32; 3] {
    [pq_eotf(rgb[0]), pq_eotf(rgb[1]), pq_eotf(rgb[2])]
}

/// Converts a luminance triplet to PQ signals.
pub fn pq_oetf_triplet(luminances: [f32; 3]) -> [f32; 3] {
    [
        pq_oetf(luminances[0]),
        pq_oetf(luminances[1]),
        pq_oetf(luminances[2]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_oetf_zero() {
        /* 0 nits → signal near 0 */
        assert!(pq_oetf(0.0) < 0.1);
    }

    #[test]
    fn test_pq_oetf_peak() {
        /* 10000 nits → signal near 1 */
        assert!((pq_oetf(PQ_MAX_LUMINANCE) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_pq_eotf_zero_signal() {
        /* Signal 0 → near 0 nits */
        assert!(pq_eotf(0.0) < 1.0);
    }

    #[test]
    fn test_pq_eotf_one_signal() {
        /* Signal 1 → near 10000 nits */
        assert!((pq_eotf(1.0) - PQ_MAX_LUMINANCE).abs() < 100.0);
    }

    #[test]
    fn test_pq_roundtrip() {
        /* OETF→EOTF roundtrip should preserve value */
        let lum = 500.0f32;
        let sig = pq_oetf(lum);
        let recovered = pq_eotf(sig);
        assert!((recovered - lum).abs() / lum < 0.01);
    }

    #[test]
    fn test_export_contains_pq() {
        /* Export should mention ST 2084 or PQ */
        assert!(export_pq_config(&PqConfig::default()).contains("PQ"));
    }

    #[test]
    fn test_validate_default_config() {
        /* Default config should validate */
        assert!(validate_pq_config(&PqConfig::default()));
    }

    #[test]
    fn test_validate_zero_peak_fails() {
        /* Zero peak should fail */
        let cfg = PqConfig {
            peak_luminance_nits: 0.0,
            black_level_nits: 0.0,
        };
        assert!(!validate_pq_config(&cfg));
    }

    #[test]
    fn test_pq_eotf_triplet_length() {
        /* Triplet should have 3 components */
        assert_eq!(pq_eotf_triplet([0.5, 0.5, 0.5]).len(), 3);
    }

    #[test]
    fn test_pq_oetf_triplet_length() {
        /* Triplet should have 3 components */
        assert_eq!(pq_oetf_triplet([100.0, 200.0, 300.0]).len(), 3);
    }
}
