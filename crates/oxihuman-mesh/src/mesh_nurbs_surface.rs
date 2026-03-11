// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! NURBS surface stub.

/// NURBS surface definition stub.
#[derive(Debug, Clone)]
pub struct NurbsSurface {
    /// Control points in a `(u_count × v_count)` grid, stored row-major.
    pub control_points: Vec<[f32; 3]>,
    /// Homogeneous weights for each control point.
    pub weights: Vec<f32>,
    pub u_count: usize,
    pub v_count: usize,
    pub u_degree: usize,
    pub v_degree: usize,
    pub u_knots: Vec<f32>,
    pub v_knots: Vec<f32>,
}

/// Clamp-open uniform knot vector for given count and degree.
pub fn uniform_knots(n: usize, degree: usize) -> Vec<f32> {
    let knot_count = n + degree + 1;
    (0..knot_count)
        .map(|i| {
            if i <= degree {
                0.0
            } else if i >= knot_count.saturating_sub(degree + 1) {
                1.0
            } else {
                (i - degree) as f32 / (n - degree) as f32
            }
        })
        .collect()
}

/// Create a NURBS surface stub with uniform weights.
pub fn new_nurbs_surface(
    control_points: Vec<[f32; 3]>,
    u_count: usize,
    v_count: usize,
    u_degree: usize,
    v_degree: usize,
) -> NurbsSurface {
    let n = control_points.len();
    let weights = vec![1.0f32; n];
    let u_knots = uniform_knots(u_count, u_degree);
    let v_knots = uniform_knots(v_count, v_degree);
    NurbsSurface {
        control_points,
        weights,
        u_count,
        v_count,
        u_degree,
        v_degree,
        u_knots,
        v_knots,
    }
}

/// Validate that the NURBS surface has consistent dimensions.
pub fn validate_nurbs(surf: &NurbsSurface) -> bool {
    surf.control_points.len() == surf.u_count * surf.v_count
        && surf.weights.len() == surf.control_points.len()
        && surf.u_knots.len() == surf.u_count + surf.u_degree + 1
        && surf.v_knots.len() == surf.v_count + surf.v_degree + 1
}

/// Return the total control point count.
pub fn nurbs_control_point_count(surf: &NurbsSurface) -> usize {
    surf.control_points.len()
}

/// Compute the bounding box of the control polygon (not the surface itself).
pub fn nurbs_control_bbox(surf: &NurbsSurface) -> ([f32; 3], [f32; 3]) {
    if surf.control_points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = surf.control_points[0];
    let mut mx = surf.control_points[0];
    for &p in &surf.control_points {
        for k in 0..3 {
            mn[k] = mn[k].min(p[k]);
            mx[k] = mx[k].max(p[k]);
        }
    }
    (mn, mx)
}

/// Tessellate the NURBS surface into a flat grid mesh (stub — uses bilinear interpolation).
pub fn tessellate_nurbs(
    surf: &NurbsSurface,
    u_divs: usize,
    v_divs: usize,
) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    if !validate_nurbs(surf) || u_divs == 0 || v_divs == 0 {
        return (vec![], vec![]);
    }
    let rows = u_divs + 1;
    let cols = v_divs + 1;
    let mut verts = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        let u = i as f32 / u_divs as f32;
        for j in 0..cols {
            let v = j as f32 / v_divs as f32;
            /* bilinear stub: sample corner control points */
            let ui = (u * (surf.u_count.saturating_sub(1)) as f32) as usize;
            let vi = (v * (surf.v_count.saturating_sub(1)) as f32) as usize;
            let ui = ui.min(surf.u_count.saturating_sub(1));
            let vi = vi.min(surf.v_count.saturating_sub(1));
            let idx = ui * surf.v_count + vi;
            verts.push(surf.control_points[idx.min(surf.control_points.len().saturating_sub(1))]);
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
    (verts, tris)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_grid(u: usize, v: usize) -> Vec<[f32; 3]> {
        (0..u)
            .flat_map(|i| (0..v).map(move |j| [i as f32, j as f32, 0.0]))
            .collect()
    }

    #[test]
    fn test_uniform_knots_length() {
        /* degree=2, count=4 → 7 knots */
        let k = uniform_knots(4, 2);
        assert_eq!(k.len(), 7);
    }

    #[test]
    fn test_uniform_knots_clamped() {
        let k = uniform_knots(5, 2);
        assert_eq!(k[0], 0.0);
        assert_eq!(*k.last().unwrap(), 1.0);
    }

    #[test]
    fn test_new_nurbs_surface_valid() {
        let cp = flat_grid(4, 3);
        let surf = new_nurbs_surface(cp, 4, 3, 2, 2);
        assert!(validate_nurbs(&surf));
    }

    #[test]
    fn test_nurbs_control_point_count() {
        let cp = flat_grid(3, 4);
        let surf = new_nurbs_surface(cp, 3, 4, 2, 2);
        assert_eq!(nurbs_control_point_count(&surf), 12);
    }

    #[test]
    fn test_nurbs_bbox_flat() {
        let cp = flat_grid(3, 3);
        let surf = new_nurbs_surface(cp, 3, 3, 2, 2);
        let (mn, mx) = nurbs_control_bbox(&surf);
        assert!(mn[2].abs() < 1e-6);
        assert!(mx[2].abs() < 1e-6);
    }

    #[test]
    fn test_tessellate_nurbs_vertex_count() {
        /* 4x4 divs → 5*5 = 25 verts */
        let cp = flat_grid(4, 4);
        let surf = new_nurbs_surface(cp, 4, 4, 2, 2);
        let (verts, _) = tessellate_nurbs(&surf, 4, 4);
        assert_eq!(verts.len(), 25);
    }

    #[test]
    fn test_tessellate_nurbs_tri_count() {
        let cp = flat_grid(4, 4);
        let surf = new_nurbs_surface(cp, 4, 4, 2, 2);
        let (_, tris) = tessellate_nurbs(&surf, 4, 4);
        assert_eq!(tris.len(), 32);
    }

    #[test]
    fn test_tessellate_empty_on_zero_divs() {
        let cp = flat_grid(4, 4);
        let surf = new_nurbs_surface(cp, 4, 4, 2, 2);
        let (v, t) = tessellate_nurbs(&surf, 0, 4);
        assert!(v.is_empty());
        assert!(t.is_empty());
    }

    #[test]
    fn test_weights_all_one() {
        let cp = flat_grid(3, 3);
        let surf = new_nurbs_surface(cp, 3, 3, 2, 2);
        assert!(surf.weights.iter().all(|&w| (w - 1.0).abs() < 1e-6));
    }

    #[test]
    fn test_nurbs_bbox_empty() {
        let surf = new_nurbs_surface(vec![], 0, 0, 0, 0);
        let (mn, mx) = nurbs_control_bbox(&surf);
        assert_eq!(mn, [0.0; 3]);
        assert_eq!(mx, [0.0; 3]);
    }
}
