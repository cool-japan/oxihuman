// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Transfer (bake) normals from a hi-res mesh to a lo-res mesh.

#[allow(dead_code)]
pub struct NormalTransferResult {
    pub vertex_normals: Vec<[f32; 3]>,
}

fn normalize_vec(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 { return [0.0, 0.0, 1.0]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[allow(dead_code)]
pub fn nt_compute_face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    normalize_vec(cross)
}

fn face_centroid(positions: &[[f32; 3]], tri: &[u32; 3]) -> [f32; 3] {
    let a = tri[0] as usize;
    let b = tri[1] as usize;
    let c = tri[2] as usize;
    [
        (positions[a][0] + positions[b][0] + positions[c][0]) / 3.0,
        (positions[a][1] + positions[b][1] + positions[c][1]) / 3.0,
        (positions[a][2] + positions[b][2] + positions[c][2]) / 3.0,
    ]
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

#[allow(dead_code)]
pub fn nt_transfer(
    src_positions: &[[f32; 3]],
    src_indices: &[[u32; 3]],
    dst_positions: &[[f32; 3]],
) -> NormalTransferResult {
    let mut vertex_normals = Vec::with_capacity(dst_positions.len());
    for dst_p in dst_positions {
        let normal = if src_indices.is_empty() {
            [0.0, 0.0, 1.0]
        } else {
            let mut best_face = 0;
            let mut best_dist = f32::MAX;
            for (i, tri) in src_indices.iter().enumerate() {
                let a = tri[0] as usize;
                let b = tri[1] as usize;
                let c = tri[2] as usize;
                if a < src_positions.len() && b < src_positions.len() && c < src_positions.len() {
                    let cen = face_centroid(src_positions, tri);
                    let d = dist_sq(*dst_p, cen);
                    if d < best_dist {
                        best_dist = d;
                        best_face = i;
                    }
                }
            }
            let tri = &src_indices[best_face];
            let a = tri[0] as usize;
            let b = tri[1] as usize;
            let c = tri[2] as usize;
            if a < src_positions.len() && b < src_positions.len() && c < src_positions.len() {
                nt_compute_face_normal(src_positions[a], src_positions[b], src_positions[c])
            } else {
                [0.0, 0.0, 1.0]
            }
        };
        vertex_normals.push(normal);
    }
    NormalTransferResult { vertex_normals }
}

#[allow(dead_code)]
pub fn nt_normal_count(result: &NormalTransferResult) -> usize {
    result.vertex_normals.len()
}

#[allow(dead_code)]
pub fn nt_average_normal(result: &NormalTransferResult) -> [f32; 3] {
    let n = result.vertex_normals.len();
    if n == 0 { return [0.0, 0.0, 1.0]; }
    let mut sum = [0f32; 3];
    for v in &result.vertex_normals {
        sum[0] += v[0]; sum[1] += v[1]; sum[2] += v[2];
    }
    normalize_vec([sum[0] / n as f32, sum[1] / n as f32, sum[2] / n as f32])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_normal_z_up() {
        let n = nt_compute_face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((n[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_normal_count_matches_vertex_count() {
        let src_pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let src_idx = vec![[0u32, 1, 2]];
        let dst_pos = vec![[0.5, 0.2, 0.0], [0.1, 0.1, 0.0]];
        let result = nt_transfer(&src_pos, &src_idx, &dst_pos);
        assert_eq!(nt_normal_count(&result), 2);
    }

    #[test]
    fn test_transfer_empty_dst() {
        let src_pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let src_idx = vec![[0u32, 1, 2]];
        let result = nt_transfer(&src_pos, &src_idx, &[]);
        assert_eq!(nt_normal_count(&result), 0);
    }

    #[test]
    fn test_transfer_no_src_faces() {
        let dst_pos = vec![[0.0, 0.0, 0.0]];
        let result = nt_transfer(&[], &[], &dst_pos);
        assert_eq!(result.vertex_normals[0], [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_average_normal_single() {
        let result = NormalTransferResult { vertex_normals: vec![[0.0, 0.0, 1.0]] };
        let avg = nt_average_normal(&result);
        assert!((avg[2] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_average_normal_empty() {
        let result = NormalTransferResult { vertex_normals: vec![] };
        let avg = nt_average_normal(&result);
        assert_eq!(avg, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_face_normal_normalized() {
        let n = nt_compute_face_normal([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
        let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_transfer_z_up_flat_mesh() {
        let src_pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let src_idx = vec![[0u32, 1, 2]];
        let dst_pos = vec![[0.3, 0.3, 0.0]];
        let result = nt_transfer(&src_pos, &src_idx, &dst_pos);
        assert!((result.vertex_normals[0][2] - 1.0).abs() < 1e-4);
    }
}
