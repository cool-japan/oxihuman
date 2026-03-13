// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Post-processing effect pipeline descriptors (pure data, no GPU calls).

// ── Types ──────────────────────────────────────────────────────────────────

/// Screen-space bloom effect configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BloomConfig {
    pub enabled: bool,
    /// Luminance threshold above which bloom is applied.
    pub threshold: f32,
    /// Bloom intensity multiplier.
    pub intensity: f32,
    /// Blur radius factor.
    pub radius: f32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        BloomConfig {
            enabled: false,
            threshold: 1.0,
            intensity: 0.3,
            radius: 1.0,
        }
    }
}

/// Screen-Space Ambient Occlusion configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SsaoConfig {
    pub enabled: bool,
    /// World-space hemisphere radius for occlusion sampling.
    pub radius: f32,
    /// Depth bias to avoid self-shadowing artifacts.
    pub bias: f32,
    /// Occlusion exponent (higher = darker shadows).
    pub power: f32,
    /// Number of SSAO sample kernel taps.
    pub sample_count: u32,
}

impl Default for SsaoConfig {
    fn default() -> Self {
        SsaoConfig {
            enabled: false,
            radius: 0.5,
            bias: 0.025,
            power: 1.0,
            sample_count: 16,
        }
    }
}

/// Supported tone-mapping operators.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ToneMapMethod {
    Linear,
    Reinhard,
    AcesFilm,
    Filmic,
    Uncharted2,
}

/// Tone-mapping and gamma configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToneMappingConfig {
    pub method: ToneMapMethod,
    /// Linear exposure multiplier applied before tone mapping.
    pub exposure: f32,
    /// Output gamma (sRGB default is 2.2).
    pub gamma: f32,
}

impl Default for ToneMappingConfig {
    fn default() -> Self {
        ToneMappingConfig {
            method: ToneMapMethod::Reinhard,
            exposure: 1.0,
            gamma: 2.2,
        }
    }
}

/// Fast-approximate anti-aliasing configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FxaaConfig {
    pub enabled: bool,
    /// Sub-pixel quality dithering (0.0 = off, 0.75 = default).
    pub quality_subpix: f32,
    /// Minimum local contrast required to trigger FXAA (lower = more AA).
    pub quality_edge_threshold: f32,
}

impl Default for FxaaConfig {
    fn default() -> Self {
        FxaaConfig {
            enabled: false,
            quality_subpix: 0.75,
            quality_edge_threshold: 0.166,
        }
    }
}

/// Complete post-processing pipeline descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostProcessPipeline {
    pub bloom: BloomConfig,
    pub ssao: SsaoConfig,
    pub tone_mapping: ToneMappingConfig,
    pub fxaa: FxaaConfig,
    /// Screen-edge darkening strength: 0 = none, 1 = full black edges.
    pub vignette_strength: f32,
    /// Lateral chromatic aberration offset (0 = none).
    pub chromatic_aberration: f32,
}

impl Default for PostProcessPipeline {
    fn default() -> Self {
        PostProcessPipeline {
            bloom: BloomConfig::default(),
            ssao: SsaoConfig::default(),
            tone_mapping: ToneMappingConfig::default(),
            fxaa: FxaaConfig::default(),
            vignette_strength: 0.0,
            chromatic_aberration: 0.0,
        }
    }
}

impl PostProcessPipeline {
    /// High-quality preset: SSAO + bloom + FXAA enabled, ACES tone mapping.
    pub fn high_quality() -> Self {
        PostProcessPipeline {
            bloom: BloomConfig {
                enabled: true,
                threshold: 0.9,
                intensity: 0.4,
                radius: 1.2,
            },
            ssao: SsaoConfig {
                enabled: true,
                radius: 0.4,
                bias: 0.02,
                power: 1.5,
                sample_count: 32,
            },
            tone_mapping: ToneMappingConfig {
                method: ToneMapMethod::AcesFilm,
                exposure: 1.0,
                gamma: 2.2,
            },
            fxaa: FxaaConfig {
                enabled: true,
                quality_subpix: 0.75,
                quality_edge_threshold: 0.125,
            },
            vignette_strength: 0.0,
            chromatic_aberration: 0.0,
        }
    }

