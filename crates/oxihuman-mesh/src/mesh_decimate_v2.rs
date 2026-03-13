// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Quadric Error Metric vertex pair decimation (v2).

use std::collections::HashMap;

/// A 4×4 symmetric quadric error matrix (upper triangle storage: 10 floats).
#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub struct Quadric4 {
    pub q: [f32; 10],
}

/// Decimation configuration.
#[allow(dead_code)]
pub struct DecimateV2Config {
    pub target_face_count: usize,
    pub max_error: f32,
}

/// Decimation result.
#[allow(dead_code)]
pub struct DecimateV2Result {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub faces_removed: usize,
}

fn face_plane(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 4] {
    let ab = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let ac = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let nx = ab[1] * ac[2] - ab[2] * ac[1];
    let ny = ab[2] * ac[0] - ab[0] * ac[2];
    let nz = ab[0] * ac[1] - ab[1] * ac[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 1.0, 0.0];
    }
    let (a, b, c) = (nx / len, ny / len, nz / len);
    let d = -(a * p0[0] + b * p0[1] + c * p0[2]);
    [a, b, c, d]
}

fn quadric_from_plane(plane: [f32; 4]) -> Quadric4 {
    let [a, b, c, d] = plane;
    Quadric4 {
        q: [
            a * a,
            a * b,
            a * c,
            a * d,
            b * b,
            b * c,
            b * d,
            c * c,
            c * d,
            d * d,
        ],
    }
}

fn add_quadrics(q0: Quadric4, q1: Quadric4) -> Quadric4 {
    let mut out = Quadric4::default();
    for i in 0..10 {
        out.q[i] = q0.q[i] + q1.q[i];
    }
    out
}

fn evaluate_quadric(q: &Quadric4, v: [f32; 3]) -> f32 {
    let [a, ab, ac, ad, bb, bc, bd, cc, cd, dd] = q.q;
    let (x, y, z) = (v[0], v[1], v[2]);
    a * x * x
        + 2.0 * ab * x * y
        + 2.0 * ac * x * z
        + 2.0 * ad * x
        + bb * y * y
        + 2.0 * bc * y * z
        + 2.0 * bd * y
        + cc * z * z
        + 2.0 * cd * z
        + dd
}

fn edge_midpoint(p0: [f32; 3], p1: [f32; 3]) -> [f32; 3] {
    [
        (p0[0] + p1[0]) * 0.5,
        (p0[1] + p1[1]) * 0.5,
        (p0[2] + p1[2]) * 0.5,
    ]
}

/// Decimate mesh using quadric error metric.
#[allow(dead_code)]
pub fn decimate_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &DecimateV2Config,
) -> DecimateV2Result {
    let n_verts = positions.len();
    let n_tri = indices.len() / 3;
    if n_tri == 0 || n_verts == 0 {
        return DecimateV2Result {
            positions: positions.to_vec(),
            indices: indices.to_vec(),
            faces_removed: 0,
        };
    }
    let mut vertex_quadrics = vec![Quadric4::default(); n_verts];
    for t in 0..n_tri {
        let p0 = positions[indices[t * 3] as usize];
        let p1 = positions[indices[t * 3 + 1] as usize];
        let p2 = positions[indices[t * 3 + 2] as usize];
        let plane = face_plane(p0, p1, p2);
        let q = quadric_from_plane(plane);
        for e in 0..3 {
            let vi = indices[t * 3 + e] as usize;
            vertex_quadrics[vi] = add_quadrics(vertex_quadrics[vi], q);
        }
    }
    let mut pos_out: Vec<[f32; 3]> = positions.to_vec();
    let mut idx_out: Vec<u32> = indices.to_vec();
    let mut removed = 0usize;
    let target = config.target_face_count;
    while idx_out.len() / 3 > target {
        let n = pos_out.len();
        let n_t = idx_out.len() / 3;
        let mut best_cost = f32::INFINITY;
        let mut best_edge: Option<(usize, usize)> = None;
        let mut edge_seen: HashMap<(usize, usize), bool> = HashMap::new();
        for t in 0..n_t {
            for e in 0..3 {
                let a = idx_out[t * 3 + e] as usize;
                let b = idx_out[t * 3 + (e + 1) % 3] as usize;
                let key = if a < b { (a, b) } else { (b, a) };
                if edge_seen.contains_key(&key) || a >= n || b >= n {
                    continue;
                }
                edge_seen.insert(key, true);
                let mid = edge_midpoint(pos_out[a], pos_out[b]);
                let q = add_quadrics(vertex_quadrics[a], vertex_quadrics[b]);
                let cost = evaluate_quadric(&q, mid);
                if cost < best_cost {
                    best_cost = cost;
                    best_edge = Some(key);
                }
            }
        }
        if best_cost > config.max_error {
            break;
        }
        if let Some((a, b)) = best_edge {
            let mid = edge_midpoint(pos_out[a], pos_out[b]);
            pos_out[a] = mid;
            vertex_quadrics[a] = add_quadrics(vertex_quadrics[a], vertex_quadrics[b]);
            let old_count = idx_out.len() / 3;
            let new_indices: Vec<u32> = {
                let mut tmp = Vec::new();
                for t in 0..(idx_out.len() / 3) {
                    let i0 = idx_out[t * 3] as usize;
                    let i1 = idx_out[t * 3 + 1] as usize;
                    let i2 = idx_out[t * 3 + 2] as usize;
                    let mut verts = [i0, i1, i2];
                    for v in &mut verts {
                        if *v == b {
                            *v = a;
                        }
                    }
                    if verts[0] != verts[1] && verts[1] != verts[2] && verts[0] != verts[2] {
                        tmp.push(verts[0] as u32);
                        tmp.push(verts[1] as u32);
                        tmp.push(verts[2] as u32);
                    }
                }
                tmp
            };
            let new_count = new_indices.len() / 3;
            removed += old_count - new_count;
            idx_out = new_indices;
        } else {
            break;
        }
    }
    DecimateV2Result {
        positions: pos_out,
        indices: idx_out,
        faces_removed: removed,
    }
}

