// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Parametric surface sampler with grid tessellation.
//! Maps (u, v) in `[0,1]`^2 to 3-D positions via a user-supplied function,
//! then builds an indexed triangle mesh.

use std::f32::consts::PI;

/// A tessellated parametric surface.
#[allow(dead_code)]
pub struct ParametricSurface {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub u_steps: usize,
    pub v_steps: usize,
}

/// Tessellate a parametric surface `f(u, v) -> [f32; 3]` with `u_steps × v_steps` quads.
#[allow(dead_code)]
pub fn tessellate_parametric<F>(f: F, u_steps: usize, v_steps: usize) -> ParametricSurface
where
    F: Fn(f32, f32) -> [f32; 3],
{
    let nu = u_steps.max(1);
    let nv = v_steps.max(1);
    let mut positions = Vec::with_capacity((nu + 1) * (nv + 1));
    let mut uvs = Vec::with_capacity((nu + 1) * (nv + 1));
    for j in 0..=nv {
        for i in 0..=nu {
            let u = i as f32 / nu as f32;
            let v = j as f32 / nv as f32;
            positions.push(f(u, v));
            uvs.push([u, v]);
        }
    }
    let mut indices = Vec::with_capacity(nu * nv * 6);
    for j in 0..nv {
        for i in 0..nu {
            let base = (j * (nu + 1) + i) as u32;
            let stride = (nu + 1) as u32;
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + stride);
            indices.push(base + 1);
            indices.push(base + stride + 1);
            indices.push(base + stride);
        }
    }
    let normals = compute_smooth_normals_ps(&positions, &indices);
    ParametricSurface {
        positions,
        indices,
        normals,
        uvs,
        u_steps: nu,
        v_steps: nv,
    }
}

/// Compute smooth per-vertex normals by averaging face normals.
#[allow(dead_code)]
pub fn compute_smooth_normals_ps(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut acc = vec![[0.0f32; 3]; n];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n3 = cross3(ab, ac);
        for &idx in &[a, b, c] {
            acc[idx][0] += n3[0];
            acc[idx][1] += n3[1];
            acc[idx][2] += n3[2];
        }
    }
    acc.iter().map(|&v| normalize3(v)).collect()
}

/// Return the vertex count for a given u/v step combination.
#[allow(dead_code)]
pub fn parametric_surf_vertex_count(u_steps: usize, v_steps: usize) -> usize {
    (u_steps + 1) * (v_steps + 1)
}

/// Return the triangle count for a given u/v step combination.
#[allow(dead_code)]
pub fn parametric_surf_triangle_count(u_steps: usize, v_steps: usize) -> usize {
    u_steps * v_steps * 2
}

/// Built-in torus parametric function.
#[allow(dead_code)]
pub fn torus_fn(major_r: f32, minor_r: f32) -> impl Fn(f32, f32) -> [f32; 3] {
    move |u, v| {
        let theta = u * 2.0 * PI;
        let phi = v * 2.0 * PI;
        let x = (major_r + minor_r * phi.cos()) * theta.cos();
        let y = minor_r * phi.sin();
        let z = (major_r + minor_r * phi.cos()) * theta.sin();
        [x, y, z]
    }
}

/// Built-in sphere parametric function.
#[allow(dead_code)]
pub fn sphere_fn(radius: f32) -> impl Fn(f32, f32) -> [f32; 3] {
    move |u, v| {
        let theta = u * PI;
        let phi = v * 2.0 * PI;
        [
            radius * theta.sin() * phi.cos(),
            radius * theta.cos(),
            radius * theta.sin() * phi.sin(),
        ]
    }
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-8 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn tessellate_vertex_count() {
        let surf = tessellate_parametric(|u, v| [u, v, 0.0], 4, 4);
        assert_eq!(surf.positions.len(), 25);
    }

    #[test]
    fn tessellate_index_count() {
        let surf = tessellate_parametric(|u, v| [u, v, 0.0], 4, 4);
        assert_eq!(surf.indices.len(), 4 * 4 * 6);
    }

    #[test]
    fn uv_range_clamped() {
        let surf = tessellate_parametric(|u, v| [u, v, 0.0], 3, 3);
        for uv in &surf.uvs {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }

    #[test]
    fn normals_unit_length() {
        let surf = tessellate_parametric(sphere_fn(1.0), 8, 8);
        for n in &surf.normals {
            let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
            assert!((len_sq - 1.0).abs() < 0.01 || len_sq < 1e-6);
        }
    }

    #[test]
    fn sphere_fn_radius_approx() {
        let f = sphere_fn(2.0);
        let p = f(0.5, 0.5);
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 2.0).abs() < 0.01);
    }

    #[test]
    fn torus_fn_produces_points() {
        let f = torus_fn(1.0, 0.25);
        let p = f(0.0, 0.0);
        assert!((p[0] - 1.25_f32).abs() < 1e-5);
    }

    #[test]
    fn pi_sin_near_zero() {
        let v = PI.sin();
        assert!(v.abs() < 1e-5);
    }

    #[test]
    fn vertex_count_formula() {
        assert_eq!(parametric_surf_vertex_count(4, 6), 35);
    }

    #[test]
    fn triangle_count_formula() {
        assert_eq!(parametric_surf_triangle_count(4, 6), 48);
    }

    #[test]
    fn single_step_mesh() {
        let surf = tessellate_parametric(|u, v| [u, 0.0, v], 1, 1);
        assert_eq!(surf.positions.len(), 4);
        assert_eq!(surf.indices.len(), 6);
    }
}
