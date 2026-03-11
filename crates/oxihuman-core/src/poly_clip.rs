// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Polygon clipping using the Sutherland-Hodgman algorithm.

#![allow(dead_code)]

/// A 2D polygon represented as a list of vertices.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Polygon2D {
    pub vertices: Vec<[f32; 2]>,
}

/// Create a polygon from a slice of vertices.
#[allow(dead_code)]
pub fn new_polygon(vertices: &[[f32; 2]]) -> Polygon2D {
    Polygon2D {
        vertices: vertices.to_vec(),
    }
}

/// Signed area of a polygon (positive = CCW, negative = CW).
#[allow(dead_code)]
pub fn polygon_signed_area(poly: &Polygon2D) -> f32 {
    let n = poly.vertices.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += poly.vertices[i][0] * poly.vertices[j][1];
        area -= poly.vertices[j][0] * poly.vertices[i][1];
    }
    area * 0.5
}

/// Area (absolute value of signed area).
#[allow(dead_code)]
pub fn polygon_area(poly: &Polygon2D) -> f32 {
    polygon_signed_area(poly).abs()
}

/// Clip a polygon against a single half-plane edge (from `e0` to `e1`).
/// Points to the left of (or on) the directed edge are inside.
fn clip_against_edge(polygon: &[[f32; 2]], e0: [f32; 2], e1: [f32; 2]) -> Vec<[f32; 2]> {
    let n = polygon.len();
    if n == 0 {
        return Vec::new();
    }
    let mut output = Vec::new();
    let inside = |p: [f32; 2]| -> bool {
        let dx = e1[0] - e0[0];
        let dy = e1[1] - e0[1];
        let px = p[0] - e0[0];
        let py = p[1] - e0[1];
        dx * py - dy * px >= 0.0
    };
    let intersect = |a: [f32; 2], b: [f32; 2]| -> [f32; 2] {
        let dx = e1[0] - e0[0];
        let dy = e1[1] - e0[1];
        let denom = dx * (b[1] - a[1]) - dy * (b[0] - a[0]);
        if denom.abs() < 1e-10 {
            return a;
        }
        let t = (dx * (a[1] - e0[1]) - dy * (a[0] - e0[0])) / denom;
        [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
    };
    for i in 0..n {
        let curr = polygon[i];
        let prev = polygon[(i + n - 1) % n];
        let curr_in = inside(curr);
        let prev_in = inside(prev);
        if curr_in {
            if !prev_in {
                output.push(intersect(prev, curr));
            }
            output.push(curr);
        } else if prev_in {
            output.push(intersect(prev, curr));
        }
    }
    output
}

/// Clip `subject` polygon against convex `clip` polygon using Sutherland-Hodgman.
/// Returns the clipped polygon.
#[allow(dead_code)]
pub fn sutherland_hodgman(subject: &Polygon2D, clip: &Polygon2D) -> Polygon2D {
    let cn = clip.vertices.len();
    if cn < 3 || subject.vertices.is_empty() {
        return Polygon2D::default();
    }
    let mut output = subject.vertices.clone();
    for i in 0..cn {
        if output.is_empty() {
            break;
        }
        let e0 = clip.vertices[i];
        let e1 = clip.vertices[(i + 1) % cn];
        output = clip_against_edge(&output, e0, e1);
    }
    Polygon2D { vertices: output }
}

/// Check if a point is inside a convex polygon using cross products.
#[allow(dead_code)]
pub fn point_in_polygon(poly: &Polygon2D, p: [f32; 2]) -> bool {
    let n = poly.vertices.len();
    if n < 3 {
        return false;
    }
    for i in 0..n {
        let a = poly.vertices[i];
        let b = poly.vertices[(i + 1) % n];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let px = p[0] - a[0];
        let py = p[1] - a[1];
        if dx * py - dy * px < 0.0 {
            return false;
        }
    }
    true
}

/// Number of vertices in the polygon.
#[allow(dead_code)]
pub fn polygon_vertex_count(poly: &Polygon2D) -> usize {
    poly.vertices.len()
}

/// Check if the polygon is empty (fewer than 3 vertices).
#[allow(dead_code)]
pub fn polygon_is_empty(poly: &Polygon2D) -> bool {
    poly.vertices.len() < 3
}

/// Compute centroid of the polygon.
#[allow(dead_code)]
pub fn polygon_centroid(poly: &Polygon2D) -> Option<[f32; 2]> {
    let n = poly.vertices.len();
    if n == 0 {
        return None;
    }
    let sum = poly
        .vertices
        .iter()
        .fold([0.0f32; 2], |acc, v| [acc[0] + v[0], acc[1] + v[1]]);
    Some([sum[0] / n as f32, sum[1] / n as f32])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square() -> Polygon2D {
        new_polygon(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
    }

    #[test]
    fn test_polygon_area() {
        let sq = unit_square();
        assert!((polygon_area(&sq) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_clip_identical() {
        let sq = unit_square();
        let result = sutherland_hodgman(&sq, &sq);
        assert!((polygon_area(&result) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_clip_overlapping() {
        let subj = unit_square();
        let clip = new_polygon(&[[0.5, 0.0], [1.5, 0.0], [1.5, 1.0], [0.5, 1.0]]);
        let result = sutherland_hodgman(&subj, &clip);
        let area = polygon_area(&result);
        assert!((area - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_clip_no_overlap() {
        let subj = unit_square();
        let clip = new_polygon(&[[2.0, 0.0], [3.0, 0.0], [3.0, 1.0], [2.0, 1.0]]);
        let result = sutherland_hodgman(&subj, &clip);
        assert!(polygon_is_empty(&result));
    }

    #[test]
    fn test_point_in_polygon() {
        let sq = unit_square();
        assert!(point_in_polygon(&sq, [0.5, 0.5]));
        assert!(!point_in_polygon(&sq, [1.5, 0.5]));
    }

    #[test]
    fn test_centroid() {
        let sq = unit_square();
        let c = polygon_centroid(&sq).unwrap();
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_vertex_count() {
        let sq = unit_square();
        assert_eq!(polygon_vertex_count(&sq), 4);
    }

    #[test]
    fn test_signed_area_ccw() {
        let sq = unit_square();
        assert!(polygon_signed_area(&sq) > 0.0);
    }

    #[test]
    fn test_empty_polygon() {
        let poly = new_polygon(&[[0.0, 0.0]]);
        assert!(polygon_is_empty(&poly));
    }

    #[test]
    fn test_clip_empty_subject() {
        let empty = new_polygon(&[]);
        let clip = unit_square();
        let result = sutherland_hodgman(&empty, &clip);
        assert!(polygon_is_empty(&result));
    }
}
