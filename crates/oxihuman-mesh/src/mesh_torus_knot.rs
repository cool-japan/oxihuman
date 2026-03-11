// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Torus knot mesh generator.

use std::f32::consts::PI;

/// Parameters for a torus knot mesh.
#[derive(Debug, Clone)]
pub struct TorusKnotParams {
    /// p-parameter (winds around torus p times).
    pub p: i32,
    /// q-parameter (winds around tube q times).
    pub q: i32,
    /// Outer torus radius.
    pub torus_radius: f32,
    /// Tube radius.
    pub tube_radius: f32,
    /// Segments along the knot curve.
    pub tube_segments: usize,
    /// Segments around the tube cross-section.
    pub radial_segments: usize,
}

impl Default for TorusKnotParams {
    fn default() -> Self {
        Self {
            p: 2,
            q: 3,
            torus_radius: 0.2,
            tube_radius: 0.05,
            tube_segments: 100,
            radial_segments: 12,
        }
    }
}

/// Generated torus knot mesh.
#[derive(Debug, Clone)]
pub struct TorusKnotMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl TorusKnotMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Evaluate the torus knot curve at parameter `t` in `[0, 2π)`.
pub fn torus_knot_point(params: &TorusKnotParams, t: f32) -> [f32; 3] {
    let p = params.p as f32;
    let q = params.q as f32;
    let r = params.torus_radius;
    let x = r * (p * t).cos() * (1.0 + (q * t).cos() * 0.5);
    let y = r * (q * t).sin() * 0.5;
    let z = r * (p * t).sin() * (1.0 + (q * t).cos() * 0.5);
    [x, y, z]
}

/// Build a torus knot mesh.
pub fn build_torus_knot(params: &TorusKnotParams) -> TorusKnotMesh {
    let ts = params.tube_segments.max(8);
    let rs = params.radial_segments.max(3);
    /* generate spine */
    let spine: Vec<[f32; 3]> = (0..ts)
        .map(|i| torus_knot_point(params, 2.0 * PI * i as f32 / ts as f32))
        .collect();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for (si, &center) in spine.iter().enumerate() {
        let next_si = (si + 1) % ts;
        let fwd = normalize3([
            spine[next_si][0] - center[0],
            spine[next_si][1] - center[1],
            spine[next_si][2] - center[2],
        ]);
        let (tu, tv) = frame_from_forward(fwd);
        let u_param = si as f32 / ts as f32;
        for j in 0..rs {
            let angle = 2.0 * PI * j as f32 / rs as f32;
            let (s, c) = angle.sin_cos();
            let nrm = [
                tu[0] * c + tv[0] * s,
                tu[1] * c + tv[1] * s,
                tu[2] * c + tv[2] * s,
            ];
            positions.push([
                center[0] + nrm[0] * params.tube_radius,
                center[1] + nrm[1] * params.tube_radius,
                center[2] + nrm[2] * params.tube_radius,
            ]);
            normals.push(nrm);
            uvs.push([u_param, j as f32 / rs as f32]);
        }
    }
    let mut indices = Vec::new();
    let rs32 = rs as u32;
    let ts32 = ts as u32;
    for i in 0..ts32 {
        for j in 0..rs32 {
            let a = i * rs32 + j;
            let b = i * rs32 + (j + 1) % rs32;
            let c = (i + 1) % ts32 * rs32 + j;
            let d = (i + 1) % ts32 * rs32 + (j + 1) % rs32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    TorusKnotMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Validate torus knot params.
pub fn validate_torus_knot_params(p: &TorusKnotParams) -> bool {
    p.p != 0
        && p.q != 0
        && p.torus_radius > p.tube_radius
        && p.tube_radius > 0.0
        && p.tube_segments >= 8
        && p.radial_segments >= 3
}

/// Expected vertex count.
pub fn expected_vertex_count(tube_seg: usize, radial_seg: usize) -> usize {
    tube_seg * radial_seg
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-10 {
        return [1.0, 0.0, 0.0];
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

fn frame_from_forward(fwd: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if fwd[1].abs() < 0.9 {
        [0.0f32, 1.0, 0.0]
    } else {
        [1.0f32, 0.0, 0.0]
    };
    let tu = normalize3(cross3(fwd, up));
    let tv = cross3(fwd, tu);
    (tu, tv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knot_has_vertices() {
        let m = build_torus_knot(&TorusKnotParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn knot_has_triangles() {
        let m = build_torus_knot(&TorusKnotParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_torus_knot(&TorusKnotParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match() {
        let m = build_torus_knot(&TorusKnotParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn uvs_match() {
        let m = build_torus_knot(&TorusKnotParams::default());
        assert_eq!(m.uvs.len(), m.positions.len());
    }

    #[test]
    fn knot_point_nonzero() {
        /* at t=0 the knot point should not be at origin */
        let pt = torus_knot_point(&TorusKnotParams::default(), 0.0);
        let len = (pt[0] * pt[0] + pt[1] * pt[1] + pt[2] * pt[2]).sqrt();
        assert!(len > 0.01);
    }

    #[test]
    fn validate_ok() {
        assert!(validate_torus_knot_params(&TorusKnotParams::default()));
    }

    #[test]
    fn validate_bad_radii() {
        let mut p = TorusKnotParams::default();
        p.tube_radius = p.torus_radius + 0.1;
        assert!(!validate_torus_knot_params(&p));
    }

    #[test]
    fn expected_vertex_count_formula() {
        /* 100 × 12 = 1200 */
        assert_eq!(expected_vertex_count(100, 12), 1200);
    }
}
