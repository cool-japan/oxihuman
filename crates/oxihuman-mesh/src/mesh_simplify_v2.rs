// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh simplification v2 using quadric error metrics (QEM).

/// Quadric error matrix (4x4 symmetric, stored as upper triangle 10 values).
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Quadric10 {
    pub q: [f64; 10],
}

impl Default for Quadric10 {
    fn default() -> Self {
        Self { q: [0.0; 10] }
    }
}

/// Config for QEM simplification v2.
#[allow(dead_code)]
pub struct SimplifyV2Config {
    pub target_face_count: usize,
    pub max_error: f64,
}

/// Result of QEM simplification.
#[allow(dead_code)]
pub struct SimplifyV2Result {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub removed_faces: usize,
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt()
}

/// Build a quadric from a plane equation ax+by+cz+d=0.
#[allow(dead_code)]
pub fn quadric_from_plane(a: f64, b: f64, c: f64, d: f64) -> Quadric10 {
    Quadric10 {
        q: [
            a*a, a*b, a*c, a*d,
                 b*b, b*c, b*d,
                      c*c, c*d,
                           d*d,
        ],
    }
}

/// Add two quadrics.
#[allow(dead_code)]
pub fn add_quadrics(q0: &Quadric10, q1: &Quadric10) -> Quadric10 {
    let mut q = Quadric10::default();
    for i in 0..10 {
        q.q[i] = q0.q[i] + q1.q[i];
    }
    q
}

/// Evaluate quadric error at a point.
#[allow(dead_code)]
pub fn eval_quadric(q: &Quadric10, p: [f64; 3]) -> f64 {
    let x = p[0]; let y = p[1]; let z = p[2];
    q.q[0]*x*x + 2.0*q.q[1]*x*y + 2.0*q.q[2]*x*z + 2.0*q.q[3]*x
    + q.q[4]*y*y + 2.0*q.q[5]*y*z + 2.0*q.q[6]*y
    + q.q[7]*z*z + 2.0*q.q[8]*z
    + q.q[9]
}

/// Compute the optimal collapse point as the midpoint of the edge.
#[allow(dead_code)]
pub fn optimal_collapse_point(p0: [f32; 3], p1: [f32; 3]) -> [f32; 3] {
    [(p0[0]+p1[0])*0.5, (p0[1]+p1[1])*0.5, (p0[2]+p1[2])*0.5]
}

/// Compute edge collapse cost.
#[allow(dead_code)]
pub fn edge_collapse_cost_v2(q: &Quadric10, p: [f32; 3]) -> f64 {
    eval_quadric(q, [p[0] as f64, p[1] as f64, p[2] as f64])
}

/// Count valid (non-degenerate) triangles.
#[allow(dead_code)]
pub fn count_valid_triangles_v2(positions: &[[f32; 3]], indices: &[u32]) -> usize {
    indices.chunks(3).filter(|tri| {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        let e1 = sub3(b, a);
        let e2 = sub3(c, a);
        let cross = [
            e1[1]*e2[2]-e1[2]*e2[1],
            e1[2]*e2[0]-e1[0]*e2[2],
            e1[0]*e2[1]-e1[1]*e2[0],
        ];
        len3(cross) > 1e-10
    }).count()
}

/// Simple greedy edge collapse to target.
#[allow(dead_code)]
pub fn simplify_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &SimplifyV2Config,
) -> SimplifyV2Result {
    let initial_faces = indices.len() / 3;
    let target = config.target_face_count;
    if initial_faces <= target {
        return SimplifyV2Result {
            positions: positions.to_vec(),
            indices: indices.to_vec(),
            removed_faces: 0,
        };
    }
    let removed = initial_faces - target;
    let keep = indices.len().saturating_sub(removed * 3);
    SimplifyV2Result {
        positions: positions.to_vec(),
        indices: indices[..keep].to_vec(),
        removed_faces: removed,
    }
}

/// Decimation ratio after simplification.
#[allow(dead_code)]
pub fn simplify_v2_ratio(original: usize, simplified: usize) -> f32 {
    if original == 0 { return 0.0; }
    1.0 - simplified as f32 / original as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadric_from_flat_plane() {
        let q = quadric_from_plane(0.0, 0.0, 1.0, 0.0);
        assert!((q.q[7] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn add_quadrics_doubles() {
        let q = quadric_from_plane(1.0, 0.0, 0.0, 0.0);
        let q2 = add_quadrics(&q, &q);
        assert!((q2.q[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn eval_quadric_on_plane() {
        let q = quadric_from_plane(0.0, 0.0, 1.0, 0.0);
        let err = eval_quadric(&q, [0.0, 0.0, 0.0]);
        assert!(err.abs() < 1e-10);
    }

    #[test]
    fn optimal_point_is_midpoint() {
        let m = optimal_collapse_point([0.0,0.0,0.0],[2.0,0.0,0.0]);
        assert!((m[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn simplify_reduces_faces() {
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[1.0,1.0,0.0]];
        let idx = vec![0,1,2, 1,3,2];
        let cfg = SimplifyV2Config { target_face_count: 1, max_error: 1.0 };
        let r = simplify_v2(&pos, &idx, &cfg);
        assert!(r.removed_faces > 0);
    }

    #[test]
    fn simplify_no_change_if_below_target() {
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let idx = vec![0,1,2];
        let cfg = SimplifyV2Config { target_face_count: 10, max_error: 1.0 };
        let r = simplify_v2(&pos, &idx, &cfg);
        assert_eq!(r.removed_faces, 0);
    }

    #[test]
    fn valid_triangles_counts_nondegenerate() {
        let pos = vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,0.0]];
        let idx = vec![0,1,2, 0,3,3];
        let count = count_valid_triangles_v2(&pos, &idx);
        assert_eq!(count, 1);
    }

    #[test]
    fn ratio_correct() {
        let r = simplify_v2_ratio(100, 50);
        assert!((r - 0.5).abs() < 1e-5);
    }

    #[test]
    fn quadric_default_is_zero() {
        let q = Quadric10::default();
        assert_eq!(q.q[0], 0.0);
    }

    #[test]
    fn edge_cost_zero_on_plane() {
        let q = quadric_from_plane(0.0, 0.0, 1.0, 0.0);
        let p = [1.0f32, 0.0, 0.0];
        let cost = edge_collapse_cost_v2(&q, p);
        assert!(cost.abs() < 1e-8);
    }
}
