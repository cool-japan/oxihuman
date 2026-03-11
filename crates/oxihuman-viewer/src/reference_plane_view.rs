// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Reference image plane overlay view stub.

/// Plane orientation axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaneAxis {
    XY,
    XZ,
    YZ,
    Custom,
}

/// Reference plane view configuration.
#[derive(Debug, Clone)]
pub struct ReferencePlaneView {
    pub axis: PlaneAxis,
    pub offset: f32,
    pub opacity: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub enabled: bool,
}

impl ReferencePlaneView {
    pub fn new() -> Self {
        ReferencePlaneView {
            axis: PlaneAxis::XY,
            offset: 0.0,
            opacity: 0.7,
            flip_x: false,
            flip_y: false,
            enabled: true,
        }
    }
}

impl Default for ReferencePlaneView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new reference plane view.
pub fn new_reference_plane_view() -> ReferencePlaneView {
    ReferencePlaneView::new()
}

/// Set plane axis.
pub fn rfpv_set_axis(view: &mut ReferencePlaneView, axis: PlaneAxis) {
    view.axis = axis;
}

/// Set plane offset along its normal.
pub fn rfpv_set_offset(view: &mut ReferencePlaneView, offset: f32) {
    view.offset = offset;
}

/// Set image opacity.
pub fn rfpv_set_opacity(view: &mut ReferencePlaneView, opacity: f32) {
    view.opacity = opacity.clamp(0.0, 1.0);
}

/// Set flip state.
pub fn rfpv_set_flip(view: &mut ReferencePlaneView, flip_x: bool, flip_y: bool) {
    view.flip_x = flip_x;
    view.flip_y = flip_y;
}

/// Enable or disable.
pub fn rfpv_set_enabled(view: &mut ReferencePlaneView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn rfpv_to_json(view: &ReferencePlaneView) -> String {
    let ax = match view.axis {
        PlaneAxis::XY => "xy",
        PlaneAxis::XZ => "xz",
        PlaneAxis::YZ => "yz",
        PlaneAxis::Custom => "custom",
    };
    format!(
        r#"{{"axis":"{}","offset":{},"opacity":{},"flip_x":{},"flip_y":{},"enabled":{}}}"#,
        ax, view.offset, view.opacity, view.flip_x, view.flip_y, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_axis() {
        let v = new_reference_plane_view();
        assert_eq!(v.axis, PlaneAxis::XY /* default axis must be XY */);
    }

    #[test]
    fn test_set_axis() {
        let mut v = new_reference_plane_view();
        rfpv_set_axis(&mut v, PlaneAxis::YZ);
        assert_eq!(v.axis, PlaneAxis::YZ /* axis must be set */);
    }

    #[test]
    fn test_set_offset() {
        let mut v = new_reference_plane_view();
        rfpv_set_offset(&mut v, 1.5);
        assert!((v.offset - 1.5).abs() < 1e-6 /* offset must be set */);
    }

    #[test]
    fn test_opacity_clamp() {
        let mut v = new_reference_plane_view();
        rfpv_set_opacity(&mut v, 2.0);
        assert!((v.opacity - 1.0).abs() < 1e-6 /* opacity clamped to 1.0 */);
    }

    #[test]
    fn test_set_flip() {
        let mut v = new_reference_plane_view();
        rfpv_set_flip(&mut v, true, false);
        assert!(v.flip_x /* flip_x must be set */);
        assert!(!v.flip_y /* flip_y must remain false */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_reference_plane_view();
        rfpv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_axis() {
        let v = new_reference_plane_view();
        let j = rfpv_to_json(&v);
        assert!(j.contains("\"axis\"") /* JSON must have axis */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_reference_plane_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_opacity() {
        let v = new_reference_plane_view();
        assert!((v.opacity - 0.7).abs() < 1e-6 /* default opacity must be 0.7 */);
    }
}
