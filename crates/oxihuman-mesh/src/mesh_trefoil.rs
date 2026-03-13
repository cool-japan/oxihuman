// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Trefoil knot mesh generation.

use std::f32::consts::PI;

/// Parameters for a trefoil knot tube mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrefoilParams {
    pub tube_radius: f32,
    pub knot_radius: f32,
    pub path_segs: usize,
    pub ring_segs: usize,
}

/// A trefoil knot mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TrefoilMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Evaluate the trefoil knot curve at parameter t ∈ [0, 2π).
#[allow(dead_code)]
pub fn trefoil_point(t: f32, r: f32) -> [f32; 3] {
    let (st, ct) = t.sin_cos();
    let (s2t, c2t) = (2.0 * t).sin_cos();
    [
        r * (ct + 2.0 * c2t),
        r * (st - 2.0 * s2t),
        r * (-(3.0 * t).sin()),
    ]
}

/// Sample the trefoil spine path.
#[allow(dead_code)]
pub fn trefoil_path(params: &TrefoilParams) -> Vec<[f32; 3]> {
    let n = params.path_segs.max(3);
    (0..=n)
        .map(|i| {
            let t = 2.0 * PI * i as f32 / n as f32;
            trefoil_point(t, params.knot_radius)
        })
        .collect()
}

/// Build a tube mesh along the trefoil knot.
#[allow(dead_code)]
pub fn build_trefoil_mesh(params: &TrefoilParams) -> TrefoilMesh {
    let path = trefoil_path(params);
    let n_ring = params.ring_segs.max(3);
    let n_path = path.len();

    let mut positions = Vec::with_capacity(n_path * n_ring);
    for (seg, &p) in path.iter().enumerate() {
        let next = if seg + 1 < n_path {
            path[seg + 1]
        } else {
            path[1]
        };
        let fwd = normalize3([next[0] - p[0], next[1] - p[1], next[2] - p[2]]);
        let (right, up) = frame_from_forward(fwd);
        for i in 0..n_ring {
            let angle = 2.0 * PI * i as f32 / n_ring as f32;
            let (s, c) = angle.sin_cos();
            positions.push([
                p[0] + (right[0] * c + up[0] * s) * params.tube_radius,
                p[1] + (right[1] * c + up[1] * s) * params.tube_radius,
                p[2] + (right[2] * c + up[2] * s) * params.tube_radius,
            ]);
        }
    }

    let mut indices = Vec::new();
    for seg in 0..(n_path - 1) {
        let b0 = (seg * n_ring) as u32;
        let b1 = ((seg + 1) * n_ring) as u32;
        for i in 0..n_ring {
            let a = b0 + i as u32;
            let b = b0 + ((i + 1) % n_ring) as u32;
            let c = b1 + i as u32;
            let d = b1 + ((i + 1) % n_ring) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    TrefoilMesh { positions, indices }
}

/// Vertex count.
#[allow(dead_code)]
pub fn trefoil_vertex_count(m: &TrefoilMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn trefoil_triangle_count(m: &TrefoilMesh) -> usize {
    m.indices.len() / 3
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn frame_from_forward(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let hint = if fwd[1].abs() < 0.9 {
        [0.0_f32, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let right = normalize3(cross3(fwd, hint));
    (right, normalize3(cross3(right, fwd)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> TrefoilParams {
        TrefoilParams {
            tube_radius: 0.1,
            knot_radius: 1.0,
            path_segs: 32,
            ring_segs: 8,
        }
    }

    #[test]
    fn path_nonempty() {
        let p = trefoil_path(&default_params());
        assert!(!p.is_empty());
    }

    #[test]
    fn path_all_finite() {
        let p = trefoil_path(&default_params());
        for pt in &p {
            assert!(pt[0].is_finite() && pt[1].is_finite() && pt[2].is_finite());
        }
    }

    #[test]
    fn trefoil_point_finite() {
        let p = trefoil_point(1.0, 1.0);
        assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
    }

    #[test]
    fn vertex_count_correct() {
        let params = default_params();
        let m = build_trefoil_mesh(&params);
        assert_eq!(m.positions.len(), (params.path_segs + 1) * params.ring_segs);
    }

    #[test]
    fn indices_multiple_of_three() {
        let m = build_trefoil_mesh(&default_params());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn trefoil_vertex_count_helper() {
        let m = build_trefoil_mesh(&default_params());
        assert_eq!(trefoil_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn trefoil_triangle_count_helper() {
        let m = build_trefoil_mesh(&default_params());
        assert_eq!(trefoil_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn all_positions_finite() {
        let m = build_trefoil_mesh(&default_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn index_max_within_bounds() {
        let m = build_trefoil_mesh(&default_params());
        let max_idx = m.indices.iter().copied().max().unwrap_or(0) as usize;
        assert!(max_idx < m.positions.len());
    }

    #[test]
    fn more_segs_more_vertices() {
        let p1 = TrefoilParams {
            tube_radius: 0.1,
            knot_radius: 1.0,
            path_segs: 16,
            ring_segs: 6,
        };
        let p2 = TrefoilParams {
            tube_radius: 0.1,
            knot_radius: 1.0,
            path_segs: 32,
            ring_segs: 6,
        };
        let m1 = build_trefoil_mesh(&p1);
        let m2 = build_trefoil_mesh(&p2);
        assert!(m2.positions.len() > m1.positions.len());
    }
}
