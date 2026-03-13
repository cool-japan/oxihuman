// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Albedo-only rendering mode override.

#![allow(dead_code)]

/// Config for albedo-only rendering mode.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AlbedoViewConfig {
    /// Enable albedo-only mode.
    pub enabled: bool,
    /// Exposure multiplier.
    pub exposure: f32,
    /// Gamma correction exponent.
    pub gamma: f32,
    /// Desaturate output.
    pub desaturate: bool,
    /// Desaturation amount: 0.0 = full color, 1.0 = grayscale.
    pub desaturate_amount: f32,
}

#[allow(dead_code)]
impl Default for AlbedoViewConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            exposure: 1.0,
            gamma: 2.2,
            desaturate: false,
            desaturate_amount: 0.0,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_albedo_view_config() -> AlbedoViewConfig {
    AlbedoViewConfig::default()
}

/// Enable albedo mode.
#[allow(dead_code)]
pub fn av_enable(cfg: &mut AlbedoViewConfig) {
    cfg.enabled = true;
}

/// Disable albedo mode.
#[allow(dead_code)]
pub fn av_disable(cfg: &mut AlbedoViewConfig) {
    cfg.enabled = false;
}

/// Set exposure.
#[allow(dead_code)]
pub fn av_set_exposure(cfg: &mut AlbedoViewConfig, value: f32) {
    cfg.exposure = value.max(0.0);
}

/// Set gamma.
#[allow(dead_code)]
pub fn av_set_gamma(cfg: &mut AlbedoViewConfig, value: f32) {
    cfg.gamma = value.max(0.01);
}

/// Apply albedo view to an RGB color.
#[allow(dead_code)]
pub fn apply_albedo_view(color: [f32; 3], cfg: &AlbedoViewConfig) -> [f32; 3] {
    let r = (color[0] * cfg.exposure)
        .clamp(0.0, 1.0)
        .powf(1.0 / cfg.gamma);
    let g = (color[1] * cfg.exposure)
        .clamp(0.0, 1.0)
        .powf(1.0 / cfg.gamma);
    let b = (color[2] * cfg.exposure)
        .clamp(0.0, 1.0)
        .powf(1.0 / cfg.gamma);
    if cfg.desaturate {
        let lum = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let t = cfg.desaturate_amount.clamp(0.0, 1.0);
        [r + (lum - r) * t, g + (lum - g) * t, b + (lum - b) * t]
    } else {
        [r, g, b]
    }
}

/// Compute luminance of an RGB color.
#[allow(dead_code)]
pub fn luminance(color: [f32; 3]) -> f32 {
    color[0] * 0.2126 + color[1] * 0.7152 + color[2] * 0.0722
}

/// Reset to default.
#[allow(dead_code)]
pub fn av_reset(cfg: &mut AlbedoViewConfig) {
    *cfg = AlbedoViewConfig::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn albedo_view_to_json(cfg: &AlbedoViewConfig) -> String {
    format!(
        r#"{{"enabled":{},"exposure":{:.4},"gamma":{:.4},"desaturate":{}}}"#,
        cfg.enabled, cfg.exposure, cfg.gamma, cfg.desaturate
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = AlbedoViewConfig::default();
        assert!(!c.enabled);
        assert!((c.gamma - 2.2).abs() < 1e-5);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = AlbedoViewConfig::default();
        av_enable(&mut c);
        assert!(c.enabled);
        av_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_apply_identity_gamma1() {
        let cfg = AlbedoViewConfig {
            enabled: true,
            exposure: 1.0,
            gamma: 1.0,
            desaturate: false,
            desaturate_amount: 0.0,
        };
        let out = apply_albedo_view([0.5, 0.5, 0.5], &cfg);
        assert!((out[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_luminance() {
        let lum = luminance([1.0, 0.0, 0.0]);
        assert!((lum - 0.2126).abs() < 1e-4);
    }

    #[test]
    fn test_desaturate() {
        let cfg = AlbedoViewConfig {
            enabled: true,
            exposure: 1.0,
            gamma: 1.0,
            desaturate: true,
            desaturate_amount: 1.0,
        };
        let out = apply_albedo_view([1.0, 0.0, 0.0], &cfg);
        assert!((out[0] - out[1]).abs() < 1e-4);
    }

    #[test]
    fn test_set_exposure_clamped() {
        let mut c = AlbedoViewConfig::default();
        av_set_exposure(&mut c, -1.0);
        assert!(c.exposure < 1e-6);
    }

    #[test]
    fn test_set_gamma_clamped() {
        let mut c = AlbedoViewConfig::default();
        av_set_gamma(&mut c, 0.0);
        assert!(c.gamma > 0.0);
    }

    #[test]
    fn test_reset() {
        let mut c = AlbedoViewConfig {
            enabled: true,
            exposure: 3.0,
            gamma: 1.0,
            desaturate: true,
            desaturate_amount: 1.0,
        };
        av_reset(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_to_json() {
        let j = albedo_view_to_json(&AlbedoViewConfig::default());
        assert!(j.contains("enabled"));
        assert!(j.contains("gamma"));
    }
}
