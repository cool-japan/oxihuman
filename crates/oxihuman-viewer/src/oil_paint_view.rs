// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Oil painting post-process view stub.

/// Oil paint brush style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushStyle {
    Bristle,
    Smooth,
    Impasto,
}

/// Oil paint view configuration.
#[derive(Debug, Clone)]
pub struct OilPaintView {
    pub brush_style: BrushStyle,
    pub brush_size: f32,
    pub smoothing: f32,
    pub saturation_boost: f32,
    pub enabled: bool,
}

impl OilPaintView {
    pub fn new() -> Self {
        OilPaintView {
            brush_style: BrushStyle::Bristle,
            brush_size: 8.0,
            smoothing: 0.5,
            saturation_boost: 1.2,
            enabled: true,
        }
    }
}

impl Default for OilPaintView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new oil paint view.
pub fn new_oil_paint_view() -> OilPaintView {
    OilPaintView::new()
}

/// Set brush style.
pub fn opv_set_brush_style(view: &mut OilPaintView, style: BrushStyle) {
    view.brush_style = style;
}

/// Set brush size.
pub fn opv_set_brush_size(view: &mut OilPaintView, size: f32) {
    view.brush_size = size.clamp(1.0, 64.0);
}

/// Set smoothing amount.
pub fn opv_set_smoothing(view: &mut OilPaintView, smoothing: f32) {
    view.smoothing = smoothing.clamp(0.0, 1.0);
}

/// Set saturation boost.
pub fn opv_set_saturation_boost(view: &mut OilPaintView, boost: f32) {
    view.saturation_boost = boost.clamp(0.0, 3.0);
}

/// Enable or disable.
pub fn opv_set_enabled(view: &mut OilPaintView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn opv_to_json(view: &OilPaintView) -> String {
    let style = match view.brush_style {
        BrushStyle::Bristle => "bristle",
        BrushStyle::Smooth => "smooth",
        BrushStyle::Impasto => "impasto",
    };
    format!(
        r#"{{"brush_style":"{}","brush_size":{},"smoothing":{},"saturation_boost":{},"enabled":{}}}"#,
        style, view.brush_size, view.smoothing, view.saturation_boost, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_brush_style() {
        let v = new_oil_paint_view();
        assert_eq!(
            v.brush_style,
            BrushStyle::Bristle /* default style must be Bristle */
        );
    }

    #[test]
    fn test_set_brush_style() {
        let mut v = new_oil_paint_view();
        opv_set_brush_style(&mut v, BrushStyle::Impasto);
        assert_eq!(
            v.brush_style,
            BrushStyle::Impasto /* style must be set */
        );
    }

    #[test]
    fn test_brush_size_clamped() {
        let mut v = new_oil_paint_view();
        opv_set_brush_size(&mut v, 128.0);
        assert!((v.brush_size - 64.0).abs() < 1e-6 /* brush_size clamped to 64.0 */);
    }

    #[test]
    fn test_smoothing_clamped() {
        let mut v = new_oil_paint_view();
        opv_set_smoothing(&mut v, 2.0);
        assert!((v.smoothing - 1.0).abs() < 1e-6 /* smoothing clamped to 1.0 */);
    }

    #[test]
    fn test_saturation_boost_clamped() {
        let mut v = new_oil_paint_view();
        opv_set_saturation_boost(&mut v, 5.0);
        assert!((v.saturation_boost - 3.0).abs() < 1e-6 /* saturation_boost clamped to 3.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_oil_paint_view();
        opv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_brush_style() {
        let v = new_oil_paint_view();
        let j = opv_to_json(&v);
        assert!(j.contains("\"brush_style\"") /* JSON must have brush_style */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_oil_paint_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_brush_size() {
        let v = new_oil_paint_view();
        assert!((v.brush_size - 8.0).abs() < 1e-6 /* default brush_size must be 8.0 */);
    }

    #[test]
    fn test_brush_size_min_clamped() {
        let mut v = new_oil_paint_view();
        opv_set_brush_size(&mut v, 0.0);
        assert!((v.brush_size - 1.0).abs() < 1e-6 /* brush_size clamped to minimum 1.0 */);
    }
}
