// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Watercolor bleed effect view stub.

/// Watercolor bleed pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BleedPattern {
    Radial,
    Directional,
    Turbulent,
}

/// Watercolor view configuration.
#[derive(Debug, Clone)]
pub struct WatercolorView {
    pub bleed_pattern: BleedPattern,
    pub bleed_amount: f32,
    pub wetness: f32,
    pub paper_texture: f32,
    pub enabled: bool,
}

impl WatercolorView {
    pub fn new() -> Self {
        WatercolorView {
            bleed_pattern: BleedPattern::Radial,
            bleed_amount: 0.4,
            wetness: 0.6,
            paper_texture: 0.3,
            enabled: true,
        }
    }
}

impl Default for WatercolorView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new watercolor view.
pub fn new_watercolor_view() -> WatercolorView {
    WatercolorView::new()
}

/// Set bleed pattern.
pub fn wcv_set_bleed_pattern(view: &mut WatercolorView, pattern: BleedPattern) {
    view.bleed_pattern = pattern;
}

/// Set bleed amount.
pub fn wcv_set_bleed_amount(view: &mut WatercolorView, amount: f32) {
    view.bleed_amount = amount.clamp(0.0, 1.0);
}

/// Set wetness.
pub fn wcv_set_wetness(view: &mut WatercolorView, wetness: f32) {
    view.wetness = wetness.clamp(0.0, 1.0);
}

/// Set paper texture intensity.
pub fn wcv_set_paper_texture(view: &mut WatercolorView, texture: f32) {
    view.paper_texture = texture.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn wcv_set_enabled(view: &mut WatercolorView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn wcv_to_json(view: &WatercolorView) -> String {
    let pattern = match view.bleed_pattern {
        BleedPattern::Radial => "radial",
        BleedPattern::Directional => "directional",
        BleedPattern::Turbulent => "turbulent",
    };
    format!(
        r#"{{"bleed_pattern":"{}","bleed_amount":{},"wetness":{},"paper_texture":{},"enabled":{}}}"#,
        pattern, view.bleed_amount, view.wetness, view.paper_texture, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pattern() {
        let v = new_watercolor_view();
        assert_eq!(
            v.bleed_pattern,
            BleedPattern::Radial /* default pattern must be Radial */
        );
    }

    #[test]
    fn test_set_pattern() {
        let mut v = new_watercolor_view();
        wcv_set_bleed_pattern(&mut v, BleedPattern::Turbulent);
        assert_eq!(
            v.bleed_pattern,
            BleedPattern::Turbulent /* pattern must be set */
        );
    }

    #[test]
    fn test_bleed_amount_clamped() {
        let mut v = new_watercolor_view();
        wcv_set_bleed_amount(&mut v, 2.0);
        assert!((v.bleed_amount - 1.0).abs() < 1e-6 /* bleed_amount clamped to 1.0 */);
    }

    #[test]
    fn test_wetness_clamped() {
        let mut v = new_watercolor_view();
        wcv_set_wetness(&mut v, -0.5);
        assert!((v.wetness).abs() < 1e-6 /* wetness clamped to 0.0 */);
    }

    #[test]
    fn test_paper_texture_clamped() {
        let mut v = new_watercolor_view();
        wcv_set_paper_texture(&mut v, 1.5);
        assert!((v.paper_texture - 1.0).abs() < 1e-6 /* paper_texture clamped to 1.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_watercolor_view();
        wcv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_pattern() {
        let v = new_watercolor_view();
        let j = wcv_to_json(&v);
        assert!(j.contains("\"bleed_pattern\"") /* JSON must have bleed_pattern */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_watercolor_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_wetness() {
        let v = new_watercolor_view();
        assert!((v.wetness - 0.6).abs() < 1e-6 /* default wetness must be 0.6 */);
    }

    #[test]
    fn test_default_trait() {
        let v = WatercolorView::default();
        assert!((v.bleed_amount - 0.4).abs() < 1e-6 /* Default trait must give 0.4 bleed */);
    }
}
