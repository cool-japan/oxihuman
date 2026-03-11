// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Grid-based deformation (FFD-lite) for mesh vertices.

use std::f32::consts::PI;

/// A 3D grid deformer with trilinear interpolation of control points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridDeform {
    /// Grid dimensions (divs_x+1, divs_y+1, divs_z+1).
    pub dims: [usize; 3],
    /// Control point displacements, stored row-major XYZ.
    pub deltas: Vec<[f32; 3]>,
    /// Grid origin.
    pub origin: [f32; 3],
    /// Grid size per axis.
    pub size: [f32; 3],
}

/// Result of grid deformation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridDeformResult {
    pub positions: Vec<[f32; 3]>,
    pub max_displacement: f32,
}

/// Create an identity grid deformer (all deltas = zero).
#[allow(dead_code)]
pub fn new_grid_deform(dims: [usize; 3], origin: [f32; 3], size: [f32; 3]) -> GridDeform {
    let total = dims[0] * dims[1] * dims[2];
    GridDeform {
        dims,
        deltas: vec![[0.0; 3]; total],
        origin,
        size,
    }
}

/// Index into deltas for grid cell (ix, iy, iz).
#[allow(dead_code)]
pub fn grid_index(dims: [usize; 3], ix: usize, iy: usize, iz: usize) -> usize {
    ix + dims[0] * (iy + dims[1] * iz)
}

/// Set displacement at grid cell.
#[allow(dead_code)]
pub fn set_grid_delta(gd: &mut GridDeform, ix: usize, iy: usize, iz: usize, delta: [f32; 3]) {
    let idx = grid_index(gd.dims, ix, iy, iz);
    if idx < gd.deltas.len() {
        gd.deltas[idx] = delta;
    }
}

/// Trilinear interpolation of a unit cell value.
#[allow(dead_code)]
pub fn trilinear(c: &[[f32; 3]; 8], u: f32, v: f32, w: f32) -> [f32; 3] {
    let mut r = [0.0f32; 3];
    let weights = [
        (1.0 - u) * (1.0 - v) * (1.0 - w),
        u * (1.0 - v) * (1.0 - w),
        (1.0 - u) * v * (1.0 - w),
        u * v * (1.0 - w),
        (1.0 - u) * (1.0 - v) * w,
        u * (1.0 - v) * w,
        (1.0 - u) * v * w,
        u * v * w,
    ];
    for (i, &wt) in weights.iter().enumerate() {
        r[0] += wt * c[i][0];
        r[1] += wt * c[i][1];
        r[2] += wt * c[i][2];
    }
    r
}

/// Deform a single vertex using the grid.
#[allow(dead_code)]
pub fn deform_vertex(gd: &GridDeform, v: [f32; 3]) -> [f32; 3] {
    let mut local = [0.0f32; 3];
    for k in 0..3 {
        let s = gd.size[k];
        if s < 1e-12 {
            local[k] = 0.0;
        } else {
            local[k] = ((v[k] - gd.origin[k]) / s).clamp(0.0, 1.0);
        }
    }
    let nx = gd.dims[0].saturating_sub(1).max(1);
    let ny = gd.dims[1].saturating_sub(1).max(1);
    let nz = gd.dims[2].saturating_sub(1).max(1);
    let fx = local[0] * nx as f32;
    let fy = local[1] * ny as f32;
    let fz = local[2] * nz as f32;
    let ix = (fx as usize).min(nx - 1);
    let iy = (fy as usize).min(ny - 1);
    let iz = (fz as usize).min(nz - 1);
    let u = fx - ix as f32;
    let vv = fy - iy as f32;
    let w = fz - iz as f32;
    let corners_idx = [
        grid_index(gd.dims, ix, iy, iz),
        grid_index(gd.dims, ix + 1, iy, iz),
        grid_index(gd.dims, ix, iy + 1, iz),
        grid_index(gd.dims, ix + 1, iy + 1, iz),
        grid_index(gd.dims, ix, iy, iz + 1),
        grid_index(gd.dims, ix + 1, iy, iz + 1),
        grid_index(gd.dims, ix, iy + 1, iz + 1),
        grid_index(gd.dims, ix + 1, iy + 1, iz + 1),
    ];
    let mut c = [[0.0f32; 3]; 8];
    for (i, &ci) in corners_idx.iter().enumerate() {
        if ci < gd.deltas.len() {
            c[i] = gd.deltas[ci];
        }
    }
    let delta = trilinear(&c, u, vv, w);
    [v[0] + delta[0], v[1] + delta[1], v[2] + delta[2]]
}

