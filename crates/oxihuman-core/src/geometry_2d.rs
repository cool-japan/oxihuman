// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Test whether point (px, py) lies within circle centred at (cx, cy) with radius r.
pub fn point_in_circle(px: f32, py: f32, cx: f32, cy: f32, r: f32) -> bool {
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= r * r
}

/// Test whether point (px, py) lies within axis-aligned rectangle.
pub fn point_in_rect(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

fn cross_2d(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

/// Test whether line segment p1-p2 intersects line segment p3-p4.
pub fn segment_intersect(p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], p4: [f32; 2]) -> bool {
    let d1 = cross_2d(p3, p4, p1);
    let d2 = cross_2d(p3, p4, p2);
    let d3 = cross_2d(p1, p2, p3);
    let d4 = cross_2d(p1, p2, p4);
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    false
}

/// Signed area of a 2D triangle (absolute value gives unsigned area).
pub fn triangle_area_2d(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    let s = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
    s.abs() * 0.5
}

/// Test whether point p lies inside triangle (a, b, c) using sign of cross products.
pub fn point_in_triangle_2d(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let d1 = cross_2d(p, a, b);
    let d2 = cross_2d(p, b, c);
    let d3 = cross_2d(p, c, a);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

/// Perimeter of a polygon given as a list of 2D points.
pub fn polygon_perimeter_2d(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        total += (dx * dx + dy * dy).sqrt();
    }
    total
}

/// Area of a circle.
pub fn circle_area(r: f32) -> f32 {
    PI * r * r
}

/// Euclidean distance between two 2D points.
pub fn dist_2d(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx * dx + dy * dy).sqrt()
}

/// Centroid of a polygon.
pub fn polygon_centroid_2d(pts: &[[f32; 2]]) -> [f32; 2] {
    let n = pts.len() as f32;
    if n == 0.0 {
        return [0.0, 0.0];
    }
    let sx: f32 = pts.iter().map(|p| p[0]).sum();
    let sy: f32 = pts.iter().map(|p| p[1]).sum();
    [sx / n, sy / n]
}

/// Area of a polygon using the shoelace formula.
pub fn polygon_area_2d(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        area += a[0] * b[1] - b[0] * a[1];
    }
    area.abs() * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_circle() {
        /* inside */
        assert!(point_in_circle(0.0, 0.0, 0.0, 0.0, 1.0));
        /* outside */
        assert!(!point_in_circle(2.0, 0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_point_in_rect() {
        /* inside */
        assert!(point_in_rect(0.5, 0.5, 0.0, 0.0, 1.0, 1.0));
        /* outside */
        assert!(!point_in_rect(2.0, 0.5, 0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn test_segment_intersect_crossing() {
        /* two crossing segments */
        let yes = segment_intersect([0.0, 0.0], [1.0, 1.0], [0.0, 1.0], [1.0, 0.0]);
        assert!(yes);
    }

    #[test]
    fn test_segment_no_intersect_parallel() {
        /* parallel non-overlapping */
        let no = segment_intersect([0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]);
        assert!(!no);
    }

    #[test]
    fn test_triangle_area_2d() {
        /* right triangle with legs 3 and 4 -> area 6 */
        let area = triangle_area_2d([0.0, 0.0], [3.0, 0.0], [0.0, 4.0]);
        assert!((area - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_point_in_triangle_2d() {
        /* centre of unit triangle */
        let a = [0.0f32, 0.0];
        let b = [1.0, 0.0];
        let c = [0.0, 1.0];
        assert!(point_in_triangle_2d([0.1, 0.1], a, b, c));
        assert!(!point_in_triangle_2d([1.0, 1.0], a, b, c));
    }

    #[test]
    fn test_polygon_perimeter_2d() {
        /* unit square perimeter = 4 */
        let pts: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let p = polygon_perimeter_2d(&pts);
        assert!((p - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_circle_area() {
        /* pi*r^2 */
        let a = circle_area(1.0);
        assert!((a - std::f32::consts::PI).abs() < 1e-5);
    }

    #[test]
    fn test_dist_2d() {
        /* distance between (0,0) and (3,4) = 5 */
        let d = dist_2d([0.0, 0.0], [3.0, 4.0]);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_polygon_area_2d_square() {
        /* unit square area = 1 */
        let pts: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let a = polygon_area_2d(&pts);
        assert!((a - 1.0).abs() < 1e-5);
    }
}
