// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Halftone dot pattern view stub.

/// Halftone dot shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DotShape {
    Circle,
    Square,
    Diamond,
    Line,
}

/// Halftone view configuration.
#[derive(Debug, Clone)]
pub struct HalftoneView {
    pub dot_shape: DotShape,
    pub dot_size: f32,
    pub angle: f32,
    pub contrast: f32,
    pub enabled: bool,
}

impl HalftoneView {
    pub fn new() -> Self {
        HalftoneView {
            dot_shape: DotShape::Circle,
            dot_size: 4.0,
            angle: 45.0,
            contrast: 1.0,
            enabled: true,
        }
    }
}

impl Default for HalftoneView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new halftone view.
pub fn new_halftone_view() -> HalftoneView {
    HalftoneView::new()
}

/// Set dot shape.
pub fn hfv_set_dot_shape(view: &mut HalftoneView, shape: DotShape) {
    view.dot_shape = shape;
}

/// Set dot size in pixels.
pub fn hfv_set_dot_size(view: &mut HalftoneView, size: f32) {
    view.dot_size = size.clamp(1.0, 32.0);
}

/// Set screen angle in degrees.
pub fn hfv_set_angle(view: &mut HalftoneView, angle: f32) {
    view.angle = angle % 180.0;
}

/// Set contrast multiplier.
pub fn hfv_set_contrast(view: &mut HalftoneView, contrast: f32) {
    view.contrast = contrast.clamp(0.0, 4.0);
}

/// Enable or disable.
pub fn hfv_set_enabled(view: &mut HalftoneView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn hfv_to_json(view: &HalftoneView) -> String {
    let shape = match view.dot_shape {
        DotShape::Circle => "circle",
        DotShape::Square => "square",
        DotShape::Diamond => "diamond",
        DotShape::Line => "line",
    };
    format!(
        r#"{{"dot_shape":"{}","dot_size":{},"angle":{},"contrast":{},"enabled":{}}}"#,
        shape, view.dot_size, view.angle, view.contrast, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_shape() {
        let v = new_halftone_view();
        assert_eq!(
            v.dot_shape,
            DotShape::Circle /* default shape must be Circle */
        );
    }

    #[test]
    fn test_set_dot_shape() {
        let mut v = new_halftone_view();
        hfv_set_dot_shape(&mut v, DotShape::Diamond);
        assert_eq!(v.dot_shape, DotShape::Diamond /* shape must be set */);
    }

    #[test]
    fn test_dot_size_clamped_min() {
        let mut v = new_halftone_view();
        hfv_set_dot_size(&mut v, 0.0);
        assert!((v.dot_size - 1.0).abs() < 1e-6 /* dot_size clamped to 1.0 */);
    }

    #[test]
    fn test_dot_size_clamped_max() {
        let mut v = new_halftone_view();
        hfv_set_dot_size(&mut v, 100.0);
        assert!((v.dot_size - 32.0).abs() < 1e-6 /* dot_size clamped to 32.0 */);
    }

    #[test]
    fn test_set_angle() {
        let mut v = new_halftone_view();
        hfv_set_angle(&mut v, 30.0);
        assert!((v.angle - 30.0).abs() < 1e-5 /* angle must be set */);
    }

    #[test]
    fn test_angle_modulo() {
        let mut v = new_halftone_view();
        hfv_set_angle(&mut v, 270.0);
        assert!((v.angle - 90.0).abs() < 1e-5 /* angle must wrap at 180 */);
    }

    #[test]
    fn test_contrast_clamped() {
        let mut v = new_halftone_view();
        hfv_set_contrast(&mut v, 10.0);
        assert!((v.contrast - 4.0).abs() < 1e-6 /* contrast clamped to 4.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_halftone_view();
        hfv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_shape() {
        let v = new_halftone_view();
        let j = hfv_to_json(&v);
        assert!(j.contains("\"dot_shape\"") /* JSON must have dot_shape */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_halftone_view();
        assert!(v.enabled /* must be enabled by default */);
    }
}
