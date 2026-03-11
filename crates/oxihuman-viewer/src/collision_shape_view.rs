// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Collision shape wireframe view — renders collider geometry outlines.

/// Collision shape display kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollisionShapeKind {
    Box,
    Sphere,
    Capsule,
    ConvexHull,
    Mesh,
}

/// Collision shape view configuration.
#[derive(Debug, Clone)]
pub struct CollisionShapeView {
    pub enabled: bool,
    pub color: [f32; 4],
    pub line_width: f32,
    pub filter_kind: Option<CollisionShapeKind>,
}

impl CollisionShapeView {
    pub fn new() -> Self {
        Self {
            enabled: false,
            color: [0.0, 1.0, 0.0, 0.8],
            line_width: 1.5,
            filter_kind: None,
        }
    }
}

impl Default for CollisionShapeView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new collision shape view.
pub fn new_collision_shape_view() -> CollisionShapeView {
    CollisionShapeView::new()
}

/// Enable or disable the collision shape overlay.
pub fn csv_set_enabled(v: &mut CollisionShapeView, enabled: bool) {
    v.enabled = enabled;
}

/// Set wireframe color (RGBA).
pub fn csv_set_color(v: &mut CollisionShapeView, r: f32, g: f32, b: f32, a: f32) {
    v.color = [
        r.clamp(0.0, 1.0),
        g.clamp(0.0, 1.0),
        b.clamp(0.0, 1.0),
        a.clamp(0.0, 1.0),
    ];
}

/// Set wireframe line width.
pub fn csv_set_line_width(v: &mut CollisionShapeView, w: f32) {
    v.line_width = w.clamp(0.5, 8.0);
}

/// Filter by specific collision shape kind.
pub fn csv_set_filter(v: &mut CollisionShapeView, kind: CollisionShapeKind) {
    v.filter_kind = Some(kind);
}

/// Clear shape kind filter (show all).
pub fn csv_clear_filter(v: &mut CollisionShapeView) {
    v.filter_kind = None;
}

/// Serialize to JSON-like string.
pub fn collision_shape_view_to_json(v: &CollisionShapeView) -> String {
    format!(
        r#"{{"enabled":{},"line_width":{:.4},"color":[{:.3},{:.3},{:.3},{:.3}]}}"#,
        v.enabled, v.line_width, v.color[0], v.color[1], v.color[2], v.color[3]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_collision_shape_view();
        assert!(!v.enabled);
        assert!(v.filter_kind.is_none());
    }

    #[test]
    fn test_enable() {
        let mut v = new_collision_shape_view();
        csv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }

    #[test]
    fn test_color_set() {
        let mut v = new_collision_shape_view();
        csv_set_color(&mut v, 1.0, 0.0, 0.0, 1.0);
        assert!((v.color[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_line_width_clamp() {
        let mut v = new_collision_shape_view();
        csv_set_line_width(&mut v, 0.0);
        assert_eq!(v.line_width, 0.5);
    }

    #[test]
    fn test_filter_set() {
        let mut v = new_collision_shape_view();
        csv_set_filter(&mut v, CollisionShapeKind::Sphere);
        assert_eq!(v.filter_kind, Some(CollisionShapeKind::Sphere));
    }

    #[test]
    fn test_filter_clear() {
        let mut v = new_collision_shape_view();
        csv_set_filter(&mut v, CollisionShapeKind::Box);
        csv_clear_filter(&mut v);
        assert!(v.filter_kind.is_none());
    }

    #[test]
    fn test_json_keys() {
        let v = new_collision_shape_view();
        let s = collision_shape_view_to_json(&v);
        assert!(s.contains("line_width"));
    }

    #[test]
    fn test_clone() {
        let v = new_collision_shape_view();
        let v2 = v.clone();
        assert_eq!(v2.enabled, v.enabled);
    }

    #[test]
    fn test_line_width_max_clamp() {
        let mut v = new_collision_shape_view();
        csv_set_line_width(&mut v, 100.0);
        assert_eq!(v.line_width, 8.0);
    }
}
