// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Error metric computation for mesh decimation (Quadric Error Metrics).

use std::f32::consts::SQRT_2;

/// Quadric error matrix (symmetric 4x4 stored as 10 floats).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct QuadricError {
    pub data: [f32; 10],
}

/// Decimation error result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DecimateErrorResult {
    pub errors: Vec<f32>,
    pub edge_pairs: Vec<(u32, u32)>,
}

/// Create a zero quadric.
#[allow(dead_code)]
pub fn zero_quadric() -> QuadricError {
    QuadricError { data: [0.0; 10] }
}

/// Create a quadric from a plane (nx, ny, nz, d).
#[allow(dead_code)]
pub fn quadric_from_plane(nx: f32, ny: f32, nz: f32, d: f32) -> QuadricError {
    QuadricError {
        data: [
            nx * nx,
            nx * ny,
            nx * nz,
            nx * d,
            ny * ny,
            ny * nz,
            ny * d,
            nz * nz,
            nz * d,
            d * d,
        ],
    }
}

/// Add two quadrics.
#[allow(dead_code)]
pub fn add_quadrics(a: &QuadricError, b: &QuadricError) -> QuadricError {
    let mut result = [0.0_f32; 10];
    #[allow(clippy::needless_range_loop)]
    for i in 0..10 {
        result[i] = a.data[i] + b.data[i];
    }
    QuadricError { data: result }
}

/// Evaluate quadric error at a point.
#[allow(dead_code)]
pub fn evaluate_quadric(q: &QuadricError, v: [f32; 3]) -> f32 {
    let d = &q.data;
    let x = v[0];
    let y = v[1];
    let z = v[2];
    d[0] * x * x
        + 2.0 * d[1] * x * y
        + 2.0 * d[2] * x * z
        + 2.0 * d[3] * x
        + d[4] * y * y
        + 2.0 * d[5] * y * z
        + 2.0 * d[6] * y
        + d[7] * z * z
        + 2.0 * d[8] * z
        + d[9]
}

/// Compute edge collapse error for all edges.
#[allow(dead_code)]
pub fn compute_edge_errors(positions: &[[f32; 3]], indices: &[u32]) -> DecimateErrorResult {
    let tri_count = indices.len() / 3;
    let mut vertex_quadrics: Vec<QuadricError> = vec![zero_quadric(); positions.len()];

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let v0 = positions[i0];
        let v1 = positions[i1];
        let v2 = positions[i2];
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len < 1e-12 {
            continue;
        }
        let (nx, ny, nz) = (nx / len, ny / len, nz / len);
        let d = -(nx * v0[0] + ny * v0[1] + nz * v0[2]);
        let q = quadric_from_plane(nx, ny, nz, d);
        for &vi in &[i0, i1, i2] {
            vertex_quadrics[vi] = add_quadrics(&vertex_quadrics[vi], &q);
        }
    }

    let mut edge_set = std::collections::HashSet::new();
    let mut errors = Vec::new();
    let mut edge_pairs = Vec::new();

    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for i in 0..3 {
            let (a, b) = if vs[i] < vs[(i + 1) % 3] {
                (vs[i], vs[(i + 1) % 3])
            } else {
                (vs[(i + 1) % 3], vs[i])
            };
            if edge_set.insert((a, b)) {
                let q = add_quadrics(&vertex_quadrics[a as usize], &vertex_quadrics[b as usize]);
                let mid = [
                    (positions[a as usize][0] + positions[b as usize][0]) * 0.5,
                    (positions[a as usize][1] + positions[b as usize][1]) * 0.5,
                    (positions[a as usize][2] + positions[b as usize][2]) * 0.5,
                ];
                errors.push(evaluate_quadric(&q, mid).max(0.0));
                edge_pairs.push((a, b));
            }
        }
    }

    DecimateErrorResult { errors, edge_pairs }
}

/// Edge count in result.
#[allow(dead_code)]
pub fn error_edge_count(r: &DecimateErrorResult) -> usize {
    r.errors.len()
}

/// Minimum error.
#[allow(dead_code)]
pub fn min_error(r: &DecimateErrorResult) -> f32 {
    r.errors.iter().cloned().fold(f32::MAX, f32::min)
}

/// Maximum error.
#[allow(dead_code)]
pub fn max_error(r: &DecimateErrorResult) -> f32 {
    r.errors.iter().cloned().fold(0.0_f32, f32::max)
}

/// SQRT_2 reference constant.
#[allow(dead_code)]
pub fn sqrt2_ref() -> f32 {
    SQRT_2
}

/// Export to JSON.
#[allow(dead_code)]
pub fn decimate_error_to_json(r: &DecimateErrorResult) -> String {
    format!(
        "{{\"edges\":{},\"min_error\":{:.6}}}",
        error_edge_count(r),
        min_error(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_quadric() {
        let q = zero_quadric();
        assert!((q.data[0]).abs() < 1e-6);
    }

    #[test]
    fn test_quadric_from_plane() {
        let q = quadric_from_plane(0.0, 0.0, 1.0, 0.0);
        assert!((q.data[7] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_quadrics() {
        let a = quadric_from_plane(1.0, 0.0, 0.0, 0.0);
        let b = quadric_from_plane(0.0, 1.0, 0.0, 0.0);
        let c = add_quadrics(&a, &b);
        assert!((c.data[0] - 1.0).abs() < 1e-6);
        assert!((c.data[4] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_quadric_on_plane() {
        let q = quadric_from_plane(0.0, 0.0, 1.0, 0.0);
        assert!((evaluate_quadric(&q, [1.0, 2.0, 0.0])).abs() < 1e-6);
    }

    #[test]
    fn test_compute_edge_errors() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let r = compute_edge_errors(&pos, &idx);
        assert_eq!(error_edge_count(&r), 3);
    }

    #[test]
    fn test_min_max_error() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = compute_edge_errors(&pos, &[0, 1, 2]);
        assert!(min_error(&r) <= max_error(&r));
    }

    #[test]
    fn test_empty() {
        let r = compute_edge_errors(&[], &[]);
        assert_eq!(error_edge_count(&r), 0);
    }

    #[test]
    fn test_sqrt2() {
        assert!((sqrt2_ref() - SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let r = DecimateErrorResult {
            errors: vec![0.1],
            edge_pairs: vec![(0, 1)],
        };
        assert!(decimate_error_to_json(&r).contains("\"edges\":1"));
    }

    #[test]
    fn test_evaluate_off_plane() {
        let q = quadric_from_plane(0.0, 0.0, 1.0, 0.0);
        let err = evaluate_quadric(&q, [0.0, 0.0, 1.0]);
        assert!((err - 1.0).abs() < 1e-5);
    }
}
