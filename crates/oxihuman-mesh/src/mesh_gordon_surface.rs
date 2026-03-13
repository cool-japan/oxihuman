// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gordon surface (net of curves) stub.

/// A Gordon surface is defined by a grid of u-curves and v-curves plus a
/// cross-interpolation grid.  This stub stores the input network and provides
/// a simple linear tessellation.
#[derive(Debug, Clone)]
pub struct GordonSurface {
    /// `u_curves[i]` is a list of 3-D points along the i-th u-direction curve.
    pub u_curves: Vec<Vec<[f32; 3]>>,
    /// `v_curves[j]` is a list of 3-D points along the j-th v-direction curve.
    pub v_curves: Vec<Vec<[f32; 3]>>,
    /// Intersection matrix `intersections[i][j]` = `u_curves[i]` ∩ `v_curves[j]`.
    pub intersections: Vec<Vec<[f32; 3]>>,
}

/// Create an empty Gordon surface network.
pub fn new_gordon_surface(
    u_curves: Vec<Vec<[f32; 3]>>,
    v_curves: Vec<Vec<[f32; 3]>>,
) -> GordonSurface {
    let nu = u_curves.len();
    let nv = v_curves.len();
    /* stub: intersections taken as first point of each u-curve per v-curve */
    let intersections = (0..nu)
        .map(|i| {
            (0..nv)
                .map(|_j| u_curves[i].first().copied().unwrap_or([0.0; 3]))
                .collect()
        })
        .collect();
    GordonSurface {
        u_curves,
        v_curves,
        intersections,
    }
}

/// Return the number of u-curves.
pub fn gordon_u_curve_count(surf: &GordonSurface) -> usize {
    surf.u_curves.len()
}

/// Return the number of v-curves.
pub fn gordon_v_curve_count(surf: &GordonSurface) -> usize {
    surf.v_curves.len()
}

/// Validate that the intersection matrix has the correct size.
pub fn validate_gordon(surf: &GordonSurface) -> bool {
    let nu = surf.u_curves.len();
    let nv = surf.v_curves.len();
    surf.intersections.len() == nu && surf.intersections.iter().all(|row| row.len() == nv)
}

/// Tessellate the Gordon surface using a simple u×v grid (stub).
/// Returns (vertices, triangles).
pub fn tessellate_gordon(surf: &GordonSurface, samples: usize) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let nu = surf.u_curves.len();
    let nv = surf.v_curves.len();
    if nu < 2 || nv < 2 || samples == 0 {
        return (vec![], vec![]);
    }
    let rows = nu.max(2);
    let cols = nv.max(2);
    let mut verts = Vec::with_capacity(rows * cols);
    for i in 0..rows {
        let ui = i.min(surf.u_curves.len().saturating_sub(1));
        for j in 0..cols {
            let vi = j.min(surf.v_curves.len().saturating_sub(1));
            let _ = samples;
            /* stub: use u_curve first point weighted with v_curve first point */
            let pu = surf.u_curves[ui].first().copied().unwrap_or([0.0; 3]);
            let pv = surf.v_curves[vi].first().copied().unwrap_or([0.0; 3]);
            verts.push([
                (pu[0] + pv[0]) * 0.5,
                (pu[1] + pv[1]) * 0.5,
                (pu[2] + pv[2]) * 0.5,
            ]);
        }
    }
    let mut tris = Vec::new();
    for i in 0..(rows - 1) {
        for j in 0..(cols - 1) {
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

/// Total point count across all u-curves.
pub fn gordon_total_u_points(surf: &GordonSurface) -> usize {
    surf.u_curves.iter().map(|c| c.len()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_u_curves(n: usize) -> Vec<Vec<[f32; 3]>> {
        (0..n)
            .map(|i| vec![[i as f32, 0.0, 0.0], [i as f32, 1.0, 0.0]])
            .collect()
    }
    fn make_v_curves(n: usize) -> Vec<Vec<[f32; 3]>> {
        (0..n)
            .map(|j| vec![[0.0, j as f32, 0.0], [1.0, j as f32, 0.0]])
            .collect()
    }

    #[test]
    fn test_gordon_u_curve_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(3));
        assert_eq!(gordon_u_curve_count(&s), 4);
    }

    #[test]
    fn test_gordon_v_curve_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(3));
        assert_eq!(gordon_v_curve_count(&s), 3);
    }

    #[test]
    fn test_validate_gordon() {
        let s = new_gordon_surface(make_u_curves(3), make_v_curves(5));
        assert!(validate_gordon(&s));
    }

    #[test]
    fn test_tessellate_gordon_vertex_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(4));
        let (v, _) = tessellate_gordon(&s, 1);
        assert_eq!(v.len(), 16);
    }

    #[test]
    fn test_tessellate_gordon_tri_count() {
        let s = new_gordon_surface(make_u_curves(4), make_v_curves(4));
        let (_, t) = tessellate_gordon(&s, 1);
        assert_eq!(t.len(), 18);
    }

    #[test]
    fn test_tessellate_gordon_empty_on_too_few() {
        let s = new_gordon_surface(make_u_curves(1), make_v_curves(4));
        let (v, _) = tessellate_gordon(&s, 2);
        assert!(v.is_empty());
    }

    #[test]
    fn test_gordon_total_u_points() {
        let s = new_gordon_surface(make_u_curves(3), make_v_curves(2));
        assert_eq!(gordon_total_u_points(&s), 6);
    }

    #[test]
    fn test_validate_gordon_empty() {
        let s = new_gordon_surface(vec![], vec![]);
        assert!(validate_gordon(&s));
    }

    #[test]
    fn test_tessellate_empty_on_zero_samples() {
        let s = new_gordon_surface(make_u_curves(3), make_v_curves(3));
        let (v, _) = tessellate_gordon(&s, 0);
        assert!(v.is_empty());
    }
}
