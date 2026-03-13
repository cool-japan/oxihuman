// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Roughness map visualization override.

#![allow(dead_code)]

/// Config for roughness map visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RoughnessViewConfig {
    /// Enable roughness-only mode.
    pub enabled: bool,
    /// Invert roughness (show smoothness instead).
    pub invert: bool,
    /// Exposure multiplier.
    pub exposure: f32,
    /// Color map: false = grayscale, true = false-color.
    pub false_color: bool,
    /// Roughness threshold for highlighting.
    pub highlight_above: Option<f32>,
}

#[allow(dead_code)]
impl Default for RoughnessViewConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            invert: false,
            exposure: 1.0,
            false_color: false,
            highlight_above: None,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_roughness_view_config() -> RoughnessViewConfig {
    RoughnessViewConfig::default()
}

/// Enable roughness view mode.
#[allow(dead_code)]
pub fn rv_enable(cfg: &mut RoughnessViewConfig) {
    cfg.enabled = true;
}

/// Disable roughness view mode.
#[allow(dead_code)]
pub fn rv_disable(cfg: &mut RoughnessViewConfig) {
    cfg.enabled = false;
}

/// Toggle invert.
#[allow(dead_code)]
pub fn rv_toggle_invert(cfg: &mut RoughnessViewConfig) {
    cfg.invert = !cfg.invert;
}

/// Set exposure.
#[allow(dead_code)]
pub fn rv_set_exposure(cfg: &mut RoughnessViewConfig, value: f32) {
    cfg.exposure = value.max(0.0);
}

/// Apply roughness view to a scalar roughness value.
#[allow(dead_code)]
pub fn apply_roughness_view(roughness: f32, cfg: &RoughnessViewConfig) -> [f32; 3] {
    let mut v = (roughness * cfg.exposure).clamp(0.0, 1.0);
    if cfg.invert {
        v = 1.0 - v;
    }
    let highlighted = cfg
        .highlight_above
        .is_some_and(|threshold| roughness > threshold);
    if highlighted {
        return [1.0, 0.2, 0.0];
    }
    if cfg.false_color {
        [v, 1.0 - v, 0.0]
    } else {
        [v, v, v]
    }
}

/// Check if roughness is in "rough" range.
#[allow(dead_code)]
pub fn is_rough(roughness: f32, threshold: f32) -> bool {
    roughness > threshold
}

/// Roughness to perceptual smoothness (Blinn convention).
#[allow(dead_code)]
pub fn roughness_to_smoothness(roughness: f32) -> f32 {
    1.0 - roughness.clamp(0.0, 1.0)
}

/// Roughness to specular power approximation.
#[allow(dead_code)]
pub fn roughness_to_specular_power(roughness: f32) -> f32 {
    let r = roughness.clamp(0.01, 1.0);
    2.0 / (r * r * r * r) - 2.0
}

/// Reset to default.
#[allow(dead_code)]
pub fn rv_reset(cfg: &mut RoughnessViewConfig) {
    *cfg = RoughnessViewConfig::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn roughness_view_to_json(cfg: &RoughnessViewConfig) -> String {
    let ha = match cfg.highlight_above {
        None => "null".to_string(),
        Some(v) => format!("{:.4}", v),
    };
    format!(
        r#"{{"enabled":{},"invert":{},"exposure":{:.4},"false_color":{},"highlight_above":{}}}"#,
        cfg.enabled, cfg.invert, cfg.exposure, cfg.false_color, ha
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = RoughnessViewConfig::default();
        assert!(!c.enabled);
        assert!(!c.invert);
    }

    #[test]
    fn test_apply_grayscale() {
        let cfg = RoughnessViewConfig {
            enabled: true,
            exposure: 1.0,
            ..Default::default()
        };
        let out = apply_roughness_view(0.5, &cfg);
        assert!((out[0] - 0.5).abs() < 1e-5);
        assert!((out[0] - out[1]).abs() < 1e-6);
    }

    #[test]
    fn test_apply_inverted() {
        let cfg = RoughnessViewConfig {
            enabled: true,
            invert: true,
            exposure: 1.0,
            ..Default::default()
        };
        let out = apply_roughness_view(1.0, &cfg);
        assert!(out[0] < 1e-5);
    }

    #[test]
    fn test_highlight_above() {
        let cfg = RoughnessViewConfig {
            enabled: true,
            highlight_above: Some(0.5),
            ..Default::default()
        };
        let out = apply_roughness_view(0.8, &cfg);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_roughness_to_smoothness() {
        assert!((roughness_to_smoothness(0.0) - 1.0).abs() < 1e-6);
        assert!((roughness_to_smoothness(1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_rough() {
        assert!(is_rough(0.8, 0.5));
        assert!(!is_rough(0.3, 0.5));
    }

    #[test]
    fn test_specular_power_positive() {
        assert!(roughness_to_specular_power(0.5) > 0.0);
    }

    #[test]
    fn test_toggle_invert() {
        let mut c = RoughnessViewConfig::default();
        rv_toggle_invert(&mut c);
        assert!(c.invert);
    }

    #[test]
    fn test_to_json() {
        let j = roughness_view_to_json(&RoughnessViewConfig::default());
        assert!(j.contains("enabled"));
    }
}
