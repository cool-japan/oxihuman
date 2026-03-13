// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dual contouring isosurface extraction.

/// A QEF (Quadric Error Function) accumulator used in dual contouring.
#[derive(Clone, Debug, Default)]
pub struct QefDc {
    ata: [[f32; 3]; 3],
    atb: [f32; 3],
    btb: f32,
    pub plane_count: usize,
}

impl QefDc {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Result of dual contouring extraction.
#[derive(Clone, Debug, Default)]
pub struct DualContourDcResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Add a plane (normal + offset) to a QEF accumulator.
pub fn qef_dc_add_plane(qef: &mut QefDc, n: [f32; 3], d: f32) {
    for i in 0..3 {
        for j in 0..3 {
            qef.ata[i][j] += n[i] * n[j];
        }
        qef.atb[i] += n[i] * d;
    }
    qef.btb += d * d;
    qef.plane_count += 1;
}

/// Evaluate residual error at a given position.
pub fn qef_dc_error(qef: &QefDc, p: [f32; 3]) -> f32 {
    let mut err = qef.btb;
    for (i, &pi) in p.iter().enumerate() {
        let row: f32 = qef.ata[i]
            .iter()
            .zip(p.iter())
            .map(|(&a, &pj)| a * pj)
            .sum();
        err += row * pi - 2.0 * qef.atb[i] * pi;
    }
    err
}

/// Solve for the minimising vertex position (simplified: uses cell centre).
pub fn qef_dc_solve(qef: &QefDc, cell_centre: [f32; 3]) -> [f32; 3] {
    if qef.plane_count == 0 {
        return cell_centre;
    }
    // Diagonal pseudo-solve: gradient descent one step from centre
    let mut g = [0.0_f32; 3];
    for (i, gi) in g.iter_mut().enumerate() {
        let v: f32 = qef.ata[i]
            .iter()
            .zip(cell_centre.iter())
            .map(|(&a, &c)| a * c)
            .sum();
        *gi = v - qef.atb[i];
    }
    let denom: f32 = g.iter().map(|&x| x * x).sum::<f32>().max(1e-12);
    let num: f32 = g
        .iter()
        .enumerate()
        .map(|(i, &gi)| {
            let ap_i: f32 = qef.ata[i]
                .iter()
                .zip(cell_centre.iter())
                .map(|(&a, &c)| a * c)
                .sum();
            (ap_i - qef.atb[i]) * gi
        })
        .sum();
    let alpha = num / denom;
    [
        cell_centre[0] - alpha * g[0],
        cell_centre[1] - alpha * g[1],
        cell_centre[2] - alpha * g[2],
    ]
}

/// Run dual contouring on a scalar field given as a closure.
///
/// `field(x,y,z)` returns the scalar value at that position.
/// `res` is the number of cells along each axis.
/// `bounds_min / bounds_max` define the grid bounding box.
pub fn dual_contour_dc<F>(
    field: F,
    res: usize,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
) -> DualContourDcResult
where
    F: Fn(f32, f32, f32) -> f32,
{
    if res < 2 {
        return DualContourDcResult::default();
    }
    let step = [
        (bounds_max[0] - bounds_min[0]) / res as f32,
        (bounds_max[1] - bounds_min[1]) / res as f32,
        (bounds_max[2] - bounds_min[2]) / res as f32,
    ];

    // Sample grid
    let mut grid = vec![0.0_f32; (res + 1).pow(3)];
    let idx = |x: usize, y: usize, z: usize| x * (res + 1) * (res + 1) + y * (res + 1) + z;

    for xi in 0..=res {
        for yi in 0..=res {
            for zi in 0..=res {
                let px = bounds_min[0] + xi as f32 * step[0];
                let py = bounds_min[1] + yi as f32 * step[1];
                let pz = bounds_min[2] + zi as f32 * step[2];
                grid[idx(xi, yi, zi)] = field(px, py, pz);
            }
        }
    }

    let mut positions: Vec<[f32; 3]> = Vec::new();
    // Map cell -> vertex index
    let mut cell_to_vert: std::collections::HashMap<(usize, usize, usize), u32> =
        std::collections::HashMap::new();

    // For each interior cell, check sign changes and place a vertex
    for xi in 0..res {
        for yi in 0..res {
            for zi in 0..res {
                let v000 = grid[idx(xi, yi, zi)];
                let v100 = grid[idx(xi + 1, yi, zi)];
                let v010 = grid[idx(xi, yi + 1, zi)];
                let v001 = grid[idx(xi, yi, zi + 1)];
                let has_sign_change = [v100, v010, v001]
                    .iter()
                    .any(|&v| (v >= 0.0) != (v000 >= 0.0));
                if has_sign_change {
                    let cx = bounds_min[0] + (xi as f32 + 0.5) * step[0];
                    let cy = bounds_min[1] + (yi as f32 + 0.5) * step[1];
                    let cz = bounds_min[2] + (zi as f32 + 0.5) * step[2];
                    let vi = positions.len() as u32;
                    positions.push([cx, cy, cz]);
                    cell_to_vert.insert((xi, yi, zi), vi);
                }
            }
        }
    }

    // Connect quads along sign-change edges
    let mut indices: Vec<u32> = Vec::new();
    for xi in 1..res {
        for yi in 1..res {
            for zi in 1..res {
                // x-edge sign change
                let va = grid[idx(xi, yi, zi)];
                let vb = grid[idx(xi + 1, yi, zi)];
                if (va >= 0.0) != (vb >= 0.0) {
                    let cells = [
                        (xi, yi, zi),
                        (xi, yi - 1, zi),
                        (xi, yi - 1, zi - 1),
                        (xi, yi, zi - 1),
                    ];
                    let verts: Vec<u32> = cells
                        .iter()
                        .filter_map(|k| cell_to_vert.get(k).copied())
                        .collect();
                    if verts.len() == 4 {
                        indices.extend_from_slice(&[verts[0], verts[1], verts[2]]);
                        indices.extend_from_slice(&[verts[0], verts[2], verts[3]]);
                    }
                }
            }
        }
    }

    DualContourDcResult { positions, indices }
}

/// Return the vertex count from a dual contouring result.
pub fn dc_vertex_count_dc(r: &DualContourDcResult) -> usize {
    r.positions.len()
}

/// Return the triangle count from a dual contouring result.
pub fn dc_triangle_count_dc(r: &DualContourDcResult) -> usize {
    r.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    /* sphere SDF centred at origin, radius 1 */
    fn sphere_sdf(x: f32, y: f32, z: f32) -> f32 {
        (x * x + y * y + z * z).sqrt() - 1.0
    }

    #[test]
    fn qef_dc_starts_zero() {
        let q = QefDc::new();
        assert_eq!(q.plane_count, 0);
    }

    #[test]
    fn qef_dc_add_plane_increments_count() {
        let mut q = QefDc::new();
        qef_dc_add_plane(&mut q, [1.0, 0.0, 0.0], 1.0);
        assert_eq!(q.plane_count, 1);
    }

    #[test]
    fn qef_dc_error_nonnegative_at_centre() {
        let mut q = QefDc::new();
        qef_dc_add_plane(&mut q, [1.0, 0.0, 0.0], 0.5);
        let err = qef_dc_error(&q, [0.0; 3]);
        assert!(err >= 0.0, "error={err}");
    }

    #[test]
    fn qef_dc_solve_returns_some_point() {
        let mut q = QefDc::new();
        qef_dc_add_plane(&mut q, [1.0, 0.0, 0.0], 1.0);
        let p = qef_dc_solve(&q, [0.5, 0.5, 0.5]);
        /* Just ensure it returns a finite point */
        assert!(p.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn dual_contour_dc_returns_vertices() {
        let r = dual_contour_dc(sphere_sdf, 4, [-2.0; 3], [2.0; 3]);
        assert!(!r.positions.is_empty());
    }

    #[test]
    fn dual_contour_dc_indices_valid() {
        let r = dual_contour_dc(sphere_sdf, 4, [-2.0; 3], [2.0; 3]);
        let n = r.positions.len() as u32;
        for &i in &r.indices {
            assert!(i < n);
        }
    }

    #[test]
    fn dc_vertex_count_dc_consistent() {
        let r = dual_contour_dc(sphere_sdf, 4, [-2.0; 3], [2.0; 3]);
        assert_eq!(dc_vertex_count_dc(&r), r.positions.len());
    }

    #[test]
    fn dc_triangle_count_dc_consistent() {
        let r = dual_contour_dc(sphere_sdf, 4, [-2.0; 3], [2.0; 3]);
        assert_eq!(dc_triangle_count_dc(&r) * 3, r.indices.len());
    }

    #[test]
    fn dual_contour_dc_low_res_noop() {
        let r = dual_contour_dc(sphere_sdf, 1, [-2.0; 3], [2.0; 3]);
        assert_eq!(r.positions.len(), 0);
    }
}
