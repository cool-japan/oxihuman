// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Curve renderer for Bezier and spline curves.

/// A 2D point.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CurvePoint {
    pub x: f32,
    pub y: f32,
}

/// Curve type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    Linear,
    Quadratic,
    Cubic,
}

/// Curve render state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveRenderState {
    pub points: Vec<CurvePoint>,
    pub curve_type: CurveType,
    pub segments: u32,
    pub line_width: f32,
    pub color: [f32; 4],
}

#[allow(dead_code)]
pub fn new_curve_point(x: f32, y: f32) -> CurvePoint {
    CurvePoint { x, y }
}

#[allow(dead_code)]
pub fn new_curve_render_state() -> CurveRenderState {
    CurveRenderState {
        points: Vec::new(),
        curve_type: CurveType::Cubic,
        segments: 32,
        line_width: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
    }
}

#[allow(dead_code)]
pub fn add_curve_point(state: &mut CurveRenderState, x: f32, y: f32) {
    state.points.push(CurvePoint { x, y });
}

#[allow(dead_code)]
pub fn set_curve_type(state: &mut CurveRenderState, ct: CurveType) {
    state.curve_type = ct;
}

#[allow(dead_code)]
pub fn set_curve_segments(state: &mut CurveRenderState, count: u32) {
    state.segments = count.clamp(2, 256);
}

#[allow(dead_code)]
pub fn lerp_point(a: CurvePoint, b: CurvePoint, t: f32) -> CurvePoint {
    CurvePoint {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    }
}

#[allow(dead_code)]
pub fn quadratic_bezier(p0: CurvePoint, p1: CurvePoint, p2: CurvePoint, t: f32) -> CurvePoint {
    let a = lerp_point(p0, p1, t);
    let b = lerp_point(p1, p2, t);
    lerp_point(a, b, t)
}

#[allow(dead_code)]
pub fn cubic_bezier(p0: CurvePoint, p1: CurvePoint, p2: CurvePoint, p3: CurvePoint, t: f32) -> CurvePoint {
    let a = lerp_point(p0, p1, t);
    let b = lerp_point(p1, p2, t);
    let c = lerp_point(p2, p3, t);
    let d = lerp_point(a, b, t);
    let e = lerp_point(b, c, t);
    lerp_point(d, e, t)
}

#[allow(dead_code)]
pub fn clear_curve_points(state: &mut CurveRenderState) {
    state.points.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_point() {
        let p = new_curve_point(1.0, 2.0);
        assert!((p.x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state() {
        let s = new_curve_render_state();
        assert!(s.points.is_empty());
        assert_eq!(s.curve_type, CurveType::Cubic);
    }

    #[test]
    fn test_add_point() {
        let mut s = new_curve_render_state();
        add_curve_point(&mut s, 1.0, 2.0);
        assert_eq!(s.points.len(), 1);
    }

    #[test]
    fn test_set_type() {
        let mut s = new_curve_render_state();
        set_curve_type(&mut s, CurveType::Linear);
        assert_eq!(s.curve_type, CurveType::Linear);
    }

    #[test]
    fn test_set_segments_clamp() {
        let mut s = new_curve_render_state();
        set_curve_segments(&mut s, 1);
        assert_eq!(s.segments, 2);
        set_curve_segments(&mut s, 1000);
        assert_eq!(s.segments, 256);
    }

    #[test]
    fn test_lerp_point() {
        let a = new_curve_point(0.0, 0.0);
        let b = new_curve_point(10.0, 10.0);
        let mid = lerp_point(a, b, 0.5);
        assert!((mid.x - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_quadratic_bezier_endpoints() {
        let p0 = new_curve_point(0.0, 0.0);
        let p1 = new_curve_point(5.0, 10.0);
        let p2 = new_curve_point(10.0, 0.0);
        let start = quadratic_bezier(p0, p1, p2, 0.0);
        assert!((start.x).abs() < 1e-6);
        let end = quadratic_bezier(p0, p1, p2, 1.0);
        assert!((end.x - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_cubic_bezier_endpoints() {
        let p0 = new_curve_point(0.0, 0.0);
        let p1 = new_curve_point(3.0, 10.0);
        let p2 = new_curve_point(7.0, 10.0);
        let p3 = new_curve_point(10.0, 0.0);
        let start = cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((start.x).abs() < 1e-6);
    }

    #[test]
    fn test_clear_points() {
        let mut s = new_curve_render_state();
        add_curve_point(&mut s, 1.0, 2.0);
        clear_curve_points(&mut s);
        assert!(s.points.is_empty());
    }

    #[test]
    fn test_lerp_identity() {
        let p = new_curve_point(5.0, 3.0);
        let r = lerp_point(p, p, 0.5);
        assert!((r.x - 5.0).abs() < 1e-6);
    }
}
