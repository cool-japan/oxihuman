// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dupin cyclide surface mesh (Villarceau circles).

use std::f32::consts::PI;

/// Parameters for a Dupin cyclide.
/// The cyclide is defined by parameters a, c (0 < c < a).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CyclideParams {
    /// Major radius.
    pub a: f32,
    /// Offset (eccentricity-like, c < a).
    pub c: f32,
    /// Minor radius: b = sqrt(a^2 - c^2).
    pub b: f32,
    pub u_segs: usize,
    pub v_segs: usize,
}

impl CyclideParams {
    #[allow(dead_code)]
    pub fn new(a: f32, c: f32, u_segs: usize, v_segs: usize) -> Self {
        let b = (a * a - c * c).abs().sqrt();
        CyclideParams {
            a,
            c,
            b,
            u_segs,
            v_segs,
        }
    }
}

/// A Dupin cyclide mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CyclideMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Evaluate the Dupin cyclide at (u, v).
/// u ∈ [0, 2π), v ∈ [0, 2π).
#[allow(dead_code)]
pub fn cyclide_point(u: f32, v: f32, a: f32, b: f32, c: f32) -> [f32; 3] {
    let (su, cu) = u.sin_cos();
    let (sv, cv) = v.sin_cos();
    let denom = a - c * cu * cv;
    let denom = if denom.abs() < 1e-8 {
        1e-8_f32.copysign(denom)
    } else {
        denom
    };
    [
        (b * b * cu + a * b * cv - b * c) / denom,
        b * su * (a - c * cv) / denom,
        b * sv * (a * cu - c) / denom,
    ]
}

/// Build a Dupin cyclide mesh.
#[allow(dead_code)]
pub fn build_cyclide_mesh(params: &CyclideParams) -> CyclideMesh {
    let nu = params.u_segs.max(4);
    let nv = params.v_segs.max(4);

    let mut positions = Vec::with_capacity((nu + 1) * (nv + 1));
    for iu in 0..=nu {
        let u = 2.0 * PI * iu as f32 / nu as f32;
        for iv in 0..=nv {
            let v = 2.0 * PI * iv as f32 / nv as f32;
            positions.push(cyclide_point(u, v, params.a, params.b, params.c));
        }
    }

    let mut indices = Vec::new();
    for iu in 0..nu {
        for iv in 0..nv {
            let a = (iu * (nv + 1) + iv) as u32;
            let b = (iu * (nv + 1) + iv + 1) as u32;
            let c = ((iu + 1) * (nv + 1) + iv) as u32;
            let d = ((iu + 1) * (nv + 1) + iv + 1) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    CyclideMesh { positions, indices }
}

/// Vertex count.
#[allow(dead_code)]
pub fn cyclide_vertex_count(m: &CyclideMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn cyclide_triangle_count(m: &CyclideMesh) -> usize {
    m.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> CyclideParams {
        CyclideParams::new(3.0, 1.0, 16, 16)
    }

    #[test]
    fn b_computed_correctly() {
        let p = default_params();
        assert!((p.b - (8.0_f32).sqrt()).abs() < 1e-4);
    }

    #[test]
    fn vertex_count_correct() {
        let p = default_params();
        let m = build_cyclide_mesh(&p);
        assert_eq!(m.positions.len(), (p.u_segs + 1) * (p.v_segs + 1));
    }

    #[test]
    fn indices_multiple_of_three() {
        let m = build_cyclide_mesh(&default_params());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn all_positions_finite() {
        let m = build_cyclide_mesh(&default_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn triangle_count_correct() {
        let p = default_params();
        let m = build_cyclide_mesh(&p);
        assert_eq!(cyclide_triangle_count(&m), p.u_segs * p.v_segs * 2);
    }

    #[test]
    fn vertex_count_helper() {
        let m = build_cyclide_mesh(&default_params());
        assert_eq!(cyclide_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn cyclide_point_finite() {
        let p = cyclide_point(1.0, 2.0, 3.0, (8.0_f32).sqrt(), 1.0);
        assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
    }

    #[test]
    fn index_max_within_bounds() {
        let m = build_cyclide_mesh(&default_params());
        let max_idx = m.indices.iter().copied().max().unwrap_or(0) as usize;
        assert!(max_idx < m.positions.len());
    }

    #[test]
    fn more_segs_more_vertices() {
        let p1 = CyclideParams::new(3.0, 1.0, 8, 8);
        let p2 = CyclideParams::new(3.0, 1.0, 16, 16);
        let m1 = build_cyclide_mesh(&p1);
        let m2 = build_cyclide_mesh(&p2);
        assert!(m2.positions.len() > m1.positions.len());
    }

    #[test]
    fn min_segs_enforced() {
        let p = CyclideParams {
            a: 3.0,
            b: (8.0_f32).sqrt(),
            c: 1.0,
            u_segs: 1,
            v_segs: 1,
        };
        let m = build_cyclide_mesh(&p);
        assert_eq!(m.positions.len(), 5 * 5);
    }
}