/// Apply grid deformation to all positions.
#[allow(dead_code)]
pub fn apply_grid_deform(positions: &[[f32; 3]], gd: &GridDeform) -> GridDeformResult {
    let mut max_disp = 0.0f32;
    let new_pos: Vec<[f32; 3]> = positions
        .iter()
        .map(|&v| {
            let d = deform_vertex(gd, v);
            let dist =
                ((d[0] - v[0]).powi(2) + (d[1] - v[1]).powi(2) + (d[2] - v[2]).powi(2)).sqrt();
            max_disp = max_disp.max(dist);
            d
        })
        .collect();
    GridDeformResult {
        positions: new_pos,
        max_displacement: max_disp,
    }
}

/// Total control point count.
#[allow(dead_code)]
pub fn control_point_count(gd: &GridDeform) -> usize {
    gd.deltas.len()
}

/// Export to JSON.
#[allow(dead_code)]
pub fn grid_deform_to_json(r: &GridDeformResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"max_displacement\":{:.6},\"pi_ref\":{:.6}}}",
        r.positions.len(),
        r.max_displacement,
        PI
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid_deform() {
        let gd = new_grid_deform([2, 2, 2], [0.0; 3], [1.0; 3]);
        assert_eq!(control_point_count(&gd), 8);
    }

    #[test]
    fn test_grid_index() {
        let idx = grid_index([3, 3, 3], 1, 1, 1);
        assert_eq!(idx, 13);
    }

    #[test]
    fn test_set_grid_delta() {
        let mut gd = new_grid_deform([2, 2, 2], [0.0; 3], [1.0; 3]);
        set_grid_delta(&mut gd, 0, 0, 0, [0.1, 0.0, 0.0]);
        assert!((gd.deltas[0][0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_trilinear_zero_deltas() {
        let c = [[0.0f32; 3]; 8];
        let r = trilinear(&c, 0.5, 0.5, 0.5);
        assert!((r[0]).abs() < 1e-9);
    }

    #[test]
    fn test_deform_vertex_identity() {
        let gd = new_grid_deform([2, 2, 2], [0.0; 3], [1.0; 3]);
        let v = [0.5, 0.5, 0.5];
        let d = deform_vertex(&gd, v);
        assert!((d[0] - v[0]).abs() < 1e-5);
    }

    #[test]
    fn test_apply_grid_deform_empty() {
        let gd = new_grid_deform([2, 2, 2], [0.0; 3], [1.0; 3]);
        let r = apply_grid_deform(&[], &gd);
        assert_eq!(r.positions.len(), 0);
        assert!((r.max_displacement).abs() < 1e-9);
    }

    #[test]
    fn test_apply_grid_deform_with_delta() {
        let mut gd = new_grid_deform([2, 2, 2], [0.0; 3], [1.0; 3]);
        set_grid_delta(&mut gd, 0, 0, 0, [1.0, 0.0, 0.0]);
        let positions = vec![[0.0, 0.0, 0.0]];
        let r = apply_grid_deform(&positions, &gd);
        assert!(r.max_displacement > 0.0);
    }

    #[test]
    fn test_grid_deform_to_json() {
        let r = GridDeformResult {
            positions: vec![[0.0; 3]],
            max_displacement: 0.1,
        };
        let j = grid_deform_to_json(&r);
        assert!(j.contains("\"vertex_count\":1"));
    }

    #[test]
    fn test_deform_vertex_out_of_bounds() {
        let gd = new_grid_deform([2, 2, 2], [0.0; 3], [1.0; 3]);
        let v = [5.0, 5.0, 5.0];
        let d = deform_vertex(&gd, v);
        // Should be clamped, no panic
        assert!(d[0].is_finite());
    }
}
