// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Catmull-Clark subdivision weight computation utilities.

use std::f32::consts::PI;

/// Catmull-Clark weight for a vertex with n adjacent edges.
#[allow(dead_code)]
pub fn vertex_weight(valence: usize) -> f32 {
    if valence < 3 {
        return 0.0;
    }
    let n = valence as f32;
    (n - 2.0) / n
}

/// Face point weight (average of face vertices).
#[allow(dead_code)]
pub fn face_point_weight() -> f32 {
    0.25
}

/// Edge point weight for a boundary edge.
#[allow(dead_code)]
pub fn boundary_edge_weight() -> f32 {
    0.5
}

/// Compute the Warren weight for extraordinary vertices.
#[allow(dead_code)]
pub fn warren_weight(valence: usize) -> f32 {
    if valence < 3 {
        return 0.0;
    }
    let n = valence as f32;
    let alpha = 3.0 / (8.0 * n);
    let beta = (3.0 / 8.0 + 0.25 * (2.0 * PI / n).cos()).powi(2);
    alpha + beta / n
}

/// Compute Loop subdivision beta weight.
#[allow(dead_code)]
pub fn loop_beta(valence: usize) -> f32 {
    if valence < 3 {
        return 0.0;
    }
    let n = valence as f32;
    let t = 3.0 / 8.0 + 0.25 * (2.0 * PI / n).cos();
    (1.0 / n) * (5.0 / 8.0 - t * t)
}

/// Compute the centroid of a set of points.
#[allow(dead_code)]
pub fn centroid(points: &[[f32; 3]]) -> [f32; 3] {
    if points.is_empty() {
        return [0.0; 3];
    }
    let n = points.len() as f32;
    let mut sum = [0.0f32; 3];
    for p in points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Linear interpolation of two points.
#[allow(dead_code)]
pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Check if a valence is regular (4 for quads, 6 for triangles).
#[allow(dead_code)]
pub fn is_regular_valence(valence: usize, quad_mesh: bool) -> bool {
    if quad_mesh {
        valence == 4
    } else {
        valence == 6
    }
}

/// Compute subdivision smoothing factor.
#[allow(dead_code)]
pub fn smoothing_factor(valence: usize) -> f32 {
    if valence < 3 {
        return 0.0;
    }
    let n = valence as f32;
    1.0 / (n + 5.0 / (4.0 - 7.0 / (4.0 * n).max(1.0)))
}

/// Convert weights info to JSON.
#[allow(dead_code)]
pub fn weights_to_json(valence: usize) -> String {
    format!(
        "{{\"valence\":{},\"vertex_weight\":{:.6},\"warren\":{:.6},\"loop_beta\":{:.6}}}",
        valence,
        vertex_weight(valence),
        warren_weight(valence),
        loop_beta(valence)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_weight_regular() {
        let w = vertex_weight(4);
        assert!((w - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_vertex_weight_low() {
        assert!((vertex_weight(2)).abs() < 1e-9);
    }

    #[test]
    fn test_face_point_weight() {
        assert!((face_point_weight() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_boundary_edge_weight() {
        assert!((boundary_edge_weight() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_warren_weight() {
        let w = warren_weight(4);
        assert!(w > 0.0);
        assert!(w < 1.0);
    }

    #[test]
    fn test_loop_beta() {
        let b = loop_beta(6);
        assert!(b > 0.0);
    }

    #[test]
    fn test_centroid() {
        let c = centroid(&[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_centroid_empty() {
        let c = centroid(&[]);
        assert!((c[0]).abs() < 1e-9);
    }

    #[test]
    fn test_lerp3() {
        let r = lerp3([0.0, 0.0, 0.0], [2.0, 4.0, 6.0], 0.5);
        assert!((r[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_regular_valence() {
        assert!(is_regular_valence(4, true));
        assert!(!is_regular_valence(5, true));
        assert!(is_regular_valence(6, false));
    }

    #[test]
    fn test_to_json() {
        let j = weights_to_json(4);
        assert!(j.contains("\"valence\":4"));
    }
}
