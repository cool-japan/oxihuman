// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Minimal surface mesh generation (Enneper, Scherk).

/// Minimal surface type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimalSurfaceType {
    Enneper,
    Scherk,
}

/// Parameters for a minimal surface mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MinimalSurfaceParams {
    pub kind: MinimalSurfaceType,
    pub u_segs: usize,
    pub v_segs: usize,
    pub u_range: [f32; 2],
    pub v_range: [f32; 2],
}

/// A minimal surface mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MinimalSurfaceMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub kind: MinimalSurfaceType,
}

/// Enneper surface: x=(u - u^3/3 + u*v^2), y=(v - v^3/3 + v*u^2), z=(u^2 - v^2).
#[allow(dead_code)]
pub fn enneper_point(u: f32, v: f32) -> [f32; 3] {
    [
        u - u * u * u / 3.0 + u * v * v,
        v - v * v * v / 3.0 + v * u * u,
        u * u - v * v,
    ]
}

/// Scherk's first surface: z = ln(cos(v)/cos(u)) (valid where |cos(u)| > 0 and |cos(v)| > 0).
#[allow(dead_code)]
pub fn scherk_point(u: f32, v: f32) -> [f32; 3] {
    let cu = u.cos();
    let cv = v.cos();
    let z = if cu.abs() < 1e-4 || cv.abs() < 1e-4 {
        0.0
    } else {
        (cv.abs() / cu.abs()).ln()
    };
    [u, v, z]
}

/// Build a minimal surface mesh.
#[allow(dead_code)]
pub fn build_minimal_surface_mesh(params: &MinimalSurfaceParams) -> MinimalSurfaceMesh {
    let nu = params.u_segs.max(2);
    let nv = params.v_segs.max(2);

    let mut positions = Vec::with_capacity((nu + 1) * (nv + 1));
    for iu in 0..=nu {
        let u = params.u_range[0] + (params.u_range[1] - params.u_range[0]) * iu as f32 / nu as f32;
        for iv in 0..=nv {
            let v =
                params.v_range[0] + (params.v_range[1] - params.v_range[0]) * iv as f32 / nv as f32;
            let p = match params.kind {
                MinimalSurfaceType::Enneper => enneper_point(u, v),
                MinimalSurfaceType::Scherk => scherk_point(u, v),
            };
            positions.push(p);
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

    MinimalSurfaceMesh {
        positions,
        indices,
        kind: params.kind,
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn minimal_surface_vertex_count(m: &MinimalSurfaceMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn minimal_surface_triangle_count(m: &MinimalSurfaceMesh) -> usize {
    m.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn enneper_params() -> MinimalSurfaceParams {
        MinimalSurfaceParams {
            kind: MinimalSurfaceType::Enneper,
            u_segs: 8,
            v_segs: 8,
            u_range: [-1.5, 1.5],
            v_range: [-1.5, 1.5],
        }
    }

    fn scherk_params() -> MinimalSurfaceParams {
        MinimalSurfaceParams {
            kind: MinimalSurfaceType::Scherk,
            u_segs: 8,
            v_segs: 8,
            u_range: [-PI * 0.4, PI * 0.4],
            v_range: [-PI * 0.4, PI * 0.4],
        }
    }

    #[test]
    fn enneper_vertex_count() {
        let p = enneper_params();
        let m = build_minimal_surface_mesh(&p);
        assert_eq!(m.positions.len(), (p.u_segs + 1) * (p.v_segs + 1));
    }

    #[test]
    fn enneper_at_origin() {
        let p = enneper_point(0.0, 0.0);
        assert!((p[0]).abs() < 1e-5 && (p[1]).abs() < 1e-5 && (p[2]).abs() < 1e-5);
    }

    #[test]
    fn scherk_vertex_count() {
        let p = scherk_params();
        let m = build_minimal_surface_mesh(&p);
        assert_eq!(m.positions.len(), (p.u_segs + 1) * (p.v_segs + 1));
    }

    #[test]
    fn enneper_indices_multiple_of_three() {
        let m = build_minimal_surface_mesh(&enneper_params());
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn scherk_all_finite() {
        let m = build_minimal_surface_mesh(&scherk_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn enneper_all_finite() {
        let m = build_minimal_surface_mesh(&enneper_params());
        for p in &m.positions {
            assert!(p[0].is_finite() && p[1].is_finite() && p[2].is_finite());
        }
    }

    #[test]
    fn vertex_count_helper() {
        let m = build_minimal_surface_mesh(&enneper_params());
        assert_eq!(minimal_surface_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn triangle_count_helper() {
        let m = build_minimal_surface_mesh(&enneper_params());
        assert_eq!(minimal_surface_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn index_max_within_bounds() {
        let m = build_minimal_surface_mesh(&enneper_params());
        let max_idx = m.indices.iter().copied().max().unwrap_or(0) as usize;
        assert!(max_idx < m.positions.len());
    }

    #[test]
    fn kind_stored() {
        let m = build_minimal_surface_mesh(&enneper_params());
        assert_eq!(m.kind, MinimalSurfaceType::Enneper);
    }
}
