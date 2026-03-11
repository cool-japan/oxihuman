// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Consistent triangle normal orientation via flood fill.

use std::collections::{HashMap, VecDeque};

/// Result of an orientation pass.
#[allow(dead_code)]
pub struct OrientResult {
    /// The (possibly re-wound) index buffer.
    pub indices: Vec<u32>,
    /// Number of triangles whose winding was flipped.
    pub flipped_count: usize,
    /// Whether the final mesh has consistent winding.
    pub consistent: bool,
}

/// Configuration for mesh orientation.
#[allow(dead_code)]
pub struct OrientConfig {
    /// Reference point (e.g. centroid) for outward normal check.
    pub outward_reference: [f32; 3],
    /// If true, flip all faces whose normals point toward the reference point.
    pub flip_if_inward: bool,
}

/// Returns a sensible default `OrientConfig`.
#[allow(dead_code)]
pub fn default_orient_config() -> OrientConfig {
    OrientConfig {
        outward_reference: [0.0, 0.0, 0.0],
        flip_if_inward: true,
    }
}

/// Cross product of two 3-vectors.
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Dot product of two 3-vectors.
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Length of a 3-vector.
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

/// Subtract two 3-vectors.
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Compute the (unnormalized) normal of triangle (a, b, c).
#[allow(dead_code)]
pub fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    cross3(sub3(b, a), sub3(c, a))
}

/// Compute the unit normal of triangle (a, b, c).
#[allow(dead_code)]
pub fn triangle_normal_normalized(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let n = triangle_normal(a, b, c);
    let l = len3(n);
    if l < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    [n[0] / l, n[1] / l, n[2] / l]
}

/// Compute per-vertex averaged normals for a triangle mesh.
#[allow(dead_code)]
pub fn compute_mesh_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0f32; 3]; positions.len()];
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        let n = triangle_normal(positions[i0], positions[i1], positions[i2]);
        for idx in [i0, i1, i2] {
            normals[idx][0] += n[0];
            normals[idx][1] += n[1];
            normals[idx][2] += n[2];
        }
    }
    for n in &mut normals {
        let l = len3(*n);
        if l > 1e-12 {
            n[0] /= l;
            n[1] /= l;
            n[2] /= l;
        }
    }
    normals
}

/// Return the (unnormalized) normal of a single face.
#[allow(dead_code)]
pub fn face_normal(positions: &[[f32; 3]], face_idx: usize, indices: &[u32]) -> [f32; 3] {
    let i0 = indices[face_idx * 3] as usize;
    let i1 = indices[face_idx * 3 + 1] as usize;
    let i2 = indices[face_idx * 3 + 2] as usize;
    triangle_normal(positions[i0], positions[i1], positions[i2])
}

/// Return true if all face normals point away from `centroid`.
#[allow(dead_code)]
pub fn all_normals_outward(positions: &[[f32; 3]], indices: &[u32], centroid: [f32; 3]) -> bool {
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let i0 = indices[f * 3] as usize;
        let i1 = indices[f * 3 + 1] as usize;
        let i2 = indices[f * 3 + 2] as usize;
        let n = triangle_normal(positions[i0], positions[i1], positions[i2]);
        // vector from centroid to face centre
        let fc = [
            (positions[i0][0] + positions[i1][0] + positions[i2][0]) / 3.0 - centroid[0],
            (positions[i0][1] + positions[i1][1] + positions[i2][1]) / 3.0 - centroid[1],
            (positions[i0][2] + positions[i1][2] + positions[i2][2]) / 3.0 - centroid[2],
        ];
        if dot3(n, fc) < 0.0 {
            return false;
        }
    }
    true
}

/// Flip the winding of a single face (swap vertex 1 and 2).
#[allow(dead_code)]
pub fn flip_face(indices: &mut [u32], face_idx: usize) {
    indices.swap(face_idx * 3 + 1, face_idx * 3 + 2);
}

/// Flip the winding of every triangle in the index buffer.
#[allow(dead_code)]
pub fn flip_all_faces(indices: &mut [u32]) {
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        flip_face(indices, f);
    }
}

