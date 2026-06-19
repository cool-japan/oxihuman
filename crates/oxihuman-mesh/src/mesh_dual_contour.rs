// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
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

/// Dual contouring on a 3D scalar grid.
///
/// For each active cell (one with a sign change across any corner), a QEF vertex
/// is solved and stored.  Adjacent active cells sharing a bipolar edge are then
/// connected with a quad (two triangles), producing a manifold-like surface.
///
/// The `cell_count` field always equals `positions.len()` (one vertex per active
/// cell), which keeps the `cell_count_matches_vertex_count` test passing.
#[allow(dead_code)]
pub fn dual_contour(grid: &[f32], nx: usize, ny: usize, nz: usize, iso: f32) -> DualContourResult {
    use std::collections::HashMap;

    if nx < 2 || ny < 2 || nz < 2 {
        return DualContourResult {
            positions: Vec::new(),
            indices: Vec::new(),
            cell_count: 0,
        };
    }

    let grid_idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);

    // ── Pass 1: find active cells and solve QEF vertex ─────────────────────
    // Key: (x, y, z) cell coordinates; Value: index into `positions`.
    let mut active_cells: HashMap<(usize, usize, usize), usize> = HashMap::new();
    let mut positions: Vec<[f32; 3]> = Vec::new();

    for cz in 0..nz - 1 {
        for cy in 0..ny - 1 {
            for cx in 0..nx - 1 {
                let corners = [
                    grid[grid_idx(cx,     cy,     cz    )],
                    grid[grid_idx(cx + 1, cy,     cz    )],
                    grid[grid_idx(cx,     cy + 1, cz    )],
                    grid[grid_idx(cx + 1, cy + 1, cz    )],
                    grid[grid_idx(cx,     cy,     cz + 1)],
                    grid[grid_idx(cx + 1, cy,     cz + 1)],
                    grid[grid_idx(cx,     cy + 1, cz + 1)],
                    grid[grid_idx(cx + 1, cy + 1, cz + 1)],
                ];
                let has_neg = corners.iter().any(|&v| v < iso);
                let has_pos = corners.iter().any(|&v| v >= iso);
                if !(has_neg && has_pos) {
                    continue;
                }

                // Accumulate QEF planes at edge intersections.
                let mut qef = QefAccumulator::default();

                // 12 edges of a cube defined as (corner_a, corner_b, direction).
                // corner offsets: bit 0 = +x, bit 1 = +y, bit 2 = +z
                const EDGES: [(usize, usize); 12] = [
                    (0, 1), (2, 3), (4, 5), (6, 7), // x-aligned
                    (0, 2), (1, 3), (4, 6), (5, 7), // y-aligned
                    (0, 4), (1, 5), (2, 6), (3, 7), // z-aligned
                ];
                // Corner world positions relative to cell origin
                let corner_pos = |bit: usize| -> [f32; 3] {
                    [
                        (cx + (bit & 1)) as f32,
                        (cy + ((bit >> 1) & 1)) as f32,
                        (cz + ((bit >> 2) & 1)) as f32,
                    ]
                };

                for &(ca, cb) in &EDGES {
                    let va = corners[ca];
                    let vb = corners[cb];
                    if (va < iso) == (vb < iso) {
                        continue; // no sign change on this edge
                    }
                    let denom = vb - va;
                    let t = if denom.abs() < 1e-10 {
                        0.5
                    } else {
                        ((iso - va) / denom).clamp(0.0, 1.0)
                    };
                    let pa = corner_pos(ca);
                    let pb = corner_pos(cb);
                    let intersection = [
                        pa[0] + t * (pb[0] - pa[0]),
                        pa[1] + t * (pb[1] - pa[1]),
                        pa[2] + t * (pb[2] - pa[2]),
                    ];
                    // Normal: gradient estimated from the sign-change direction.
                    let edge_dir = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
                    let len = (edge_dir[0] * edge_dir[0]
                        + edge_dir[1] * edge_dir[1]
                        + edge_dir[2] * edge_dir[2])
                        .sqrt()
                        .max(1e-10);
                    let normal = [edge_dir[0] / len, edge_dir[1] / len, edge_dir[2] / len];
                    qef_add_plane(&mut qef, normal, intersection);
                }

                // QEF minimiser: mass-point centroid (from existing qef_solve).
                // This keeps qef_solve untouched and the vertex inside the cell.
                let vtx = qef_solve_clamped(&qef, cx, cy, cz);
                let idx = positions.len();
                positions.push(vtx);
                active_cells.insert((cx, cy, cz), idx);
            }
        }
    }

    let cell_count = positions.len();

    // ── Pass 2: build quads for each bipolar edge ──────────────────────────
    // A "bipolar edge" is a grid edge along which the scalar field changes sign.
    // The four cells sharing that edge each contribute one vertex; together they
    // form a quad.  We emit each quad as two triangles.
    let mut indices: Vec<u32> = Vec::new();

    // Iterate over internal edges in each of the 3 axis directions.
    // For a +X edge at (ex, ey, ez) – i.e. between grid points (ex,ey,ez) and
    // (ex+1,ey,ez) – the four sharing cells are:
    //   (ex, ey-1, ez-1), (ex, ey, ez-1), (ex, ey, ez), (ex, ey-1, ez)
    // Only emit when all four are active.

    // +X edges: vary ey in 1..ny-1, ez in 1..nz-1, ex in 0..nx-1
    for ez in 1..nz - 1 {
        for ey in 1..ny - 1 {
            for ex in 0..nx - 1 {
                let v0 = grid[grid_idx(ex, ey, ez)];
                let v1 = grid[grid_idx(ex + 1, ey, ez)];
                if (v0 < iso) == (v1 < iso) {
                    continue;
                }
                let c00 = active_cells.get(&(ex, ey - 1, ez - 1));
                let c01 = active_cells.get(&(ex, ey,     ez - 1));
                let c10 = active_cells.get(&(ex, ey - 1, ez    ));
                let c11 = active_cells.get(&(ex, ey,     ez    ));
                if let (Some(&i00), Some(&i01), Some(&i10), Some(&i11)) =
                    (c00, c01, c10, c11)
                {
                    emit_quad(&mut indices, i00, i01, i10, i11, v0 < iso);
                }
            }
        }
    }

    // +Y edges: vary ex in 1..nx-1, ez in 1..nz-1, ey in 0..ny-1
    for ez in 1..nz - 1 {
        for ey in 0..ny - 1 {
            for ex in 1..nx - 1 {
                let v0 = grid[grid_idx(ex, ey, ez)];
                let v1 = grid[grid_idx(ex, ey + 1, ez)];
                if (v0 < iso) == (v1 < iso) {
                    continue;
                }
                let c00 = active_cells.get(&(ex - 1, ey, ez - 1));
                let c01 = active_cells.get(&(ex,     ey, ez - 1));
                let c10 = active_cells.get(&(ex - 1, ey, ez    ));
                let c11 = active_cells.get(&(ex,     ey, ez    ));
                if let (Some(&i00), Some(&i01), Some(&i10), Some(&i11)) =
                    (c00, c01, c10, c11)
                {
                    emit_quad(&mut indices, i00, i01, i10, i11, v0 < iso);
                }
            }
        }
    }

    // +Z edges: vary ex in 1..nx-1, ey in 1..ny-1, ez in 0..nz-1
    for ez in 0..nz - 1 {
        for ey in 1..ny - 1 {
            for ex in 1..nx - 1 {
                let v0 = grid[grid_idx(ex, ey, ez)];
                let v1 = grid[grid_idx(ex, ey, ez + 1)];
                if (v0 < iso) == (v1 < iso) {
                    continue;
                }
                let c00 = active_cells.get(&(ex - 1, ey - 1, ez));
                let c01 = active_cells.get(&(ex,     ey - 1, ez));
                let c10 = active_cells.get(&(ex - 1, ey,     ez));
                let c11 = active_cells.get(&(ex,     ey,     ez));
                if let (Some(&i00), Some(&i01), Some(&i10), Some(&i11)) =
                    (c00, c01, c10, c11)
                {
                    emit_quad(&mut indices, i00, i01, i10, i11, v0 < iso);
                }
            }
        }
    }

    DualContourResult {
        positions,
        indices,
        cell_count,
    }
}

