// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cavity map visualization overlay.

#![allow(dead_code)]

/// Config for cavity map overlay.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CavityMapViewConfig {
    /// Enable cavity map overlay.
    pub enabled: bool,
    /// Intensity: 0.0 = no effect, 1.0 = full cavity darkening.
    pub intensity: f32,
    /// Ridge highlight strength.
    pub ridge_strength: f32,
    /// Valley shadow strength.
    pub valley_strength: f32,
    /// Blur radius for cavity computation.
    pub blur_radius: u32,
}

#[allow(dead_code)]
impl Default for CavityMapViewConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 1.0,
            ridge_strength: 0.5,
            valley_strength: 0.5,
            blur_radius: 4,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_cavity_map_view_config() -> CavityMapViewConfig {
    CavityMapViewConfig::default()
}

/// Enable cavity map.
#[allow(dead_code)]
pub fn cmv_enable(cfg: &mut CavityMapViewConfig) {
    cfg.enabled = true;
}

/// Disable cavity map.
#[allow(dead_code)]
pub fn cmv_disable(cfg: &mut CavityMapViewConfig) {
    cfg.enabled = false;
}

/// Set intensity.
#[allow(dead_code)]
pub fn cmv_set_intensity(cfg: &mut CavityMapViewConfig, value: f32) {
    cfg.intensity = value.clamp(0.0, 1.0);
}

/// Set ridge strength.
#[allow(dead_code)]
pub fn cmv_set_ridge_strength(cfg: &mut CavityMapViewConfig, value: f32) {
    cfg.ridge_strength = value.clamp(0.0, 1.0);
}

/// Set valley strength.
#[allow(dead_code)]
pub fn cmv_set_valley_strength(cfg: &mut CavityMapViewConfig, value: f32) {
    cfg.valley_strength = value.clamp(0.0, 1.0);
}

/// Apply cavity map to an RGB color.
/// `cavity`: 0.0 = deep valley, 0.5 = flat, 1.0 = ridge.
#[allow(dead_code)]
pub fn apply_cavity_map(color: [f32; 3], cavity: f32, cfg: &CavityMapViewConfig) -> [f32; 3] {
    let cavity = cavity.clamp(0.0, 1.0);
    let valley = (0.5 - cavity).max(0.0) * 2.0 * cfg.valley_strength;
    let ridge = (cavity - 0.5).max(0.0) * 2.0 * cfg.ridge_strength;
    let factor = (1.0 - valley + ridge).clamp(0.0, 2.0);
    let t = cfg.intensity;
    [
        (color[0] * (1.0 - t + t * factor)).clamp(0.0, 1.0),
        (color[1] * (1.0 - t + t * factor)).clamp(0.0, 1.0),
        (color[2] * (1.0 - t + t * factor)).clamp(0.0, 1.0),
    ]
}

/// Compute cavity value from local curvature estimate.
/// `curvature`: negative = concave (valley), positive = convex (ridge).
#[allow(dead_code)]
pub fn curvature_to_cavity(curvature: f32, scale: f32) -> f32 {
    (0.5 + curvature * scale).clamp(0.0, 1.0)
}

/// Reset to default.
#[allow(dead_code)]
pub fn cmv_reset(cfg: &mut CavityMapViewConfig) {
    *cfg = CavityMapViewConfig::default();
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn cavity_map_view_to_json(cfg: &CavityMapViewConfig) -> String {
    format!(
        r#"{{"enabled":{},"intensity":{:.4},"ridge_strength":{:.4},"valley_strength":{:.4},"blur_radius":{}}}"#,
        cfg.enabled, cfg.intensity, cfg.ridge_strength, cfg.valley_strength, cfg.blur_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = CavityMapViewConfig::default();
        assert!(!c.enabled);
        assert!((c.intensity - 1.0).abs() < 1e-6);
        assert_eq!(c.blur_radius, 4);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = CavityMapViewConfig::default();
        cmv_enable(&mut c);
        assert!(c.enabled);
        cmv_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_apply_flat_unchanged() {
        let cfg = CavityMapViewConfig {
            enabled: true,
            ..Default::default()
        };
        let color = [0.5f32, 0.5, 0.5];
        let out = apply_cavity_map(color, 0.5, &cfg);
        assert!((out[0] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_apply_valley_darkens() {
        let cfg = CavityMapViewConfig {
            enabled: true,
            valley_strength: 1.0,
            ..Default::default()
        };
        let color = [1.0f32, 1.0, 1.0];
        let out = apply_cavity_map(color, 0.0, &cfg);
        assert!(out[0] < 1.0);
    }

    #[test]
    fn test_curvature_to_cavity_flat() {
        let c = curvature_to_cavity(0.0, 1.0);
        assert!((c - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_curvature_to_cavity_clamped() {
        let c = curvature_to_cavity(100.0, 1.0);
        assert!((c - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_intensity_clamped() {
        let mut c = CavityMapViewConfig::default();
        cmv_set_intensity(&mut c, 5.0);
        assert!((c.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut c = CavityMapViewConfig {
            enabled: true,
            intensity: 0.1,
            ridge_strength: 0.9,
            valley_strength: 0.9,
            blur_radius: 16,
        };
        cmv_reset(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_to_json() {
        let j = cavity_map_view_to_json(&CavityMapViewConfig::default());
        assert!(j.contains("intensity"));
        assert!(j.contains("blur_radius"));
    }
}
