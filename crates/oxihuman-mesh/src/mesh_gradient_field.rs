// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Compute gradient vector fields on triangle mesh surfaces from scalar functions.

use std::f32::consts::PI;

/// Per-face gradient vector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FaceGradient {
    pub face_index: usize,
    pub gradient: [f32; 3],
}

/// Compute gradient of a scalar field on a single triangle.
#[allow(dead_code)]
pub fn triangle_gradient(
    p0: [f32; 3], p1: [f32; 3], p2: [f32; 3],
    s0: f32, s1: f32, s2: f32,
) -> [f32; 3] {
    let e1 = sub3(p1, p0);
    let e2 = sub3(p2, p0);
    let n = cross3(e1, e2);
    let area2 = dot3(n, n);
    if area2 < 1e-12 { return [0.0; 3]; }
    // Gradient = (1/2A) * sum(si * (N x ei))
    let n_unit = {
        let len = area2.sqrt();
        [n[0] / len, n[1] / len, n[2] / len]
    };
    let e01 = sub3(p1, p0);
    let e12 = sub3(p2, p1);
    let e20 = sub3(p0, p2);
    let g0 = cross3(n_unit, e12);
    let g1 = cross3(n_unit, e20);
    let g2 = cross3(n_unit, e01);
    let inv_2a = 1.0 / area2.sqrt();
    [
        (s0 * g0[0] + s1 * g1[0] + s2 * g2[0]) * inv_2a,
        (s0 * g0[1] + s1 * g1[1] + s2 * g2[1]) * inv_2a,
        (s0 * g0[2] + s1 * g1[2] + s2 * g2[2]) * inv_2a,
    ]
}

/// Compute gradient field for a per-vertex scalar function over the mesh.
#[allow(dead_code)]
pub fn compute_gradient_field(
    positions: &[[f32; 3]],
    indices: &[u32],
    scalars: &[f32],
) -> Vec<FaceGradient> {
    let tc = indices.len() / 3;
    (0..tc).map(|t| {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let g = triangle_gradient(
            positions[i0], positions[i1], positions[i2],
            scalars[i0], scalars[i1], scalars[i2],
        );
        FaceGradient { face_index: t, gradient: g }
    }).collect()
}

/// Magnitude of a gradient vector.
#[allow(dead_code)]
pub fn gradient_magnitude(g: [f32; 3]) -> f32 {
    (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt()
}

/// Average gradient magnitude across all faces.
#[allow(dead_code)]
pub fn average_gradient_magnitude(field: &[FaceGradient]) -> f32 {
    if field.is_empty() { return 0.0; }
    let sum: f32 = field.iter().map(|fg| gradient_magnitude(fg.gradient)).sum();
    sum / field.len() as f32
}

/// Max gradient magnitude.
#[allow(dead_code)]
pub fn max_gradient_magnitude(field: &[FaceGradient]) -> f32 {
    field.iter().map(|fg| gradient_magnitude(fg.gradient))
        .fold(0.0f32, f32::max)
}

/// Divergence of the gradient field (sum of outgoing gradient flux per vertex).
#[allow(dead_code)]
pub fn gradient_divergence(
    positions: &[[f32; 3]],
    indices: &[u32],
    field: &[FaceGradient],
) -> Vec<f32> {
    let _ = PI; // reference constant
    let n = positions.len();
    let mut div = vec![0.0f32; n];
    let tc = indices.len() / 3;
    for t in 0..tc {
        if t >= field.len() { break; }
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let g = field[t].gradient;
        let mag = gradient_magnitude(g) / 3.0;
        div[i0] += mag;
        div[i1] += mag;
        div[i2] += mag;
    }
    div
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_field_zero_grad() {
        let g = triangle_gradient(
            [0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            5.0, 5.0, 5.0,
        );
        assert!(gradient_magnitude(g) < 1e-5);
    }

    #[test]
    fn test_linear_field_nonzero() {
        let g = triangle_gradient(
            [0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
            0.0, 1.0, 0.0,
        );
        assert!(gradient_magnitude(g) > 0.1);
    }

    #[test]
    fn test_compute_field_count() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let scalars = vec![0.0, 1.0, 2.0];
        let field = compute_gradient_field(&pos, &[0, 1, 2], &scalars);
        assert_eq!(field.len(), 1);
    }

    #[test]
    fn test_avg_magnitude() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let scalars = vec![0.0, 1.0, 0.0];
        let field = compute_gradient_field(&pos, &[0, 1, 2], &scalars);
        assert!(average_gradient_magnitude(&field) > 0.0);
    }

    #[test]
    fn test_max_magnitude() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let scalars = vec![0.0, 1.0, 0.0];
        let field = compute_gradient_field(&pos, &[0, 1, 2], &scalars);
        assert!(max_gradient_magnitude(&field) > 0.0);
    }

    #[test]
    fn test_divergence_count() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let scalars = vec![0.0, 1.0, 0.0];
        let field = compute_gradient_field(&pos, &[0, 1, 2], &scalars);
        let div = gradient_divergence(&pos, &[0, 1, 2], &field);
        assert_eq!(div.len(), 3);
    }

    #[test]
    fn test_degenerate_triangle_gradient() {
        let g = triangle_gradient([0.0; 3], [0.0; 3], [0.0; 3], 0.0, 1.0, 2.0);
        assert!(gradient_magnitude(g) < 1e-5);
    }

    #[test]
    fn test_gradient_magnitude_fn() {
        assert!((gradient_magnitude([3.0, 4.0, 0.0]) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_empty_field() {
        let avg = average_gradient_magnitude(&[]);
        assert!((avg - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_empty() {
        let max = max_gradient_magnitude(&[]);
        assert!((max - 0.0).abs() < 1e-5);
    }

}
