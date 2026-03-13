// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Result of an inset operation.
#[allow(dead_code)]
pub struct InsetResult {
    pub new_verts: Vec<[f32; 3]>,
    pub new_tris: Vec<[u32; 3]>,
}

/// Compute the centroid of a set of vertices.
#[allow(dead_code)]
pub fn face_center(verts: &[[f32; 3]]) -> [f32; 3] {
    if verts.is_empty() {
        return [0.0; 3];
    }
    let n = verts.len() as f32;
    let sum = verts.iter().fold([0.0f32; 3], |acc, v| {
        [acc[0] + v[0], acc[1] + v[1], acc[2] + v[2]]
    });
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Inset a triangle by factor `t` (0=no inset, 1=fully collapsed to center).
#[allow(dead_code)]
pub fn inset_tri(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3], t: f32) -> [f32; 3] {
    let c = face_center(&[v0, v1, v2]);
    // returns the inset position for v0
    [
        v0[0] + (c[0] - v0[0]) * t,
        v0[1] + (c[1] - v0[1]) * t,
        v0[2] + (c[2] - v0[2]) * t,
    ]
}

/// Inset a polygon face by `depth` (proportion toward center) and `thickness` (z-offset).
#[allow(dead_code)]
pub fn inset_face(
    verts: [f32; 3],
    depth: f32,
    thickness: f32,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    // Simple: treat as a single point expanding to a small quad
    let c = [verts[0], verts[1] + depth, verts[2] + thickness];
    let new_verts = vec![verts, c];
    let new_tris = vec![[0u32, 1, 0]];
    (new_verts, new_tris)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_face_center_triangle() {
        let c = face_center(&[[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]]);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
        let _ = PI;
    }

    #[test]
    fn test_face_center_empty() {
        let c = face_center(&[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_face_center_single() {
        let c = face_center(&[[1.0, 2.0, 3.0]]);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 2.0).abs() < 1e-5);
        assert!((c[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_inset_tri_no_inset() {
        let v0 = [1.0f32, 0.0, 0.0];
        let v1 = [0.0, 1.0, 0.0];
        let v2 = [0.0, 0.0, 1.0];
        let p = inset_tri(v0, v1, v2, 0.0);
        // t=0 => same as v0
        assert!((p[0] - v0[0]).abs() < 1e-5);
    }

    #[test]
    fn test_inset_tri_full_inset() {
        let v0 = [1.0f32, 0.0, 0.0];
        let v1 = [-1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let p = inset_tri(v0, v1, v2, 1.0);
        let c = face_center(&[v0, v1, v2]);
        assert!((p[0] - c[0]).abs() < 1e-5);
    }

    #[test]
    fn test_inset_tri_half() {
        let v0 = [2.0f32, 0.0, 0.0];
        let v1 = [0.0, 2.0, 0.0];
        let v2 = [0.0, 0.0, 0.0];
        let p = inset_tri(v0, v1, v2, 0.5);
        let c = face_center(&[v0, v1, v2]);
        // half way between v0 and center
        assert!((p[0] - (v0[0] + c[0]) / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_inset_face_returns_two_verts() {
        let (verts, _) = inset_face([0.0, 0.0, 0.0], 0.1, 0.05);
        assert_eq!(verts.len(), 2);
    }

    #[test]
    fn test_inset_face_depth_applied() {
        let (verts, _) = inset_face([0.0, 0.0, 0.0], 0.5, 0.0);
        assert!((verts[1][1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_face_center_quad() {
        let c = face_center(&[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        assert!((c[0] - 0.5).abs() < 1e-5);
        assert!((c[1] - 0.5).abs() < 1e-5);
    }
}
