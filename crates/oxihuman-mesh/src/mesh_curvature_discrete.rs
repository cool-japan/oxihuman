// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Discrete curvature computation for triangle meshes.

use std::f32::consts::PI;

/// Per-vertex discrete curvature.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiscreteCurvature {
    pub gaussian: Vec<f32>,
    pub mean: Vec<f32>,
}

/// Compute the angle at vertex `v` in triangle (v, a, b).
#[allow(dead_code)]
pub fn vertex_angle(v: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let va = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
    let vb = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];
    let dot = va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2];
    let la = (va[0] * va[0] + va[1] * va[1] + va[2] * va[2]).sqrt();
    let lb = (vb[0] * vb[0] + vb[1] * vb[1] + vb[2] * vb[2]).sqrt();
    let denom = la * lb;
    if denom < 1e-12 {
        return 0.0;
    }
    (dot / denom).clamp(-1.0, 1.0).acos()
}

/// Compute discrete Gaussian curvature via angle defect.
#[allow(dead_code)]
pub fn gaussian_curvature(positions: &[[f32; 3]], indices: &[u32]) -> Vec<f32> {
    let n = positions.len();
    let mut angle_sums = vec![0.0f32; n];
    let tri_count = indices.len() / 3;

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        angle_sums[i0] += vertex_angle(positions[i0], positions[i1], positions[i2]);
        angle_sums[i1] += vertex_angle(positions[i1], positions[i0], positions[i2]);
        angle_sums[i2] += vertex_angle(positions[i2], positions[i0], positions[i1]);
    }

    let two_pi = 2.0 * PI;
    angle_sums.iter().map(|&s| two_pi - s).collect()
}

/// Compute mean curvature as half the Laplacian magnitude.
#[allow(dead_code)]
pub fn mean_curvature_simple(positions: &[[f32; 3]], indices: &[u32]) -> Vec<f32> {
    let n = positions.len();
    let mut neighbors: Vec<Vec<usize>> = vec![vec![]; n];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        neighbors[i0].push(i1);
        neighbors[i0].push(i2);
        neighbors[i1].push(i0);
        neighbors[i1].push(i2);
        neighbors[i2].push(i0);
        neighbors[i2].push(i1);
    }

    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        neighbors[i].sort_unstable();
        neighbors[i].dedup();
        if neighbors[i].is_empty() {
            result.push(0.0);
            continue;
        }
        let mut lap = [0.0f32; 3];
        let k = neighbors[i].len() as f32;
        for &j in &neighbors[i] {
            lap[0] += positions[j][0] - positions[i][0];
            lap[1] += positions[j][1] - positions[i][1];
            lap[2] += positions[j][2] - positions[i][2];
        }
        lap[0] /= k;
        lap[1] /= k;
        lap[2] /= k;
        result.push(0.5 * (lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2]).sqrt());
    }
    result
}

/// Compute both gaussian and mean curvature.
#[allow(dead_code)]
pub fn compute_discrete_curvature(positions: &[[f32; 3]], indices: &[u32]) -> DiscreteCurvature {
    DiscreteCurvature {
        gaussian: gaussian_curvature(positions, indices),
        mean: mean_curvature_simple(positions, indices),
    }
}

/// Average curvature value.
#[allow(dead_code)]
pub fn avg_curvature(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

/// Max absolute curvature.
#[allow(dead_code)]
pub fn max_abs_curvature(values: &[f32]) -> f32 {
    values.iter().map(|v| v.abs()).fold(0.0f32, f32::max)
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn curvature_to_json(dc: &DiscreteCurvature) -> String {
    format!(
        "{{\"vertex_count\":{},\"avg_gaussian\":{:.6},\"avg_mean\":{:.6}}}",
        dc.gaussian.len(),
        avg_curvature(&dc.gaussian),
        avg_curvature(&dc.mean)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn test_vertex_angle_right() {
        let angle = vertex_angle([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_gaussian_curvature_flat() {
        let (pos, idx) = flat_mesh();
        let gc = gaussian_curvature(&pos, &idx);
        assert_eq!(gc.len(), 4);
        // Interior vertex should have curvature near 0
    }

    #[test]
    fn test_mean_curvature_flat() {
        let (pos, idx) = flat_mesh();
        let mc = mean_curvature_simple(&pos, &idx);
        assert_eq!(mc.len(), 4);
    }

    #[test]
    fn test_compute_discrete() {
        let (pos, idx) = flat_mesh();
        let dc = compute_discrete_curvature(&pos, &idx);
        assert_eq!(dc.gaussian.len(), dc.mean.len());
    }

    #[test]
    fn test_avg_curvature() {
        assert!((avg_curvature(&[1.0, 3.0]) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_avg_curvature_empty() {
        assert!((avg_curvature(&[])).abs() < 1e-9);
    }

    #[test]
    fn test_max_abs() {
        assert!((max_abs_curvature(&[-5.0, 3.0]) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let dc = DiscreteCurvature {
            gaussian: vec![0.1],
            mean: vec![0.2],
        };
        let j = curvature_to_json(&dc);
        assert!(j.contains("\"vertex_count\":1"));
    }

    #[test]
    fn test_empty_mesh() {
        let dc = compute_discrete_curvature(&[], &[]);
        assert!(dc.gaussian.is_empty());
    }

    #[test]
    fn test_single_triangle() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let dc = compute_discrete_curvature(&pos, &idx);
        assert_eq!(dc.gaussian.len(), 3);
    }
}
