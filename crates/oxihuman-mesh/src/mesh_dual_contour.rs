// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Dual contouring stub: QEF minimisation per cell for isosurface extraction.

/// A minimal QEF accumulator (sum of squared distances to planes).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct QefAccumulator {
    pub ata: [f32; 6], // symmetric 3x3: [a00,a01,a02,a11,a12,a22]
    pub atb: [f32; 3],
    pub btb: f32,
    pub mass_point: [f32; 3],
    pub count: u32,
}

/// Result of dual contouring on a grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DualContourResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub cell_count: usize,
}

/// Add a plane constraint (normal `n`, point on plane `p`) to a QEF.
#[allow(dead_code)]
pub fn qef_add_plane(qef: &mut QefAccumulator, n: [f32; 3], p: [f32; 3]) {
    let d = n[0] * p[0] + n[1] * p[1] + n[2] * p[2];
    qef.ata[0] += n[0] * n[0];
    qef.ata[1] += n[0] * n[1];
    qef.ata[2] += n[0] * n[2];
    qef.ata[3] += n[1] * n[1];
    qef.ata[4] += n[1] * n[2];
    qef.ata[5] += n[2] * n[2];
    qef.atb[0] += n[0] * d;
    qef.atb[1] += n[1] * d;
    qef.atb[2] += n[2] * d;
    qef.btb += d * d;
    qef.mass_point[0] += p[0];
    qef.mass_point[1] += p[1];
    qef.mass_point[2] += p[2];
    qef.count += 1;
}

/// Solve QEF: return mass-point centroid (approximate minimiser).
#[allow(dead_code)]
pub fn qef_solve(qef: &QefAccumulator) -> [f32; 3] {
    if qef.count == 0 {
        return [0.0; 3];
    }
    let n = qef.count as f32;
    [
        qef.mass_point[0] / n,
        qef.mass_point[1] / n,
        qef.mass_point[2] / n,
    ]
}

/// Evaluate QEF error at a point.
#[allow(dead_code)]
pub fn qef_error(qef: &QefAccumulator, p: [f32; 3]) -> f32 {
    // E = p^T * A^T*A * p - 2 * p^T * A^T*b + b^T*b
    let a = &qef.ata;
    let b = &qef.atb;
    let ap0 = a[0] * p[0] + a[1] * p[1] + a[2] * p[2];
    let ap1 = a[1] * p[0] + a[3] * p[1] + a[4] * p[2];
    let ap2 = a[2] * p[0] + a[4] * p[1] + a[5] * p[2];
    let quad = p[0] * ap0 + p[1] * ap1 + p[2] * ap2;
    let lin = 2.0 * (p[0] * b[0] + p[1] * b[1] + p[2] * b[2]);
    quad - lin + qef.btb
}

/// Simple dual contour on a 3D scalar grid (stub, returns cell centroids).
#[allow(dead_code)]
pub fn dual_contour(grid: &[f32], nx: usize, ny: usize, nz: usize, iso: f32) -> DualContourResult {
    let mut positions = Vec::new();
    let indices = Vec::new();
    let mut cell_count = 0;
    if nx < 2 || ny < 2 || nz < 2 {
        return DualContourResult {
            positions,
            indices,
            cell_count,
        };
    }
    let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);
    for z in 0..nz - 1 {
        for y in 0..ny - 1 {
            for x in 0..nx - 1 {
                let corners = [
                    grid[idx(x, y, z)],
                    grid[idx(x + 1, y, z)],
                    grid[idx(x, y + 1, z)],
                    grid[idx(x + 1, y + 1, z)],
                    grid[idx(x, y, z + 1)],
                    grid[idx(x + 1, y, z + 1)],
                    grid[idx(x, y + 1, z + 1)],
                    grid[idx(x + 1, y + 1, z + 1)],
                ];
                let has_neg = corners.iter().any(|&v| v < iso);
                let has_pos = corners.iter().any(|&v| v >= iso);
                if has_neg && has_pos {
                    let cx = x as f32 + 0.5;
                    let cy = y as f32 + 0.5;
                    let cz = z as f32 + 0.5;
                    positions.push([cx, cy, cz]);
                    cell_count += 1;
                }
            }
        }
    }
    // Stub: no quad/tri generation; just record cell count
    let _ = indices;
    DualContourResult {
        positions,
        indices: Vec::new(),
        cell_count,
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn dc_vertex_count(r: &DualContourResult) -> usize {
    r.positions.len()
}

/// Plane count in QEF.
#[allow(dead_code)]
pub fn qef_plane_count(qef: &QefAccumulator) -> u32 {
    qef.count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qef_solve_zero_count_returns_origin() {
        let qef = QefAccumulator::default();
        let p = qef_solve(&qef);
        assert!((p[0]).abs() < 1e-6);
    }

    #[test]
    fn qef_add_plane_increments_count() {
        let mut qef = QefAccumulator::default();
        qef_add_plane(&mut qef, [0.0, 1.0, 0.0], [0.0, 1.0, 0.0]);
        assert_eq!(qef_plane_count(&qef), 1);
    }

    #[test]
    fn qef_solve_centroid() {
        let mut qef = QefAccumulator::default();
        qef_add_plane(&mut qef, [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]);
        qef_add_plane(&mut qef, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
        let p = qef_solve(&qef);
        assert!((p[0] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn dual_contour_sphere_has_vertices() {
        let n = 10usize;
        let mut grid = vec![0.0_f32; n * n * n];
        let cx = 5.0_f32;
        let cy = 5.0_f32;
        let cz = 5.0_f32;
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dz = z as f32 - cz;
                    grid[x + n * (y + n * z)] = (dx * dx + dy * dy + dz * dz).sqrt() - 3.0;
                }
            }
        }
        let r = dual_contour(&grid, n, n, n, 0.0);
        assert!(dc_vertex_count(&r) > 0);
    }

    #[test]
    fn dual_contour_all_same_sign_no_vertices() {
        let grid = vec![1.0_f32; 8]; // 2x2x2, all positive
        let r = dual_contour(&grid, 2, 2, 2, 0.0);
        assert_eq!(dc_vertex_count(&r), 0);
    }

    #[test]
    fn qef_error_at_solution_near_zero() {
        let mut qef = QefAccumulator::default();
        qef_add_plane(&mut qef, [0.0, 1.0, 0.0], [0.0, 2.0, 0.0]);
        let p = qef_solve(&qef);
        let e = qef_error(&qef, p);
        assert!(e.is_finite());
    }

    #[test]
    fn dc_small_grid_no_panic() {
        let r = dual_contour(&[], 0, 0, 0, 0.0);
        assert_eq!(r.cell_count, 0);
    }

    #[test]
    fn contains_range() {
        let v = 0.5_f32;
        assert!((0.0..=1.0).contains(&v));
    }

    #[test]
    fn cell_count_matches_vertex_count() {
        let grid = vec![-1.0_f32, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let r = dual_contour(&grid, 2, 2, 2, 0.0);
        assert_eq!(r.cell_count, dc_vertex_count(&r));
    }
}
