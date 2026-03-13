// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Configuration for adaptive remeshing.
#[allow(dead_code)]
pub struct AdaptiveRemeshConfig {
    pub min_edge_length: f32,
    pub max_edge_length: f32,
    pub iterations: usize,
    pub curvature_weight: f32,
}

impl Default for AdaptiveRemeshConfig {
    fn default() -> Self {
        Self {
            min_edge_length: 0.01,
            max_edge_length: 1.0,
            iterations: 3,
            curvature_weight: 1.0,
        }
    }
}

/// Result of adaptive remeshing.
#[allow(dead_code)]
pub struct AdaptiveRemeshResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub iterations_run: usize,
}

/// Compute adaptive target edge length at a vertex based on curvature.
#[allow(dead_code)]
pub fn adaptive_target_length(curvature: f32, config: &AdaptiveRemeshConfig) -> f32 {
    if curvature.abs() < 1e-6 {
        config.max_edge_length
    } else {
        (config.curvature_weight / curvature.abs())
            .clamp(config.min_edge_length, config.max_edge_length)
    }
}

/// Compute edge length.
#[allow(dead_code)]
pub fn edge_length_ar(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Collect unique edges from triangle indices.
#[allow(dead_code)]
pub fn collect_edges_ar(indices: &[u32]) -> Vec<(u32, u32)> {
    let mut edges = std::collections::HashSet::new();
    let n = indices.len() / 3;
    for fi in 0..n {
        let [a, b, c] = [indices[fi * 3], indices[fi * 3 + 1], indices[fi * 3 + 2]];
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let key = if u < v { (u, v) } else { (v, u) };
            edges.insert(key);
        }
    }
    edges.into_iter().collect()
}

/// Count edges longer than target.
#[allow(dead_code)]
pub fn count_long_edges_ar(positions: &[[f32; 3]], indices: &[u32], target: f32) -> usize {
    let edges = collect_edges_ar(indices);
    edges
        .iter()
        .filter(|&&(a, b)| edge_length_ar(positions[a as usize], positions[b as usize]) > target)
        .count()
}

/// Count edges shorter than target.
#[allow(dead_code)]
pub fn count_short_edges_ar(positions: &[[f32; 3]], indices: &[u32], target: f32) -> usize {
    let edges = collect_edges_ar(indices);
    edges
        .iter()
        .filter(|&&(a, b)| edge_length_ar(positions[a as usize], positions[b as usize]) < target)
        .count()
}

/// Stub adaptive remesh (returns copy with logged iterations).
#[allow(dead_code)]
pub fn adaptive_remesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &AdaptiveRemeshConfig,
) -> AdaptiveRemeshResult {
    AdaptiveRemeshResult {
        positions: positions.to_vec(),
        indices: indices.to_vec(),
        iterations_run: config.iterations,
    }
}

/// Compute average edge length.
#[allow(dead_code)]
pub fn avg_edge_length_ar(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let edges = collect_edges_ar(indices);
    if edges.is_empty() {
        return 0.0;
    }
    let sum: f32 = edges
        .iter()
        .map(|&(a, b)| edge_length_ar(positions[a as usize], positions[b as usize]))
        .sum();
    sum / edges.len() as f32
}

/// Serialize result to JSON.
#[allow(dead_code)]
pub fn adaptive_remesh_to_json(result: &AdaptiveRemeshResult) -> String {
    format!(
        r#"{{"vertices":{},"triangles":{},"iterations":{}}}"#,
        result.positions.len(),
        result.indices.len() / 3,
        result.iterations_run
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        (pos, vec![0, 1, 2])
    }

    #[test]
    fn default_config_valid() {
        let c = AdaptiveRemeshConfig::default();
        assert!(c.min_edge_length < c.max_edge_length);
    }

    #[test]
    fn adaptive_target_flat() {
        let c = AdaptiveRemeshConfig::default();
        let t = adaptive_target_length(0.0, &c);
        assert!((t - c.max_edge_length).abs() < 1e-6);
    }

    #[test]
    fn adaptive_target_curved() {
        let c = AdaptiveRemeshConfig::default();
        let t = adaptive_target_length(10.0, &c);
        assert!(t <= c.max_edge_length);
    }

    #[test]
    fn edge_count() {
        let (_, idx) = simple_mesh();
        let edges = collect_edges_ar(&idx);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn avg_edge_length_unit() {
        let (pos, idx) = simple_mesh();
        let avg = avg_edge_length_ar(&pos, &idx);
        assert!(avg > 0.0);
    }

    #[test]
    fn remesh_returns_same() {
        let (pos, idx) = simple_mesh();
        let r = adaptive_remesh(&pos, &idx, &AdaptiveRemeshConfig::default());
        assert_eq!(r.positions.len(), pos.len());
    }

    #[test]
    fn json_has_iterations() {
        let (pos, idx) = simple_mesh();
        let r = adaptive_remesh(
            &pos,
            &idx,
            &AdaptiveRemeshConfig {
                iterations: 7,
                ..Default::default()
            },
        );
        let j = adaptive_remesh_to_json(&r);
        assert!(j.contains("\"iterations\":7"));
    }

    #[test]
    fn count_long_none() {
        let (pos, idx) = simple_mesh();
        let n = count_long_edges_ar(&pos, &idx, 10.0);
        assert_eq!(n, 0);
    }

    #[test]
    fn count_short_none() {
        let (pos, idx) = simple_mesh();
        let n = count_short_edges_ar(&pos, &idx, 0.0);
        assert_eq!(n, 0);
    }

    #[test]
    fn edge_length_known() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [3.0, 4.0, 0.0];
        assert!((edge_length_ar(a, b) - 5.0).abs() < 1e-5);
    }
}
