// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Recalculate face/vertex normals.

#[derive(Debug, Clone)]
pub struct RecalcNormalsResult {
    pub vertex_normals: Vec<[f32; 3]>,
    pub face_normals: Vec<[f32; 3]>,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute per-face normals.
pub fn compute_face_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    indices
        .chunks(3)
        .filter(|c| c.len() == 3)
        .map(|tri| {
            let a = positions[tri[0] as usize];
            let b = positions[tri[1] as usize];
            let c = positions[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            normalize3(cross3(ab, ac))
        })
        .collect()
}

/// Compute per-vertex normals (area-weighted average of adjacent face normals).
pub fn compute_vertex_normals_recalc(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut accum = vec![[0.0f32; 3]; n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let fac = cross3(ab, ac);
        for idx in [a, b, c] {
            accum[idx][0] += fac[0];
            accum[idx][1] += fac[1];
            accum[idx][2] += fac[2];
        }
    }
    accum.iter().map(|&v| normalize3(v)).collect()
}

/// Compute angle-weighted vertex normals.
pub fn compute_angle_weighted_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut accum = vec![[0.0f32; 3]; n];
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        let (ai, bi, ci) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if ai >= n || bi >= n || ci >= n {
            continue;
        }
        let verts = [positions[ai], positions[bi], positions[ci]];
        let face_n = {
            let ab = [
                verts[1][0] - verts[0][0],
                verts[1][1] - verts[0][1],
                verts[1][2] - verts[0][2],
            ];
            let ac = [
                verts[2][0] - verts[0][0],
                verts[2][1] - verts[0][1],
                verts[2][2] - verts[0][2],
            ];
            normalize3(cross3(ab, ac))
        };
        let idxs = [ai, bi, ci];
        for k in 0..3 {
            let prev = verts[(k + 2) % 3];
            let curr = verts[k];
            let next = verts[(k + 1) % 3];
            let d0 = normalize3([prev[0] - curr[0], prev[1] - curr[1], prev[2] - curr[2]]);
            let d1 = normalize3([next[0] - curr[0], next[1] - curr[1], next[2] - curr[2]]);
            let angle = dot3(d0, d1).clamp(-1.0, 1.0).acos();
            let w = angle;
            accum[idxs[k]][0] += face_n[0] * w;
            accum[idxs[k]][1] += face_n[1] * w;
            accum[idxs[k]][2] += face_n[2] * w;
        }
    }
    accum.iter().map(|&v| normalize3(v)).collect()
}

/// Recalculate all normals for a mesh.
pub fn recalc_normals(positions: &[[f32; 3]], indices: &[u32]) -> RecalcNormalsResult {
    let face_normals = compute_face_normals(positions, indices);
    let vertex_normals = compute_vertex_normals_recalc(positions, indices);
    RecalcNormalsResult {
        vertex_normals,
        face_normals,
    }
}

/// Smooth existing normals using Laplacian averaging.
pub fn smooth_normals_recalc(
    normals: &[[f32; 3]],
    adj: &[Vec<usize>],
    factor: f32,
) -> Vec<[f32; 3]> {
    let mut out = normals.to_vec();
    for (i, nbs) in adj.iter().enumerate() {
        if nbs.is_empty() {
            continue;
        }
        let n = nbs.len() as f32;
        let avg = nbs.iter().fold([0.0f32; 3], |a, &j| {
            [
                a[0] + normals[j][0],
                a[1] + normals[j][1],
                a[2] + normals[j][2],
            ]
        });
        let avg = normalize3([avg[0] / n, avg[1] / n, avg[2] / n]);
        let cur = normals[i];
        out[i] = normalize3([
            cur[0] + factor * (avg[0] - cur[0]),
            cur[1] + factor * (avg[1] - cur[1]),
            cur[2] + factor * (avg[2] - cur[2]),
        ]);
    }
    out
}

pub fn normal_deviation(a: [f32; 3], b: [f32; 3]) -> f32 {
    dot3(normalize3(a), normalize3(b)).clamp(-1.0, 1.0).acos()
}

pub fn average_face_normal(face_normals: &[[f32; 3]]) -> [f32; 3] {
    if face_normals.is_empty() {
        return [0.0, 0.0, 1.0];
    }
    let n = face_normals.len() as f32;
    let s = face_normals
        .iter()
        .fold([0.0f32; 3], |a, &v| [a[0] + v[0], a[1] + v[1], a[2] + v[2]]);
    normalize3([s[0] / n, s[1] / n, s[2] / n])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0u32, 1, 2],
        )
    }

    #[test]
    fn test_compute_face_normals_z() {
        let (pos, idx) = flat_mesh();
        let fn_ = compute_face_normals(&pos, &idx);
        assert_eq!(fn_.len(), 1);
        assert!(fn_[0][2].abs() > 0.5);
    }

    #[test]
    fn test_compute_vertex_normals_count() {
        let (pos, idx) = flat_mesh();
        let vn = compute_vertex_normals_recalc(&pos, &idx);
        assert_eq!(vn.len(), 3);
    }

    #[test]
    fn test_recalc_normals() {
        let (pos, idx) = flat_mesh();
        let res = recalc_normals(&pos, &idx);
        assert_eq!(res.face_normals.len(), 1);
        assert_eq!(res.vertex_normals.len(), 3);
    }

    #[test]
    fn test_normal_deviation_same() {
        let n = [0.0f32, 0.0, 1.0];
        assert!(normal_deviation(n, n).abs() < 1e-5);
    }

    #[test]
    fn test_normal_deviation_perpendicular() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0f32, 1.0, 0.0];
        let d = normal_deviation(a, b);
        assert!((d - std::f32::consts::PI / 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_average_face_normal() {
        let n = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let avg = average_face_normal(&n);
        assert!((avg[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_weighted_normals_count() {
        let (pos, idx) = flat_mesh();
        let vn = compute_angle_weighted_normals(&pos, &idx);
        assert_eq!(vn.len(), 3);
    }

    #[test]
    fn test_smooth_normals_recalc() {
        let n = vec![[0.0f32, 0.0, 1.0], [0.0, 0.0, 1.0]];
        let adj = vec![vec![1usize], vec![0]];
        let out = smooth_normals_recalc(&n, &adj, 0.5);
        assert_eq!(out.len(), 2);
    }
}
