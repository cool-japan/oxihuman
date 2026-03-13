// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear-clipping polygon triangulation.

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EcPoint {
    pub x: f32,
    pub y: f32,
}

impl EcPoint {
    pub fn new(x: f32, y: f32) -> Self {
        EcPoint { x, y }
    }
}

fn cross_2d(o: &EcPoint, a: &EcPoint, b: &EcPoint) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

fn point_in_triangle(p: &EcPoint, a: &EcPoint, b: &EcPoint, c: &EcPoint) -> bool {
    let d1 = cross_2d(a, b, p);
    let d2 = cross_2d(b, c, p);
    let d3 = cross_2d(c, a, p);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

fn polygon_area_2(pts: &[EcPoint]) -> f32 {
    let n = pts.len();
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i].x * pts[j].y;
        area -= pts[j].x * pts[i].y;
    }
    area
}

fn is_ear(idx: usize, poly: &[usize], pts: &[EcPoint]) -> bool {
    let n = poly.len();
    let prev = poly[(idx + n - 1) % n];
    let curr = poly[idx];
    let next = poly[(idx + 1) % n];
    let a = &pts[prev];
    let b = &pts[curr];
    let c = &pts[next];
    /* Must be convex (CCW polygon => positive cross product) */
    if cross_2d(a, b, c) <= 0.0 {
        return false;
    }
    /* No other vertex inside triangle */
    for (i, &vi) in poly.iter().enumerate() {
        if i == (idx + n - 1) % n || i == idx || i == (idx + 1) % n {
            continue;
        }
        if point_in_triangle(&pts[vi], a, b, c) {
            return false;
        }
    }
    true
}

/// Triangulate a simple polygon using ear clipping.
/// Returns a list of triangle index triples (into the input vertex array).
pub fn ear_clip(vertices: &[EcPoint]) -> Vec<[usize; 3]> {
    let n = vertices.len();
    if n < 3 {
        return vec![];
    }
    if n == 3 {
        return vec![[0, 1, 2]];
    }

    let mut poly: Vec<usize> = (0..n).collect();
    /* Ensure CCW orientation */
    if polygon_area_2(vertices) < 0.0 {
        poly.reverse();
    }

    let mut triangles = Vec::with_capacity(n - 2);
    let mut attempts = 0;
    let mut idx = 0;
    while poly.len() > 3 && attempts < poly.len() * poly.len() + 1 {
        let m = poly.len();
        if is_ear(idx % m, &poly, vertices) {
            let im = idx % m;
            let prev = poly[(im + m - 1) % m];
            let curr = poly[im];
            let next = poly[(im + 1) % m];
            triangles.push([prev, curr, next]);
            poly.remove(im);
            attempts = 0;
            if im > 0 {
                idx = im - 1;
            }
        } else {
            idx += 1;
            attempts += 1;
        }
    }
    if poly.len() == 3 {
        triangles.push([poly[0], poly[1], poly[2]]);
    }
    triangles
}

/// Triangulate and return flat index array [i0, i1, i2, i0, i1, i2, ...].
pub fn ear_clip_flat(vertices: &[EcPoint]) -> Vec<usize> {
    ear_clip(vertices)
        .into_iter()
        .flat_map(|t| t.into_iter())
        .collect()
}

/// Return the signed area of the polygon (positive = CCW).
pub fn polygon_signed_area(vertices: &[EcPoint]) -> f32 {
    polygon_area_2(vertices) * 0.5
}

/// Check if a polygon is convex (all cross products same sign).
pub fn is_convex(vertices: &[EcPoint]) -> bool {
    let n = vertices.len();
    if n < 3 {
        return false;
    }
    let mut sign = 0i32;
    for i in 0..n {
        let a = &vertices[i];
        let b = &vertices[(i + 1) % n];
        let c = &vertices[(i + 2) % n];
        let cp = cross_2d(a, b, c);
        if cp > 1e-9 {
            if sign == -1 {
                return false;
            }
            sign = 1;
        } else if cp < -1e-9 {
            if sign == 1 {
                return false;
            }
            sign = -1;
        }
    }
    true
}

/// Compute bounding box of a polygon.
pub fn polygon_bbox(vertices: &[EcPoint]) -> Option<([f32; 2], [f32; 2])> {
    if vertices.is_empty() {
        return None;
    }
    let min_x = vertices.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = vertices
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = vertices.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = vertices
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    Some(([min_x, min_y], [max_x, max_y]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_passthrough() {
        let pts = vec![
            EcPoint::new(0.0, 0.0),
            EcPoint::new(1.0, 0.0),
            EcPoint::new(0.5, 1.0),
        ];
        let tris = ear_clip(&pts);
        assert_eq!(tris.len(), 1);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn test_quad_triangulation() {
        let pts = vec![
            EcPoint::new(0.0, 0.0),
            EcPoint::new(1.0, 0.0),
            EcPoint::new(1.0, 1.0),
            EcPoint::new(0.0, 1.0),
        ];
        let tris = ear_clip(&pts);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn test_pentagon() {
        let pts: Vec<EcPoint> = (0..5)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 5.0;
                EcPoint::new(angle.cos(), angle.sin())
            })
            .collect();
        let tris = ear_clip(&pts);
        assert_eq!(tris.len(), 3);
    }

    #[test]
    fn test_empty_and_degenerate() {
        assert!(ear_clip(&[]).is_empty());
        let pts = vec![EcPoint::new(0.0, 0.0), EcPoint::new(1.0, 0.0)];
        assert!(ear_clip(&pts).is_empty());
    }

    #[test]
    fn test_signed_area_positive_ccw() {
        let pts = vec![
            EcPoint::new(0.0, 0.0),
            EcPoint::new(1.0, 0.0),
            EcPoint::new(0.0, 1.0),
        ];
        assert!(polygon_signed_area(&pts) > 0.0);
    }

    #[test]
    fn test_is_convex_square() {
        let pts = vec![
            EcPoint::new(0.0, 0.0),
            EcPoint::new(1.0, 0.0),
            EcPoint::new(1.0, 1.0),
            EcPoint::new(0.0, 1.0),
        ];
        assert!(is_convex(&pts));
    }

    #[test]
    fn test_polygon_bbox() {
        let pts = vec![EcPoint::new(-1.0, -2.0), EcPoint::new(3.0, 4.0)];
        let (mn, mx) = polygon_bbox(&pts).expect("should succeed");
        assert!((mn[0] - (-1.0)).abs() < 1e-5);
        assert!((mx[1] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_flat_output() {
        let pts = vec![
            EcPoint::new(0.0, 0.0),
            EcPoint::new(1.0, 0.0),
            EcPoint::new(0.5, 1.0),
        ];
        let flat = ear_clip_flat(&pts);
        assert_eq!(flat.len(), 3);
    }
}
