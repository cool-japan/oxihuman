// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Light probe sphere visualization stub.

/// Light probe sphere rendering config.
#[derive(Debug, Clone)]
pub struct LightProbeViewConfig {
    pub sphere_radius: f32,
    pub exposure: f32,
    pub enabled: bool,
    pub show_sh_coefficients: bool,
}

impl Default for LightProbeViewConfig {
    fn default() -> Self {
        LightProbeViewConfig {
            sphere_radius: 0.1,
            exposure: 1.0,
            enabled: true,
            show_sh_coefficients: false,
        }
    }
}

/// Create a new light probe view config.
pub fn new_light_probe_view() -> LightProbeViewConfig {
    LightProbeViewConfig::default()
}

/// Set the sphere radius.
pub fn lpv_set_radius(cfg: &mut LightProbeViewConfig, radius: f32) {
    cfg.sphere_radius = radius.max(0.001);
}

/// Set the exposure.
pub fn lpv_set_exposure(cfg: &mut LightProbeViewConfig, exposure: f32) {
    cfg.exposure = exposure.max(0.0);
}

/// Enable or disable the view.
pub fn lpv_set_enabled(cfg: &mut LightProbeViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle SH coefficient display.
pub fn lpv_toggle_sh(cfg: &mut LightProbeViewConfig) {
    cfg.show_sh_coefficients = !cfg.show_sh_coefficients;
}

/// Compute sphere surface area.
pub fn lpv_sphere_area(cfg: &LightProbeViewConfig) -> f32 {
    4.0 * std::f32::consts::PI * cfg.sphere_radius * cfg.sphere_radius
}

/// Return a JSON-like string.
pub fn lpv_to_json(cfg: &LightProbeViewConfig) -> String {
    format!(
        r#"{{"radius":{:.4},"exposure":{:.4},"enabled":{}}}"#,
        cfg.sphere_radius, cfg.exposure, cfg.enabled
    )
}

/// Sample the irradiance color for a given normal direction (stub returns grey).
pub fn lpv_sample_direction(cfg: &LightProbeViewConfig, _normal: [f32; 3]) -> [f32; 3] {
    let v = 0.18 * cfg.exposure;
    [v, v, v]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_radius() {
        let c = new_light_probe_view();
        assert!((c.sphere_radius - 0.1).abs() < 1e-5, /* default radius is 0.1 */);
    }

    #[test]
    fn test_set_radius() {
        let mut c = new_light_probe_view();
        lpv_set_radius(&mut c, 0.5);
        assert!((c.sphere_radius - 0.5).abs() < 1e-5, /* radius must be set */);
    }

    #[test]
    fn test_set_radius_minimum() {
        let mut c = new_light_probe_view();
        lpv_set_radius(&mut c, -5.0);
        assert!(c.sphere_radius >= 0.001, /* radius must not go below minimum */);
    }

    #[test]
    fn test_set_exposure() {
        let mut c = new_light_probe_view();
        lpv_set_exposure(&mut c, 3.0);
        assert!((c.exposure - 3.0).abs() < 1e-5, /* exposure must be set */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_light_probe_view();
        lpv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_sh() {
        let mut c = new_light_probe_view();
        lpv_toggle_sh(&mut c);
        assert!(c.show_sh_coefficients /* sh should be toggled on */,);
    }

    #[test]
    fn test_sphere_area_positive() {
        let c = new_light_probe_view();
        assert!(lpv_sphere_area(&c) > 0.0, /* sphere area must be positive */);
    }

    #[test]
    fn test_to_json_contains_radius() {
        let c = new_light_probe_view();
        let j = lpv_to_json(&c);
        assert!(j.contains("radius") /* JSON must contain radius */,);
    }

    #[test]
    fn test_sample_direction_returns_three() {
        let c = new_light_probe_view();
        let s = lpv_sample_direction(&c, [0.0, 1.0, 0.0]);
        assert_eq!(s.len(), 3 /* sample must return RGB */,);
    }

    #[test]
    fn test_default_sh_hidden() {
        let c = new_light_probe_view();
        assert!(!c.show_sh_coefficients, /* SH coefficients hidden by default */);
    }
}
