// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Metallic mask visualization override.

#![allow(dead_code)]

/// Config for metallic mask visualization.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MetallicViewConfig {
    /// Enable metallic-only mode.
    pub enabled: bool,
    /// Threshold for binary metal/non-metal display.
    pub threshold: f32,
    /// Show as binary (true) or grayscale (false).
    pub binary_mode: bool,
    /// Color for metallic regions.
    pub metal_color: [f32; 3],
    /// Color for non-metallic regions.
    pub non_metal_color: [f32; 3],
}

#[allow(dead_code)]
impl Default for MetallicViewConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.5,
            binary_mode: false,
            metal_color: [1.0, 0.8, 0.0],
            non_metal_color: [0.2, 0.2, 0.8],
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_metallic_view_config() -> MetallicViewConfig {
    MetallicViewConfig::default()
}

/// Enable metallic view.
#[allow(dead_code)]
pub fn mv_enable(cfg: &mut MetallicViewConfig) {
    cfg.enabled = true;
}

/// Disable metallic view.
#[allow(dead_code)]
pub fn mv_disable(cfg: &mut MetallicViewConfig) {
    cfg.enabled = false;
}

/// Set threshold.
#[allow(dead_code)]
pub fn mv_set_threshold(cfg: &mut MetallicViewConfig, value: f32) {
    cfg.threshold = value.clamp(0.0, 1.0);
}

/// Toggle binary mode.
#[allow(dead_code)]
pub fn mv_toggle_binary(cfg: &mut MetallicViewConfig) {
    cfg.binary_mode = !cfg.binary_mode;
}

/// Apply metallic view to a scalar metallic value.
#[allow(dead_code)]
pub fn apply_metallic_view(metallic: f32, cfg: &MetallicViewConfig) -> [f32; 3] {
    let m = metallic.clamp(0.0, 1.0);
    if cfg.binary_mode {
        if m >= cfg.threshold {
            cfg.metal_color
        } else {
            cfg.non_metal_color
        }
    } else {
        let mc = cfg.metal_color;
        let nc = cfg.non_metal_color;
        [
            nc[0] + (mc[0] - nc[0]) * m,
            nc[1] + (mc[1] - nc[1]) * m,
            nc[2] + (mc[2] - nc[2]) * m,
        ]
    }
}

/// Is this value considered metallic?
#[allow(dead_code)]
pub fn is_metallic(value: f32, threshold: f32) -> bool {
    value >= threshold
}

/// Reset to default.
#[allow(dead_code)]
pub fn mv_reset(cfg: &mut MetallicViewConfig) {
    *cfg = MetallicViewConfig::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn metallic_view_to_json(cfg: &MetallicViewConfig) -> String {
    format!(
        r#"{{"enabled":{},"threshold":{:.4},"binary_mode":{}}}"#,
        cfg.enabled, cfg.threshold, cfg.binary_mode
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = MetallicViewConfig::default();
        assert!(!c.enabled);
        assert!((c.threshold - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_grayscale_interpolation() {
        let cfg = MetallicViewConfig {
            enabled: true,
            binary_mode: false,
            ..Default::default()
        };
        let mid = apply_metallic_view(0.5, &cfg);
        assert!(mid[0] > 0.0 && mid[0] < 1.0);
    }

    #[test]
    fn test_apply_binary_metal() {
        let cfg = MetallicViewConfig {
            enabled: true,
            binary_mode: true,
            threshold: 0.5,
            ..Default::default()
        };
        let out = apply_metallic_view(1.0, &cfg);
        assert!((out[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_binary_non_metal() {
        let cfg = MetallicViewConfig {
            enabled: true,
            binary_mode: true,
            threshold: 0.5,
            ..Default::default()
        };
        let out = apply_metallic_view(0.0, &cfg);
        assert!((out[2] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_is_metallic() {
        assert!(is_metallic(0.8, 0.5));
        assert!(!is_metallic(0.3, 0.5));
    }

    #[test]
    fn test_toggle_binary() {
        let mut c = MetallicViewConfig::default();
        mv_toggle_binary(&mut c);
        assert!(c.binary_mode);
    }

    #[test]
    fn test_set_threshold_clamped() {
        let mut c = MetallicViewConfig::default();
        mv_set_threshold(&mut c, 5.0);
        assert!((c.threshold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut c = MetallicViewConfig {
            enabled: true,
            threshold: 0.9,
            binary_mode: true,
            ..Default::default()
        };
        mv_reset(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_to_json() {
        let j = metallic_view_to_json(&MetallicViewConfig::default());
        assert!(j.contains("enabled"));
        assert!(j.contains("threshold"));
    }
}
