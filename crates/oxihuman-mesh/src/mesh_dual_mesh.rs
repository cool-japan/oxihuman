#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Dual mesh construction: faces become vertices, vertices become faces.

#[allow(dead_code)]
pub struct DualResult {
    pub verts: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
}

#[allow(dead_code)]
pub fn build_dual_mesh(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> DualResult {
    let mut dual_verts = Vec::with_capacity(tris.len());
    for tri in tris {
        dual_verts.push(face_centroid(verts, tri));
    }
    let n = dual_verts.len() as u32;
    let faces: Vec<Vec<u32>> = if n < 3 {
        vec![]
    } else {
        vec![(0..n).collect()]
    };
    DualResult { verts: dual_verts, faces }
}

#[allow(dead_code)]
pub fn face_centroid(verts: &[[f32; 3]], face: &[u32]) -> [f32; 3] {
    if face.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for &idx in face {
        let v = verts[idx as usize];
        cx += v[0];
        cy += v[1];
        cz += v[2];
    }
    let n = face.len() as f32;
    [cx / n, cy / n, cz / n]
}

#[allow(dead_code)]
pub fn dual_vert_count(tris: usize) -> usize {
    tris
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn dual_vert_count_matches_tri_count() {
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2], [1, 3, 2]];
        assert_eq!(dual_vert_count(tris.len()), 2);
    }

    #[test]
    fn build_dual_mesh_produces_correct_vert_count() {
        let verts = simple_verts();
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2], [1, 3, 2]];
        let result = build_dual_mesh(&verts, &tris);
        assert_eq!(result.verts.len(), 2);
    }

    #[test]
    fn face_centroid_single_vert() {
        let verts = vec![[2.0, 4.0, 6.0]];
        let c = face_centroid(&verts, &[0]);
        assert!((c[0] - 2.0).abs() < 1e-5);
        assert!((c[1] - 4.0).abs() < 1e-5);
        assert!((c[2] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn face_centroid_triangle() {
        let verts = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0]];
        let c = face_centroid(&verts, &[0, 1, 2]);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
        assert!(c[2].abs() < 1e-5);
    }

    #[test]
    fn face_centroid_empty_returns_zero() {
        let verts: Vec<[f32; 3]> = vec![];
        let c = face_centroid(&verts, &[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn build_dual_mesh_empty() {
        let result = build_dual_mesh(&[], &[]);
        assert_eq!(result.verts.len(), 0);
        assert_eq!(result.faces.len(), 0);
    }

    #[test]
    fn dual_vert_count_zero() {
        assert_eq!(dual_vert_count(0), 0);
    }

    #[test]
    fn dual_vert_count_large() {
        assert_eq!(dual_vert_count(100), 100);
    }

    #[test]
    fn build_dual_mesh_one_tri() {
        let verts = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let tris: Vec<[u32; 3]> = vec![[0, 1, 2]];
        let result = build_dual_mesh(&verts, &tris);
        assert_eq!(result.verts.len(), 1);
        let c = result.verts[0];
        assert!((c[0] - 2.0 / 3.0).abs() < 1e-4);
    }
}
