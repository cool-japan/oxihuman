// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fresnel term debug visualization view.

/// Configuration for Fresnel debug view.
#[derive(Debug, Clone)]
pub struct FresnelDebugConfig {
    pub f0: f32,
    pub power: f32,
    pub show_grazing: bool,
}

impl Default for FresnelDebugConfig {
    fn default() -> Self {
        Self { f0: 0.04, power: 5.0, show_grazing: true }
    }
}

/// State for Fresnel debug visualization.
#[derive(Debug, Clone)]
pub struct FresnelDebugView {
    pub config: FresnelDebugConfig,
    pub enabled: bool,
}

impl Default for FresnelDebugView {
    fn default() -> Self {
        Self { config: FresnelDebugConfig::default(), enabled: false }
    }
}

/// Enable Fresnel debug view.
pub fn frdv_enable(view: &mut FresnelDebugView) {
    view.enabled = true;
}

/// Disable Fresnel debug view.
pub fn frdv_disable(view: &mut FresnelDebugView) {
    view.enabled = false;
}

/// Compute the Schlick Fresnel approximation.
pub fn frdv_schlick(f0: f32, cos_theta: f32) -> f32 {
    let cos = cos_theta.clamp(0.0, 1.0);
    f0 + (1.0 - f0) * (1.0 - cos).powf(5.0)
}

/// Map a Fresnel value to a display color.
pub fn frdv_to_color(fresnel: f32, config: &FresnelDebugConfig) -> [f32; 4] {
    let v = fresnel.clamp(0.0, 1.0);
    if config.show_grazing && v > 0.9 {
        [1.0, 0.5, 0.0, 1.0] /* highlight grazing angle in orange */
    } else {
        [v, v, v, 1.0]
    }
}

/// Set the F0 reflectance value.
pub fn frdv_set_f0(view: &mut FresnelDebugView, f0: f32) {
    view.config.f0 = f0.clamp(0.0, 1.0);
}

/// Export config to JSON string (stub).
pub fn frdv_to_json(view: &FresnelDebugView) -> String {
    format!(
        r#"{{"f0":{:.4},"power":{:.2},"enabled":{}}}"#,
        view.config.f0, view.config.power, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = FresnelDebugView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = FresnelDebugView::default();
        frdv_enable(&mut v);
        assert!(v.enabled);
        frdv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_f0() {
        /* f0 should be stored */
        let mut v = FresnelDebugView::default();
        frdv_set_f0(&mut v, 0.1);
        assert!((v.config.f0 - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_schlick_normal_incidence() {
        /* at normal incidence (cos=1) Fresnel should equal F0 */
        let f0 = 0.04;
        assert!((frdv_schlick(f0, 1.0) - f0).abs() < 1e-6);
    }

    #[test]
    fn test_schlick_grazing() {
        /* at grazing (cos=0) Fresnel should be 1.0 */
        assert!((frdv_schlick(0.04, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_color_greyscale() {
        /* non-grazing fresnel should produce greyscale */
        let cfg = FresnelDebugConfig { show_grazing: false, ..Default::default() };
        let c = frdv_to_color(0.5, &cfg);
        assert!((c[0] - c[1]).abs() < 1e-6);
    }

    #[test]
    fn test_to_color_grazing_highlight() {
        /* grazing angle should show orange highlight */
        let cfg = FresnelDebugConfig { show_grazing: true, ..Default::default() };
        let c = frdv_to_color(0.95, &cfg);
        assert!(c[1] < c[0]);
    }

    #[test]
    fn test_alpha_one() {
        /* alpha should always be 1.0 */
        let cfg = FresnelDebugConfig::default();
        let c = frdv_to_color(0.5, &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_f0() {
        /* JSON should contain f0 field */
        let v = FresnelDebugView::default();
        let json = frdv_to_json(&v);
        assert!(json.contains("f0"));
    }
}
