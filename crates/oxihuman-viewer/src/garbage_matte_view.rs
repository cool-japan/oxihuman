// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Garbage matte overlay view for compositing region exclusion.

/// A single 2D point in normalized viewport coordinates.
#[derive(Debug, Clone, Copy)]
pub struct MattePoint {
    pub x: f32,
    pub y: f32,
}

impl MattePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: x.clamp(0.0, 1.0),
            y: y.clamp(0.0, 1.0),
        }
    }
}

/// Garbage matte view configuration.
#[derive(Debug, Clone)]
pub struct GarbageMatteView {
    pub points: Vec<MattePoint>,
    pub invert: bool,
    pub feather: f32,
    pub enabled: bool,
    pub color: [f32; 4],
}

impl GarbageMatteView {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            invert: false,
            feather: 0.0,
            enabled: false,
            color: [1.0, 0.0, 1.0, 0.5],
        }
    }
}

impl Default for GarbageMatteView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new garbage matte view.
pub fn new_garbage_matte_view() -> GarbageMatteView {
    GarbageMatteView::new()
}

/// Add a point to the matte polygon.
pub fn gmv_add_point(view: &mut GarbageMatteView, x: f32, y: f32) {
    view.points.push(MattePoint::new(x, y));
}

/// Clear all matte polygon points.
pub fn gmv_clear_points(view: &mut GarbageMatteView) {
    view.points.clear();
}

/// Set feather radius in normalized units.
pub fn gmv_set_feather(view: &mut GarbageMatteView, feather: f32) {
    view.feather = feather.clamp(0.0, 0.5);
}

/// Toggle matte inversion.
pub fn gmv_set_invert(view: &mut GarbageMatteView, invert: bool) {
    view.invert = invert;
}

/// Toggle garbage matte overlay.
pub fn gmv_set_enabled(view: &mut GarbageMatteView, enabled: bool) {
    view.enabled = enabled;
}

/// Compute approximate bounding area of the polygon.
pub fn gmv_bounding_area(view: &GarbageMatteView) -> f32 {
    let n = view.points.len();
    if n < 3 {
        return 0.0;
    }
    /* Shoelace formula */
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += view.points[i].x * view.points[j].y;
        area -= view.points[j].x * view.points[i].y;
    }
    (area / 2.0).abs()
}

/// Serialize to JSON-like string.
pub fn garbage_matte_view_to_json(view: &GarbageMatteView) -> String {
    format!(
        r#"{{"point_count":{},"invert":{},"feather":{:.4},"enabled":{}}}"#,
        view.points.len(),
        view.invert,
        view.feather,
        view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_garbage_matte_view();
        assert!(v.points.is_empty());
        assert!(!v.enabled);
    }

    #[test]
    fn test_add_points() {
        let mut v = new_garbage_matte_view();
        gmv_add_point(&mut v, 0.1, 0.2);
        gmv_add_point(&mut v, 0.9, 0.2);
        assert_eq!(v.points.len(), 2);
    }

    #[test]
    fn test_clear_points() {
        let mut v = new_garbage_matte_view();
        gmv_add_point(&mut v, 0.5, 0.5);
        gmv_clear_points(&mut v);
        assert!(v.points.is_empty());
    }

    #[test]
    fn test_feather_clamp() {
        let mut v = new_garbage_matte_view();
        gmv_set_feather(&mut v, 1.0);
        assert_eq!(v.feather, 0.5);
    }

    #[test]
    fn test_invert_toggle() {
        let mut v = new_garbage_matte_view();
        gmv_set_invert(&mut v, true);
        assert!(v.invert);
    }

    #[test]
    fn test_bounding_area_triangle() {
        let mut v = new_garbage_matte_view();
        gmv_add_point(&mut v, 0.0, 0.0);
        gmv_add_point(&mut v, 1.0, 0.0);
        gmv_add_point(&mut v, 0.0, 1.0);
        let area = gmv_bounding_area(&v);
        assert!((area - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_bounding_area_too_few_points() {
        let mut v = new_garbage_matte_view();
        gmv_add_point(&mut v, 0.0, 0.0);
        assert_eq!(gmv_bounding_area(&v), 0.0);
    }

    #[test]
    fn test_json() {
        let v = new_garbage_matte_view();
        let s = garbage_matte_view_to_json(&v);
        assert!(s.contains("point_count"));
    }

    #[test]
    fn test_clone() {
        let v = new_garbage_matte_view();
        let v2 = v.clone();
        assert_eq!(v2.invert, v.invert);
    }

    #[test]
    fn test_enabled_toggle() {
        let mut v = new_garbage_matte_view();
        gmv_set_enabled(&mut v, true);
        assert!(v.enabled);
    }
}
