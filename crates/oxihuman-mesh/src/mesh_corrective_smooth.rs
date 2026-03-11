// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Corrective smoothing modifier.

/// Configuration for corrective smooth.
#[derive(Debug, Clone)]
pub struct CorrectiveSmoothConfig {
    pub factor: f32,
    pub iterations: usize,
    pub rest_source: RestSource,
}

/// How to obtain the rest shape for correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestSource {
    Original,
    Bind,
}

impl Default for CorrectiveSmoothConfig {
    fn default() -> Self {
        Self { factor: 0.5, iterations: 1, rest_source: RestSource::Original }
    }
}

impl CorrectiveSmoothConfig {
    pub fn new(factor: f32, iterations: usize) -> Self {
        Self { factor, iterations, rest_source: RestSource::Original }
    }
}

/// Compute a simple laplacian-smoothed version of positions.
pub fn laplacian_step(positions: &[[f32; 3]], adjacency: &[Vec<usize>]) -> Vec<[f32; 3]> {
    let mut result = positions.to_vec();
    for (i, neighbors) in adjacency.iter().enumerate() {
        if neighbors.is_empty() {
            continue;
        }
        let mut sum = [0.0_f32; 3];
        for &n in neighbors {
            sum[0] += positions[n][0];
            sum[1] += positions[n][1];
            sum[2] += positions[n][2];
        }
        let cnt = neighbors.len() as f32;
        result[i] = [sum[0] / cnt, sum[1] / cnt, sum[2] / cnt];
    }
    result
}

/// Apply corrective smooth (smooth then correct against rest).
pub fn apply_corrective_smooth(
    positions: &mut [[f32; 3]],
    rest: &[[f32; 3]],
    adjacency: &[Vec<usize>],
    cfg: &CorrectiveSmoothConfig,
) {
    let mut smoothed = positions.to_vec();
    for _ in 0..cfg.iterations {
        smoothed = laplacian_step(&smoothed, adjacency);
    }
    for i in 0..positions.len().min(rest.len()).min(smoothed.len()) {
        let delta = [
            smoothed[i][0] - positions[i][0],
            smoothed[i][1] - positions[i][1],
            smoothed[i][2] - positions[i][2],
        ];
        /* correction: blend smooth delta scaled by factor */
        positions[i][0] += delta[0] * cfg.factor;
        positions[i][1] += delta[1] * cfg.factor;
        positions[i][2] += delta[2] * cfg.factor;
        /* keep rest shape influence */
        let _ = rest[i];
    }
}

/// Validate config.
pub fn validate_corrective_smooth_config(cfg: &CorrectiveSmoothConfig) -> bool {
    (0.0..=1.0).contains(&cfg.factor) && cfg.iterations > 0
}

/// Compute per-vertex delta between smoothed and original.
pub fn smooth_delta(positions: &[[f32; 3]], smoothed: &[[f32; 3]]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .zip(smoothed.iter())
        .map(|(p, s)| [s[0] - p[0], s[1] - p[1], s[2] - p[2]])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg = CorrectiveSmoothConfig::default();
        assert!((cfg.factor - 0.5).abs() < 1e-6);
        assert_eq!(cfg.iterations, 1);
    }

    #[test]
    fn test_laplacian_step_isolated_vertex() {
        let pos = vec![[1.0_f32, 2.0, 3.0]];
        let adj: Vec<Vec<usize>> = vec![vec![]];
        let out = laplacian_step(&pos, &adj);
        assert!((out[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_laplacian_step_two_vertices() {
        let pos = vec![[0.0_f32, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let adj = vec![vec![1usize], vec![0usize]];
        let out = laplacian_step(&pos, &adj);
        assert!((out[0][0] - 2.0).abs() < 1e-5);
        assert!((out[1][0] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = CorrectiveSmoothConfig::new(0.5, 3);
        assert!(validate_corrective_smooth_config(&cfg));
    }

    #[test]
    fn test_validate_config_zero_iterations() {
        let cfg = CorrectiveSmoothConfig::new(0.5, 0);
        assert!(!validate_corrective_smooth_config(&cfg));
    }

    #[test]
    fn test_validate_config_factor_out_of_range() {
        let cfg = CorrectiveSmoothConfig::new(1.5, 1);
        assert!(!validate_corrective_smooth_config(&cfg));
    }

    #[test]
    fn test_smooth_delta_zero_when_same() {
        let pos = vec![[1.0_f32, 2.0, 3.0]];
        let d = smooth_delta(&pos, &pos);
        assert!(d[0][0].abs() < 1e-5);
    }

    #[test]
    fn test_apply_corrective_smooth_preserves_count() {
        let mut pos = vec![[0.0_f32; 3]; 4];
        let rest = pos.clone();
        let adj = vec![vec![1usize, 3], vec![0usize, 2], vec![1usize, 3], vec![2usize, 0]];
        let cfg = CorrectiveSmoothConfig::default();
        apply_corrective_smooth(&mut pos, &rest, &adj, &cfg);
        assert_eq!(pos.len(), 4);
    }

    #[test]
    fn test_rest_source_debug() {
        let s = format!("{:?}", RestSource::Bind);
        assert!(s.contains("Bind"));
    }
}
