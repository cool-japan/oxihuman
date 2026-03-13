#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Transfer face normals from source to target mesh.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceNormalTransfer {
    pub source_normals: Vec<[f32; 3]>,
    pub target_normals: Vec<[f32; 3]>,
}

#[allow(dead_code)]
pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[allow(dead_code)]
pub fn find_nearest_normal(
    pos: [f32; 3],
    src_verts: &[[f32; 3]],
    src_normals: &[[f32; 3]],
) -> [f32; 3] {
    if src_verts.is_empty() {
        return [0.0, 1.0, 0.0];
    }
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, v) in src_verts.iter().enumerate() {
        let dx = pos[0] - v[0];
        let dy = pos[1] - v[1];
        let dz = pos[2] - v[2];
        let d2 = dx * dx + dy * dy + dz * dz;
        if d2 < best_dist {
            best_dist = d2;
            best_idx = i;
        }
    }
    if best_idx < src_normals.len() {
        normalize3(src_normals[best_idx])
    } else {
        [0.0, 1.0, 0.0]
    }
}

#[allow(dead_code)]
pub fn transfer_normals(
    src_verts: &[[f32; 3]],
    src_normals: &[[f32; 3]],
    dst_verts: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    dst_verts
        .iter()
        .map(|v| find_nearest_normal(*v, src_verts, src_normals))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize3_unit_y() {
        let n = normalize3([0.0, 5.0, 0.0]);
        assert!((n[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normalize3_zero_gives_fallback() {
        let n = normalize3([0.0, 0.0, 0.0]);
        assert!((n[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn find_nearest_empty_src() {
        let n = find_nearest_normal([0.0, 0.0, 0.0], &[], &[]);
        assert!((n[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn find_nearest_single_vertex() {
        let verts = vec![[0.0f32, 0.0, 0.0]];
        let normals = vec![[0.0f32, 0.0, 1.0]];
        let n = find_nearest_normal([1.0, 0.0, 0.0], &verts, &normals);
        assert!((n[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transfer_normals_count_matches_dst() {
        let src_v = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let src_n = vec![[0.0f32, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let dst_v = vec![[0.1f32, 0.0, 0.0], [0.9, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let out = transfer_normals(&src_v, &src_n, &dst_v);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn transfer_normals_nearest_match() {
        let src_v = vec![[0.0f32, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let src_n = vec![[0.0f32, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let dst_v = vec![[0.1f32, 0.0, 0.0]];
        let out = transfer_normals(&src_v, &src_n, &dst_v);
        assert!((out[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transfer_empty_dst() {
        let src_v = vec![[0.0f32, 0.0, 0.0]];
        let src_n = vec![[0.0f32, 1.0, 0.0]];
        let out = transfer_normals(&src_v, &src_n, &[]);
        assert!(out.is_empty());
    }

    #[test]
    fn normalize3_diagonal() {
        let n = normalize3([1.0, 1.0, 1.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn face_normal_transfer_struct() {
        let fnt = FaceNormalTransfer {
            source_normals: vec![[0.0, 1.0, 0.0]],
            target_normals: vec![[0.0, 0.0, 1.0]],
        };
        assert_eq!(fnt.source_normals.len(), 1);
        assert_eq!(fnt.target_normals.len(), 1);
    }
}
