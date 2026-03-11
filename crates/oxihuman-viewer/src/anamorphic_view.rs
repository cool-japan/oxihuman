// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Anamorphic lens simulation stub.

/// Anamorphic squeeze ratio preset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SqueezeRatio {
    S1_25x,
    S1_33x,
    S1_5x,
    S2x,
}

/// Anamorphic view configuration.
#[derive(Debug, Clone)]
pub struct AnamorphicView {
    pub squeeze: SqueezeRatio,
    pub lens_flare_intensity: f32,
    pub horizontal_streak: f32,
    pub bokeh_oval_ratio: f32,
    pub enabled: bool,
}

impl AnamorphicView {
    pub fn new() -> Self {
        AnamorphicView {
            squeeze: SqueezeRatio::S2x,
            lens_flare_intensity: 0.5,
            horizontal_streak: 0.8,
            bokeh_oval_ratio: 0.5,
            enabled: true,
        }
    }
}

impl Default for AnamorphicView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new anamorphic view.
pub fn new_anamorphic_view() -> AnamorphicView {
    AnamorphicView::new()
}

/// Get squeeze ratio as float.
pub fn anv_squeeze_factor(anv: &AnamorphicView) -> f32 {
    match anv.squeeze {
        SqueezeRatio::S1_25x => 1.25,
        SqueezeRatio::S1_33x => 1.33,
        SqueezeRatio::S1_5x => 1.5,
        SqueezeRatio::S2x => 2.0,
    }
}

/// Set squeeze ratio.
pub fn anv_set_squeeze(anv: &mut AnamorphicView, squeeze: SqueezeRatio) {
    anv.squeeze = squeeze;
}

/// Set lens flare intensity.
pub fn anv_set_flare_intensity(anv: &mut AnamorphicView, intensity: f32) {
    anv.lens_flare_intensity = intensity.clamp(0.0, 1.0);
}

/// Set horizontal streak strength.
pub fn anv_set_streak(anv: &mut AnamorphicView, streak: f32) {
    anv.horizontal_streak = streak.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn anv_set_enabled(anv: &mut AnamorphicView, enabled: bool) {
    anv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn anv_to_json(anv: &AnamorphicView) -> String {
    format!(
        r#"{{"squeeze":{},"flare_intensity":{},"streak":{},"enabled":{}}}"#,
        anv_squeeze_factor(anv),
        anv.lens_flare_intensity,
        anv.horizontal_streak,
        anv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_squeeze_2x() {
        let a = new_anamorphic_view();
        assert_eq!(
            a.squeeze,
            SqueezeRatio::S2x, /* default squeeze must be 2x */
        );
    }

    #[test]
    fn test_squeeze_factor_2x() {
        let a = new_anamorphic_view();
        assert!((anv_squeeze_factor(&a) - 2.0).abs() < 1e-5, /* 2x squeeze factor */);
    }

    #[test]
    fn test_squeeze_factor_1_33x() {
        let mut a = new_anamorphic_view();
        anv_set_squeeze(&mut a, SqueezeRatio::S1_33x);
        assert!((anv_squeeze_factor(&a) - 1.33).abs() < 1e-3, /* 1.33x squeeze factor */);
    }

    #[test]
    fn test_set_squeeze() {
        let mut a = new_anamorphic_view();
        anv_set_squeeze(&mut a, SqueezeRatio::S1_5x);
        assert_eq!(
            a.squeeze,
            SqueezeRatio::S1_5x, /* squeeze must be set */
        );
    }

    #[test]
    fn test_set_flare_clamped() {
        let mut a = new_anamorphic_view();
        anv_set_flare_intensity(&mut a, 2.0);
        assert!((a.lens_flare_intensity - 1.0).abs() < 1e-6, /* flare clamped to 1.0 */);
    }

    #[test]
    fn test_set_streak_clamped() {
        let mut a = new_anamorphic_view();
        anv_set_streak(&mut a, -0.5);
        assert!((a.horizontal_streak).abs() < 1e-6, /* streak clamped to 0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut a = new_anamorphic_view();
        anv_set_enabled(&mut a, false);
        assert!(!a.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_squeeze() {
        let a = new_anamorphic_view();
        let j = anv_to_json(&a);
        assert!(j.contains("\"squeeze\""), /* json must contain squeeze */);
    }

    #[test]
    fn test_enabled_default() {
        let a = new_anamorphic_view();
        assert!(a.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_bokeh_oval_ratio_default() {
        let a = new_anamorphic_view();
        assert!((a.bokeh_oval_ratio - 0.5).abs() < 1e-5, /* default bokeh ratio must be 0.5 */);
    }
}
