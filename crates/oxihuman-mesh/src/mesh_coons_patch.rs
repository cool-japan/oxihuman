// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Coons bi-linear patch mesh generation.

/// A Coons bi-linear patch defined by four boundary curves.
/// c0u: curve at v=0, c1u: curve at v=1, c0v: curve at u=0, c1v: curve at u=1.
/// All curves must have the same sample count.
#[derive(Debug, Clone)]
pub struct CoonsPatch {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub grid_u: usize,
    pub grid_v: usize,
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Evaluate the Coons bi-linear patch at parameter (u, v) ∈ `[0,1]`².
/// Corners: p00 = `c0u[0]` = `c0v[0]`, p10 = `c0u[last]` = `c1v[0]`,
///          p01 = `c1u[0]` = `c0v[last]`, p11 = `c1u[last]` = `c1v[last]`.
pub fn coons_eval(
    c0u: &[[f32; 3]],
    c1u: &[[f32; 3]],
    c0v: &[[f32; 3]],
    c1v: &[[f32; 3]],
    u: f32,
    v: f32,
) -> [f32; 3] {
    let n = c0u.len().saturating_sub(1).max(1) as f32;
    let ui = (u * n).clamp(0.0, n) as usize;
    let vi = (v * n).clamp(0.0, n) as usize;
    let ui = ui.min(c0u.len().saturating_sub(1));
    let vi = vi.min(c0v.len().saturating_sub(1));

    let lc = lerp3(c0v[vi], c1v[vi], u);
    let lc2 = lerp3(c0u[ui], c1u[ui], v);
    let p00 = c0u[0];
    let p10 = c0u[c0u.len().saturating_sub(1)];
    let p01 = c1u[0];
    let p11 = c1u[c1u.len().saturating_sub(1)];
    let bilin = add3(
        add3(lerp3(lerp3(p00, p10, u), lerp3(p01, p11, u), v), [0.0; 3]),
        [0.0; 3],
    );
    sub3(add3(lc, lc2), bilin)
}

/// Build a Coons patch mesh with `nu` × `nv` grid divisions.
pub fn build_coons_patch(
    c0u: &[[f32; 3]],
    c1u: &[[f32; 3]],
    c0v: &[[f32; 3]],
    c1v: &[[f32; 3]],
    nu: usize,
    nv: usize,
) -> CoonsPatch {
    let n_samples = c0u.len();
    if n_samples < 2
        || c1u.len() != n_samples
        || c0v.len() != n_samples
        || c1v.len() != n_samples
        || nu < 1
        || nv < 1
    {
        return CoonsPatch {
            verts: vec![],
            tris: vec![],
            grid_u: 0,
            grid_v: 0,
        };
    }
    let rows = nu + 1;
    let cols = nv + 1;
    let mut verts = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        let u = i as f32 / nu as f32;
        for j in 0..cols {
            let v = j as f32 / nv as f32;
            verts.push(coons_eval(c0u, c1u, c0v, c1v, u, v));
        }
    }
    let mut tris = Vec::new();
    for i in 0..nu {
        for j in 0..nv {
            let a = (i * cols + j) as u32;
            let b = (i * cols + j + 1) as u32;
            let c = ((i + 1) * cols + j) as u32;
            let d = ((i + 1) * cols + j + 1) as u32;
            tris.push([a, c, b]);
            tris.push([b, c, d]);
        }
    }
    CoonsPatch {
        verts,
        tris,
        grid_u: rows,
        grid_v: cols,
    }
}

/// Return vertex count of a Coons patch.
pub fn coons_vertex_count(patch: &CoonsPatch) -> usize {
    patch.verts.len()
}

/// Return triangle count of a Coons patch.
pub fn coons_tri_count(patch: &CoonsPatch) -> usize {
    patch.tris.len()
}

/// Validate that all triangle indices are within range.
pub fn validate_coons_patch(patch: &CoonsPatch) -> bool {
    let n = patch.verts.len() as u32;
    patch.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_curve_x(n: usize, y: f32) -> Vec<[f32; 3]> {
        (0..n)
            .map(|i| [i as f32 / (n - 1) as f32, y, 0.0])
            .collect()
    }
    fn flat_curve_y(n: usize, x: f32) -> Vec<[f32; 3]> {
        (0..n)
            .map(|i| [x, i as f32 / (n - 1) as f32, 0.0])
            .collect()
    }

    #[test]
    fn test_coons_vertex_count() {
        /* 4x4 grid → 5*5 = 25 verts */
        let c0u = flat_curve_x(5, 0.0);
        let c1u = flat_curve_x(5, 1.0);
        let c0v = flat_curve_y(5, 0.0);
        let c1v = flat_curve_y(5, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 4, 4);
        assert_eq!(coons_vertex_count(&p), 25);
    }

    #[test]
    fn test_coons_tri_count() {
        /* 4*4 quads * 2 = 32 tris */
        let c0u = flat_curve_x(5, 0.0);
        let c1u = flat_curve_x(5, 1.0);
        let c0v = flat_curve_y(5, 0.0);
        let c1v = flat_curve_y(5, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 4, 4);
        assert_eq!(coons_tri_count(&p), 32);
    }

    #[test]
    fn test_coons_empty_on_mismatch() {
        let c0u = flat_curve_x(4, 0.0);
        let c1u = flat_curve_x(6, 1.0);
        let c0v = flat_curve_y(4, 0.0);
        let c1v = flat_curve_y(4, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 3, 3);
        assert_eq!(coons_vertex_count(&p), 0);
    }

    #[test]
    fn test_coons_validate() {
        let c0u = flat_curve_x(5, 0.0);
        let c1u = flat_curve_x(5, 1.0);
        let c0v = flat_curve_y(5, 0.0);
        let c1v = flat_curve_y(5, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 3, 3);
        assert!(validate_coons_patch(&p));
    }

    #[test]
    fn test_coons_grid_dims() {
        let c0u = flat_curve_x(5, 0.0);
        let c1u = flat_curve_x(5, 1.0);
        let c0v = flat_curve_y(5, 0.0);
        let c1v = flat_curve_y(5, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 3, 5);
        assert_eq!(p.grid_u, 4);
        assert_eq!(p.grid_v, 6);
    }

    #[test]
    fn test_lerp3_identity() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [5.0f32, 6.0, 7.0];
        assert_eq!(lerp3(a, a, 0.5), a);
        assert_eq!(lerp3(b, b, 0.5), b);
    }

    #[test]
    fn test_coons_empty_on_too_short() {
        let c0u = flat_curve_x(1, 0.0);
        let c1u = flat_curve_x(1, 1.0);
        let c0v = flat_curve_y(1, 0.0);
        let c1v = flat_curve_y(1, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 2, 2);
        assert_eq!(coons_vertex_count(&p), 0);
    }

    #[test]
    fn test_add3_sub3() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, 5.0, 6.0];
        assert_eq!(add3(a, b), [5.0, 7.0, 9.0]);
        assert_eq!(sub3(b, a), [3.0, 3.0, 3.0]);
    }

    #[test]
    fn test_coons_single_cell() {
        /* nu=1, nv=1 → 4 verts, 2 tris */
        let c0u = flat_curve_x(5, 0.0);
        let c1u = flat_curve_x(5, 1.0);
        let c0v = flat_curve_y(5, 0.0);
        let c1v = flat_curve_y(5, 1.0);
        let p = build_coons_patch(&c0u, &c1u, &c0v, &c1v, 1, 1);
        assert_eq!(coons_vertex_count(&p), 4);
        assert_eq!(coons_tri_count(&p), 2);
    }
}
