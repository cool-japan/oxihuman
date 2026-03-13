// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn smooth_vertex_laplacian(center: [f32; 3], neighbors: &[[f32; 3]], factor: f32) -> [f32; 3] {
    if neighbors.is_empty() {
        return center;
    }
    let n = neighbors.len() as f32;
    let avg = [
        neighbors.iter().map(|v| v[0]).sum::<f32>() / n,
        neighbors.iter().map(|v| v[1]).sum::<f32>() / n,
        neighbors.iter().map(|v| v[2]).sum::<f32>() / n,
    ];
    let f = factor.clamp(0.0, 1.0);
    [
        center[0] * (1.0 - f) + avg[0] * f,
        center[1] * (1.0 - f) + avg[1] * f,
        center[2] * (1.0 - f) + avg[2] * f,
    ]
}

pub fn smooth_mesh_pass(positions: &mut [[f32; 3]], adjacency: &[Vec<usize>], factor: f32) {
    let old = positions.to_owned();
    for (i, adj) in adjacency.iter().enumerate() {
        if i >= positions.len() {
            break;
        }
        let neighbors: Vec<[f32; 3]> = adj.iter().filter_map(|&j| old.get(j).copied()).collect();
        positions[i] = smooth_vertex_laplacian(old[i], &neighbors, factor);
    }
}

pub fn smooth_passes(
    positions: &mut [[f32; 3]],
    adjacency: &[Vec<usize>],
    factor: f32,
    iterations: usize,
) {
    for _ in 0..iterations {
        smooth_mesh_pass(positions, adjacency, factor);
    }
}

pub fn smooth_cotangent_weight(a: [f32; 3], b: [f32; 3], _c: [f32; 3]) -> f32 {
    /* stub: 1/distance between a and b */
    let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
    if d < 1e-10 {
        1.0
    } else {
        1.0 / d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smooth_laplacian_no_neighbors() {
        /* no neighbors returns center unchanged */
        let out = smooth_vertex_laplacian([1.0, 2.0, 3.0], &[], 0.5);
        assert!((out[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_laplacian_factor_zero() {
        /* factor=0 returns center unchanged */
        let neighbors = [[0.0f32, 0.0, 0.0]; 4];
        let out = smooth_vertex_laplacian([1.0, 1.0, 1.0], &neighbors, 0.0);
        assert!((out[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_laplacian_factor_one() {
        /* factor=1 returns average of neighbors */
        let neighbors = [[2.0f32, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let out = smooth_vertex_laplacian([0.0, 0.0, 0.0], &neighbors, 1.0);
        assert!((out[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_mesh_pass_moves_toward_neighbors() {
        /* pass with factor=1 converges to average */
        let mut pos = vec![[0.0f32, 0.0, 0.0], [4.0, 0.0, 0.0]];
        let adj = vec![vec![1usize], vec![0usize]];
        smooth_mesh_pass(&mut pos, &adj, 1.0);
        assert!((pos[0][0] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_passes_idempotent() {
        /* flat mesh stays flat after multiple passes */
        let mut pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let adj = vec![vec![1usize], vec![0, 2], vec![1usize]];
        smooth_passes(&mut pos, &adj, 0.5, 5);
        assert!(pos[1][1].abs() < 1e-5);
    }

    #[test]
    fn test_smooth_cotangent_weight() {
        /* weight is 1/distance */
        let w = smooth_cotangent_weight([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
        assert!((w - 0.5).abs() < 1e-5);
    }
}