    /// Performance preset: minimal effects, no SSAO, FXAA only, Reinhard tone mapping.
    pub fn performance() -> Self {
        PostProcessPipeline {
            bloom: BloomConfig {
                enabled: false,
                ..BloomConfig::default()
            },
            ssao: SsaoConfig {
                enabled: false,
                ..SsaoConfig::default()
            },
            tone_mapping: ToneMappingConfig {
                method: ToneMapMethod::Reinhard,
                exposure: 1.0,
                gamma: 2.2,
            },
            fxaa: FxaaConfig {
                enabled: true,
                quality_subpix: 0.5,
                quality_edge_threshold: 0.25,
            },
            vignette_strength: 0.0,
            chromatic_aberration: 0.0,
        }
    }

    /// Cinematic preset: ACES + bloom + vignette + chromatic aberration.
    pub fn cinematic() -> Self {
        PostProcessPipeline {
            bloom: BloomConfig {
                enabled: true,
                threshold: 0.8,
                intensity: 0.6,
                radius: 1.5,
            },
            ssao: SsaoConfig {
                enabled: true,
                radius: 0.5,
                bias: 0.025,
                power: 2.0,
                sample_count: 64,
            },
            tone_mapping: ToneMappingConfig {
                method: ToneMapMethod::AcesFilm,
                exposure: 1.2,
                gamma: 2.2,
            },
            fxaa: FxaaConfig {
                enabled: true,
                quality_subpix: 0.75,
                quality_edge_threshold: 0.125,
            },
            vignette_strength: 0.35,
            chromatic_aberration: 0.003,
        }
    }

    /// Serialize to a compact JSON string.
    pub fn to_json(&self) -> String {
        let method_str = match self.tone_mapping.method {
            ToneMapMethod::Linear => "Linear",
            ToneMapMethod::Reinhard => "Reinhard",
            ToneMapMethod::AcesFilm => "AcesFilm",
            ToneMapMethod::Filmic => "Filmic",
            ToneMapMethod::Uncharted2 => "Uncharted2",
        };
        format!(
            r#"{{"bloom":{{"enabled":{},"threshold":{:.4},"intensity":{:.4},"radius":{:.4}}},"ssao":{{"enabled":{},"radius":{:.4},"bias":{:.4},"power":{:.4},"sample_count":{}}},"tone_mapping":{{"method":"{}","exposure":{:.4},"gamma":{:.4}}},"fxaa":{{"enabled":{},"quality_subpix":{:.4},"quality_edge_threshold":{:.4}}},"vignette_strength":{:.4},"chromatic_aberration":{:.6}}}"#,
            self.bloom.enabled,
            self.bloom.threshold,
            self.bloom.intensity,
            self.bloom.radius,
            self.ssao.enabled,
            self.ssao.radius,
            self.ssao.bias,
            self.ssao.power,
            self.ssao.sample_count,
            method_str,
            self.tone_mapping.exposure,
            self.tone_mapping.gamma,
            self.fxaa.enabled,
            self.fxaa.quality_subpix,
            self.fxaa.quality_edge_threshold,
            self.vignette_strength,
            self.chromatic_aberration,
        )
    }
}

// ── Tone-mapping functions ─────────────────────────────────────────────────

/// Reinhard tone operator: maps [0, ∞) → [0, 1).
#[inline]
pub fn tone_map_reinhard(x: f32) -> f32 {
    x / (1.0 + x)
}

/// Approximate ACES filmic tone mapping (Narkowicz 2015).
///
/// Numerically stable for very bright inputs.
#[inline]
pub fn tone_map_aces_approx(x: f32) -> f32 {
    let a = 2.51_f32;
    let b = 0.03_f32;
    let c = 2.43_f32;
    let d = 0.59_f32;
    let e = 0.14_f32;
    ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0)
}

/// Linear tone mapping with exposure and gamma correction.
///
/// `pow(x * exposure, 1 / gamma)`.  Returns 0 for negative inputs.
#[inline]
pub fn tone_map_linear(x: f32, exposure: f32, gamma: f32) -> f32 {
    let v = (x * exposure).max(0.0);
    if gamma <= 0.0 {
        return v;
    }
    v.powf(1.0 / gamma)
}

