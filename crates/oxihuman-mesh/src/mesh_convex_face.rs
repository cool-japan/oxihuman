// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Convexity testing for mesh faces (polygons).

/// Result of face convexity analysis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConvexFaceResult {
    /// True for each face if convex.
    pub is_convex: Vec<bool>,
    /// Number of convex faces.
    pub convex_count: usize,
    /// Number of concave faces.
    pub concave_count: usize,
}

/// Cross product of two 3D vectors.
#[allow(dead_code)]
pub fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Dot product.
#[allow(dead_code)]
pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Test if a triangle is convex (always true for triangles).
#[allow(dead_code)]
pub fn is_triangle_convex() -> bool {
    true
}

/// Test if a quad (4 vertices) is convex by checking cross product signs.
#[allow(dead_code)]
pub fn is_quad_convex(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> bool {
    let edges = [
        [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]],
        [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]],
        [v3[0] - v2[0], v3[1] - v2[1], v3[2] - v2[2]],
        [v0[0] - v3[0], v0[1] - v3[1], v0[2] - v3[2]],
    ];
    let normal = cross3(edges[0], edges[1]);
    for i in 1..4 {
        let c = cross3(edges[i], edges[(i + 1) % 4]);
        if dot3(normal, c) < 0.0 {
            return false;
        }
    }
    true
}

/// Analyze triangle faces for convexity (all triangles are convex).
#[allow(dead_code)]
pub fn analyze_triangle_convexity(indices: &[u32]) -> ConvexFaceResult {
    let tri_count = indices.len() / 3;
    ConvexFaceResult {
        is_convex: vec![true; tri_count],
        convex_count: tri_count,
        concave_count: 0,
    }
}

/// Compute face normal from triangle vertices.
#[allow(dead_code)]
pub fn face_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    cross3(e1, e2)
}

/// Triangle area from face normal length.
#[allow(dead_code)]
pub fn face_area(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let n = face_normal(v0, v1, v2);
    0.5 * (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
}

/// Convex face count.
#[allow(dead_code)]
pub fn convex_count(result: &ConvexFaceResult) -> usize {
    result.convex_count
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn convex_face_to_json(result: &ConvexFaceResult) -> String {
    format!(
        "{{\"convex\":{},\"concave\":{}}}",
        result.convex_count, result.concave_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross3() {
        let c = cross3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot3() {
        assert!((dot3([1.0, 0.0, 0.0], [0.0, 1.0, 0.0])).abs() < 1e-9);
    }

    #[test]
    fn test_is_triangle_convex() {
        assert!(is_triangle_convex());
    }

    #[test]
    fn test_quad_convex() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [1.0, 1.0, 0.0];
        let v3 = [0.0, 1.0, 0.0];
        assert!(is_quad_convex(v0, v1, v2, v3));
    }

    #[test]
    fn test_quad_concave() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.3, 0.3, 0.0]; // concave
        let v3 = [0.0, 1.0, 0.0];
        assert!(!is_quad_convex(v0, v1, v2, v3));
    }

    #[test]
    fn test_analyze_convexity() {
        let indices = vec![0, 1, 2, 3, 4, 5];
        let result = analyze_triangle_convexity(&indices);
        assert_eq!(result.convex_count, 2);
        assert_eq!(result.concave_count, 0);
    }

    #[test]
    fn test_face_normal() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2] > 0.0);
    }

    #[test]
    fn test_face_area() {
        let a = face_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_convex_count() {
        let r = ConvexFaceResult {
            is_convex: vec![true, true],
            convex_count: 2,
            concave_count: 0,
        };
        assert_eq!(convex_count(&r), 2);
    }

    #[test]
    fn test_to_json() {
        let r = ConvexFaceResult {
            is_convex: vec![true],
            convex_count: 1,
            concave_count: 0,
        };
        let j = convex_face_to_json(&r);
        assert!(j.contains("\"convex\":1"));
    }
}
