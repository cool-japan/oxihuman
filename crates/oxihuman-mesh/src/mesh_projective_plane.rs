// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Klein bottle mesh stub (embedded in 4-D, rendered as 3-D immersion).

use std::f32::consts::PI;

/// Parameters for the Klein bottle mesh.
#[derive(Debug, Clone)]
pub struct KleinBottleParams {
    /// Scale factor.
    pub scale: f32,
    /// Segments in the u direction.
    pub u_segments: usize,
    /// Segments in the v direction.
    pub v_segments: usize,
}

impl Default for KleinBottleParams {
    fn default() -> Self {
        Self {
            scale: 0.25,
            u_segments: 40,
            v_segments: 20,
        }
    }
}

/// Generated Klein bottle mesh (3-D immersion).
#[derive(Debug, Clone)]
pub struct KleinBottleMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl KleinBottleMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Evaluate the "figure-8" Klein bottle immersion at (u, v).
/// Uses the standard parametric equations from differential geometry.
pub fn klein_point(params: &KleinBottleParams, u: f32, v: f32) -> [f32; 3] {
    let s = params.scale;
    let (su, cu) = u.sin_cos();
    let (sv, cv) = v.sin_cos();
    let (su2, cu2) = (u * 0.5).sin_cos();
    let x = cu * (cu2 * (2.0 + cv) + su2 * sv * cv);
    let y = su * (cu2 * (2.0 + cv) + su2 * sv * cv);
    let z = -su2 * (2.0 + cv) + cu2 * sv * cv;
    [x * s, y * s, z * s]
}

/// Build a Klein bottle mesh using the figure-8 immersion.
pub fn build_klein_bottle(params: &KleinBottleParams) -> KleinBottleMesh {
    let us = params.u_segments.max(8);
    let vs = params.v_segments.max(4);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for i in 0..=us {
        let u = i as f32 / us as f32 * 2.0 * PI;
        for j in 0..=vs {
            let v = j as f32 / vs as f32 * 2.0 * PI;
            let pos = klein_point(params, u, v);
            /* finite-difference normal */
            let du = 0.001;
            let dv = 0.001;
            let pu = klein_point(params, u + du, v);
            let pv = klein_point(params, u, v + dv);
            let tang_u = normalize3([pu[0] - pos[0], pu[1] - pos[1], pu[2] - pos[2]]);
            let tang_v = normalize3([pv[0] - pos[0], pv[1] - pos[1], pv[2] - pos[2]]);
            let nrm = normalize3(cross3(tang_u, tang_v));
            positions.push(pos);
            normals.push(nrm);
            uvs.push([i as f32 / us as f32, j as f32 / vs as f32]);
        }
    }
    let row = (vs + 1) as u32;
    let mut indices = Vec::new();
    for i in 0..(us as u32) {
        for j in 0..(vs as u32) {
            let a = i * row + j;
            let b = i * row + j + 1;
            let c = (i + 1) * row + j;
            let d = (i + 1) * row + j + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    KleinBottleMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Validate Klein bottle params.
pub fn validate_klein_params(p: &KleinBottleParams) -> bool {
    p.scale > 0.0 && p.u_segments >= 8 && p.v_segments >= 4
}

/// Expected vertex count.
pub fn expected_vertex_count(us: usize, vs: usize) -> usize {
    (us + 1) * (vs + 1)
}

/// Expected triangle count.
pub fn expected_triangle_count(us: usize, vs: usize) -> usize {
    us * vs * 2
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
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

    #[test]
    fn klein_has_vertices() {
        let m = build_klein_bottle(&KleinBottleParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn klein_has_triangles() {
        let m = build_klein_bottle(&KleinBottleParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_klein_bottle(&KleinBottleParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match() {
        let m = build_klein_bottle(&KleinBottleParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn uvs_match() {
        let m = build_klein_bottle(&KleinBottleParams::default());
        assert_eq!(m.uvs.len(), m.positions.len());
    }

    #[test]
    fn validate_ok() {
        assert!(validate_klein_params(&KleinBottleParams::default()));
    }

    #[test]
    fn validate_bad_scale() {
        let p = KleinBottleParams {
            scale: 0.0,
            ..KleinBottleParams::default()
        };
        assert!(!validate_klein_params(&p));
    }

    #[test]
    fn expected_counts_correct() {
        let p = KleinBottleParams::default();
        let m = build_klein_bottle(&p);
        assert_eq!(m.vertex_count(), expected_vertex_count(40, 20));
        assert_eq!(m.triangle_count(), expected_triangle_count(40, 20));
    }

    #[test]
    fn klein_point_not_origin() {
        let pos = klein_point(&KleinBottleParams::default(), 0.5, 1.0);
        let len = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
        assert!(len > 0.01);
    }
}