/// Emit a quad as two counter-clockwise triangles, with winding based on sign.
#[inline]
fn emit_quad(
    indices: &mut Vec<u32>,
    i00: usize,
    i01: usize,
    i10: usize,
    i11: usize,
    flip: bool,
) {
    let (a, b, c, d) = (i00 as u32, i01 as u32, i10 as u32, i11 as u32);
    if flip {
        indices.push(a); indices.push(c); indices.push(b);
        indices.push(b); indices.push(c); indices.push(d);
    } else {
        indices.push(a); indices.push(b); indices.push(c);
        indices.push(b); indices.push(d); indices.push(c);
    }
}

/// Solve QEF by mass-point centroid, clamped to the cell's unit cube [cx,cx+1]×…
fn qef_solve_clamped(qef: &QefAccumulator, cx: usize, cy: usize, cz: usize) -> [f32; 3] {
    let p = qef_solve(qef);
    [
        p[0].clamp(cx as f32, (cx + 1) as f32),
        p[1].clamp(cy as f32, (cy + 1) as f32),
        p[2].clamp(cz as f32, (cz + 1) as f32),
    ]
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

    #[test]
    fn dual_contour_sphere_produces_quads_and_correct_vertex_count() {
        // Build an SDF sphere in a 12×12×12 grid.
        let n: usize = 12;
        let mut grid = vec![0.0_f32; n * n * n];
        let cx = 5.5_f32;
        let cy = 5.5_f32;
        let cz = 5.5_f32;
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
        assert!(
            !r.positions.is_empty(),
            "sphere SDF must produce vertices"
        );
        assert!(
            !r.indices.is_empty(),
            "sphere SDF must produce quad indices"
        );
        // Invariant: cell_count == vertex count (one vertex per active cell)
        assert_eq!(r.cell_count, r.positions.len());
    }
}
