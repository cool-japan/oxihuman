#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Transfer UV coordinates from one mesh to another via nearest-point search.

#[allow(dead_code)]
pub fn nearest_uv(
    pos: [f32; 3],
    src_verts: &[[f32; 3]],
    src_uvs: &[[f32; 2]],
) -> [f32; 2] {
    if src_verts.is_empty() {
        return [0.0, 0.0];
    }
    let mut best = 0;
    let mut best_dist = f32::MAX;
    for (i, v) in src_verts.iter().enumerate() {
        let dx = pos[0] - v[0];
        let dy = pos[1] - v[1];
        let dz = pos[2] - v[2];
        let d = dx * dx + dy * dy + dz * dz;
        if d < best_dist {
            best_dist = d;
            best = i;
        }
    }
    if best < src_uvs.len() {
        src_uvs[best]
    } else {
        [0.0, 0.0]
    }
}

#[allow(dead_code)]
pub fn transfer_uv(
    src_verts: &[[f32; 3]],
    src_uvs: &[[f32; 2]],
    dst_verts: &[[f32; 3]],
) -> Vec<[f32; 2]> {
    dst_verts
        .iter()
        .map(|&p| nearest_uv(p, src_verts, src_uvs))
        .collect()
}

#[allow(dead_code)]
pub fn uv_deviation(src_uvs: &[[f32; 2]], dst_uvs: &[[f32; 2]]) -> f32 {
    let n = src_uvs.len().min(dst_uvs.len());
    if n == 0 {
        return 0.0;
    }
    let mut sum = 0.0f32;
    for i in 0..n {
        let du = src_uvs[i][0] - dst_uvs[i][0];
        let dv = src_uvs[i][1] - dst_uvs[i][1];
        sum += (du * du + dv * dv).sqrt();
    }
    sum / n as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src_verts() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
    }

    fn src_uvs() -> Vec<[f32; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
    }

    #[test]
    fn transfer_uv_exact_match() {
        let dst = src_verts();
        let result = transfer_uv(&src_verts(), &src_uvs(), &dst);
        assert!((result[0][0] - 0.0).abs() < 1e-6);
        assert!((result[1][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transfer_uv_count_matches_dst() {
        let dst = vec![[0.5f32, 0.5, 0.0], [0.0, 0.0, 0.0]];
        let result = transfer_uv(&src_verts(), &src_uvs(), &dst);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn nearest_uv_closest_vertex() {
        let result = nearest_uv([0.9, 0.0, 0.0], &src_verts(), &src_uvs());
        assert!((result[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn nearest_uv_empty_src_returns_zero() {
        let result = nearest_uv([1.0, 0.0, 0.0], &[], &[]);
        assert!((result[0]).abs() < 1e-6);
        assert!((result[1]).abs() < 1e-6);
    }

    #[test]
    fn uv_deviation_identical_is_zero() {
        let uvs = src_uvs();
        let dev = uv_deviation(&uvs, &uvs);
        assert!(dev.abs() < 1e-6);
    }

    #[test]
    fn uv_deviation_different() {
        let a = vec![[0.0f32, 0.0], [1.0, 0.0]];
        let b = vec![[1.0f32, 0.0], [0.0, 0.0]];
        let dev = uv_deviation(&a, &b);
        assert!(dev > 0.0);
    }

    #[test]
    fn uv_deviation_empty() {
        let dev = uv_deviation(&[], &[]);
        assert!(dev.abs() < 1e-6);
    }

    #[test]
    fn transfer_uv_empty_dst() {
        let result = transfer_uv(&src_verts(), &src_uvs(), &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn transfer_uv_origin_maps_to_first_uv() {
        let dst = vec![[0.0f32, 0.0, 0.0]];
        let result = transfer_uv(&src_verts(), &src_uvs(), &dst);
        assert!((result[0][0]).abs() < 1e-6);
        assert!((result[0][1]).abs() < 1e-6);
    }
}
