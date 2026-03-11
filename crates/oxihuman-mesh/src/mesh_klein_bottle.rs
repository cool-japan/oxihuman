// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Klein bottle surface mesh (immersed in 3D).

use std::f32::consts::PI;

/// Parameters for a Klein bottle mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KleinParams {
    pub u_segs: usize,
    pub v_segs: usize,
}

/// A Klein bottle mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KleinMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Evaluate the "figure-8" immersion of the Klein bottle (Lawson parametrization).
/// u ∈ [0, 2π), v ∈ [0, 2π).
#[allow(dead_code)]
pub fn klein_point(u: f32, v: f32) -> [f32; 3] {
    let (su, cu) = u.sin_cos();
    let (sv, cv) = v.sin_cos();
    let half_u = u * 0.5;
    let (sh, ch) = half_u.sin_cos();
    // Lawson/Parametric Klein bottle immersion in R3
    let r = 4.0 * (1.0 - ch * 0.5);
    let x = (2.5 + r * cv) * cu;
    let y = (2.5 + r * cv) * su;
    let z = r * sv + 2.0 * sh;
    [x, y, z]
}

/// Build a Klein bottle mesh.
#[allow(dead_code)]
pub fn build_klein_mesh(params: &KleinParams) -> KleinMesh {
    let nu = params.u_segs.max(4);
    let nv = params.v_segs.max(4);

    let mut positions = Vec::with_capacity((nu + 1) * (nv + 1));
    for iu in 0..=nu {
        let u = 2.0 * PI * iu as f32 / nu as f32;
        for iv in 0..=nv {
            let v = 2.0 * PI * iv as f32 / nv as f32;
            positions.push(klein_point(u, v));
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

    KleinMesh { positions, indices }
}

/// Vertex count.
#[allow(dead_code)]
pub fn klein_vertex_count(m: &KleinMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn klein_triangle_count(m: &KleinMesh) -> usize {
    m.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> KleinParams {
        KleinParams {
            u_segs: 16,
            v_segs: 16,
        }
    }

    #[test]
    fn vertex_count_correct() {
        let p = default_params();
        let m = build_klein_mesh(&p);
        assert_eq!(m.positions.len(), (p.u_segs + 1) * (p.v_segs + 1));
    }

    #[test]
    fn indices_multiple_of_three() {
        let m = build_klein_mesh(&default_params());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn triangle_count_correct() {
        let p = default_params();
        let m = build_klein_mesh(&p);
        assert_eq!(klein_triangle_count(&m), p.u_segs * p.v_segs * 2);
    }

    #[test]
    fn vertex_count_helper() {
        let m = build_klein_mesh(&default_params());
        assert_eq!(klein_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn all_positions_finite() {
        let m = build_klein_mesh(&default_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn klein_point_finite() {
        let p = klein_point(1.0, 2.0);
        assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
    }

    #[test]
    fn min_segs_enforced() {
        let p = KleinParams {
            u_segs: 1,
            v_segs: 1,
        };
        let m = build_klein_mesh(&p);
        assert_eq!(m.positions.len(), 5 * 5);
    }

    #[test]
    fn more_segs_more_vertices() {
        let p1 = KleinParams {
            u_segs: 8,
            v_segs: 8,
        };
        let p2 = KleinParams {
            u_segs: 16,
            v_segs: 16,
        };
        let m1 = build_klein_mesh(&p1);
        let m2 = build_klein_mesh(&p2);
        assert!(m2.positions.len() > m1.positions.len());
    }

    #[test]
    fn u0_v0_point_defined() {
        let p = klein_point(0.0, 0.0);
        assert!(p.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn index_max_within_bounds() {
        let m = build_klein_mesh(&default_params());
        let max_idx = m.indices.iter().copied().max().unwrap_or(0) as usize;
        assert!(max_idx < m.positions.len());
    }
}
