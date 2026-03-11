// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Möbius strip mesh generation.

use std::f32::consts::PI;

/// Parameters for a Möbius strip.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MobiusParams {
    /// Outer radius.
    pub radius: f32,
    /// Half-width of the strip.
    pub half_width: f32,
    /// Segments around the loop.
    pub u_segs: usize,
    /// Segments across the width.
    pub v_segs: usize,
}

/// A Möbius strip mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MobiusMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Evaluate the Möbius surface at parameters (u ∈ [0,2π), v ∈ [-1,1]).
#[allow(dead_code)]
pub fn mobius_point(u: f32, v: f32, radius: f32, half_width: f32) -> [f32; 3] {
    let half_u = u * 0.5;
    let (su, cu) = u.sin_cos();
    let (sh, ch) = half_u.sin_cos();
    let vw = v * half_width;
    [(radius + vw * ch) * cu, (radius + vw * ch) * su, vw * sh]
}

/// Build a Möbius strip mesh.
#[allow(dead_code)]
pub fn build_mobius_mesh(params: &MobiusParams) -> MobiusMesh {
    let nu = params.u_segs.max(3);
    let nv = params.v_segs.max(1);

    let mut positions = Vec::with_capacity((nu + 1) * (nv + 1));
    let mut normals = Vec::with_capacity((nu + 1) * (nv + 1));

    for iu in 0..=nu {
        let u = 2.0 * PI * iu as f32 / nu as f32;
        for iv in 0..=nv {
            let v = -1.0 + 2.0 * iv as f32 / nv as f32;
            let p = mobius_point(u, v, params.radius, params.half_width);
            positions.push(p);
            let dp_dv = mobius_dv(u, params.half_width);
            let du = 2.0 * PI / nu as f32;
            let p_next = mobius_point(u + du * 0.01, v, params.radius, params.half_width);
            let dp_du = normalize3([p_next[0] - p[0], p_next[1] - p[1], p_next[2] - p[2]]);
            normals.push(normalize3(cross3(dp_du, dp_dv)));
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

    MobiusMesh {
        positions,
        normals,
        indices,
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn mobius_vertex_count(m: &MobiusMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn mobius_triangle_count(m: &MobiusMesh) -> usize {
    m.indices.len() / 3
}

fn mobius_dv(u: f32, half_width: f32) -> [f32; 3] {
    let half_u = u * 0.5;
    let (su, cu) = u.sin_cos();
    let (sh, ch) = half_u.sin_cos();
    let w = half_width;
    normalize3([w * ch * cu, w * ch * su, w * sh])
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        [0.0, 0.0, 1.0]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> MobiusParams {
        MobiusParams {
            radius: 1.0,
            half_width: 0.3,
            u_segs: 32,
            v_segs: 4,
        }
    }

    #[test]
    fn mobius_point_u0_v0_on_circle() {
        let p = mobius_point(0.0, 0.0, 1.0, 0.3);
        assert!((p[0] - 1.0).abs() < 1e-5);
        assert!((p[1]).abs() < 1e-5);
    }

    #[test]
    fn vertex_count_correct() {
        let params = default_params();
        let m = build_mobius_mesh(&params);
        assert_eq!(m.positions.len(), (params.u_segs + 1) * (params.v_segs + 1));
    }

    #[test]
    fn normals_count_matches_vertices() {
        let m = build_mobius_mesh(&default_params());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn indices_multiple_of_three() {
        let m = build_mobius_mesh(&default_params());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn triangle_count_helper() {
        let m = build_mobius_mesh(&default_params());
        assert_eq!(mobius_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn vertex_count_helper() {
        let m = build_mobius_mesh(&default_params());
        assert_eq!(mobius_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn normals_unit_length() {
        let m = build_mobius_mesh(&default_params());
        for n in &m.normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4);
        }
    }

    #[test]
    fn more_segs_more_tris() {
        let p1 = MobiusParams {
            radius: 1.0,
            half_width: 0.3,
            u_segs: 8,
            v_segs: 2,
        };
        let p2 = MobiusParams {
            radius: 1.0,
            half_width: 0.3,
            u_segs: 16,
            v_segs: 2,
        };
        let m1 = build_mobius_mesh(&p1);
        let m2 = build_mobius_mesh(&p2);
        assert!(m2.positions.len() > m1.positions.len());
    }

    #[test]
    fn positions_finite() {
        let m = build_mobius_mesh(&default_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn u_full_circle_connects() {
        let p = default_params();
        let p_start = mobius_point(0.0, 0.0, p.radius, p.half_width);
        let p_end = mobius_point(2.0 * PI, 0.0, p.radius, p.half_width);
        assert!((p_start[0] - p_end[0]).abs() < 1e-4);
    }
}
