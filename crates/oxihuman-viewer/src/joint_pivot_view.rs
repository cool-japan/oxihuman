// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Joint pivot point visualization — renders joint axes and pivot markers.

/// Joint pivot view configuration.
#[derive(Debug, Clone)]
pub struct JointPivotView {
    pub enabled: bool,
    pub axis_length: f32,
    pub pivot_radius: f32,
    pub color_x: [f32; 4],
    pub color_y: [f32; 4],
    pub color_z: [f32; 4],
}

impl JointPivotView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            axis_length: 0.1,
            pivot_radius: 0.02,
            color_x: [1.0, 0.0, 0.0, 1.0],
            color_y: [0.0, 1.0, 0.0, 1.0],
            color_z: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

impl Default for JointPivotView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new joint pivot view.
pub fn new_joint_pivot_view() -> JointPivotView {
    JointPivotView::new()
}

/// Enable or disable the pivot view.
pub fn jpv_set_enabled(v: &mut JointPivotView, enabled: bool) {
    v.enabled = enabled;
}

/// Set axis indicator length.
pub fn jpv_set_axis_length(v: &mut JointPivotView, len: f32) {
    v.axis_length = len.clamp(0.001, 1.0);
}

/// Set pivot sphere radius.
pub fn jpv_set_pivot_radius(v: &mut JointPivotView, r: f32) {
    v.pivot_radius = r.clamp(0.001, 0.5);
}

/// Set X-axis color.
pub fn jpv_set_color_x(v: &mut JointPivotView, color: [f32; 4]) {
    v.color_x = color;
}

/// Set Y-axis color.
pub fn jpv_set_color_y(v: &mut JointPivotView, color: [f32; 4]) {
    v.color_y = color;
}

/// Serialize to JSON-like string.
pub fn joint_pivot_view_to_json(v: &JointPivotView) -> String {
    format!(
        r#"{{"enabled":{},"axis_length":{:.4},"pivot_radius":{:.4}}}"#,
        v.enabled, v.axis_length, v.pivot_radius
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_joint_pivot_view();
        assert!(!v.enabled);
        assert!((v.axis_length - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        let mut v = new_joint_pivot_view();
        jpv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_axis_length_clamp_min() {
        let mut v = new_joint_pivot_view();
        jpv_set_axis_length(&mut v, 0.0);
        assert_eq!(v.axis_length, 0.001);
    }

    #[test]
    fn test_axis_length_set() {
        let mut v = new_joint_pivot_view();
        jpv_set_axis_length(&mut v, 0.5);
        assert!((v.axis_length - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_pivot_radius_clamp() {
        let mut v = new_joint_pivot_view();
        jpv_set_pivot_radius(&mut v, 10.0);
        assert_eq!(v.pivot_radius, 0.5);
    }

    #[test]
    fn test_color_x() {
        let mut v = new_joint_pivot_view();
        jpv_set_color_x(&mut v, [0.5, 0.5, 0.5, 1.0]);
        assert!((v.color_x[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_color_y() {
        let mut v = new_joint_pivot_view();
        jpv_set_color_y(&mut v, [1.0, 1.0, 0.0, 1.0]);
        assert!((v.color_y[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_joint_pivot_view();
        let s = joint_pivot_view_to_json(&v);
        assert!(s.contains("axis_length"));
    }

    #[test]
    fn test_clone() {
        let v = new_joint_pivot_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }
}
