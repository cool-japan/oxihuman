// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ambient occlusion term visualization debug view.

/// Configuration for AO visualization.
#[derive(Debug, Clone)]
pub struct AoViewConfig {
    pub brightness: f32,
    pub contrast: f32,
    pub show_raw: bool,
}

impl Default for AoViewConfig {
    fn default() -> Self {
        Self { brightness: 1.0, contrast: 1.0, show_raw: true }
    }
}

/// State for AO term visualization.
#[derive(Debug, Clone)]
pub struct AmbientOcclusionView {
    pub config: AoViewConfig,
    pub enabled: bool,
}

impl Default for AmbientOcclusionView {
    fn default() -> Self {
        Self { config: AoViewConfig::default(), enabled: false }
    }
}

/// Enable AO visualization.
pub fn aov_enable(view: &mut AmbientOcclusionView) {
    view.enabled = true;
}

/// Disable AO visualization.
pub fn aov_disable(view: &mut AmbientOcclusionView) {
    view.enabled = false;
}

/// Set brightness for the AO visualization.
pub fn aov_set_brightness(view: &mut AmbientOcclusionView, brightness: f32) {
    view.config.brightness = brightness.clamp(0.0, 4.0);
}

/// Set contrast for the AO visualization.
pub fn aov_set_contrast(view: &mut AmbientOcclusionView, contrast: f32) {
    view.config.contrast = contrast.clamp(0.0, 4.0);
}

/// Apply brightness and contrast to a raw AO sample.
pub fn aov_apply(sample: f32, config: &AoViewConfig) -> f32 {
    let s = sample.clamp(0.0, 1.0);
    let adjusted = (s - 0.5) * config.contrast + 0.5;
    (adjusted * config.brightness).clamp(0.0, 1.0)
}

/// Export config to JSON string (stub).
pub fn aov_to_json(view: &AmbientOcclusionView) -> String {
    format!(
        r#"{{"brightness":{:.2},"contrast":{:.2},"enabled":{}}}"#,
        view.config.brightness, view.config.contrast, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = AmbientOcclusionView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable and disable should toggle */
        let mut v = AmbientOcclusionView::default();
        aov_enable(&mut v);
        assert!(v.enabled);
        aov_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_brightness_clamp() {
        /* brightness above 4.0 should be clamped */
        let mut v = AmbientOcclusionView::default();
        aov_set_brightness(&mut v, 10.0);
        assert!((v.config.brightness - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_contrast_clamp() {
        /* contrast below 0 should be clamped */
        let mut v = AmbientOcclusionView::default();
        aov_set_contrast(&mut v, -1.0);
        assert_eq!(v.config.contrast, 0.0);
    }

    #[test]
    fn test_apply_identity() {
        /* identity config should leave 0.5 unchanged */
        let cfg = AoViewConfig::default();
        let out = aov_apply(0.5, &cfg);
        assert!((out - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_apply_clamps_output() {
        /* output should always be in [0, 1] */
        let cfg = AoViewConfig { brightness: 4.0, contrast: 4.0, show_raw: false };
        let out = aov_apply(1.0, &cfg);
        assert!((0.0..=1.0).contains(&out));
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should reflect enabled state */
        let mut v = AmbientOcclusionView::default();
        aov_enable(&mut v);
        let json = aov_to_json(&v);
        assert!(json.contains("true"));
    }

    #[test]
    fn test_default_brightness() {
        /* default brightness should be 1.0 */
        let v = AmbientOcclusionView::default();
        assert!((v.config.brightness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_zero_brightness() {
        /* zero brightness should output zero */
        let cfg = AoViewConfig { brightness: 0.0, contrast: 1.0, show_raw: false };
        assert_eq!(aov_apply(1.0, &cfg), 0.0);
    }
}
