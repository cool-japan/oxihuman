// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Trim mesh with a cutting plane: keep vertices (and their faces) above the plane.

/// A trimming plane defined by a normal and a point on the plane.
#[allow(dead_code)]
pub struct TrimPlane {
    pub normal: [f32; 3],
    pub point: [f32; 3],
}

/// Result of a trim operation.
#[allow(dead_code)]
pub struct TrimResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub kept_vertex_count: usize,
}

impl TrimPlane {
    /// Signed distance from a point to the plane (positive = above/front side).
    #[allow(dead_code)]
    pub fn signed_distance(&self, p: [f32; 3]) -> f32 {
        let n = self.normal;
        let q = self.point;
        n[0] * (p[0] - q[0]) + n[1] * (p[1] - q[1]) + n[2] * (p[2] - q[2])
    }

    /// A horizontal plane at height `y` with normal pointing up.
    #[allow(dead_code)]
    pub fn horizontal(y: f32) -> Self {
        TrimPlane {
            normal: [0.0, 1.0, 0.0],
            point: [0.0, y, 0.0],
        }
    }
}

/// Trim a mesh, keeping only triangles where ALL three vertices are above the plane.
/// Builds a compact mesh from the surviving faces.
#[allow(dead_code)]
pub fn trim_mesh(positions: &[[f32; 3]], indices: &[u32], plane: &TrimPlane) -> TrimResult {
    let above: Vec<bool> = positions
        .iter()
        .map(|&p| plane.signed_distance(p) >= 0.0)
        .collect();
    let mut kept: Vec<u32> = Vec::new();
    for tri in indices.chunks_exact(3) {
        if above[tri[0] as usize] && above[tri[1] as usize] && above[tri[2] as usize] {
            kept.extend_from_slice(tri);
        }
    }
    // compact: remap only used vertices
    let mut remap = vec![u32::MAX; positions.len()];
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    for &idx in &kept {
        if remap[idx as usize] == u32::MAX {
            remap[idx as usize] = new_positions.len() as u32;
            new_positions.push(positions[idx as usize]);
        }
    }
    let new_indices: Vec<u32> = kept.iter().map(|&i| remap[i as usize]).collect();
    let kept_vertex_count = new_positions.len();
    TrimResult {
        positions: new_positions,
        indices: new_indices,
        kept_vertex_count,
    }
}

/// Count how many vertices are above the plane.
#[allow(dead_code)]
pub fn count_above_plane(positions: &[[f32; 3]], plane: &TrimPlane) -> usize {
    positions
        .iter()
        .filter(|&&p| plane.signed_distance(p) >= 0.0)
        .count()
}

/// Count how many triangles survived trimming.
#[allow(dead_code)]
pub fn trim_triangle_count(result: &TrimResult) -> usize {
    result.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 3, 4];
        (positions, indices)
    }

    #[test]
    fn trim_keeps_above_faces() {
        let (pos, idx) = simple_mesh();
        let plane = TrimPlane::horizontal(0.0);
        let res = trim_mesh(&pos, &idx, &plane);
        assert_eq!(trim_triangle_count(&res), 1);
    }

    #[test]
    fn trim_removes_below_faces() {
        let (pos, idx) = simple_mesh();
        let plane = TrimPlane::horizontal(0.5);
        let res = trim_mesh(&pos, &idx, &plane);
        assert_eq!(trim_triangle_count(&res), 0);
    }

    #[test]
    fn trim_all_above() {
        let pos = vec![[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 2.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let plane = TrimPlane::horizontal(0.0);
        let res = trim_mesh(&pos, &idx, &plane);
        assert_eq!(trim_triangle_count(&res), 1);
    }

    #[test]
    fn signed_distance_positive_above() {
        let plane = TrimPlane::horizontal(0.0);
        assert!(plane.signed_distance([0.0, 1.0, 0.0]) > 0.0);
    }

    #[test]
    fn signed_distance_negative_below() {
        let plane = TrimPlane::horizontal(0.0);
        assert!(plane.signed_distance([0.0, -1.0, 0.0]) < 0.0);
    }

    #[test]
    fn count_above_plane_correct() {
        let pos = vec![[0.0, -1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let plane = TrimPlane::horizontal(0.0);
        assert_eq!(count_above_plane(&pos, &plane), 2);
    }

    #[test]
    fn compact_indices_in_range() {
        let (pos, idx) = simple_mesh();
        let plane = TrimPlane::horizontal(0.0);
        let res = trim_mesh(&pos, &idx, &plane);
        let nv = res.positions.len() as u32;
        for &i in &res.indices {
            assert!(i < nv);
        }
    }

    #[test]
    fn kept_vertex_count_matches() {
        let (pos, idx) = simple_mesh();
        let plane = TrimPlane::horizontal(0.0);
        let res = trim_mesh(&pos, &idx, &plane);
        assert_eq!(res.kept_vertex_count, res.positions.len());
    }

    #[test]
    fn trim_empty_mesh() {
        let res = trim_mesh(&[], &[], &TrimPlane::horizontal(0.0));
        assert_eq!(trim_triangle_count(&res), 0);
    }

    #[test]
    fn horizontal_plane_normal() {
        let p = TrimPlane::horizontal(1.0);
        assert!((p.normal[1] - 1.0).abs() < 1e-6);
        assert!((p.point[1] - 1.0).abs() < 1e-6);
    }
}