/// Rec. 709 / sRGB luminance: `0.2126 R + 0.7152 G + 0.0722 B`.
#[inline]
pub fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Apply the configured tone-mapping operator per-channel and return the result.
///
/// The output may still exceed 1.0 for `Linear` with high exposure; callers
/// should clamp if writing to an 8-bit target.
pub fn apply_tone_map(color: [f32; 3], cfg: &ToneMappingConfig) -> [f32; 3] {
    match cfg.method {
        ToneMapMethod::Linear => [
            tone_map_linear(color[0], cfg.exposure, cfg.gamma),
            tone_map_linear(color[1], cfg.exposure, cfg.gamma),
            tone_map_linear(color[2], cfg.exposure, cfg.gamma),
        ],
        ToneMapMethod::Reinhard => {
            let ec = [
                color[0] * cfg.exposure,
                color[1] * cfg.exposure,
                color[2] * cfg.exposure,
            ];
            [
                tone_map_reinhard(ec[0]).max(0.0),
                tone_map_reinhard(ec[1]).max(0.0),
                tone_map_reinhard(ec[2]).max(0.0),
            ]
        }
        ToneMapMethod::AcesFilm => {
            let ec = [
                color[0] * cfg.exposure,
                color[1] * cfg.exposure,
                color[2] * cfg.exposure,
            ];
            [
                tone_map_aces_approx(ec[0]),
                tone_map_aces_approx(ec[1]),
                tone_map_aces_approx(ec[2]),
            ]
        }
        ToneMapMethod::Filmic => {
            // Simple filmic S-curve approximation
            let ec = [
                color[0] * cfg.exposure,
                color[1] * cfg.exposure,
                color[2] * cfg.exposure,
            ];
            [
                tone_map_reinhard(ec[0] * 1.6).max(0.0),
                tone_map_reinhard(ec[1] * 1.6).max(0.0),
                tone_map_reinhard(ec[2] * 1.6).max(0.0),
            ]
        }
        ToneMapMethod::Uncharted2 => {
            // Uncharted 2 "Hable" filmic curve
            fn hable(x: f32) -> f32 {
                let a = 0.15_f32;
                let b = 0.50_f32;
                let c = 0.10_f32;
                let d = 0.20_f32;
                let e = 0.02_f32;
                let f = 0.30_f32;
                (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f
            }
            let white = hable(11.2);
            let ec = [
                color[0] * cfg.exposure * 2.0,
                color[1] * cfg.exposure * 2.0,
                color[2] * cfg.exposure * 2.0,
            ];
            [
                (hable(ec[0]) / white).clamp(0.0, 1.0),
                (hable(ec[1]) / white).clamp(0.0, 1.0),
                (hable(ec[2]) / white).clamp(0.0, 1.0),
            ]
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── tone_map_reinhard ────────────────────────────────────────────────

    #[test]
    fn tone_map_reinhard_at_one() {
        let v = tone_map_reinhard(1.0);
        assert!((v - 0.5).abs() < 1e-6, "reinhard(1) should be 0.5, got {v}");
    }

    #[test]
    fn tone_map_reinhard_monotone() {
        assert!(tone_map_reinhard(2.0) > tone_map_reinhard(1.0));
        assert!(tone_map_reinhard(10.0) > tone_map_reinhard(2.0));
    }

    #[test]
    fn tone_map_reinhard_approaches_one() {
        let v = tone_map_reinhard(1_000_000.0);
        assert!(v < 1.0 + 1e-4 && v > 0.999);
    }

    // ── tone_map_aces_approx ─────────────────────────────────────────────

    #[test]
    fn tone_map_aces_stable_for_bright() {
        // Should not panic or overflow for very large inputs
        let v = tone_map_aces_approx(1_000.0);
        assert!(
            (0.0..=1.0).contains(&v),
            "ACES should clamp to [0,1], got {v}"
        );
    }

    #[test]
    fn tone_map_aces_zero_is_zero() {
        assert!(tone_map_aces_approx(0.0).abs() < 1e-4);
    }

    #[test]
    fn tone_map_aces_one_is_reasonable() {
        let v = tone_map_aces_approx(1.0);
        // At exposure=1, ACES(1.0) should be in a reasonable range
        assert!(v > 0.7 && v <= 1.0, "ACES(1.0) should be ~0.8+, got {v}");
    }

    // ── luminance ────────────────────────────────────────────────────────

    #[test]
    fn luminance_white_is_one() {
        let l = luminance(1.0, 1.0, 1.0);
        assert!(
            (l - 1.0).abs() < 1e-5,
            "luminance(1,1,1) should be 1.0, got {l}"
        );
    }

    #[test]
    fn luminance_black_is_zero() {
        assert!(luminance(0.0, 0.0, 0.0).abs() < 1e-6);
    }

    #[test]
    fn luminance_formula() {
        let l = luminance(1.0, 0.0, 0.0);
        assert!(
            (l - 0.2126).abs() < 1e-4,
            "expected 0.2126 for pure red, got {l}"
        );
    }

    #[test]
    fn luminance_green_heaviest() {
        let lr = luminance(1.0, 0.0, 0.0);
        let lg = luminance(0.0, 1.0, 0.0);
        let lb = luminance(0.0, 0.0, 1.0);
        assert!(lg > lr, "green should dominate luminance");
        assert!(lg > lb, "green should dominate luminance over blue");
    }

    // ── apply_tone_map ───────────────────────────────────────────────────

    #[test]
    fn apply_tone_map_non_negative_output() {
        let cfg = ToneMappingConfig::default();
        let out = apply_tone_map([0.5, 1.0, 2.0], &cfg);
        for ch in out {
            assert!(ch >= 0.0, "output channel must be non-negative, got {ch}");
        }
    }

    #[test]
    fn apply_tone_map_aces_clamps() {
        let cfg = ToneMappingConfig {
            method: ToneMapMethod::AcesFilm,
            exposure: 1.0,
            gamma: 2.2,
        };
        let out = apply_tone_map([1000.0, 1000.0, 1000.0], &cfg);
        for ch in out {
            assert!(ch <= 1.0 + 1e-4, "ACES output must be ≤ 1.0, got {ch}");
        }
    }

    // ── PostProcessPipeline presets ──────────────────────────────────────

    #[test]
    fn high_quality_ssao_enabled() {
        assert!(PostProcessPipeline::high_quality().ssao.enabled);
    }

    #[test]
    fn high_quality_fxaa_enabled() {
        assert!(PostProcessPipeline::high_quality().fxaa.enabled);
    }

    #[test]
    fn high_quality_bloom_enabled() {
        assert!(PostProcessPipeline::high_quality().bloom.enabled);
    }

    #[test]
    fn performance_ssao_disabled() {
        assert!(!PostProcessPipeline::performance().ssao.enabled);
    }

    #[test]
    fn performance_bloom_disabled() {
        assert!(!PostProcessPipeline::performance().bloom.enabled);
    }

    #[test]
    fn cinematic_vignette_positive() {
        assert!(
            PostProcessPipeline::cinematic().vignette_strength > 0.0,
            "cinematic preset should have vignette"
        );
    }

    #[test]
    fn cinematic_chromatic_aberration_nonzero() {
        assert!(PostProcessPipeline::cinematic().chromatic_aberration > 0.0);
    }

    // ── Default values ────────────────────────────────────────────────────

    #[test]
    fn default_bloom_threshold_is_one() {
        let cfg = BloomConfig::default();
        assert!((cfg.threshold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn all_presets_have_valid_gamma() {
        let presets = [
            PostProcessPipeline::default(),
            PostProcessPipeline::high_quality(),
            PostProcessPipeline::performance(),
            PostProcessPipeline::cinematic(),
        ];
        for p in &presets {
            assert!(
                p.tone_mapping.gamma > 0.0,
                "gamma must be positive, got {}",
                p.tone_mapping.gamma
            );
        }
    }

    // ── to_json ───────────────────────────────────────────────────────────

    #[test]
    fn to_json_non_empty() {
        let json = PostProcessPipeline::default().to_json();
        assert!(!json.is_empty());
    }

    #[test]
    fn to_json_contains_bloom_key() {
        let json = PostProcessPipeline::high_quality().to_json();
        assert!(json.contains("\"bloom\""), "JSON should contain 'bloom'");
    }

    #[test]
    fn to_json_contains_tone_mapping() {
        let json = PostProcessPipeline::cinematic().to_json();
        assert!(
            json.contains("AcesFilm"),
            "cinematic JSON should mention AcesFilm"
        );
    }
}
