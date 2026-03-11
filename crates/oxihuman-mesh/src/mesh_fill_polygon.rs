// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// Result of polygon fill (triangulation of a polygon).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FillPolygonResult {
    pub indices: Vec<u32>,
    pub triangle_count: usize,
}

/// Fan triangulate a convex polygon given as ordered vertex indices.
#[allow(dead_code)]
pub fn fill_polygon_fan(polygon: &[u32]) -> FillPolygonResult {
    if polygon.len() < 3 {
        return FillPolygonResult { indices: Vec::new(), triangle_count: 0 };
    }
    let mut indices = Vec::new();
    let pivot = polygon[0];
    for i in 1..polygon.len() - 1 {
        indices.push(pivot);
        indices.push(polygon[i]);
        indices.push(polygon[i + 1]);
    }
    let triangle_count = indices.len() / 3;
    FillPolygonResult { indices, triangle_count }
}

/// Ear-clipping triangulation for a simple 2D polygon.
#[allow(dead_code)]
pub fn fill_polygon_ear_clip(vertices_2d: &[[f32; 2]], polygon: &[u32]) -> FillPolygonResult {
    if polygon.len() < 3 {
        return FillPolygonResult { indices: Vec::new(), triangle_count: 0 };
    }
    let mut remaining: Vec<u32> = polygon.to_vec();
    let mut indices = Vec::new();
    let mut safety = remaining.len() * remaining.len();
    while remaining.len() > 2 && safety > 0 {
        safety -= 1;
        let n = remaining.len();
        let mut found = false;
        for i in 0..n {
            let prev = remaining[(i + n - 1) % n];
            let curr = remaining[i];
            let next = remaining[(i + 1) % n];
            let a = &vertices_2d[prev as usize];
            let b = &vertices_2d[curr as usize];
            let c = &vertices_2d[next as usize];
            let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            if cross > 0.0 {
                indices.push(prev);
                indices.push(curr);
                indices.push(next);
                remaining.remove(i);
                found = true;
                break;
            }
        }
        if !found {
            break;
        }
    }
    let triangle_count = indices.len() / 3;
    FillPolygonResult { indices, triangle_count }
}

/// Check if a polygon is convex (2D).
#[allow(dead_code)]
pub fn is_convex_2d(vertices_2d: &[[f32; 2]], polygon: &[u32]) -> bool {
    let n = polygon.len();
    if n < 3 { return false; }
    let mut sign = 0;
    for i in 0..n {
        let a = &vertices_2d[polygon[i] as usize];
        let b = &vertices_2d[polygon[(i + 1) % n] as usize];
        let c = &vertices_2d[polygon[(i + 2) % n] as usize];
        let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if cross > 1e-8 {
            if sign < 0 { return false; }
            sign = 1;
        } else if cross < -1e-8 {
            if sign > 0 { return false; }
            sign = -1;
        }
    }
    true
}

/// Compute polygon area (2D, signed).
#[allow(dead_code)]
pub fn polygon_area_2d(vertices_2d: &[[f32; 2]], polygon: &[u32]) -> f32 {
    let n = polygon.len();
    let mut area = 0.0f32;
    for i in 0..n {
        let a = &vertices_2d[polygon[i] as usize];
        let b = &vertices_2d[polygon[(i + 1) % n] as usize];
        area += a[0] * b[1] - b[0] * a[1];
    }
    area * 0.5
}

/// Serialize fill result to JSON.
#[allow(dead_code)]
pub fn fill_to_json(result: &FillPolygonResult) -> String {
    format!("{{\"triangles\":{}}}", result.triangle_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_triangle() {
        let r = fill_polygon_fan(&[0, 1, 2]);
        assert_eq!(r.triangle_count, 1);
    }

    #[test]
    fn test_fan_quad() {
        let r = fill_polygon_fan(&[0, 1, 2, 3]);
        assert_eq!(r.triangle_count, 2);
    }

    #[test]
    fn test_fan_pentagon() {
        let r = fill_polygon_fan(&[0, 1, 2, 3, 4]);
        assert_eq!(r.triangle_count, 3);
    }

    #[test]
    fn test_fan_degenerate() {
        let r = fill_polygon_fan(&[0, 1]);
        assert_eq!(r.triangle_count, 0);
    }

    #[test]
    fn test_ear_clip_triangle() {
        let verts = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let r = fill_polygon_ear_clip(&verts, &[0, 1, 2]);
        assert_eq!(r.triangle_count, 1);
    }

    #[test]
    fn test_is_convex() {
        let verts = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(is_convex_2d(&verts, &[0, 1, 2, 3]));
    }

    #[test]
    fn test_polygon_area() {
        let verts = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let area = polygon_area_2d(&verts, &[0, 1, 2, 3]);
        assert!((area - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_polygon() {
        let r = fill_polygon_fan(&[]);
        assert_eq!(r.triangle_count, 0);
    }

    #[test]
    fn test_fill_to_json() {
        let r = fill_polygon_fan(&[0, 1, 2, 3]);
        let json = fill_to_json(&r);
        assert!(json.contains("triangles"));
    }

    #[test]
    fn test_polygon_area_triangle() {
        let verts = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let area = polygon_area_2d(&verts, &[0, 1, 2]);
        assert!((area - 0.5).abs() < 1e-5);
    }
}
