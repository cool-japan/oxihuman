// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Detect and tag sharp edges and vertices by dihedral angle threshold.

use std::collections::HashMap;

/// A detected sharp feature edge.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SharpFeatureEdge {
    pub v0: u32,
    pub v1: u32,
    pub dihedral_deg: f32,
}

/// Result of sharp feature detection.
#[allow(dead_code)]
pub struct SharpFeatureResult {
    pub sharp_edges: Vec<SharpFeatureEdge>,
    pub sharp_vertices: Vec<u32>,
    pub threshold_deg: f32,
}

fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let ab = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let ac = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-9 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

fn dihedral_angle_between(n0: [f32; 3], n1: [f32; 3]) -> f32 {
    let d = (n0[0] * n1[0] + n0[1] * n1[1] + n0[2] * n1[2]).clamp(-1.0, 1.0);
    d.acos().to_degrees()
}

/// Detect sharp edges exceeding `threshold_deg` dihedral angle.
#[allow(dead_code)]
pub fn detect_sharp_features(
    positions: &[[f32; 3]],
    indices: &[u32],
    threshold_deg: f32,
) -> SharpFeatureResult {
    let n_tri = indices.len() / 3;
    let mut face_normals: Vec<[f32; 3]> = Vec::with_capacity(n_tri);
    for t in 0..n_tri {
        let p0 = positions[indices[t * 3] as usize];
        let p1 = positions[indices[t * 3 + 1] as usize];
        let p2 = positions[indices[t * 3 + 2] as usize];
        face_normals.push(face_normal(p0, p1, p2));
    }
    #[allow(clippy::type_complexity)]
    let mut edge_faces: HashMap<(u32, u32), Vec<(usize, [f32; 3])>> = HashMap::new();
    for t in 0..n_tri {
        let verts = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for e in 0..3 {
            let a = verts[e];
            let b = verts[(e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces
                .entry(key)
                .or_default()
                .push((t, face_normals[t]));
        }
    }
    let mut sharp_edges = Vec::new();
    let mut sharp_vert_set = std::collections::HashSet::new();
    for ((v0, v1), faces) in &edge_faces {
        if faces.len() < 2 {
            continue;
        }
        let n0 = faces[0].1;
        let n1 = faces[1].1;
        let angle = dihedral_angle_between(n0, n1);
        if angle > threshold_deg {
            sharp_edges.push(SharpFeatureEdge {
                v0: *v0,
                v1: *v1,
                dihedral_deg: angle,
            });
            sharp_vert_set.insert(*v0);
            sharp_vert_set.insert(*v1);
        }
    }
    let mut sharp_vertices: Vec<u32> = sharp_vert_set.into_iter().collect();
    sharp_vertices.sort();
    SharpFeatureResult {
        sharp_edges,
        sharp_vertices,
        threshold_deg,
    }
}

/// Count of sharp edges detected.
#[allow(dead_code)]
pub fn sharp_feature_edge_count(result: &SharpFeatureResult) -> usize {
    result.sharp_edges.len()
}

/// Count of sharp vertices detected.
#[allow(dead_code)]
pub fn sharp_feature_vertex_count(result: &SharpFeatureResult) -> usize {
    result.sharp_vertices.len()
}

/// Maximum dihedral angle among sharp edges.
#[allow(dead_code)]
pub fn max_sharp_dihedral(result: &SharpFeatureResult) -> f32 {
    result
        .sharp_edges
        .iter()
        .map(|e| e.dihedral_deg)
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Check if vertex is tagged as sharp.
#[allow(dead_code)]
pub fn is_sharp_vertex(vertex_idx: u32, result: &SharpFeatureResult) -> bool {
    result.sharp_vertices.binary_search(&vertex_idx).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 6, 3, 3, 6, 7, 1, 6, 2, 1, 5,
            6, 0, 3, 7, 0, 7, 4,
        ];
        (positions, indices)
    }

    #[test]
    fn cube_sharp_at_90_deg() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 45.0);
        assert!(!result.sharp_edges.is_empty());
    }

    #[test]
    fn no_sharp_at_high_threshold() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 150.0);
        assert_eq!(result.sharp_edges.len(), 0);
    }

    #[test]
    fn sharp_feature_edge_count_consistent() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 45.0);
        assert_eq!(sharp_feature_edge_count(&result), result.sharp_edges.len());
    }

    #[test]
    fn sharp_feature_vertex_count_consistent() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 45.0);
        assert_eq!(
            sharp_feature_vertex_count(&result),
            result.sharp_vertices.len()
        );
    }

    #[test]
    fn max_sharp_dihedral_above_threshold() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 45.0);
        if !result.sharp_edges.is_empty() {
            assert!(max_sharp_dihedral(&result) > 45.0);
        }
    }

    #[test]
    fn is_sharp_vertex_finds_tagged() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 45.0);
        if let Some(v) = result.sharp_vertices.first() {
            assert!(is_sharp_vertex(*v, &result));
        }
    }

    #[test]
    fn face_normal_unit() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn dihedral_angle_parallel_faces_zero() {
        let n = [0.0, 0.0, 1.0];
        let angle = dihedral_angle_between(n, n);
        assert!(angle < 1e-4);
    }

    #[test]
    fn dihedral_angle_perpendicular_90() {
        let n0 = [0.0, 0.0, 1.0];
        let n1 = [1.0, 0.0, 0.0];
        let angle = dihedral_angle_between(n0, n1);
        assert!((angle - 90.0).abs() < 1e-3);
    }

    #[test]
    fn threshold_stored_in_result() {
        let (pos, idx) = cube_mesh();
        let result = detect_sharp_features(&pos, &idx, 60.0);
        assert!((result.threshold_deg - 60.0).abs() < 1e-5);
    }
}
