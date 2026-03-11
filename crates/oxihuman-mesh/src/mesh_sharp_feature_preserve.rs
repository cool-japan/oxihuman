// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sharp feature (edge/corner) detection and preservation.

use std::collections::HashMap;

/// A detected sharp feature edge.
#[derive(Clone, Debug)]
pub struct SharpFeatureEdge {
    pub v0: u32,
    pub v1: u32,
    /// Dihedral angle in radians between the two adjacent faces.
    pub dihedral_angle: f32,
}

/// A detected sharp corner vertex.
#[derive(Clone, Debug)]
pub struct SharpFeatureCorner {
    pub vertex: u32,
    pub valence: usize,
}

/// Result of sharp feature detection.
#[derive(Clone, Debug, Default)]
pub struct SharpFeatureResult {
    pub sharp_edges: Vec<SharpFeatureEdge>,
    pub sharp_corners: Vec<SharpFeatureCorner>,
}

fn face_normal_sfp(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = n.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-12);
    [n[0] / len, n[1] / len, n[2] / len]
}

fn dot3_sfp(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b.iter()).map(|(u, v)| u * v).sum()
}

/// Detect sharp edges and corners using a dihedral-angle threshold.
pub fn detect_sharp_features(
    positions: &[[f32; 3]],
    indices: &[u32],
    angle_threshold_rad: f32,
) -> SharpFeatureResult {
    let tri_count = indices.len() / 3;
    // Build edge → face list
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tri_count {
        let verts = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for k in 0..3 {
            let a = verts[k];
            let b = verts[(k + 1) % 3];
            let key = (a.min(b), a.max(b));
            edge_faces.entry(key).or_default().push(t);
        }
    }

    let normals: Vec<[f32; 3]> = (0..tri_count)
        .map(|t| {
            face_normal_sfp(
                positions[indices[t * 3] as usize],
                positions[indices[t * 3 + 1] as usize],
                positions[indices[t * 3 + 2] as usize],
            )
        })
        .collect();

    let mut sharp_edges = Vec::new();
    let mut sharp_edge_vertex_count: HashMap<u32, usize> = HashMap::new();

    for (&(v0, v1), faces) in &edge_faces {
        if faces.len() == 2 {
            let n0 = normals[faces[0]];
            let n1 = normals[faces[1]];
            let cos_a = dot3_sfp(n0, n1).clamp(-1.0, 1.0);
            let angle = cos_a.acos();
            if angle > angle_threshold_rad {
                sharp_edges.push(SharpFeatureEdge {
                    v0,
                    v1,
                    dihedral_angle: angle,
                });
                *sharp_edge_vertex_count.entry(v0).or_insert(0) += 1;
                *sharp_edge_vertex_count.entry(v1).or_insert(0) += 1;
            }
        }
    }

    // Corner: vertex incident to 3+ sharp edges
    let sharp_corners: Vec<SharpFeatureCorner> = sharp_edge_vertex_count
        .iter()
        .filter(|(_, &count)| count >= 3)
        .map(|(&vertex, &valence)| SharpFeatureCorner { vertex, valence })
        .collect();

    SharpFeatureResult {
        sharp_edges,
        sharp_corners,
    }
}

/// Count sharp edges.
pub fn sharp_feature_edge_count(r: &SharpFeatureResult) -> usize {
    r.sharp_edges.len()
}

/// Count sharp corners.
pub fn sharp_feature_corner_count(r: &SharpFeatureResult) -> usize {
    r.sharp_corners.len()
}

/// Maximum dihedral angle among sharp edges.
pub fn sharp_feature_max_angle(r: &SharpFeatureResult) -> f32 {
    r.sharp_edges
        .iter()
        .map(|e| e.dihedral_angle)
        .fold(0.0_f32, f32::max)
}

/// Check whether a vertex index is a sharp corner.
pub fn is_sharp_corner(r: &SharpFeatureResult, vertex: u32) -> bool {
    r.sharp_corners.iter().any(|c| c.vertex == vertex)
}

#[cfg(test)]
mod tests {
    use super::*;

    /* A cube-like shape with sharp edges */
    fn two_tris_planar() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    fn two_tris_fold() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.0, 1.0],
            [0.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn planar_no_sharp_edges() {
        let (pos, idx) = two_tris_planar();
        let r = detect_sharp_features(&pos, &idx, std::f32::consts::FRAC_PI_4);
        assert_eq!(r.sharp_edges.len(), 0);
    }

    #[test]
    fn fold_has_sharp_edge() {
        let (pos, idx) = two_tris_fold();
        let r = detect_sharp_features(&pos, &idx, 0.1);
        assert!(!r.sharp_edges.is_empty());
    }

    #[test]
    fn sharp_feature_edge_count_consistent() {
        let (pos, idx) = two_tris_fold();
        let r = detect_sharp_features(&pos, &idx, 0.1);
        assert_eq!(sharp_feature_edge_count(&r), r.sharp_edges.len());
    }

    #[test]
    fn sharp_feature_corner_count_consistent() {
        let (pos, idx) = two_tris_fold();
        let r = detect_sharp_features(&pos, &idx, 0.1);
        assert_eq!(sharp_feature_corner_count(&r), r.sharp_corners.len());
    }

    #[test]
    fn sharp_feature_max_angle_nonneg() {
        let (pos, idx) = two_tris_fold();
        let r = detect_sharp_features(&pos, &idx, 0.1);
        assert!(sharp_feature_max_angle(&r) >= 0.0);
    }

    #[test]
    fn is_sharp_corner_false_when_no_corners() {
        let (pos, idx) = two_tris_planar();
        let r = detect_sharp_features(&pos, &idx, 0.01);
        assert!(!is_sharp_corner(&r, 0));
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let r = detect_sharp_features(&[], &[], 0.5);
        assert_eq!(r.sharp_edges.len(), 0);
        assert_eq!(r.sharp_corners.len(), 0);
    }

    #[test]
    fn dihedral_angles_finite() {
        let (pos, idx) = two_tris_fold();
        let r = detect_sharp_features(&pos, &idx, 0.01);
        for e in &r.sharp_edges {
            assert!(e.dihedral_angle.is_finite());
        }
    }
}
