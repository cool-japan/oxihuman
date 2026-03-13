// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Contact point debug view — renders contact manifold points and normals.

/// Contact point view configuration.
#[derive(Debug, Clone)]
pub struct ContactPointView {
    pub enabled: bool,
    pub point_size: f32,
    pub normal_length: f32,
    pub color_point: [f32; 4],
    pub color_normal: [f32; 4],
}

impl ContactPointView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            point_size: 4.0,
            normal_length: 0.05,
            color_point: [1.0, 0.0, 0.0, 1.0],
            color_normal: [1.0, 1.0, 0.0, 1.0],
        }
    }
}

impl Default for ContactPointView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new contact point view.
pub fn new_contact_point_view() -> ContactPointView {
    ContactPointView::new()
}

/// Enable or disable the contact point overlay.
pub fn cpv_set_enabled(v: &mut ContactPointView, enabled: bool) {
    v.enabled = enabled;
}

/// Set point glyph size in pixels.
pub fn cpv_set_point_size(v: &mut ContactPointView, size: f32) {
    v.point_size = size.clamp(1.0, 32.0);
}

/// Set contact normal arrow length.
pub fn cpv_set_normal_length(v: &mut ContactPointView, len: f32) {
    v.normal_length = len.clamp(0.001, 1.0);
}

/// Set contact point colour.
pub fn cpv_set_color_point(v: &mut ContactPointView, color: [f32; 4]) {
    v.color_point = color;
}

/// Set normal arrow colour.
pub fn cpv_set_color_normal(v: &mut ContactPointView, color: [f32; 4]) {
    v.color_normal = color;
}

/// Serialize to JSON-like string.
pub fn contact_point_view_to_json(v: &ContactPointView) -> String {
    format!(
        r#"{{"enabled":{},"point_size":{:.4},"normal_length":{:.4}}}"#,
        v.enabled, v.point_size, v.normal_length
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_contact_point_view();
        assert!(!v.enabled);
        assert!((v.point_size - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable() {
        let mut v = new_contact_point_view();
        cpv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_point_size_clamp_min() {
        let mut v = new_contact_point_view();
        cpv_set_point_size(&mut v, 0.0);
        assert_eq!(v.point_size, 1.0);
    }

    #[test]
    fn test_point_size_clamp_max() {
        let mut v = new_contact_point_view();
        cpv_set_point_size(&mut v, 100.0);
        assert_eq!(v.point_size, 32.0);
    }

    #[test]
    fn test_normal_length() {
        let mut v = new_contact_point_view();
        cpv_set_normal_length(&mut v, 0.1);
        assert!((v.normal_length - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_color_point() {
        let mut v = new_contact_point_view();
        cpv_set_color_point(&mut v, [0.0, 1.0, 0.0, 1.0]);
        assert!((v.color_point[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_color_normal() {
        let mut v = new_contact_point_view();
        cpv_set_color_normal(&mut v, [1.0, 0.5, 0.0, 1.0]);
        assert!((v.color_normal[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_json_keys() {
        let v = new_contact_point_view();
        let s = contact_point_view_to_json(&v);
        assert!(s.contains("normal_length"));
    }

    #[test]
    fn test_clone() {
        let v = new_contact_point_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }
}