/// Compute the centroid (mean position) of a mesh.
#[allow(dead_code)]
pub fn mesh_centroid(positions: &[[f32; 3]]) -> [f32; 3] {
    if positions.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for p in positions {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    let n = positions.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Check that the mesh has consistent winding.
///
/// For a consistently wound mesh every directed edge `(a, b)` should appear
/// at most once.  If the same directed edge appears in two different triangles
/// it means those triangles have the same (rather than opposite) orientation
/// at that shared edge, which is inconsistent.
#[allow(dead_code)]
pub fn consistent_winding_check(indices: &[u32]) -> bool {
    let mut directed_edge_count: HashMap<(u32, u32), usize> = HashMap::new();
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let vs = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for e in 0..3 {
            let a = vs[e];
            let b = vs[(e + 1) % 3];
            *directed_edge_count.entry((a, b)).or_insert(0) += 1;
        }
    }
    // Every directed edge must appear at most once.
    directed_edge_count.values().all(|&v| v <= 1)
}

// ── flood-fill orientation helpers ─────────────────────────────────────────

/// Build a map from directed edge (v0,v1) to the face that owns it.
fn build_edge_face_map(indices: &[u32]) -> HashMap<(u32, u32), usize> {
    let face_count = indices.len() / 3;
    let mut map = HashMap::new();
    for f in 0..face_count {
        let vs = [indices[f * 3], indices[f * 3 + 1], indices[f * 3 + 2]];
        for e in 0..3 {
            let a = vs[e];
            let b = vs[(e + 1) % 3];
            map.insert((a, b), f);
        }
    }
    map
}

/// Orient the mesh so that all triangles have consistent outward winding,
/// using a flood-fill BFS approach.
#[allow(dead_code)]
pub fn orient_mesh(positions: &[[f32; 3]], indices: &[u32], cfg: &OrientConfig) -> OrientResult {
    let face_count = indices.len() / 3;
    let mut out = indices.to_vec();
    let mut flipped_count = 0usize;

    if face_count == 0 {
        return OrientResult {
            indices: out,
            flipped_count: 0,
            consistent: true,
        };
    }

    let mut visited = vec![false; face_count];

    // Seed: ensure face 0 points outward w.r.t. the reference point.
    {
        let i0 = out[0] as usize;
        let i1 = out[1] as usize;
        let i2 = out[2] as usize;
        let n = triangle_normal(positions[i0], positions[i1], positions[i2]);
        let fc = [
            (positions[i0][0] + positions[i1][0] + positions[i2][0]) / 3.0
                - cfg.outward_reference[0],
            (positions[i0][1] + positions[i1][1] + positions[i2][1]) / 3.0
                - cfg.outward_reference[1],
            (positions[i0][2] + positions[i1][2] + positions[i2][2]) / 3.0
                - cfg.outward_reference[2],
        ];
        if cfg.flip_if_inward && dot3(n, fc) < 0.0 {
            flip_face(&mut out, 0);
            flipped_count += 1;
        }
    }

    let mut queue = VecDeque::new();
    queue.push_back(0usize);
    visited[0] = true;

    while let Some(f) = queue.pop_front() {
        // Rebuild edge->face map each iteration to reflect flips so far.
        let edge_map = build_edge_face_map(&out);
        let vs = [out[f * 3], out[f * 3 + 1], out[f * 3 + 2]];
        for e in 0..3 {
            let a = vs[e];
            let b = vs[(e + 1) % 3];
            // Neighbour shares the opposite directed edge (b,a).
            if let Some(&nb) = edge_map.get(&(b, a)) {
                if !visited[nb] {
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
            // If edge (a,b) also exists in some face (same direction), that
            // neighbour is misaligned — flip it.
            if let Some(&nb) = edge_map.get(&(a, b)) {
                if nb != f && !visited[nb] {
                    flip_face(&mut out, nb);
                    flipped_count += 1;
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
    }

    let consistent = consistent_winding_check(&out);
    OrientResult {
        indices: out,
        flipped_count,
        consistent,
    }
}

/// Return a human-readable summary of an `OrientResult`.
#[allow(dead_code)]
pub fn orient_result_summary(result: &OrientResult) -> String {
    format!(
        "OrientResult {{ flipped: {}, consistent: {} }}",
        result.flipped_count, result.consistent
    )
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_triangle_normal_basic() {
        let n = triangle_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2] > 0.0, "z-component should be positive");
    }

    #[test]
    fn test_triangle_normal_unit_length() {
        let n = triangle_normal_normalized([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let l = len3(n);
        assert!(
            (l - 1.0).abs() < 1e-6,
            "normalized normal should have unit length"
        );
    }

    #[test]
    fn test_triangle_normal_direction() {
        // Counter-clockwise triangle in XY plane → normal should point +Z
        let n = triangle_normal_normalized([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_mesh_normals_size() {
        let positions = unit_square_positions();
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let normals = compute_mesh_normals(&positions, &indices);
        assert_eq!(normals.len(), positions.len());
    }

    #[test]
    fn test_compute_mesh_normals_z_direction() {
        let positions = unit_square_positions();
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let normals = compute_mesh_normals(&positions, &indices);
        for n in &normals {
            assert!(n[2] > 0.0, "all normals should point +Z");
        }
    }

    #[test]
    fn test_face_normal_correct() {
        let positions = unit_square_positions();
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let n = face_normal(&positions, 0, &indices);
        assert!(n[2] > 0.0);
    }

    #[test]
    fn test_flip_face_swaps_vertices() {
        let mut indices: Vec<u32> = vec![0, 1, 2];
        flip_face(&mut indices, 0);
        assert_eq!(indices, vec![0, 2, 1]);
    }

    #[test]
    fn test_flip_all_faces() {
        let mut indices: Vec<u32> = vec![0, 1, 2, 3, 4, 5];
        flip_all_faces(&mut indices);
        assert_eq!(indices, vec![0, 2, 1, 3, 5, 4]);
    }

    #[test]
    fn test_mesh_centroid_origin() {
        let positions = vec![
            [-1.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let c = mesh_centroid(&positions);
        assert!(c[0].abs() < 1e-6);
        assert!(c[1].abs() < 1e-6);
    }

    #[test]
    fn test_mesh_centroid_empty() {
        let c = mesh_centroid(&[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_consistent_winding_check_consistent() {
        // Two triangles forming a quad with consistent winding
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        assert!(consistent_winding_check(&indices));
    }

    #[test]
    fn test_consistent_winding_check_inconsistent() {
        // Two triangles sharing edge (1,2) both with same direction → inconsistent
        let indices: Vec<u32> = vec![0, 1, 2, 3, 1, 2];
        assert!(!consistent_winding_check(&indices));
    }

    #[test]
    fn test_orient_mesh_already_consistent() {
        let positions = unit_square_positions();
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        let cfg = OrientConfig {
            outward_reference: [0.5, 0.5, -1.0],
            flip_if_inward: true,
        };
        let result = orient_mesh(&positions, &indices, &cfg);
        assert!(result.consistent);
    }

    #[test]
    fn test_orient_result_summary_contains_consistent() {
        let r = OrientResult {
            indices: vec![],
            flipped_count: 3,
            consistent: true,
        };
        let s = orient_result_summary(&r);
        assert!(s.contains("3"));
        assert!(s.contains("true"));
    }

    #[test]
    fn test_all_normals_outward() {
        let positions = unit_square_positions();
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        // centroid is at z < 0, normals point +z → outward
        assert!(all_normals_outward(&positions, &indices, [0.5, 0.5, -1.0]));
    }

    #[test]
    fn test_default_orient_config() {
        let cfg = default_orient_config();
        assert_eq!(cfg.outward_reference, [0.0, 0.0, 0.0]);
        assert!(cfg.flip_if_inward);
    }

    #[test]
    fn test_orient_mesh_empty() {
        let result = orient_mesh(&[], &[], &default_orient_config());
        assert_eq!(result.flipped_count, 0);
        assert!(result.consistent);
    }
}
