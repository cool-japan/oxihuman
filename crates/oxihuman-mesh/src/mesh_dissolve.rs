// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a dissolve operation.
#[allow(dead_code)]
pub struct DissolveResult {
    pub new_verts: Vec<[f32; 3]>,
    pub new_tris: Vec<[u32; 3]>,
    pub dissolved_count: usize,
}

/// Return the number of dissolved edges from a result.
#[allow(dead_code)]
pub fn dissolved_edge_count(result: &DissolveResult) -> usize {
    result.dissolved_count
}

/// Check if three collinear vertices lie on the same line within tolerance.
#[allow(dead_code)]
pub fn edges_are_collinear(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], tol: f32) -> bool {
    let ab = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let ac = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len_sq = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
    len_sq < tol * tol
}

/// Dissolve collinear edges (vertices that lie exactly on adjacent edges).
#[allow(dead_code)]
pub fn dissolve_collinear_edges(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    threshold: f32,
) -> DissolveResult {
    let mut dissolved_count = 0usize;
    let mut kept_tris = Vec::with_capacity(tris.len());

    for tri in tris {
        let v0 = verts[tri[0] as usize];
        let v1 = verts[tri[1] as usize];
        let v2 = verts[tri[2] as usize];
        if edges_are_collinear(v0, v1, v2, threshold) {
            dissolved_count += 1;
        } else {
            kept_tris.push(*tri);
        }
    }

    DissolveResult {
        new_verts: verts.to_vec(),
        new_tris: kept_tris,
        dissolved_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collinear_on_x_axis() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [2.0, 0.0, 0.0];
        assert!(edges_are_collinear(v0, v1, v2, 1e-4));
    }

    #[test]
    fn test_not_collinear_triangle() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.5, 1.0, 0.0];
        assert!(!edges_are_collinear(v0, v1, v2, 1e-4));
    }

    #[test]
    fn test_dissolve_removes_degenerate() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        // tri 0: degenerate (collinear), tri 1: valid
        let tris = vec![[0u32, 1, 2], [0, 1, 3]];
        let result = dissolve_collinear_edges(&verts, &tris, 1e-4);
        assert_eq!(result.dissolved_count, 1);
        assert_eq!(result.new_tris.len(), 1);
    }

    #[test]
    fn test_dissolve_no_degenerate() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 2]];
        let result = dissolve_collinear_edges(&verts, &tris, 1e-4);
        assert_eq!(result.dissolved_count, 0);
        assert_eq!(result.new_tris.len(), 1);
    }

    #[test]
    fn test_dissolved_edge_count_helper() {
        let r = DissolveResult {
            new_verts: Vec::new(),
            new_tris: Vec::new(),
            dissolved_count: 5,
        };
        assert_eq!(dissolved_edge_count(&r), 5);
    }

    #[test]
    fn test_dissolve_empty() {
        let result = dissolve_collinear_edges(&[], &[], 1e-4);
        assert_eq!(result.dissolved_count, 0);
        assert!(result.new_tris.is_empty());
    }

    #[test]
    fn test_collinear_diagonal_2d() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 1.0, 0.0];
        let v2 = [2.0, 2.0, 0.0];
        assert!(edges_are_collinear(v0, v1, v2, 1e-4));
    }

    #[test]
    fn test_collinear_3d() {
        let v0 = [0.0f32, 0.0, 0.0];
        let v1 = [1.0, 1.0, 1.0];
        let v2 = [2.0, 2.0, 2.0];
        assert!(edges_are_collinear(v0, v1, v2, 1e-4));
    }

    #[test]
    fn test_dissolve_verts_preserved() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let result = dissolve_collinear_edges(&verts, &tris, 1e-4);
        assert_eq!(result.new_verts.len(), verts.len());
    }
}
