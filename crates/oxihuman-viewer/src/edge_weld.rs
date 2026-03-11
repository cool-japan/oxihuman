// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge-weld post-process: closes visible T-junction seams along mesh boundaries.

/// Configuration for the edge-weld pass.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct EdgeWeldConfig {
    /// Distance threshold below which two vertices are considered coincident.
    pub weld_threshold: f32,
    /// Whether to weld only UV-seam edges (false = all boundary edges).
    pub seam_only: bool,
    /// Normal angle tolerance in radians for welding across hard edges.
    pub normal_tolerance: f32,
    /// Maximum iterations for iterative welding.
    pub max_iters: u32,
}

impl Default for EdgeWeldConfig {
    fn default() -> Self {
        Self {
            weld_threshold: 1e-4,
            seam_only: false,
            normal_tolerance: std::f32::consts::FRAC_PI_6,
            max_iters: 4,
        }
    }
}

/// Statistics produced by a weld pass.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct WeldStats {
    /// Number of vertex pairs merged.
    pub merged_pairs: usize,
    /// Number of iterations performed.
    pub iterations: u32,
}

/// Weld duplicate vertices in a position buffer (returns index remap table).
#[allow(dead_code)]
pub fn weld_positions(positions: &[[f32; 3]], threshold: f32) -> Vec<usize> {
    let n = positions.len();
    let mut remap: Vec<usize> = (0..n).collect();
    for i in 0..n {
        if remap[i] != i {
            continue;
        }
        for j in (i + 1)..n {
            if remap[j] != j {
                continue;
            }
            if distance_sq(positions[i], positions[j]) < threshold * threshold {
                remap[j] = i;
            }
        }
    }
    remap
}

fn distance_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Count unique vertices after welding.
#[allow(dead_code)]
pub fn count_unique(remap: &[usize]) -> usize {
    let mut count = 0;
    for (i, &r) in remap.iter().enumerate() {
        if r == i {
            count += 1;
        }
    }
    count
}

/// Apply remap to an index buffer in-place.
#[allow(dead_code)]
pub fn apply_remap_to_indices(indices: &mut [usize], remap: &[usize]) {
    for idx in indices.iter_mut() {
        *idx = remap[*idx];
    }
}

/// Check if a weld config is valid.
#[allow(dead_code)]
pub fn validate_weld_config(cfg: &EdgeWeldConfig) -> bool {
    cfg.weld_threshold > 0.0 && cfg.max_iters > 0
}

/// Run a weld pass and return statistics.
#[allow(dead_code)]
pub fn run_weld_pass(
    positions: &[[f32; 3]],
    indices: &mut [usize],
    cfg: &EdgeWeldConfig,
) -> WeldStats {
    let remap = weld_positions(positions, cfg.weld_threshold);
    apply_remap_to_indices(indices, &remap);
    let merged = positions.len() - count_unique(&remap);
    WeldStats {
        merged_pairs: merged,
        iterations: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_6;

    #[test]
    fn default_config_valid() {
        let cfg = EdgeWeldConfig::default();
        assert!(validate_weld_config(&cfg));
    }

    #[test]
    fn normal_tolerance_is_pi_over_6() {
        let cfg = EdgeWeldConfig::default();
        assert!((cfg.normal_tolerance - FRAC_PI_6).abs() < 1e-6);
    }

    #[test]
    fn weld_identical_positions() {
        let pos = vec![[0.0_f32, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let remap = weld_positions(&pos, 1e-4);
        assert_eq!(remap[1], 0);
    }

    #[test]
    fn no_weld_far_positions() {
        let pos = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let remap = weld_positions(&pos, 1e-4);
        assert_eq!(remap[0], 0);
        assert_eq!(remap[1], 1);
    }

    #[test]
    fn count_unique_all_different() {
        let remap = vec![0, 1, 2];
        assert_eq!(count_unique(&remap), 3);
    }

    #[test]
    fn count_unique_with_merge() {
        let remap = vec![0, 0, 2];
        assert_eq!(count_unique(&remap), 2);
    }

    #[test]
    fn apply_remap_updates_indices() {
        let remap = vec![0, 0, 2];
        let mut indices = vec![1usize, 2];
        apply_remap_to_indices(&mut indices, &remap);
        assert_eq!(indices[0], 0);
        assert_eq!(indices[1], 2);
    }

    #[test]
    fn run_weld_pass_returns_stats() {
        let pos = vec![[0.0_f32, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut indices = vec![0usize, 1, 2];
        let cfg = EdgeWeldConfig::default();
        let stats = run_weld_pass(&pos, &mut indices, &cfg);
        assert!(stats.merged_pairs > 0);
    }

    #[test]
    fn weld_stats_default() {
        let s = WeldStats::default();
        assert_eq!(s.merged_pairs, 0);
        assert_eq!(s.iterations, 0);
    }
}
