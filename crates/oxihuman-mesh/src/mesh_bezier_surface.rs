// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Bicubic Bézier surface tessellation.

/// 4×4 grid of control points for a bicubic Bézier patch.
pub type BezierControlGrid = [[f32; 3]; 16];

/// Tessellated result of a Bézier surface.
#[derive(Debug, Clone)]
pub struct BezierSurface {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub u_divs: usize,
    pub v_divs: usize,
}

/// Cubic Bernstein basis values at parameter `t`.
pub fn bernstein3(t: f32) -> [f32; 4] {
    let s = 1.0 - t;
    [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t]
}

/// Evaluate a bicubic Bézier patch at (u, v).
pub fn eval_bezier_surface(ctrl: &BezierControlGrid, u: f32, v: f32) -> [f32; 3] {
    let bu = bernstein3(u);
    let bv = bernstein3(v);
    let mut p = [0.0f32; 3];
    for i in 0..4 {
        for j in 0..4 {
            let w = bu[i] * bv[j];
            let cp = ctrl[i * 4 + j];
            p[0] += w * cp[0];
            p[1] += w * cp[1];
            p[2] += w * cp[2];
        }
    }
    p
}

/// Tessellate a bicubic Bézier surface patch into a triangle mesh.
pub fn tessellate_bezier_surface(
    ctrl: &BezierControlGrid,
    u_divs: usize,
    v_divs: usize,
) -> BezierSurface {
    if u_divs == 0 || v_divs == 0 {
        return BezierSurface {
            verts: vec![],
            tris: vec![],
            u_divs: 0,
            v_divs: 0,
        };
    }
    let rows = u_divs + 1;
    let cols = v_divs + 1;
    let mut verts = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        let u = i as f32 / u_divs as f32;
        for j in 0..cols {
            let v = j as f32 / v_divs as f32;
            verts.push(eval_bezier_surface(ctrl, u, v));
        }
    }
    let mut tris = Vec::new();
    for i in 0..u_divs {
        for j in 0..v_divs {
            let a = (i * cols + j) as u32;
            let b = (i * cols + j + 1) as u32;
            let c = ((i + 1) * cols + j) as u32;
            let d = ((i + 1) * cols + j + 1) as u32;
            tris.push([a, c, b]);
            tris.push([b, c, d]);
        }
    }
    BezierSurface {
        verts,
        tris,
        u_divs,
        v_divs,
    }
}

/// Return the vertex count.
pub fn bezier_surface_vertex_count(surf: &BezierSurface) -> usize {
    surf.verts.len()
}

/// Return the triangle count.
pub fn bezier_surface_tri_count(surf: &BezierSurface) -> usize {
    surf.tris.len()
}

/// Validate that all triangle indices are within range.
pub fn validate_bezier_surface(surf: &BezierSurface) -> bool {
    let n = surf.verts.len() as u32;
    surf.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_ctrl() -> BezierControlGrid {
        let mut ctrl = [[0.0f32; 3]; 16];
        for i in 0..4 {
            for j in 0..4 {
                ctrl[i * 4 + j] = [i as f32, j as f32, 0.0];
            }
        }
        ctrl
    }

    #[test]
    fn test_bernstein3_sum_to_one() {
        let b = bernstein3(0.3);
        let sum: f32 = b.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bernstein3_endpoints() {
        let b0 = bernstein3(0.0);
        let b1 = bernstein3(1.0);
        assert!((b0[0] - 1.0).abs() < 1e-6);
        assert!((b1[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_eval_bezier_corner() {
        /* flat grid: at u=0,v=0 should equal control point [0,0] */
        let ctrl = flat_ctrl();
        let p = eval_bezier_surface(&ctrl, 0.0, 0.0);
        assert!((p[0]).abs() < 1e-5);
        assert!((p[1]).abs() < 1e-5);
    }

    #[test]
    fn test_tessellate_vertex_count() {
        /* 4 u-divs, 4 v-divs → 5*5 = 25 verts */
        let ctrl = flat_ctrl();
        let surf = tessellate_bezier_surface(&ctrl, 4, 4);
        assert_eq!(bezier_surface_vertex_count(&surf), 25);
    }

    #[test]
    fn test_tessellate_tri_count() {
        /* 4*4 quads * 2 = 32 tris */
        let ctrl = flat_ctrl();
        let surf = tessellate_bezier_surface(&ctrl, 4, 4);
        assert_eq!(bezier_surface_tri_count(&surf), 32);
    }

    #[test]
    fn test_tessellate_empty_on_zero() {
        let ctrl = flat_ctrl();
        let surf = tessellate_bezier_surface(&ctrl, 0, 4);
        assert_eq!(bezier_surface_vertex_count(&surf), 0);
    }

    #[test]
    fn test_validate_bezier_surface() {
        let ctrl = flat_ctrl();
        let surf = tessellate_bezier_surface(&ctrl, 3, 3);
        assert!(validate_bezier_surface(&surf));
    }

    #[test]
    fn test_eval_bezier_midpoint_z_zero() {
        /* flat patch: z should always be 0 */
        let ctrl = flat_ctrl();
        let p = eval_bezier_surface(&ctrl, 0.5, 0.5);
        assert!(p[2].abs() < 1e-5);
    }

    #[test]
    fn test_surface_u_v_divs_stored() {
        let ctrl = flat_ctrl();
        let surf = tessellate_bezier_surface(&ctrl, 6, 8);
        assert_eq!(surf.u_divs, 6);
        assert_eq!(surf.v_divs, 8);
    }
}
