// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pencil sketch edge render stub.

/// Sketch line style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SketchStyle {
    HB,
    TwoB,
    FourB,
    Mechanical,
}

/// Pencil sketch view configuration.
#[derive(Debug, Clone)]
pub struct PencilSketchView {
    pub style: SketchStyle,
    pub line_weight: f32,
    pub hatching: bool,
    pub paper_grain: f32,
    pub edge_threshold: f32,
    pub enabled: bool,
}

impl PencilSketchView {
    pub fn new() -> Self {
        PencilSketchView {
            style: SketchStyle::HB,
            line_weight: 1.0,
            hatching: false,
            paper_grain: 0.2,
            edge_threshold: 0.3,
            enabled: true,
        }
    }
}

impl Default for PencilSketchView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new pencil sketch view.
pub fn new_pencil_sketch_view() -> PencilSketchView {
    PencilSketchView::new()
}

/// Set sketch style.
pub fn psv_set_style(view: &mut PencilSketchView, style: SketchStyle) {
    view.style = style;
}

/// Set line weight.
pub fn psv_set_line_weight(view: &mut PencilSketchView, weight: f32) {
    view.line_weight = weight.clamp(0.1, 5.0);
}

/// Toggle cross-hatching.
pub fn psv_set_hatching(view: &mut PencilSketchView, hatching: bool) {
    view.hatching = hatching;
}

/// Set paper grain intensity.
pub fn psv_set_paper_grain(view: &mut PencilSketchView, grain: f32) {
    view.paper_grain = grain.clamp(0.0, 1.0);
}

/// Set edge detection threshold.
pub fn psv_set_edge_threshold(view: &mut PencilSketchView, threshold: f32) {
    view.edge_threshold = threshold.clamp(0.0, 1.0);
}

/// Enable or disable.
pub fn psv_set_enabled(view: &mut PencilSketchView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn psv_to_json(view: &PencilSketchView) -> String {
    let style = match view.style {
        SketchStyle::HB => "HB",
        SketchStyle::TwoB => "2B",
        SketchStyle::FourB => "4B",
        SketchStyle::Mechanical => "mechanical",
    };
    format!(
        r#"{{"style":"{}","line_weight":{},"hatching":{},"paper_grain":{},"edge_threshold":{},"enabled":{}}}"#,
        style, view.line_weight, view.hatching, view.paper_grain, view.edge_threshold, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style() {
        let v = new_pencil_sketch_view();
        assert_eq!(v.style, SketchStyle::HB /* default style must be HB */);
    }

    #[test]
    fn test_set_style() {
        let mut v = new_pencil_sketch_view();
        psv_set_style(&mut v, SketchStyle::FourB);
        assert_eq!(v.style, SketchStyle::FourB /* style must be set */);
    }

    #[test]
    fn test_line_weight_clamped_min() {
        let mut v = new_pencil_sketch_view();
        psv_set_line_weight(&mut v, 0.0);
        assert!((v.line_weight - 0.1).abs() < 1e-6 /* line_weight clamped to 0.1 */);
    }

    #[test]
    fn test_line_weight_clamped_max() {
        let mut v = new_pencil_sketch_view();
        psv_set_line_weight(&mut v, 10.0);
        assert!((v.line_weight - 5.0).abs() < 1e-6 /* line_weight clamped to 5.0 */);
    }

    #[test]
    fn test_set_hatching() {
        let mut v = new_pencil_sketch_view();
        psv_set_hatching(&mut v, true);
        assert!(v.hatching /* hatching must be enabled */);
    }

    #[test]
    fn test_paper_grain_clamped() {
        let mut v = new_pencil_sketch_view();
        psv_set_paper_grain(&mut v, 2.0);
        assert!((v.paper_grain - 1.0).abs() < 1e-6 /* paper_grain clamped to 1.0 */);
    }

    #[test]
    fn test_edge_threshold_clamped() {
        let mut v = new_pencil_sketch_view();
        psv_set_edge_threshold(&mut v, -1.0);
        assert!((v.edge_threshold).abs() < 1e-6 /* edge_threshold clamped to 0.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_pencil_sketch_view();
        psv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_style() {
        let v = new_pencil_sketch_view();
        let j = psv_to_json(&v);
        assert!(j.contains("\"style\"") /* JSON must have style */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_pencil_sketch_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
