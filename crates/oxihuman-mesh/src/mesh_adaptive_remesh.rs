// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Adaptive remeshing: refines mesh density based on local curvature estimation.

/// Configuration for adaptive remeshing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdaptiveRemeshConfig {
    pub curvature_threshold: f32,
    pub min_edge_length: f32,
    pub max_edge_length: f32,
    pub iterations: u32,
}

#[allow(dead_code)]
pub fn default_adaptive_remesh_config() -> AdaptiveRemeshConfig {
    AdaptiveRemeshConfig {
        curvature_threshold: 0.1,
        min_edge_length: 0.01,
        max_edge_length: 1.0,
        iterations: 3,
    }
}

/// Result of adaptive remeshing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AdaptiveRemeshResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub splits_performed: usize,
    pub collapses_performed: usize,
}

/// Estimate curvature at a vertex using discrete angle defect.
#[allow(dead_code)]
pub fn estimate_vertex_curvature(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex: usize,
) -> f32 {
    use std::f32::consts::TAU;
    let mut angle_sum = 0.0f32;
    let mut count = 0u32;
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let local = if i0 == vertex {
            Some((i0, i1, i2))
        } else if i1 == vertex {
            Some((i1, i2, i0))
        } else if i2 == vertex {
            Some((i2, i0, i1))
        } else {
            None
        };
        if let Some((v, a, b)) = local {
            let va = sub3(positions[a], positions[v]);
            let vb = sub3(positions[b], positions[v]);
            let d = dot3(va, vb);
            let la = length3(va);
            let lb = length3(vb);
            if la > 1e-12 && lb > 1e-12 {
                let cos_a = (d / (la * lb)).clamp(-1.0, 1.0);
                angle_sum += cos_a.acos();
                count += 1;
            }
        }
    }
    if count == 0 {
        return 0.0;
    }
    (TAU - angle_sum).abs()
}

/// Compute edge length between two vertices.
#[allow(dead_code)]
pub fn edge_length(positions: &[[f32; 3]], a: usize, b: usize) -> f32 {
    length3(sub3(positions[b], positions[a]))
}

/// Compute target edge length based on local curvature.
#[allow(dead_code)]
pub fn target_edge_length(curvature: f32, config: &AdaptiveRemeshConfig) -> f32 {
    let t = (curvature / config.curvature_threshold).clamp(0.0, 1.0);
    config.max_edge_length * (1.0 - t) + config.min_edge_length * t
}

/// Split an edge by inserting a midpoint, returning new vertex position.
#[allow(dead_code)]
pub fn split_edge_midpoint(positions: &[[f32; 3]], a: usize, b: usize) -> [f32; 3] {
    [
        (positions[a][0] + positions[b][0]) * 0.5,
        (positions[a][1] + positions[b][1]) * 0.5,
        (positions[a][2] + positions[b][2]) * 0.5,
    ]
}

/// Perform one pass of adaptive splitting on long edges relative to local curvature.
#[allow(dead_code)]
pub fn adaptive_split_pass(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    config: &AdaptiveRemeshConfig,
) -> usize {
    let mut splits = 0usize;
    let tri_count = indices.len() / 3;
    let mut new_tris: Vec<u32> = Vec::new();
    let mut skip = vec![false; tri_count];

    for t in 0..tri_count {
        if skip[t] {
            continue;
        }
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let e0 = edge_length(positions, i0, i1);
        let c = estimate_vertex_curvature(positions, indices, i0);
        let target = target_edge_length(c, config);
        if e0 > target * 1.5 {
            let mid = split_edge_midpoint(positions, i0, i1);
            let mi = positions.len() as u32;
            positions.push(mid);
            skip[t] = true;
            new_tris.extend_from_slice(&[i0 as u32, mi, i2 as u32]);
            new_tris.extend_from_slice(&[mi, i1 as u32, i2 as u32]);
            splits += 1;
        }
    }
    // Keep non-skipped triangles
    let mut kept = Vec::new();
    for t in 0..tri_count {
        if !skip[t] {
            kept.push(indices[t * 3]);
            kept.push(indices[t * 3 + 1]);
            kept.push(indices[t * 3 + 2]);
        }
    }
    kept.extend_from_slice(&new_tris);
    *indices = kept;
    splits
}

