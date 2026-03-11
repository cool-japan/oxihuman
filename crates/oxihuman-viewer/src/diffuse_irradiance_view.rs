// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Diffuse irradiance channel debug visualization.

/// Configuration for diffuse irradiance view.
#[derive(Debug, Clone)]
pub struct DiffuseIrradianceViewConfig {
    pub exposure: f32,
    pub show_sh_bands: bool,
    pub tonemap: bool,
}

impl Default for DiffuseIrradianceViewConfig {
    fn default() -> Self {
        Self { exposure: 1.0, show_sh_bands: false, tonemap: true }
    }
}

/// State for diffuse irradiance visualization.
#[derive(Debug, Clone)]
pub struct DiffuseIrradianceView {
    pub config: DiffuseIrradianceViewConfig,
    pub enabled: bool,
}

impl Default for DiffuseIrradianceView {
    fn default() -> Self {
        Self { config: DiffuseIrradianceViewConfig::default(), enabled: false }
    }
}

/// Enable diffuse irradiance view.
pub fn div_enable(view: &mut DiffuseIrradianceView) {
    view.enabled = true;
}

/// Disable diffuse irradiance view.
pub fn div_disable(view: &mut DiffuseIrradianceView) {
    view.enabled = false;
}

/// Set the display exposure.
pub fn div_set_exposure(view: &mut DiffuseIrradianceView, ev: f32) {
    view.config.exposure = ev.clamp(0.0, 32.0);
}

/// Apply exposure and optional Reinhard tonemapping to an irradiance value.
pub fn div_apply(value: f32, config: &DiffuseIrradianceViewConfig) -> f32 {
    let v = value * config.exposure;
    if config.tonemap {
        v / (1.0 + v)
    } else {
        v.clamp(0.0, 1.0)
    }
}

/// Map an irradiance RGB triplet to a display color.
pub fn div_to_color(ir: [f32; 3], config: &DiffuseIrradianceViewConfig) -> [f32; 4] {
    [div_apply(ir[0], config), div_apply(ir[1], config), div_apply(ir[2], config), 1.0]
}

/// Estimate the luminance of an irradiance sample.
pub fn div_luminance(ir: [f32; 3]) -> f32 {
    ir[0] * 0.2126 + ir[1] * 0.7152 + ir[2] * 0.0722
}

/// Export config to JSON string (stub).
pub fn div_to_json(view: &DiffuseIrradianceView) -> String {
    format!(
        r#"{{"exposure":{:.2},"tonemap":{},"enabled":{}}}"#,
        view.config.exposure, view.config.tonemap, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = DiffuseIrradianceView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = DiffuseIrradianceView::default();
        div_enable(&mut v);
        assert!(v.enabled);
        div_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_exposure() {
        /* exposure should be stored */
        let mut v = DiffuseIrradianceView::default();
        div_set_exposure(&mut v, 2.0);
        assert!((v.config.exposure - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_exposure_clamp() {
        /* exposure above maximum should be clamped */
        let mut v = DiffuseIrradianceView::default();
        div_set_exposure(&mut v, 999.0);
        assert!((v.config.exposure - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_tonemap() {
        /* tonemapped value should be less than 1 for large inputs */
        let cfg = DiffuseIrradianceViewConfig { tonemap: true, exposure: 1.0, show_sh_bands: false };
        let v = div_apply(10.0, &cfg);
        assert!(v < 1.0);
    }

    #[test]
    fn test_apply_no_tonemap_clamp() {
        /* without tonemap large values should clamp to 1 */
        let cfg = DiffuseIrradianceViewConfig { tonemap: false, exposure: 1.0, show_sh_bands: false };
        let v = div_apply(10.0, &cfg);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_color_alpha_one() {
        /* alpha should always be 1.0 */
        let cfg = DiffuseIrradianceViewConfig::default();
        let c = div_to_color([0.5, 0.5, 0.5], &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_luminance() {
        /* pure red should have luminance ~0.2126 */
        let lum = div_luminance([1.0, 0.0, 0.0]);
        assert!((lum - 0.2126).abs() < 1e-4);
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should contain enabled field */
        let mut v = DiffuseIrradianceView::default();
        div_enable(&mut v);
        let json = div_to_json(&v);
        assert!(json.contains("true"));
    }
}
