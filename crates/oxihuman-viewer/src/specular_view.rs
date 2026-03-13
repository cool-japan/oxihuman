// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Specular / roughness channel visualization debug view.

/// Configuration for specular view.
#[derive(Debug, Clone)]
pub struct SpecularViewConfig {
    pub show_roughness: bool,
    pub roughness_scale: f32,
    pub f0_scale: f32,
}

impl Default for SpecularViewConfig {
    fn default() -> Self {
        Self { show_roughness: true, roughness_scale: 1.0, f0_scale: 1.0 }
    }
}

/// State for specular channel visualization.
#[derive(Debug, Clone)]
pub struct SpecularView {
    pub config: SpecularViewConfig,
    pub enabled: bool,
}

impl Default for SpecularView {
    fn default() -> Self {
        Self { config: SpecularViewConfig::default(), enabled: false }
    }
}

/// Enable specular view.
pub fn spv_enable(view: &mut SpecularView) {
    view.enabled = true;
}

/// Disable specular view.
pub fn spv_disable(view: &mut SpecularView) {
    view.enabled = false;
}

/// Set roughness display scale.
pub fn spv_set_roughness_scale(view: &mut SpecularView, scale: f32) {
    view.config.roughness_scale = scale.clamp(0.0, 4.0);
}

/// Map roughness and F0 to a display color.
pub fn spv_to_color(roughness: f32, f0: f32, config: &SpecularViewConfig) -> [f32; 4] {
    let r = (roughness * config.roughness_scale).clamp(0.0, 1.0);
    let f = (f0 * config.f0_scale).clamp(0.0, 1.0);
    if config.show_roughness {
        [r, r, r, 1.0]
    } else {
        [f, f, f, 1.0]
    }
}

/// Return the perceptual roughness from linear roughness.
pub fn spv_perceptual_roughness(linear: f32) -> f32 {
    linear.clamp(0.0, 1.0).sqrt()
}

/// Return the F0 reflectance from IOR.
pub fn spv_f0_from_ior(ior: f32) -> f32 {
    let n = ior.max(1.0);
    let ratio = (n - 1.0) / (n + 1.0);
    ratio * ratio
}

/// Export config to JSON string (stub).
pub fn spv_to_json(view: &SpecularView) -> String {
    format!(
        r#"{{"show_roughness":{},"roughness_scale":{:.2},"enabled":{}}}"#,
        view.config.show_roughness, view.config.roughness_scale, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = SpecularView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = SpecularView::default();
        spv_enable(&mut v);
        assert!(v.enabled);
        spv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_roughness_scale() {
        /* roughness scale should be stored */
        let mut v = SpecularView::default();
        spv_set_roughness_scale(&mut v, 2.0);
        assert!((v.config.roughness_scale - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_roughness_scale_clamp() {
        /* scale above 4 should be clamped */
        let mut v = SpecularView::default();
        spv_set_roughness_scale(&mut v, 100.0);
        assert!((v.config.roughness_scale - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_color_roughness_mode() {
        /* roughness mode should produce greyscale */
        let cfg = SpecularViewConfig { show_roughness: true, ..Default::default() };
        let c = spv_to_color(0.5, 0.04, &cfg);
        assert!((c[0] - c[1]).abs() < 1e-6);
    }

    #[test]
    fn test_perceptual_roughness() {
        /* perceptual roughness of 1 should be 1 */
        assert!((spv_perceptual_roughness(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_perceptual_roughness_quarter() {
        /* perceptual roughness of 0.25 should be 0.5 */
        assert!((spv_perceptual_roughness(0.25) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_f0_from_ior() {
        /* glass IOR ~1.5 should give F0 ~0.04 */
        let f0 = spv_f0_from_ior(1.5);
        assert!((f0 - 0.04).abs() < 0.001);
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should reflect enabled state */
        let mut v = SpecularView::default();
        spv_enable(&mut v);
        let json = spv_to_json(&v);
        assert!(json.contains("true"));
    }
}
