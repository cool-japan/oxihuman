// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export curve control point data (Bezier / NURBS control polygons).

/// Curve type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveType {
    Bezier,
    Nurbs,
    CatmullRom,
}

/// A control point with optional tangents.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ControlPoint {
    pub position: [f32; 3],
    pub tangent_in: [f32; 3],
    pub tangent_out: [f32; 3],
    pub weight: f32,
}

impl Default for ControlPoint {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            tangent_in: [0.0; 3],
            tangent_out: [0.0; 3],
            weight: 1.0,
        }
    }
}

/// A curve with its control points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveControlExport {
    pub name: String,
    pub curve_type: CurveType,
    pub control_points: Vec<ControlPoint>,
    pub closed: bool,
}

/// Create a new curve export.
#[allow(dead_code)]
pub fn new_curve_export(name: &str, curve_type: CurveType) -> CurveControlExport {
    CurveControlExport {
        name: name.to_string(),
        curve_type,
        control_points: vec![],
        closed: false,
    }
}

/// Add a control point.
#[allow(dead_code)]
pub fn add_control_point(curve: &mut CurveControlExport, cp: ControlPoint) {
    curve.control_points.push(cp);
}

/// Compute the arc-length approximation of the control polygon.
#[allow(dead_code)]
pub fn control_polygon_length(curve: &CurveControlExport) -> f32 {
    let pts = &curve.control_points;
    if pts.len() < 2 {
        return 0.0;
    }
    pts.windows(2)
        .map(|w| {
            let a = w[0].position;
            let b = w[1].position;
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum()
}

/// Compute the AABB of the control points.
#[allow(dead_code)]
pub fn control_point_aabb(curve: &CurveControlExport) -> Option<([f32; 3], [f32; 3])> {
    let pts = &curve.control_points;
    if pts.is_empty() {
        return None;
    }
    let first = pts[0].position;
    let (mn, mx) = pts.iter().fold((first, first), |(mn, mx), cp| {
        let p = cp.position;
        (
            [mn[0].min(p[0]), mn[1].min(p[1]), mn[2].min(p[2])],
            [mx[0].max(p[0]), mx[1].max(p[1]), mx[2].max(p[2])],
        )
    });
    Some((mn, mx))
}

/// Serialise curve to flat f32 buffer: `[px,py,pz,tix,tiy,tiz,tox,toy,toz,w]` per point.
#[allow(dead_code)]
pub fn serialise_curve(curve: &CurveControlExport) -> Vec<f32> {
    curve
        .control_points
        .iter()
        .flat_map(|cp| {
            let p = cp.position;
            let ti = cp.tangent_in;
            let to_ = cp.tangent_out;
            [
                p[0], p[1], p[2], ti[0], ti[1], ti[2], to_[0], to_[1], to_[2], cp.weight,
            ]
        })
        .collect()
}

/// Reverse the control point order (useful for direction reversal).
#[allow(dead_code)]
pub fn reverse_curve(curve: &mut CurveControlExport) {
    curve.control_points.reverse();
    for cp in &mut curve.control_points {
        std::mem::swap(&mut cp.tangent_in, &mut cp.tangent_out);
    }
}

/// Check if the curve has at least `n` control points.
#[allow(dead_code)]
pub fn has_enough_points(curve: &CurveControlExport, n: usize) -> bool {
    curve.control_points.len() >= n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_curve() -> CurveControlExport {
        let mut c = new_curve_export("line", CurveType::Bezier);
        add_control_point(
            &mut c,
            ControlPoint {
                position: [0.0, 0.0, 0.0],
                ..Default::default()
            },
        );
        add_control_point(
            &mut c,
            ControlPoint {
                position: [1.0, 0.0, 0.0],
                ..Default::default()
            },
        );
        c
    }

    #[test]
    fn test_new_curve_empty() {
        let c = new_curve_export("x", CurveType::Nurbs);
        assert!(c.control_points.is_empty());
    }

    #[test]
    fn test_add_control_point() {
        let c = line_curve();
        assert_eq!(c.control_points.len(), 2);
    }

    #[test]
    fn test_polygon_length() {
        let c = line_curve();
        assert!((control_polygon_length(&c) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_some() {
        let c = line_curve();
        let (mn, mx) = control_point_aabb(&c).unwrap();
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_none_empty() {
        let c = new_curve_export("e", CurveType::Bezier);
        assert!(control_point_aabb(&c).is_none());
    }

    #[test]
    fn test_serialise_length() {
        let c = line_curve();
        assert_eq!(serialise_curve(&c).len(), 20); // 2 pts * 10 floats each
    }

    #[test]
    fn test_reverse_curve() {
        let mut c = line_curve();
        reverse_curve(&mut c);
        assert!((c.control_points[0].position[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_has_enough_points() {
        let c = line_curve();
        assert!(has_enough_points(&c, 2));
        assert!(!has_enough_points(&c, 3));
    }

    #[test]
    fn test_default_weight() {
        let cp = ControlPoint::default();
        assert!((cp.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_closed_flag() {
        let mut c = new_curve_export("c", CurveType::CatmullRom);
        c.closed = true;
        assert!(c.closed);
    }
}