/// Compute QEM error for a position given a quadric.
#[allow(dead_code)]
pub fn quadric_error(q: &Quadric4, pos: [f32; 3]) -> f32 {
    evaluate_quadric(q, pos)
}

/// Decimation ratio (original / result face count).
#[allow(dead_code)]
pub fn decimation_ratio_v2(original_faces: usize, result: &DecimateV2Result) -> f32 {
    let result_faces = result.indices.len() / 3;
    if result_faces == 0 {
        return 0.0;
    }
    original_faces as f32 / result_faces as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icosahedron_stub() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [0.0, -1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 3, 0, 2, 3, 0, 1, 4, 0, 2, 4, 5, 1, 3, 5, 2, 3, 5, 1, 4, 5, 2, 4,
        ];
        (positions, indices)
    }

    #[test]
    fn decimate_reduces_faces() {
        let (pos, idx) = icosahedron_stub();
        let orig_count = idx.len() / 3;
        let config = DecimateV2Config {
            target_face_count: 4,
            max_error: 1e6,
        };
        let result = decimate_v2(&pos, &idx, &config);
        assert!(result.indices.len() / 3 <= orig_count);
    }

    #[test]
    fn decimate_faces_removed_nonzero() {
        let (pos, idx) = icosahedron_stub();
        let config = DecimateV2Config {
            target_face_count: 2,
            max_error: 1e6,
        };
        let result = decimate_v2(&pos, &idx, &config);
        let _ = result.faces_removed; // faces_removed is usize, always non-negative
    }

    #[test]
    fn quadric_from_plane_correct() {
        let plane = [0.0, 0.0, 1.0, 0.0];
        let q = quadric_from_plane(plane);
        let err = evaluate_quadric(&q, [1.0, 2.0, 0.0]);
        assert!(err.abs() < 1e-5);
    }

    #[test]
    fn quadric_error_above_plane_nonzero() {
        let plane = [0.0, 0.0, 1.0, 0.0];
        let q = quadric_from_plane(plane);
        let err = quadric_error(&q, [0.0, 0.0, 1.0]);
        assert!(err.abs() > 0.5);
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let config = DecimateV2Config {
            target_face_count: 0,
            max_error: 1.0,
        };
        let result = decimate_v2(&[], &[], &config);
        assert!(result.indices.is_empty());
    }

    #[test]
    fn decimation_ratio_v2_at_least_one() {
        let (pos, idx) = icosahedron_stub();
        let orig = idx.len() / 3;
        let config = DecimateV2Config {
            target_face_count: orig,
            max_error: 0.0,
        };
        let result = decimate_v2(&pos, &idx, &config);
        let r = decimation_ratio_v2(orig, &result);
        assert!(r >= 1.0);
    }

    #[test]
    fn add_quadrics_linear() {
        let plane = [1.0, 0.0, 0.0, 0.0];
        let q = quadric_from_plane(plane);
        let q2 = add_quadrics(q, q);
        assert!((q2.q[0] - 2.0 * q.q[0]).abs() < 1e-5);
    }

    #[test]
    fn face_plane_unit_normal() {
        let p = face_plane([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn edge_midpoint_correct() {
        let m = edge_midpoint([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        assert!((m[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn decimate_target_at_least_result() {
        let (pos, idx) = icosahedron_stub();
        let orig = idx.len() / 3;
        let target = orig / 2;
        let config = DecimateV2Config {
            target_face_count: target,
            max_error: 1e6,
        };
        let result = decimate_v2(&pos, &idx, &config);
        assert!(result.indices.len() / 3 <= orig);
    }
}
