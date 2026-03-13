// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Vertex position smoothing: Laplacian, Taubin, and constrained variants.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SmoothV2Config {
    pub iterations: u32,
    pub lambda: f32,
    pub mu: f32,
    pub preserve_boundary: bool,
}

#[allow(dead_code)]
pub fn default_smooth_v2_config() -> SmoothV2Config {
    SmoothV2Config {
        iterations: 3,
        lambda: 0.5,
        mu: -0.53,
        preserve_boundary: true,
    }
}

#[allow(dead_code)]
pub fn build_adjacency_v2(positions: &[[f32; 3]], indices: &[u32]) -> Vec<Vec<u32>> {
    let n = positions.len();
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a < n && b < n && c < n {
            if !adj[a].contains(&tri[1]) {
                adj[a].push(tri[1]);
            }
            if !adj[b].contains(&tri[2]) {
                adj[b].push(tri[2]);
            }
            if !adj[c].contains(&tri[0]) {
                adj[c].push(tri[0]);
            }
            if !adj[b].contains(&tri[0]) {
                adj[b].push(tri[0]);
            }
            if !adj[c].contains(&tri[1]) {
                adj[c].push(tri[1]);
            }
            if !adj[a].contains(&tri[2]) {
                adj[a].push(tri[2]);
            }
        }
    }
    adj
}

#[allow(dead_code)]
pub fn laplacian_step_v2(positions: &[[f32; 3]], adj: &[Vec<u32>], lambda: f32) -> Vec<[f32; 3]> {
    let mut out = positions.to_vec();
    for (i, neighbors) in adj.iter().enumerate() {
        if neighbors.is_empty() {
            continue;
        }
        let mut sum = [0.0f32; 3];
        for &nb in neighbors {
            let p = positions[nb as usize];
            sum[0] += p[0];
            sum[1] += p[1];
            sum[2] += p[2];
        }
        let n = neighbors.len() as f32;
        let avg = [sum[0] / n, sum[1] / n, sum[2] / n];
        let p = positions[i];
        out[i] = [
            p[0] + lambda * (avg[0] - p[0]),
            p[1] + lambda * (avg[1] - p[1]),
            p[2] + lambda * (avg[2] - p[2]),
        ];
    }
    out
}

#[allow(dead_code)]
pub fn taubin_smooth_v2(
    positions: &[[f32; 3]],
    adj: &[Vec<u32>],
    config: &SmoothV2Config,
) -> Vec<[f32; 3]> {
    let mut pos = positions.to_vec();
    for _ in 0..config.iterations {
        pos = laplacian_step_v2(&pos, adj, config.lambda);
        pos = laplacian_step_v2(&pos, adj, config.mu);
    }
    pos
}

#[allow(dead_code)]
pub fn smooth_displacement(original: &[[f32; 3]], smoothed: &[[f32; 3]]) -> f32 {
    if original.is_empty() || original.len() != smoothed.len() {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(smoothed.iter())
        .map(|(o, s)| {
            let d = [s[0] - o[0], s[1] - o[1], s[2] - o[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum();
    sum / original.len() as f32
}

#[allow(dead_code)]
pub fn smooth_v2_to_json(config: &SmoothV2Config) -> String {
    format!(
        "{{\"iterations\":{},\"lambda\":{},\"mu\":{}}}",
        config.iterations, config.lambda, config.mu
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn square_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_default_config() {
        let c = default_smooth_v2_config();
        assert_eq!(c.iterations, 3);
    }

    #[test]
    fn test_build_adjacency_count() {
        let pos = square_positions();
        let idx = square_indices();
        let adj = build_adjacency_v2(&pos, &idx);
        assert_eq!(adj.len(), 4);
    }

    #[test]
    fn test_laplacian_step_same_size() {
        let pos = square_positions();
        let idx = square_indices();
        let adj = build_adjacency_v2(&pos, &idx);
        let out = laplacian_step_v2(&pos, &adj, 0.5);
        assert_eq!(out.len(), pos.len());
    }

    #[test]
    fn test_taubin_smooth_same_size() {
        let pos = square_positions();
        let idx = square_indices();
        let adj = build_adjacency_v2(&pos, &idx);
        let config = default_smooth_v2_config();
        let out = taubin_smooth_v2(&pos, &adj, &config);
        assert_eq!(out.len(), pos.len());
    }

    #[test]
    fn test_smooth_displacement_zero_same() {
        let pos = square_positions();
        let d = smooth_displacement(&pos, &pos);
        assert!((d).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_displacement_positive() {
        let orig = vec![[0.0f32; 3]; 3];
        let shifted = vec![[1.0f32, 0.0, 0.0]; 3];
        let d = smooth_displacement(&orig, &shifted);
        assert!(d > 0.0);
    }

    #[test]
    fn test_json_output() {
        let c = default_smooth_v2_config();
        let j = smooth_v2_to_json(&c);
        assert!(j.contains("iterations"));
    }

    #[test]
    fn test_lambda_in_range() {
        let c = default_smooth_v2_config();
        assert!((0.0..=1.0).contains(&c.lambda));
    }

    #[test]
    fn test_empty_positions_smooth() {
        let adj: Vec<Vec<u32>> = Vec::new();
        let out = laplacian_step_v2(&[], &adj, 0.5);
        assert!(out.is_empty());
    }

    #[test]
    fn test_preserve_boundary_flag() {
        let c = default_smooth_v2_config();
        assert!(c.preserve_boundary);
    }
}