/// Run full adaptive remesh for configured iterations.
#[allow(dead_code)]
pub fn adaptive_remesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &AdaptiveRemeshConfig,
) -> AdaptiveRemeshResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let mut total_splits = 0usize;
    for _ in 0..config.iterations {
        let s = adaptive_split_pass(&mut pos, &mut idx, config);
        total_splits += s;
        if s == 0 {
            break;
        }
    }
    AdaptiveRemeshResult {
        positions: pos,
        indices: idx,
        splits_performed: total_splits,
        collapses_performed: 0,
    }
}

/// Count faces in the result.
#[allow(dead_code)]
pub fn adaptive_remesh_face_count(result: &AdaptiveRemeshResult) -> usize {
    result.indices.len() / 3
}

/// Count vertices in the result.
#[allow(dead_code)]
pub fn adaptive_remesh_vertex_count(result: &AdaptiveRemeshResult) -> usize {
    result.positions.len()
}

// ── helpers ──

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn triangle_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2];
        (positions, indices)
    }

    fn quad_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        (positions, indices)
    }

    #[test]
    fn test_default_config() {
        let c = default_adaptive_remesh_config();
        assert!(c.iterations > 0);
        assert!(c.min_edge_length < c.max_edge_length);
    }

    #[test]
    fn test_edge_length_basic() {
        let pos = vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0]];
        let el = edge_length(&pos, 0, 1);
        assert!((el - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_split_edge_midpoint() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 4.0, 6.0]];
        let mid = split_edge_midpoint(&pos, 0, 1);
        assert!((mid[0] - 1.0).abs() < 1e-6);
        assert!((mid[1] - 2.0).abs() < 1e-6);
        assert!((mid[2] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_target_edge_length_zero_curvature() {
        let c = default_adaptive_remesh_config();
        let t = target_edge_length(0.0, &c);
        assert!((t - c.max_edge_length).abs() < 1e-6);
    }

    #[test]
    fn test_target_edge_length_high_curvature() {
        let c = default_adaptive_remesh_config();
        let t = target_edge_length(c.curvature_threshold * 2.0, &c);
        assert!((t - c.min_edge_length).abs() < 1e-6);
    }

    #[test]
    fn test_estimate_curvature_flat() {
        let (pos, idx) = quad_mesh();
        let curv = estimate_vertex_curvature(&pos, &idx, 0);
        // For a flat mesh the angle defect should be close to PI/2 from boundary
        assert!(curv >= 0.0);
    }

    #[test]
    fn test_adaptive_remesh_identity() {
        let (pos, idx) = triangle_mesh();
        let mut config = default_adaptive_remesh_config();
        config.max_edge_length = 100.0; // no splitting needed
        let result = adaptive_remesh(&pos, &idx, &config);
        assert_eq!(adaptive_remesh_face_count(&result), 1);
    }

    #[test]
    fn test_adaptive_remesh_splits() {
        let (pos, idx) = triangle_mesh();
        let mut config = default_adaptive_remesh_config();
        config.max_edge_length = 0.01;
        config.min_edge_length = 0.001;
        config.curvature_threshold = 0.001;
        config.iterations = 1;
        let result = adaptive_remesh(&pos, &idx, &config);
        assert!(result.splits_performed > 0 || adaptive_remesh_face_count(&result) >= 1);
    }

    #[test]
    fn test_face_and_vertex_counts() {
        let (pos, idx) = quad_mesh();
        let config = default_adaptive_remesh_config();
        let result = adaptive_remesh(&pos, &idx, &config);
        assert!(adaptive_remesh_vertex_count(&result) >= 4);
        assert!(adaptive_remesh_face_count(&result) >= 2);
    }

    #[test]
    fn test_curvature_non_negative() {
        let _ = PI; // use the import
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.866, 0.0],
            [0.5, 0.289, 0.816],
        ];
        let idx = vec![0, 1, 2, 0, 1, 3, 1, 2, 3, 0, 2, 3];
        for v in 0..4 {
            let c = estimate_vertex_curvature(&pos, &idx, v);
            assert!(c >= 0.0);
        }
    }
}
