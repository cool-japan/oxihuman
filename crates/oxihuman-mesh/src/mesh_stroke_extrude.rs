// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A point on a 2D stroke with associated width.
pub struct StrokePoint {
    pub pos: [f32; 2],
    pub width: f32,
}

pub fn new_stroke_point(x: f32, y: f32, width: f32) -> StrokePoint {
    StrokePoint {
        pos: [x, y],
        width: width.max(0.0),
    }
}

pub fn stroke_extrude_quad(a: &StrokePoint, b: &StrokePoint) -> [[f32; 2]; 4] {
    let dx = b.pos[0] - a.pos[0];
    let dy = b.pos[1] - a.pos[1];
    let len = (dx * dx + dy * dy).sqrt().max(1e-9);
    // perpendicular (rotate 90 deg)
    let nx = -dy / len;
    let ny = dx / len;
    let ha = a.width * 0.5;
    let hb = b.width * 0.5;
    [
        [a.pos[0] - nx * ha, a.pos[1] - ny * ha],
        [a.pos[0] + nx * ha, a.pos[1] + ny * ha],
        [b.pos[0] + nx * hb, b.pos[1] + ny * hb],
        [b.pos[0] - nx * hb, b.pos[1] - ny * hb],
    ]
}

pub fn stroke_total_length(points: &[StrokePoint]) -> f32 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n - 1 {
        let dx = points[i + 1].pos[0] - points[i].pos[0];
        let dy = points[i + 1].pos[1] - points[i].pos[1];
        total += (dx * dx + dy * dy).sqrt();
    }
    total
}

pub fn stroke_area(points: &[StrokePoint]) -> f32 {
    let n = points.len();
    if n < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for i in 0..n - 1 {
        let quad = stroke_extrude_quad(&points[i], &points[i + 1]);
        // shoelace on quad
        let area = quad_area(&quad);
        total += area;
    }
    total
}

fn quad_area(q: &[[f32; 2]; 4]) -> f32 {
    // split into two triangles
    let t1 = triangle_area_2d(q[0], q[1], q[2]);
    let t2 = triangle_area_2d(q[0], q[2], q[3]);
    (t1 + t2).abs()
}

fn triangle_area_2d(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]))
}

pub fn stroke_vertex_count(point_count: usize) -> usize {
    if point_count < 2 {
        return 0;
    }
    point_count * 2
}

pub fn stroke_face_count(point_count: usize) -> usize {
    if point_count < 2 {
        return 0;
    }
    point_count - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stroke_point() {
        let p = new_stroke_point(1.0, 2.0, 0.5);
        assert_eq!(p.pos, [1.0, 2.0]);
        assert!((p.width - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_stroke_extrude_quad_symmetric() {
        /* horizontal stroke: left/right sides should mirror */
        let a = new_stroke_point(0.0, 0.0, 2.0);
        let b = new_stroke_point(4.0, 0.0, 2.0);
        let quad = stroke_extrude_quad(&a, &b);
        assert!((quad[0][1] + quad[1][1]).abs() < 1e-5);
    }

    #[test]
    fn test_stroke_total_length() {
        let pts = vec![
            new_stroke_point(0.0, 0.0, 1.0),
            new_stroke_point(3.0, 4.0, 1.0),
        ];
        assert!((stroke_total_length(&pts) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_stroke_area_positive() {
        let pts = vec![
            new_stroke_point(0.0, 0.0, 1.0),
            new_stroke_point(1.0, 0.0, 1.0),
            new_stroke_point(2.0, 0.0, 1.0),
        ];
        assert!(stroke_area(&pts) > 0.0);
    }

    #[test]
    fn test_stroke_vertex_count() {
        assert_eq!(stroke_vertex_count(5), 10);
        assert_eq!(stroke_vertex_count(1), 0);
    }

    #[test]
    fn test_stroke_face_count() {
        assert_eq!(stroke_face_count(5), 4);
        assert_eq!(stroke_face_count(0), 0);
    }

    #[test]
    fn test_stroke_width_clamped() {
        let p = new_stroke_point(0.0, 0.0, -1.0);
        assert!(p.width >= 0.0);
    }
}
