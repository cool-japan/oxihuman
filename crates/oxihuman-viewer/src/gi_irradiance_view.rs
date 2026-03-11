// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Global illumination irradiance debug view stub.

/// GI irradiance view config.
#[derive(Debug, Clone)]
pub struct GiIrradianceViewConfig {
    pub exposure: f32,
    pub enabled: bool,
    pub show_probes: bool,
    pub show_direct: bool,
}

impl Default for GiIrradianceViewConfig {
    fn default() -> Self {
        GiIrradianceViewConfig {
            exposure: 1.0,
            enabled: true,
            show_probes: true,
            show_direct: false,
        }
    }
}

/// Create a new GI irradiance view config.
pub fn new_gi_irradiance_view() -> GiIrradianceViewConfig {
    GiIrradianceViewConfig::default()
}

/// Set exposure.
pub fn giv_set_exposure(cfg: &mut GiIrradianceViewConfig, exposure: f32) {
    cfg.exposure = exposure.max(0.0);
}

/// Enable or disable.
pub fn giv_set_enabled(cfg: &mut GiIrradianceViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Toggle probe visualization.
pub fn giv_toggle_probes(cfg: &mut GiIrradianceViewConfig) {
    cfg.show_probes = !cfg.show_probes;
}

/// Toggle direct light contribution.
pub fn giv_toggle_direct(cfg: &mut GiIrradianceViewConfig) {
    cfg.show_direct = !cfg.show_direct;
}

/// Tonemap irradiance value for display.
pub fn giv_tonemap(irradiance: f32, exposure: f32) -> f32 {
    let v = irradiance * exposure;
    v / (1.0 + v)
}

/// Return a JSON-like string.
pub fn giv_to_json(cfg: &GiIrradianceViewConfig) -> String {
    format!(
        r#"{{"exposure":{:.4},"enabled":{},"show_probes":{},"show_direct":{}}}"#,
        cfg.exposure, cfg.enabled, cfg.show_probes, cfg.show_direct
    )
}

/// Return debug color for an irradiance value.
pub fn giv_irradiance_to_color(irradiance: f32, exposure: f32) -> [f32; 3] {
    let v = giv_tonemap(irradiance, exposure);
    [v, v * 0.9, v * 0.8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_enabled() {
        let c = new_gi_irradiance_view();
        assert!(c.enabled /* enabled by default */,);
    }

    #[test]
    fn test_set_exposure() {
        let mut c = new_gi_irradiance_view();
        giv_set_exposure(&mut c, 2.5);
        assert!((c.exposure - 2.5).abs() < 1e-5, /* exposure must match */);
    }

    #[test]
    fn test_set_exposure_negative_clamps() {
        let mut c = new_gi_irradiance_view();
        giv_set_exposure(&mut c, -1.0);
        assert!((c.exposure).abs() < 1e-6, /* negative exposure clamped to 0 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_gi_irradiance_view();
        giv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_toggle_probes() {
        let mut c = new_gi_irradiance_view();
        giv_toggle_probes(&mut c);
        assert!(!c.show_probes /* probes should be toggled off */,);
    }

    #[test]
    fn test_toggle_direct() {
        let mut c = new_gi_irradiance_view();
        giv_toggle_direct(&mut c);
        assert!(c.show_direct /* direct should be toggled on */,);
    }

    #[test]
    fn test_tonemap_zero_is_zero() {
        let v = giv_tonemap(0.0, 1.0);
        assert!((v).abs() < 1e-6 /* tonemap of 0 is 0 */,);
    }

    #[test]
    fn test_tonemap_positive() {
        let v = giv_tonemap(1.0, 1.0);
        assert!(v > 0.0 && v < 1.0, /* tonemapped value should be in 0-1 */);
    }

    #[test]
    fn test_to_json_contains_exposure() {
        let c = new_gi_irradiance_view();
        let j = giv_to_json(&c);
        assert!(j.contains("exposure") /* JSON must contain exposure */,);
    }

    #[test]
    fn test_irradiance_to_color_returns_three() {
        let color = giv_irradiance_to_color(1.0, 1.0);
        assert_eq!(color.len(), 3 /* color must be RGB */,);
    }
}
