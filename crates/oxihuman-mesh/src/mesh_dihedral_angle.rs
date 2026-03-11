// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Dihedral angle computation between adjacent faces.

use std::collections::HashMap;

/// Dihedral angle information for an edge.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DihedralEdge {
    pub v0: u32,
    pub v1: u32,
    pub angle_rad: f32,
}

/// Result of dihedral angle analysis.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DihedralResult {
    pub edges: Vec<DihedralEdge>,
    pub min_angle: f32,
    pub max_angle: f32,
    pub avg_angle: f32,
}

/// Compute cross product.
#[allow(dead_code)]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Dot product.
#[allow(dead_code)]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Normalize a vector.
#[allow(dead_code)]
fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = dot(v, v).sqrt();
    if l < 1e-12 {
        [0.0; 3]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Compute face normal from three vertices.
#[allow(dead_code)]
pub fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    norm(cross(e1, e2))
}

/// Compute dihedral angle between two face normals in radians [0, PI].
#[allow(dead_code)]
pub fn dihedral_angle_from_normals(n1: [f32; 3], n2: [f32; 3]) -> f32 {
    let d = dot(n1, n2).clamp(-1.0, 1.0);
    d.acos()
}

/// Compute all dihedral angles for internal edges of a triangle mesh.
#[allow(dead_code)]
pub fn compute_dihedral_angles(positions: &[[f32; 3]], indices: &[u32]) -> DihedralResult {
    let tri_count = indices.len() / 3;
    // Map edge -> list of face indices
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for t in 0..tri_count {
        let verts = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for k in 0..3 {
            let a = verts[k];
            let b = verts[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(t);
        }
    }

    // Compute face normals
    let normals: Vec<[f32; 3]> = (0..tri_count)
        .map(|t| {
            face_normal(
                positions[indices[t * 3] as usize],
                positions[indices[t * 3 + 1] as usize],
                positions[indices[t * 3 + 2] as usize],
            )
        })
        .collect();

    let mut edges = Vec::new();
    for (&(v0, v1), faces) in &edge_faces {
        if faces.len() == 2 {
            let angle = dihedral_angle_from_normals(normals[faces[0]], normals[faces[1]]);
            edges.push(DihedralEdge {
                v0,
                v1,
                angle_rad: angle,
            });
        }
    }

    let min = edges.iter().map(|e| e.angle_rad).fold(f32::MAX, f32::min);
    let max = edges.iter().map(|e| e.angle_rad).fold(f32::MIN, f32::max);
    let avg = if !edges.is_empty() {
        edges.iter().map(|e| e.angle_rad).sum::<f32>() / edges.len() as f32
    } else {
        0.0
    };

    DihedralResult {
        edges,
        min_angle: if min == f32::MAX { 0.0 } else { min },
        max_angle: if max == f32::MIN { 0.0 } else { max },
        avg_angle: avg,
    }
}

/// Count of internal edges with dihedral angles.
#[allow(dead_code)]
pub fn dihedral_edge_count(result: &DihedralResult) -> usize {
    result.edges.len()
}

/// Find edges sharper than a threshold (angle > threshold).
#[allow(dead_code)]
pub fn sharp_edges_by_angle(result: &DihedralResult, threshold: f32) -> Vec<&DihedralEdge> {
    result
        .edges
        .iter()
        .filter(|e| e.angle_rad > threshold)
        .collect()
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn dihedral_result_to_json(result: &DihedralResult) -> String {
    format!(
        "{{\"edge_count\":{},\"min\":{:.6},\"max\":{:.6},\"avg\":{:.6}}}",
        result.edges.len(),
        result.min_angle,
        result.max_angle,
        result.avg_angle,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn flat_two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
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
    fn test_face_normal() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_dihedral_coplanar() {
        let (pos, idx) = flat_two_tris();
        let result = compute_dihedral_angles(&pos, &idx);
        // shared edge should have angle ~0
        for e in &result.edges {
            assert!(e.angle_rad < 0.01);
        }
    }

    #[test]
    fn test_dihedral_perpendicular() {
        let angle = dihedral_angle_from_normals([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
        assert!((angle - PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_dihedral_same_normal() {
        let angle = dihedral_angle_from_normals([0.0, 0.0, 1.0], [0.0, 0.0, 1.0]);
        assert!(angle.abs() < 1e-5);
    }

    #[test]
    fn test_sharp_edges_by_angle() {
        let (pos, idx) = flat_two_tris();
        let result = compute_dihedral_angles(&pos, &idx);
        let sharp = sharp_edges_by_angle(&result, PI / 4.0);
        assert!(sharp.is_empty()); // flat mesh, no sharp edges
    }

    #[test]
    fn test_empty_mesh() {
        let result = compute_dihedral_angles(&[], &[]);
        assert_eq!(dihedral_edge_count(&result), 0);
    }

    #[test]
    fn test_single_tri_no_internal() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let result = compute_dihedral_angles(&pos, &idx);
        assert_eq!(dihedral_edge_count(&result), 0);
    }

    #[test]
    fn test_dihedral_result_to_json() {
        let result = DihedralResult {
            edges: vec![],
            min_angle: 0.0,
            max_angle: 0.0,
            avg_angle: 0.0,
        };
        let json = dihedral_result_to_json(&result);
        assert!(json.contains("\"edge_count\":0"));
    }

    #[test]
    fn test_edge_count() {
        let (pos, idx) = flat_two_tris();
        let result = compute_dihedral_angles(&pos, &idx);
        assert_eq!(dihedral_edge_count(&result), 1);
    }

    #[test]
    fn test_dihedral_opposite() {
        let angle = dihedral_angle_from_normals([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]);
        assert!((angle - PI).abs() < 1e-5);
    }
}
