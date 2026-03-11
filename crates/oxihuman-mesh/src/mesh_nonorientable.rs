// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Non-orientable surface meshes (Möbius strip variant).

use std::f32::consts::PI;

/// Parameters for a Möbius strip mesh.
#[derive(Debug, Clone)]
pub struct MobiusStripParams {
    /// Radius from center to strip midline.
    pub radius: f32,
    /// Half-width of the strip.
    pub half_width: f32,
    /// Segments along the strip.
    pub length_segments: usize,
    /// Segments across the strip width.
    pub width_segments: usize,
    /// Number of half-twists (1 = standard Möbius).
    pub twists: i32,
}

impl Default for MobiusStripParams {
    fn default() -> Self {
        Self {
            radius: 0.3,
            half_width: 0.1,
            length_segments: 64,
            width_segments: 4,
            twists: 1,
        }
    }
}

/// Generated Möbius strip mesh.
#[derive(Debug, Clone)]
pub struct MobiusMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl MobiusMesh {
    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Vertex count.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Evaluate a point on the Möbius strip.
pub fn mobius_point(params: &MobiusStripParams, u: f32, v: f32) -> ([f32; 3], [f32; 3]) {
    let phi = u * 2.0 * PI;
    let twist = params.twists as f32 * u * PI;
    let (sphi, cphi) = phi.sin_cos();
    let (st, ct) = twist.sin_cos();
    let scale = v * params.half_width;
    let x = (params.radius + scale * ct) * cphi;
    let y = scale * st;
    let z = (params.radius + scale * ct) * sphi;
    /* approximate normal via partial derivative cross product */
    let du = 0.001;
    let phi2 = (u + du) * 2.0 * PI;
    let twist2 = params.twists as f32 * (u + du) * PI;
    let (sphi2, cphi2) = phi2.sin_cos();
    let (st2, ct2) = twist2.sin_cos();
    let x2 = (params.radius + scale * ct2) * cphi2;
    let y2 = scale * st2;
    let z2 = (params.radius + scale * ct2) * sphi2;
    let tang = normalize3([x2 - x, y2 - y, z2 - z]);
    let bitang = normalize3([ct * (-sphi), st, ct * cphi]);
    let nrm = normalize3(cross3(tang, bitang));
    ([x, y, z], nrm)
}

/// Build a Möbius strip mesh.
pub fn build_mobius_strip(params: &MobiusStripParams) -> MobiusMesh {
    let ls = params.length_segments.max(8);
    let ws = params.width_segments.max(1);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    for i in 0..=ls {
        let u = i as f32 / ls as f32;
        for j in 0..=ws {
            let v = j as f32 / ws as f32 * 2.0 - 1.0;
            let (pos, nrm) = mobius_point(params, u, v);
            positions.push(pos);
            normals.push(nrm);
            uvs.push([u, (v + 1.0) * 0.5]);
        }
    }
    let row = (ws + 1) as u32;
    let mut indices = Vec::new();
    for i in 0..(ls as u32) {
        for j in 0..(ws as u32) {
            let a = i * row + j;
            let b = i * row + j + 1;
            let c = (i + 1) * row + j;
            let d = (i + 1) * row + j + 1;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    MobiusMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

/// Validate Möbius strip params.
pub fn validate_mobius_params(p: &MobiusStripParams) -> bool {
    p.radius > 0.0
        && p.half_width > 0.0
        && p.radius > p.half_width
        && p.length_segments >= 8
        && p.width_segments >= 1
}

/// Expected vertex count.
pub fn expected_vertex_count(ls: usize, ws: usize) -> usize {
    (ls + 1) * (ws + 1)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_has_vertices() {
        let m = build_mobius_strip(&MobiusStripParams::default());
        assert!(m.vertex_count() > 0);
    }

    #[test]
    fn strip_has_triangles() {
        let m = build_mobius_strip(&MobiusStripParams::default());
        assert!(m.triangle_count() > 0);
    }

    #[test]
    fn indices_in_bounds() {
        let m = build_mobius_strip(&MobiusStripParams::default());
        let n = m.positions.len() as u32;
        assert!(m.indices.iter().all(|&i| i < n));
    }

    #[test]
    fn normals_match() {
        let m = build_mobius_strip(&MobiusStripParams::default());
        assert_eq!(m.normals.len(), m.positions.len());
    }

    #[test]
    fn uvs_match() {
        let m = build_mobius_strip(&MobiusStripParams::default());
        assert_eq!(m.uvs.len(), m.positions.len());
    }

    #[test]
    fn validate_ok() {
        assert!(validate_mobius_params(&MobiusStripParams::default()));
    }

    #[test]
    fn validate_bad_width() {
        /* half_width > radius → invalid */
        let mut p = MobiusStripParams::default();
        p.half_width = p.radius + 0.1;
        assert!(!validate_mobius_params(&p));
    }

    #[test]
    fn expected_vertex_count_formula() {
        /* (64+1) × (4+1) = 325 */
        assert_eq!(expected_vertex_count(64, 4), 325);
    }

    #[test]
    fn mobius_point_not_origin() {
        let (pos, _) = mobius_point(&MobiusStripParams::default(), 0.0, 0.0);
        let len = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
        assert!(len > 0.1);
    }
}
