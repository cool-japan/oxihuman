// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Emissive channel visualization debug view.

/// Configuration for emission view.
#[derive(Debug, Clone)]
pub struct EmissionViewConfig {
    pub exposure: f32,
    pub show_hdr_clipping: bool,
    pub hdr_threshold: f32,
}

impl Default for EmissionViewConfig {
    fn default() -> Self {
        Self { exposure: 1.0, show_hdr_clipping: false, hdr_threshold: 1.0 }
    }
}

/// State for emissive channel visualization.
#[derive(Debug, Clone)]
pub struct EmissionView {
    pub config: EmissionViewConfig,
    pub enabled: bool,
}

impl Default for EmissionView {
    fn default() -> Self {
        Self { config: EmissionViewConfig::default(), enabled: false }
    }
}

/// Enable emission view.
pub fn emv_enable(view: &mut EmissionView) {
    view.enabled = true;
}

/// Disable emission view.
pub fn emv_disable(view: &mut EmissionView) {
    view.enabled = false;
}

/// Set the exposure (EV) for emissive display.
pub fn emv_set_exposure(view: &mut EmissionView, ev: f32) {
    view.config.exposure = ev.clamp(0.0, 64.0);
}

/// Apply exposure to a raw emissive value.
pub fn emv_apply_exposure(value: f32, config: &EmissionViewConfig) -> f32 {
    (value * config.exposure).clamp(0.0, 65504.0)
}

/// Return whether a value exceeds the HDR threshold.
pub fn emv_is_clipped(value: f32, config: &EmissionViewConfig) -> bool {
    value > config.hdr_threshold
}

/// Map an emissive value to a display RGBA color.
pub fn emv_to_color(value: f32, config: &EmissionViewConfig) -> [f32; 4] {
    let v = (value * config.exposure).clamp(0.0, 1.0);
    if config.show_hdr_clipping && emv_is_clipped(value, config) {
        [1.0, 0.0, 1.0, 1.0] /* magenta for HDR clipping */
    } else {
        [v, v * 0.8, 0.0, 1.0]
    }
}

/// Export config to JSON string (stub).
pub fn emv_to_json(view: &EmissionView) -> String {
    format!(
        r#"{{"exposure":{:.2},"hdr_threshold":{:.2},"enabled":{}}}"#,
        view.config.exposure, view.config.hdr_threshold, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = EmissionView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = EmissionView::default();
        emv_enable(&mut v);
        assert!(v.enabled);
        emv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_exposure() {
        /* exposure should be stored */
        let mut v = EmissionView::default();
        emv_set_exposure(&mut v, 2.0);
        assert!((v.config.exposure - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_exposure_clamp() {
        /* exposure above maximum should be clamped */
        let mut v = EmissionView::default();
        emv_set_exposure(&mut v, 9999.0);
        assert!((v.config.exposure - 64.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_exposure() {
        /* exposure should scale the value */
        let cfg = EmissionViewConfig { exposure: 2.0, ..Default::default() };
        assert!((emv_apply_exposure(0.5, &cfg) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_clipped_true() {
        /* value above threshold should be clipped */
        let cfg = EmissionViewConfig { hdr_threshold: 0.5, ..Default::default() };
        assert!(emv_is_clipped(0.8, &cfg));
    }

    #[test]
    fn test_is_clipped_false() {
        /* value below threshold should not be clipped */
        let cfg = EmissionViewConfig::default();
        assert!(!emv_is_clipped(0.5, &cfg));
    }

    #[test]
    fn test_to_color_alpha() {
        /* alpha should always be 1.0 */
        let cfg = EmissionViewConfig::default();
        let c = emv_to_color(0.5, &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should reflect enabled state */
        let mut v = EmissionView::default();
        emv_enable(&mut v);
        let json = emv_to_json(&v);
        assert!(json.contains("true"));
    }
}
