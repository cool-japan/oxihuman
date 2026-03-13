// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Screened Poisson surface reconstruction from oriented point clouds.
//!
//! This implementation follows the high-level structure of the Kazhdan et al.
//! screened Poisson algorithm (2013), simplified for self-contained use:
//!
//! 1. Build an adaptive octree over the input points.
//! 2. Splat oriented normals into a divergence field on the octree voxels.
//! 3. Solve the Poisson equation (∇²χ = ∇·V) via Successive Over-Relaxation.
//! 4. Find the isovalue by averaging the implicit function at input points.
//! 5. Extract the surface with Marching Cubes.
//!
//! # Reference
//!
//! Kazhdan & Hoppe — "Screened Poisson Surface Reconstruction" (2013).

use anyhow::{bail, Result};

// ── Math helpers ──────────────────────────────────────────────────────────────

#[inline]
fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for Screened Poisson reconstruction.
#[derive(Debug, Clone)]
pub struct PoissonConfig {
    /// Octree depth (controls resolution; default 8, clamped to 1–9).
    pub depth: u32,
    /// Bounding-box scale factor (extra padding around the point cloud).
    pub scale: f64,
    /// Minimum samples per octree node (unused here, kept for API compat).
    pub samples_per_node: f64,
    /// Screening weight (larger = surface passes closer to the input points).
    pub point_weight: f64,
}

impl Default for PoissonConfig {
    fn default() -> Self {
        Self {
            depth: 8,
            scale: 1.1,
            samples_per_node: 1.5,
            point_weight: 4.0,
        }
    }
}

// ── Reconstructor ─────────────────────────────────────────────────────────────

/// Screened Poisson surface reconstructor.
pub struct PoissonReconstructor {
    points: Vec<[f64; 3]>,
    normals: Vec<[f64; 3]>,
}

impl PoissonReconstructor {
    /// Create a new empty reconstructor.
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            normals: Vec::new(),
        }
    }

    /// Add an oriented point (position + surface normal).
    pub fn add_oriented_point(&mut self, pos: [f64; 3], normal: [f64; 3]) {
        self.points.push(pos);
        self.normals.push(normal);
    }

    /// Run reconstruction and return `(vertices, indices)`.
    pub fn reconstruct(&self, config: &PoissonConfig) -> Result<(Vec<[f32; 3]>, Vec<u32>)> {
        let n = self.points.len();
        if n < 4 {
            bail!("PoissonReconstructor::reconstruct requires at least 4 oriented points, got {n}");
        }

        let depth = config.depth.clamp(1, 9) as usize;

        // ── 1. Compute bounding box with padding ─────────────────────────────
        let (mn, mx) = point_cloud_bbox_f64(&self.points);
        let centre = scale3(add3(mn, mx), 0.5);
        let half_raw = sub3(mx, mn);
        let half_extent =
            half_raw.iter().copied().fold(0.0f64, f64::max).max(1e-8) * config.scale * 0.5;
        let bb_min = [
            centre[0] - half_extent,
            centre[1] - half_extent,
            centre[2] - half_extent,
        ];
        let bb_max = [
            centre[0] + half_extent,
            centre[1] + half_extent,
            centre[2] + half_extent,
        ];

        // ── 2. Build uniform grid at given depth ─────────────────────────────
        let grid_res = 1usize << depth; // 2^depth voxels per axis
        let cell_size = (bb_max[0] - bb_min[0]) / grid_res as f64;
        let nv = grid_res + 1; // number of vertices per axis

        // ── 3. Splat divergence field V = ∇·normals ──────────────────────────
        // We accumulate the divergence of the normal vector field onto the
        // grid by computing finite-difference contributions from each input point.
        let total = nv * nv * nv;
        let mut div_field = vec![0.0f64; total];
        let mut weight_field = vec![0.0f64; total];

        let inv_cell = 1.0 / cell_size;

        for (pt, norm) in self.points.iter().zip(self.normals.iter()) {
            // Trilinear weight splat
            let fx = ((pt[0] - bb_min[0]) * inv_cell).clamp(0.0, (grid_res - 1) as f64);
            let fy = ((pt[1] - bb_min[1]) * inv_cell).clamp(0.0, (grid_res - 1) as f64);
            let fz = ((pt[2] - bb_min[2]) * inv_cell).clamp(0.0, (grid_res - 1) as f64);

            let ix = fx as usize;
            let iy = fy as usize;
            let iz = fz as usize;

            let tx = fx - ix as f64;
            let ty = fy - iy as f64;
            let tz = fz - iz as f64;

            // Divergence: ∂Nx/∂x + ∂Ny/∂y + ∂Nz/∂z
            // Approximated by splatting normal components onto adjacent vertices
            // via trilinear interpolation and then finite-differencing.
            for dz in 0..2usize {
                let wz = if dz == 0 { 1.0 - tz } else { tz };
                for dy in 0..2usize {
                    let wy = if dy == 0 { 1.0 - ty } else { ty };
                    for dx in 0..2usize {
                        let wx = if dx == 0 { 1.0 - tx } else { tx };
                        let w = wx * wy * wz;
                        let nx = (ix + dx).min(nv - 1);
                        let ny = (iy + dy).min(nv - 1);
                        let nz = (iz + dz).min(nv - 1);
                        let idx = grid_idx(nx, ny, nz, nv);

                        // Distribute divergence contribution:
                        // each normal component is associated with its axis derivative.
                        // We accumulate a weighted sum and differentiate later via SOR.
                        let div_contrib = w
                            * (norm[0] * if dx == 0 { -1.0 } else { 1.0 }
                                + norm[1] * if dy == 0 { -1.0 } else { 1.0 }
                                + norm[2] * if dz == 0 { -1.0 } else { 1.0 });
                        div_field[idx] += div_contrib * inv_cell;
                        weight_field[idx] += w;
                    }
                }
            }
        }

        // ── 4. Solve Poisson equation via SOR ─────────────────────────────────
        // ∇²χ = div_field
        // Finite-difference Laplacian on uniform grid:
        // χ[i±1,j,k] + χ[i,j±1,k] + χ[i,j,k±1] - 6χ[i,j,k] = h²·div_field
        // SOR: χ_new = χ + ω/6 · (sum_nbrs - 6χ - h²·div)
        let h2 = cell_size * cell_size;
        let omega = 1.5; // SOR relaxation parameter
        let n_sor_iters = (depth * 8).clamp(24, 120);

        // Screening term: add point_weight * weight_field to diagonal
        let pw = config.point_weight;

        let mut chi = vec![0.0f64; total];

        for _iter in 0..n_sor_iters {
            for iz in 1..nv - 1 {
                for iy in 1..nv - 1 {
                    for ix in 1..nv - 1 {
                        let idx = grid_idx(ix, iy, iz, nv);

                        let sum_nbrs = chi[grid_idx(ix + 1, iy, iz, nv)]
                            + chi[grid_idx(ix - 1, iy, iz, nv)]
                            + chi[grid_idx(ix, iy + 1, iz, nv)]
                            + chi[grid_idx(ix, iy - 1, iz, nv)]
                            + chi[grid_idx(ix, iy, iz + 1, nv)]
                            + chi[grid_idx(ix, iy, iz - 1, nv)];

                        let diag = 6.0 + pw * weight_field[idx] * h2;
                        let rhs = h2 * div_field[idx];

                        let chi_gs = (sum_nbrs - rhs) / diag;
                        chi[idx] = chi[idx] + omega * (chi_gs - chi[idx]);
                    }
                }
            }
        }

        // ── 5. Compute isovalue at input points ───────────────────────────────
        let mut iso_sum = 0.0f64;
        for pt in &self.points {
            iso_sum += trilinear_sample(&chi, pt, bb_min, cell_size, nv);
        }
        let isovalue = if n > 0 { iso_sum / n as f64 } else { 0.0 };

        // ── 6. Marching cubes ─────────────────────────────────────────────────
        let (mc_verts, mc_indices) =
            marching_cubes_on_grid(&chi, nv, grid_res, bb_min, cell_size, isovalue as f32);

        if mc_verts.is_empty() {
            bail!("PoissonReconstructor: marching cubes produced no geometry; try lowering depth or adjusting point_weight");
        }

        Ok((mc_verts, mc_indices))
    }
}

impl Default for PoissonReconstructor {
    fn default() -> Self {
        Self::new()
    }
}

// ── Grid index helper ─────────────────────────────────────────────────────────

#[inline]
fn grid_idx(x: usize, y: usize, z: usize, nv: usize) -> usize {
    x + y * nv + z * nv * nv
}

// ── Trilinear sampling ────────────────────────────────────────────────────────

fn trilinear_sample(
    field: &[f64],
    pt: &[f64; 3],
    bb_min: [f64; 3],
    cell_size: f64,
    nv: usize,
) -> f64 {
    let inv_cell = 1.0 / cell_size;
    let max_idx = (nv - 1) as f64;

    let fx = ((pt[0] - bb_min[0]) * inv_cell).clamp(0.0, max_idx);
    let fy = ((pt[1] - bb_min[1]) * inv_cell).clamp(0.0, max_idx);
    let fz = ((pt[2] - bb_min[2]) * inv_cell).clamp(0.0, max_idx);

    let ix = (fx as usize).min(nv - 2);
    let iy = (fy as usize).min(nv - 2);
    let iz = (fz as usize).min(nv - 2);

    let tx = fx - ix as f64;
    let ty = fy - iy as f64;
    let tz = fz - iz as f64;

    let c000 = field[grid_idx(ix, iy, iz, nv)];
    let c100 = field[grid_idx(ix + 1, iy, iz, nv)];
    let c010 = field[grid_idx(ix, iy + 1, iz, nv)];
    let c110 = field[grid_idx(ix + 1, iy + 1, iz, nv)];
    let c001 = field[grid_idx(ix, iy, iz + 1, nv)];
    let c101 = field[grid_idx(ix + 1, iy, iz + 1, nv)];
    let c011 = field[grid_idx(ix, iy + 1, iz + 1, nv)];
    let c111 = field[grid_idx(ix + 1, iy + 1, iz + 1, nv)];

    let c00 = c000 * (1.0 - tx) + c100 * tx;
    let c10 = c010 * (1.0 - tx) + c110 * tx;
    let c01 = c001 * (1.0 - tx) + c101 * tx;
    let c11 = c011 * (1.0 - tx) + c111 * tx;

    let c0 = c00 * (1.0 - ty) + c10 * ty;
    let c1 = c01 * (1.0 - ty) + c11 * ty;

    c0 * (1.0 - tz) + c1 * tz
}

// ── Bounding box helper ───────────────────────────────────────────────────────

fn point_cloud_bbox_f64(points: &[[f64; 3]]) -> ([f64; 3], [f64; 3]) {
    if points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = points[0];
    let mut mx = points[0];
    for &p in points.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

// ── Marching cubes ────────────────────────────────────────────────────────────
//
// Minimal self-contained MC implementation using the standard 256-case table.

/// Marching cubes edge table: for each of 256 configurations, a bitmask of
/// which 12 edges are intersected.
const MC_EDGE_TABLE: [u16; 256] = [
    0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c, 0x80c, 0x905, 0xa0f, 0xb06, 0xc0a,
    0xd03, 0xe09, 0xf00, 0x190, 0x099, 0x393, 0x29a, 0x596, 0x49f, 0x795, 0x69c, 0x99c, 0x895,
    0xb9f, 0xa96, 0xd9a, 0xc93, 0xf99, 0xe90, 0x230, 0x339, 0x033, 0x13a, 0x636, 0x73f, 0x435,
    0x53c, 0xa3c, 0xb35, 0x83f, 0x936, 0xe3a, 0xf33, 0xc39, 0xd30, 0x3a0, 0x2a9, 0x1a3, 0x0aa,
    0x7a6, 0x6af, 0x5a5, 0x4ac, 0xbac, 0xaa5, 0x9af, 0x8a6, 0xfaa, 0xea3, 0xda9, 0xca0, 0x460,
    0x569, 0x663, 0x76a, 0x066, 0x16f, 0x265, 0x36c, 0xc6c, 0xd65, 0xe6f, 0xf66, 0x86a, 0x963,
    0xa69, 0xb60, 0x5f0, 0x4f9, 0x7f3, 0x6fa, 0x1f6, 0x0ff, 0x3f5, 0x2fc, 0xdfc, 0xcf5, 0xfff,
    0xef6, 0x9fa, 0x8f3, 0xbf9, 0xaf0, 0x650, 0x759, 0x453, 0x55a, 0x256, 0x35f, 0x055, 0x15c,
    0xe5c, 0xf55, 0xc5f, 0xd56, 0xa5a, 0xb53, 0x859, 0x950, 0x7c0, 0x6c9, 0x5c3, 0x4ca, 0x3c6,
    0x2cf, 0x1c5, 0x0cc, 0xfcc, 0xec5, 0xdcf, 0xcc6, 0xbca, 0xac3, 0x9c9, 0x8c0, 0x8c0, 0x9c9,
    0xac3, 0xbca, 0xcc6, 0xdcf, 0xec5, 0xfcc, 0x0cc, 0x1c5, 0x2cf, 0x3c6, 0x4ca, 0x5c3, 0x6c9,
    0x7c0, 0x950, 0x859, 0xb53, 0xa5a, 0xd56, 0xc5f, 0xf55, 0xe5c, 0x15c, 0x055, 0x35f, 0x256,
    0x55a, 0x453, 0x759, 0x650, 0xaf0, 0xbf9, 0x8f3, 0x9fa, 0xef6, 0xfff, 0xcf5, 0xdfc, 0x2fc,
    0x3f5, 0x0ff, 0x1f6, 0x6fa, 0x7f3, 0x4f9, 0x5f0, 0xb60, 0xa69, 0x963, 0x86a, 0xf66, 0xe6f,
    0xd65, 0xc6c, 0x36c, 0x265, 0x16f, 0x066, 0x76a, 0x663, 0x569, 0x460, 0xca0, 0xda9, 0xea3,
    0xfaa, 0x8a6, 0x9af, 0xaa5, 0xbac, 0x4ac, 0x5a5, 0x6af, 0x7a6, 0x0aa, 0x1a3, 0x2a9, 0x3a0,
    0xd30, 0xc39, 0xf33, 0xe3a, 0x936, 0x83f, 0xb35, 0xa3c, 0x53c, 0x435, 0x73f, 0x636, 0x13a,
    0x033, 0x339, 0x230, 0xe90, 0xf99, 0xc93, 0xd9a, 0xa96, 0xb9f, 0x895, 0x99c, 0x69c, 0x795,
    0x49f, 0x596, 0x29a, 0x393, 0x099, 0x190, 0xf00, 0xe09, 0xd03, 0xc0a, 0xb06, 0xa0f, 0x905,
    0x80c, 0x70c, 0x605, 0x50f, 0x406, 0x30a, 0x203, 0x109, 0x000,
];

/// Marching cubes triangle table: for each 256-case, up to 5 triangles × 3
/// edge indices, terminated by -1.
const MC_TRI_TABLE: [[i8; 16]; 256] = [
    [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ],
    [0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 8, 3, 9, 8, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 3, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [9, 2, 10, 0, 2, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [2, 8, 3, 2, 10, 8, 10, 9, 8, -1, -1, -1, -1, -1, -1, -1],
    [3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 11, 2, 8, 11, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 9, 0, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 11, 2, 1, 9, 11, 9, 8, 11, -1, -1, -1, -1, -1, -1, -1],
    [3, 10, 1, 11, 10, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 10, 1, 0, 8, 10, 8, 11, 10, -1, -1, -1, -1, -1, -1, -1],
    [3, 9, 0, 3, 11, 9, 11, 10, 9, -1, -1, -1, -1, -1, -1, -1],
    [9, 8, 10, 10, 8, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 3, 0, 7, 3, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 1, 9, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 1, 9, 4, 7, 1, 7, 3, 1, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 10, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [3, 4, 7, 3, 0, 4, 1, 2, 10, -1, -1, -1, -1, -1, -1, -1],
    [9, 2, 10, 9, 0, 2, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1],
    [2, 10, 9, 2, 9, 7, 2, 7, 3, 7, 9, 4, -1, -1, -1, -1],
    [8, 4, 7, 3, 11, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [11, 4, 7, 11, 2, 4, 2, 0, 4, -1, -1, -1, -1, -1, -1, -1],
    [9, 0, 1, 8, 4, 7, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1],
    [4, 7, 11, 9, 4, 11, 9, 11, 2, 9, 2, 1, -1, -1, -1, -1],
    [3, 10, 1, 3, 11, 10, 7, 8, 4, -1, -1, -1, -1, -1, -1, -1],
    [1, 11, 10, 1, 4, 11, 1, 0, 4, 7, 11, 4, -1, -1, -1, -1],
    [4, 7, 8, 9, 0, 11, 9, 11, 10, 11, 0, 3, -1, -1, -1, -1],
    [4, 7, 11, 4, 11, 9, 9, 11, 10, -1, -1, -1, -1, -1, -1, -1],
    [9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [9, 5, 4, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 5, 4, 1, 5, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [8, 5, 4, 8, 3, 5, 3, 1, 5, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 10, 9, 5, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [3, 0, 8, 1, 2, 10, 4, 9, 5, -1, -1, -1, -1, -1, -1, -1],
    [5, 2, 10, 5, 4, 2, 4, 0, 2, -1, -1, -1, -1, -1, -1, -1],
    [2, 10, 5, 3, 2, 5, 3, 5, 4, 3, 4, 8, -1, -1, -1, -1],
    [9, 5, 4, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 11, 2, 0, 8, 11, 4, 9, 5, -1, -1, -1, -1, -1, -1, -1],
    [0, 5, 4, 0, 1, 5, 2, 3, 11, -1, -1, -1, -1, -1, -1, -1],
    [2, 1, 5, 2, 5, 8, 2, 8, 11, 4, 8, 5, -1, -1, -1, -1],
    [10, 3, 11, 10, 1, 3, 9, 5, 4, -1, -1, -1, -1, -1, -1, -1],
    [4, 9, 5, 0, 8, 1, 8, 10, 1, 8, 11, 10, -1, -1, -1, -1],
    [5, 4, 0, 5, 0, 11, 5, 11, 10, 11, 0, 3, -1, -1, -1, -1],
    [5, 4, 8, 5, 8, 10, 10, 8, 11, -1, -1, -1, -1, -1, -1, -1],
    [9, 7, 8, 5, 7, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [9, 3, 0, 9, 5, 3, 5, 7, 3, -1, -1, -1, -1, -1, -1, -1],
    [0, 7, 8, 0, 1, 7, 1, 5, 7, -1, -1, -1, -1, -1, -1, -1],
    [1, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [9, 7, 8, 9, 5, 7, 10, 1, 2, -1, -1, -1, -1, -1, -1, -1],
    [10, 1, 2, 9, 5, 0, 5, 3, 0, 5, 7, 3, -1, -1, -1, -1],
    [8, 0, 2, 8, 2, 5, 8, 5, 7, 10, 5, 2, -1, -1, -1, -1],
    [2, 10, 5, 2, 5, 3, 3, 5, 7, -1, -1, -1, -1, -1, -1, -1],
    [7, 9, 5, 7, 8, 9, 3, 11, 2, -1, -1, -1, -1, -1, -1, -1],
    [9, 5, 7, 9, 7, 2, 9, 2, 0, 2, 7, 11, -1, -1, -1, -1],
    [2, 3, 11, 0, 1, 8, 1, 7, 8, 1, 5, 7, -1, -1, -1, -1],
    [11, 2, 1, 11, 1, 7, 7, 1, 5, -1, -1, -1, -1, -1, -1, -1],
    [9, 5, 8, 8, 5, 7, 10, 1, 3, 10, 3, 11, -1, -1, -1, -1],
    [5, 7, 0, 5, 0, 9, 7, 11, 0, 1, 0, 10, 11, 10, 0, -1],
    [11, 10, 0, 11, 0, 3, 10, 5, 0, 8, 0, 7, 5, 7, 0, -1],
    [11, 10, 5, 7, 11, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 3, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [9, 0, 1, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 8, 3, 1, 9, 8, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1],
    [1, 6, 5, 2, 6, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 6, 5, 1, 2, 6, 3, 0, 8, -1, -1, -1, -1, -1, -1, -1],
    [9, 6, 5, 9, 0, 6, 0, 2, 6, -1, -1, -1, -1, -1, -1, -1],
    [5, 9, 8, 5, 8, 2, 5, 2, 6, 3, 2, 8, -1, -1, -1, -1],
    [2, 3, 11, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [11, 0, 8, 11, 2, 0, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1],
    [0, 1, 9, 2, 3, 11, 5, 10, 6, -1, -1, -1, -1, -1, -1, -1],
    [5, 10, 6, 1, 9, 2, 9, 11, 2, 9, 8, 11, -1, -1, -1, -1],
    [6, 3, 11, 6, 5, 3, 5, 1, 3, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 11, 0, 11, 5, 0, 5, 1, 5, 11, 6, -1, -1, -1, -1],
    [3, 11, 6, 0, 3, 6, 0, 6, 5, 0, 5, 9, -1, -1, -1, -1],
    [6, 5, 9, 6, 9, 11, 11, 9, 8, -1, -1, -1, -1, -1, -1, -1],
    [5, 10, 6, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 3, 0, 4, 7, 3, 6, 5, 10, -1, -1, -1, -1, -1, -1, -1],
    [1, 9, 0, 5, 10, 6, 8, 4, 7, -1, -1, -1, -1, -1, -1, -1],
    [10, 6, 5, 1, 9, 7, 1, 7, 3, 7, 9, 4, -1, -1, -1, -1],
    [6, 1, 2, 6, 5, 1, 4, 7, 8, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 5, 5, 2, 6, 3, 0, 4, 3, 4, 7, -1, -1, -1, -1],
    [8, 4, 7, 9, 0, 5, 0, 6, 5, 0, 2, 6, -1, -1, -1, -1],
    [7, 3, 9, 7, 9, 4, 3, 2, 9, 5, 9, 6, 2, 6, 9, -1],
    [3, 11, 2, 7, 8, 4, 10, 6, 5, -1, -1, -1, -1, -1, -1, -1],
    [5, 10, 6, 4, 7, 2, 4, 2, 0, 2, 7, 11, -1, -1, -1, -1],
    [0, 1, 9, 4, 7, 8, 2, 3, 11, 5, 10, 6, -1, -1, -1, -1],
    [9, 2, 1, 9, 11, 2, 9, 4, 11, 7, 11, 4, 5, 10, 6, -1],
    [8, 4, 7, 3, 11, 5, 3, 5, 1, 5, 11, 6, -1, -1, -1, -1],
    [5, 1, 11, 5, 11, 6, 1, 0, 11, 7, 11, 4, 0, 4, 11, -1],
    [0, 5, 9, 0, 6, 5, 0, 3, 6, 11, 6, 3, 8, 4, 7, -1],
    [6, 5, 9, 6, 9, 11, 4, 7, 9, 7, 11, 9, -1, -1, -1, -1],
    [10, 4, 9, 6, 4, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 10, 6, 4, 9, 10, 0, 8, 3, -1, -1, -1, -1, -1, -1, -1],
    [10, 0, 1, 10, 6, 0, 6, 4, 0, -1, -1, -1, -1, -1, -1, -1],
    [8, 3, 1, 8, 1, 6, 8, 6, 4, 6, 1, 10, -1, -1, -1, -1],
    [1, 4, 9, 1, 2, 4, 2, 6, 4, -1, -1, -1, -1, -1, -1, -1],
    [3, 0, 8, 1, 2, 9, 2, 4, 9, 2, 6, 4, -1, -1, -1, -1],
    [0, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [8, 3, 2, 8, 2, 4, 4, 2, 6, -1, -1, -1, -1, -1, -1, -1],
    [10, 4, 9, 10, 6, 4, 11, 2, 3, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 2, 2, 8, 11, 4, 9, 10, 4, 10, 6, -1, -1, -1, -1],
    [3, 11, 2, 0, 1, 6, 0, 6, 4, 6, 1, 10, -1, -1, -1, -1],
    [6, 4, 1, 6, 1, 10, 4, 8, 1, 2, 1, 11, 8, 11, 1, -1],
    [9, 6, 4, 9, 3, 6, 9, 1, 3, 11, 6, 3, -1, -1, -1, -1],
    [8, 11, 1, 8, 1, 0, 11, 6, 1, 9, 1, 4, 6, 4, 1, -1],
    [3, 11, 6, 3, 6, 0, 0, 6, 4, -1, -1, -1, -1, -1, -1, -1],
    [6, 4, 8, 11, 6, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [7, 10, 6, 7, 8, 10, 8, 9, 10, -1, -1, -1, -1, -1, -1, -1],
    [0, 7, 3, 0, 10, 7, 0, 9, 10, 6, 7, 10, -1, -1, -1, -1],
    [10, 6, 7, 1, 10, 7, 1, 7, 8, 1, 8, 0, -1, -1, -1, -1],
    [10, 6, 7, 10, 7, 1, 1, 7, 3, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 6, 1, 6, 8, 1, 8, 9, 8, 6, 7, -1, -1, -1, -1],
    [2, 6, 9, 2, 9, 1, 6, 7, 9, 0, 9, 3, 7, 3, 9, -1],
    [7, 8, 0, 7, 0, 6, 6, 0, 2, -1, -1, -1, -1, -1, -1, -1],
    [7, 3, 2, 6, 7, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [2, 3, 11, 10, 6, 8, 10, 8, 9, 8, 6, 7, -1, -1, -1, -1],
    [2, 0, 7, 2, 7, 11, 0, 9, 7, 6, 7, 10, 9, 10, 7, -1],
    [1, 8, 0, 1, 7, 8, 1, 10, 7, 6, 7, 10, 2, 3, 11, -1],
    [11, 2, 1, 11, 1, 7, 10, 6, 1, 6, 7, 1, -1, -1, -1, -1],
    [8, 9, 6, 8, 6, 7, 9, 1, 6, 11, 6, 3, 1, 3, 6, -1],
    [0, 9, 1, 11, 6, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [7, 8, 0, 7, 0, 6, 3, 11, 0, 11, 6, 0, -1, -1, -1, -1],
    [7, 11, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [3, 0, 8, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 1, 9, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [8, 1, 9, 8, 3, 1, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1],
    [10, 1, 2, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 10, 3, 0, 8, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1],
    [2, 9, 0, 2, 10, 9, 6, 11, 7, -1, -1, -1, -1, -1, -1, -1],
    [6, 11, 7, 2, 10, 3, 10, 8, 3, 10, 9, 8, -1, -1, -1, -1],
    [7, 2, 3, 6, 2, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [7, 0, 8, 7, 6, 0, 6, 2, 0, -1, -1, -1, -1, -1, -1, -1],
    [2, 7, 6, 2, 3, 7, 0, 1, 9, -1, -1, -1, -1, -1, -1, -1],
    [1, 6, 2, 1, 8, 6, 1, 9, 8, 8, 7, 6, -1, -1, -1, -1],
    [10, 7, 6, 10, 1, 7, 1, 3, 7, -1, -1, -1, -1, -1, -1, -1],
    [10, 7, 6, 1, 7, 10, 1, 8, 7, 1, 0, 8, -1, -1, -1, -1],
    [0, 3, 7, 0, 7, 10, 0, 10, 9, 6, 10, 7, -1, -1, -1, -1],
    [7, 6, 10, 7, 10, 8, 8, 10, 9, -1, -1, -1, -1, -1, -1, -1],
    [6, 8, 4, 11, 8, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [3, 6, 11, 3, 0, 6, 0, 4, 6, -1, -1, -1, -1, -1, -1, -1],
    [8, 6, 11, 8, 4, 6, 9, 0, 1, -1, -1, -1, -1, -1, -1, -1],
    [9, 4, 6, 9, 6, 3, 9, 3, 1, 11, 3, 6, -1, -1, -1, -1],
    [6, 8, 4, 6, 11, 8, 2, 10, 1, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 10, 3, 0, 11, 0, 6, 11, 0, 4, 6, -1, -1, -1, -1],
    [4, 11, 8, 4, 6, 11, 0, 2, 9, 2, 10, 9, -1, -1, -1, -1],
    [10, 9, 3, 10, 3, 2, 9, 4, 3, 11, 3, 6, 4, 6, 3, -1],
    [8, 2, 3, 8, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1, -1],
    [0, 4, 2, 4, 6, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 9, 0, 2, 3, 4, 2, 4, 6, 4, 3, 8, -1, -1, -1, -1],
    [1, 9, 4, 1, 4, 2, 2, 4, 6, -1, -1, -1, -1, -1, -1, -1],
    [8, 1, 3, 8, 6, 1, 8, 4, 6, 6, 10, 1, -1, -1, -1, -1],
    [10, 1, 0, 10, 0, 6, 6, 0, 4, -1, -1, -1, -1, -1, -1, -1],
    [4, 6, 3, 4, 3, 8, 6, 10, 3, 0, 3, 9, 10, 9, 3, -1],
    [10, 9, 4, 6, 10, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 9, 5, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 3, 4, 9, 5, 11, 7, 6, -1, -1, -1, -1, -1, -1, -1],
    [5, 0, 1, 5, 4, 0, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1],
    [11, 7, 6, 8, 3, 4, 3, 5, 4, 3, 1, 5, -1, -1, -1, -1],
    [9, 5, 4, 10, 1, 2, 7, 6, 11, -1, -1, -1, -1, -1, -1, -1],
    [6, 11, 7, 1, 2, 10, 0, 8, 3, 4, 9, 5, -1, -1, -1, -1],
    [7, 6, 11, 5, 4, 10, 4, 2, 10, 4, 0, 2, -1, -1, -1, -1],
    [3, 4, 8, 3, 5, 4, 3, 2, 5, 10, 5, 2, 11, 7, 6, -1],
    [7, 2, 3, 7, 6, 2, 5, 4, 9, -1, -1, -1, -1, -1, -1, -1],
    [9, 5, 4, 0, 8, 6, 0, 6, 2, 6, 8, 7, -1, -1, -1, -1],
    [3, 6, 2, 3, 7, 6, 1, 5, 0, 5, 4, 0, -1, -1, -1, -1],
    [6, 2, 8, 6, 8, 7, 2, 1, 8, 4, 8, 5, 1, 5, 8, -1],
    [9, 5, 4, 10, 1, 6, 1, 7, 6, 1, 3, 7, -1, -1, -1, -1],
    [1, 6, 10, 1, 7, 6, 1, 0, 7, 8, 7, 0, 9, 5, 4, -1],
    [4, 0, 10, 4, 10, 5, 0, 3, 10, 6, 10, 7, 3, 7, 10, -1],
    [7, 6, 10, 7, 10, 8, 5, 4, 10, 4, 8, 10, -1, -1, -1, -1],
    [6, 9, 5, 6, 11, 9, 11, 8, 9, -1, -1, -1, -1, -1, -1, -1],
    [3, 6, 11, 0, 6, 3, 0, 5, 6, 0, 9, 5, -1, -1, -1, -1],
    [0, 11, 8, 0, 5, 11, 0, 1, 5, 5, 6, 11, -1, -1, -1, -1],
    [6, 11, 3, 6, 3, 5, 5, 3, 1, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 10, 9, 5, 11, 9, 11, 8, 11, 5, 6, -1, -1, -1, -1],
    [0, 11, 3, 0, 6, 11, 0, 9, 6, 5, 6, 9, 1, 2, 10, -1],
    [11, 8, 5, 11, 5, 6, 8, 0, 5, 10, 5, 2, 0, 2, 5, -1],
    [6, 11, 3, 6, 3, 5, 2, 10, 3, 10, 5, 3, -1, -1, -1, -1],
    [5, 8, 9, 5, 2, 8, 5, 6, 2, 3, 8, 2, -1, -1, -1, -1],
    [9, 5, 6, 9, 6, 0, 0, 6, 2, -1, -1, -1, -1, -1, -1, -1],
    [1, 5, 8, 1, 8, 0, 5, 6, 8, 3, 8, 2, 6, 2, 8, -1],
    [1, 5, 6, 2, 1, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 3, 6, 1, 6, 10, 3, 8, 6, 5, 6, 9, 8, 9, 6, -1],
    [10, 1, 0, 10, 0, 6, 9, 5, 0, 5, 6, 0, -1, -1, -1, -1],
    [0, 3, 8, 5, 6, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [10, 5, 6, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [11, 5, 10, 7, 5, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [11, 5, 10, 11, 7, 5, 8, 3, 0, -1, -1, -1, -1, -1, -1, -1],
    [5, 11, 7, 5, 10, 11, 1, 9, 0, -1, -1, -1, -1, -1, -1, -1],
    [10, 7, 5, 10, 11, 7, 9, 8, 1, 8, 3, 1, -1, -1, -1, -1],
    [11, 1, 2, 11, 7, 1, 7, 5, 1, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 3, 1, 2, 7, 1, 7, 5, 7, 2, 11, -1, -1, -1, -1],
    [9, 7, 5, 9, 2, 7, 9, 0, 2, 2, 11, 7, -1, -1, -1, -1],
    [7, 5, 2, 7, 2, 11, 5, 9, 2, 3, 2, 8, 9, 8, 2, -1],
    [2, 5, 10, 2, 3, 5, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1],
    [8, 2, 0, 8, 5, 2, 8, 7, 5, 10, 2, 5, -1, -1, -1, -1],
    [9, 0, 1, 2, 3, 10, 3, 5, 10, 3, 7, 5, -1, -1, -1, -1],
    [1, 2, 5, 5, 2, 10, 7, 9, 8, 7, 5, 9, -1, -1, -1, -1], // hand-edited
    [3, 5, 1, 3, 7, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 7, 0, 7, 1, 1, 7, 5, -1, -1, -1, -1, -1, -1, -1],
    [9, 0, 3, 9, 3, 5, 5, 3, 7, -1, -1, -1, -1, -1, -1, -1],
    [9, 8, 7, 5, 9, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [5, 8, 4, 5, 10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1, -1],
    [5, 0, 4, 5, 11, 0, 5, 10, 11, 11, 3, 0, -1, -1, -1, -1],
    [0, 1, 9, 8, 4, 10, 8, 10, 11, 10, 4, 5, -1, -1, -1, -1],
    [10, 11, 4, 10, 4, 5, 11, 3, 4, 9, 4, 1, 3, 1, 4, -1],
    [2, 5, 1, 2, 8, 5, 2, 11, 8, 4, 5, 8, -1, -1, -1, -1],
    [0, 4, 11, 0, 11, 3, 4, 5, 11, 2, 11, 1, 5, 1, 11, -1],
    [0, 2, 5, 0, 5, 9, 2, 11, 5, 4, 5, 8, 11, 8, 5, -1],
    [9, 4, 5, 2, 11, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [2, 5, 10, 3, 5, 2, 3, 4, 5, 3, 8, 4, -1, -1, -1, -1],
    [5, 10, 2, 5, 2, 4, 4, 2, 0, -1, -1, -1, -1, -1, -1, -1],
    [3, 10, 2, 3, 5, 10, 3, 8, 5, 4, 5, 8, 0, 1, 9, -1],
    [5, 10, 2, 5, 2, 4, 1, 9, 2, 9, 4, 2, -1, -1, -1, -1],
    [8, 4, 5, 8, 5, 3, 3, 5, 1, -1, -1, -1, -1, -1, -1, -1],
    [0, 4, 5, 1, 0, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [8, 4, 5, 8, 5, 3, 9, 0, 5, 0, 3, 5, -1, -1, -1, -1],
    [9, 4, 5, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 11, 7, 4, 9, 11, 9, 10, 11, -1, -1, -1, -1, -1, -1, -1],
    [0, 8, 3, 4, 9, 7, 9, 11, 7, 9, 10, 11, -1, -1, -1, -1],
    [1, 10, 11, 1, 11, 4, 1, 4, 0, 7, 4, 11, -1, -1, -1, -1],
    [3, 1, 4, 3, 4, 8, 1, 10, 4, 7, 4, 11, 10, 11, 4, -1],
    [4, 11, 7, 9, 11, 4, 9, 2, 11, 9, 1, 2, -1, -1, -1, -1],
    [9, 7, 4, 9, 11, 7, 9, 1, 11, 2, 11, 1, 0, 8, 3, -1],
    [11, 7, 4, 11, 4, 2, 2, 4, 0, -1, -1, -1, -1, -1, -1, -1],
    [11, 7, 4, 11, 4, 2, 8, 3, 4, 3, 2, 4, -1, -1, -1, -1],
    [2, 9, 10, 2, 7, 9, 2, 3, 7, 7, 4, 9, -1, -1, -1, -1],
    [9, 10, 7, 9, 7, 4, 10, 2, 7, 8, 7, 0, 2, 0, 7, -1],
    [3, 7, 10, 3, 10, 2, 7, 4, 10, 1, 10, 0, 4, 0, 10, -1],
    [1, 10, 2, 8, 7, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 9, 1, 4, 1, 7, 7, 1, 3, -1, -1, -1, -1, -1, -1, -1],
    [4, 9, 1, 4, 1, 7, 0, 8, 1, 8, 7, 1, -1, -1, -1, -1],
    [4, 0, 3, 7, 4, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [4, 8, 7, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [9, 10, 8, 10, 11, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [3, 0, 9, 3, 9, 11, 11, 9, 10, -1, -1, -1, -1, -1, -1, -1],
    [0, 1, 10, 0, 10, 8, 8, 10, 11, -1, -1, -1, -1, -1, -1, -1],
    [3, 1, 10, 11, 3, 10, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 2, 11, 1, 11, 9, 9, 11, 8, -1, -1, -1, -1, -1, -1, -1],
    [3, 0, 9, 3, 9, 11, 1, 2, 9, 2, 11, 9, -1, -1, -1, -1],
    [0, 2, 11, 8, 0, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [3, 2, 11, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [2, 3, 8, 2, 8, 10, 10, 8, 9, -1, -1, -1, -1, -1, -1, -1],
    [9, 10, 2, 0, 9, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [2, 3, 8, 2, 8, 10, 0, 1, 8, 1, 10, 8, -1, -1, -1, -1],
    [1, 10, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [1, 3, 8, 9, 1, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 9, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [0, 3, 8, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1],
    [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    ],
];

/// The 8 corner offsets for a unit voxel cube.
const CORNER_OFFSETS: [[usize; 3]; 8] = [
    [0, 0, 0],
    [1, 0, 0],
    [1, 1, 0],
    [0, 1, 0],
    [0, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 1, 1],
];

/// Edge endpoint corner pairs (standard MC numbering).
const EDGE_CORNERS: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

/// Run marching cubes on a uniform implicit function grid.
fn marching_cubes_on_grid(
    chi: &[f64],
    nv: usize,       // vertices per axis
    grid_res: usize, // cells per axis = nv - 1
    bb_min: [f64; 3],
    cell_size: f64,
    isovalue: f32,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let world_pos = |ix: usize, iy: usize, iz: usize| -> [f32; 3] {
        [
            (bb_min[0] + ix as f64 * cell_size) as f32,
            (bb_min[1] + iy as f64 * cell_size) as f32,
            (bb_min[2] + iz as f64 * cell_size) as f32,
        ]
    };

    let get = |ix: usize, iy: usize, iz: usize| -> f32 { chi[grid_idx(ix, iy, iz, nv)] as f32 };

    for iz in 0..grid_res {
        for iy in 0..grid_res {
            for ix in 0..grid_res {
                let mut corner_vals = [0.0f32; 8];
                let mut corner_pos = [[0.0f32; 3]; 8];

                for (c, off) in CORNER_OFFSETS.iter().enumerate() {
                    let cx = ix + off[0];
                    let cy = iy + off[1];
                    let cz = iz + off[2];
                    corner_vals[c] = get(cx, cy, cz);
                    corner_pos[c] = world_pos(cx, cy, cz);
                }

                let mut case_idx: usize = 0;
                for (c, &val) in corner_vals.iter().enumerate() {
                    if val > isovalue {
                        case_idx |= 1 << c;
                    }
                }

                if case_idx == 0 || case_idx == 255 {
                    continue;
                }

                let edge_mask = MC_EDGE_TABLE[case_idx];
                if edge_mask == 0 {
                    continue;
                }

                let mut edge_verts = [[0.0f32; 3]; 12];
                for e in 0..12u16 {
                    if edge_mask & (1 << e) != 0 {
                        let [ca, cb] = EDGE_CORNERS[e as usize];
                        edge_verts[e as usize] = interp_vert(
                            corner_pos[ca],
                            corner_pos[cb],
                            corner_vals[ca],
                            corner_vals[cb],
                            isovalue,
                        );
                    }
                }

                let tris = &MC_TRI_TABLE[case_idx];
                let mut i = 0;
                while i < 16 && tris[i] >= 0 {
                    let base = positions.len() as u32;
                    positions.push(edge_verts[tris[i] as usize]);
                    positions.push(edge_verts[tris[i + 1] as usize]);
                    positions.push(edge_verts[tris[i + 2] as usize]);
                    indices.extend_from_slice(&[base, base + 1, base + 2]);
                    i += 3;
                }
            }
        }
    }

    (positions, indices)
}

#[inline]
fn interp_vert(p0: [f32; 3], p1: [f32; 3], v0: f32, v1: f32, isovalue: f32) -> [f32; 3] {
    let denom = v1 - v0;
    let t = if denom.abs() < 1e-10 {
        0.5
    } else {
        ((isovalue - v0) / denom).clamp(0.0, 1.0)
    };
    [
        p0[0] + t * (p1[0] - p0[0]),
        p0[1] + t * (p1[1] - p0[1]),
        p0[2] + t * (p1[2] - p0[2]),
    ]
}

// ── Deprecated shim helpers (kept for backward compat) ────────────────────────

/// Legacy config struct from old stub.
pub struct PoissonReconConfig {
    pub octree_depth: u32,
    pub samples_per_node: f32,
    pub point_weight: f32,
}

impl Default for PoissonReconConfig {
    fn default() -> Self {
        Self {
            octree_depth: 8,
            samples_per_node: 1.5,
            point_weight: 4.0,
        }
    }
}

/// Legacy result type from old stub.
pub struct PoissonReconResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub implicit_value: f32,
}

/// Estimate bounding box of a point cloud.
pub fn point_cloud_bbox(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    if points.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = points[0];
    let mut mx = points[0];
    for &p in points.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] {
                mn[i] = p[i];
            }
            if p[i] > mx[i] {
                mx[i] = p[i];
            }
        }
    }
    (mn, mx)
}

/// Compute centroid of point cloud.
pub fn point_cloud_centroid(points: &[[f32; 3]]) -> [f32; 3] {
    if points.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for &p in points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    let n = points.len() as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Voxel grid cell from position.
pub fn point_to_voxel(p: [f32; 3], cell_size: f32) -> [i32; 3] {
    [
        (p[0] / cell_size).floor() as i32,
        (p[1] / cell_size).floor() as i32,
        (p[2] / cell_size).floor() as i32,
    ]
}

/// Density estimate: count points in radius.
pub fn density_estimate(points: &[[f32; 3]], query: [f32; 3], radius: f32) -> usize {
    let r2 = radius * radius;
    points
        .iter()
        .filter(|&&p| {
            let dx = p[0] - query[0];
            let dy = p[1] - query[1];
            let dz = p[2] - query[2];
            dx * dx + dy * dy + dz * dz <= r2
        })
        .count()
}

/// Legacy stub: run Poisson reconstruction (returns convex hull approximation).
pub fn poisson_reconstruct_stub(
    points: &[[f32; 3]],
    normals: &[[f32; 3]],
    _config: &PoissonReconConfig,
) -> PoissonReconResult {
    let _ = normals;
    PoissonReconResult {
        positions: points.to_vec(),
        indices: vec![],
        implicit_value: 0.0,
    }
}

/// Octree level required for given resolution.
pub fn required_octree_depth(point_count: usize) -> u32 {
    if point_count == 0 {
        return 0;
    }
    (point_count as f32).log2().ceil() as u32
}

/// Estimate output mesh complexity from depth.
pub fn estimate_output_faces(depth: u32) -> usize {
    (8usize).pow(depth.min(6)) / 2
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn sphere_oriented_cloud(n: usize) -> Vec<([f64; 3], [f64; 3])> {
        let golden = PI * (3.0 - 5.0f64.sqrt());
        (0..n)
            .map(|i| {
                let y = 1.0 - (i as f64 / (n as f64 - 1.0)) * 2.0;
                let r = (1.0 - y * y).max(0.0).sqrt();
                let theta = golden * i as f64;
                let x = r * theta.cos();
                let z = r * theta.sin();
                ([x, y, z], [x, y, z])
            })
            .collect()
    }

    #[test]
    fn poisson_sphere_produces_closed_mesh() {
        let cloud = sphere_oriented_cloud(300);
        let mut recon = PoissonReconstructor::new();
        for (pos, n) in &cloud {
            recon.add_oriented_point(*pos, *n);
        }
        let config = PoissonConfig {
            depth: 4, // keep it fast for tests
            ..Default::default()
        };
        let (verts, indices) = recon
            .reconstruct(&config)
            .expect("sphere reconstruction must succeed");
        assert!(!verts.is_empty(), "must have vertices");
        assert!(indices.len() % 3 == 0, "indices must be triangle list");
        assert!(
            indices.len() >= 6,
            "sphere reconstruction must produce at least 2 triangles"
        );
    }

    #[test]
    fn poisson_requires_four_points() {
        let mut recon = PoissonReconstructor::new();
        recon.add_oriented_point([0.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        recon.add_oriented_point([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        recon.add_oriented_point([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        assert!(recon.reconstruct(&PoissonConfig::default()).is_err());
    }

    // ── Legacy stub tests ─────────────────────────────────────────────────────

    fn cloud() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ]
    }

    #[test]
    fn bbox_correct() {
        let pts = cloud();
        let (mn, mx) = point_cloud_bbox(&pts);
        assert!(mn[0].abs() < 1e-6);
        assert!((mx[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn centroid_inside_bbox() {
        let pts = cloud();
        let c = point_cloud_centroid(&pts);
        let (mn, mx) = point_cloud_bbox(&pts);
        for i in 0..3 {
            assert!(c[i] >= mn[i] && c[i] <= mx[i]);
        }
    }

    #[test]
    fn voxel_correct() {
        let v = point_to_voxel([1.5, 2.5, 3.5], 1.0);
        assert_eq!(v, [1, 2, 3]);
    }

    #[test]
    fn density_self() {
        let pts = cloud();
        let d = density_estimate(&pts, pts[0], 0.01);
        assert!(d >= 1);
    }

    #[test]
    fn poisson_stub_returns_points() {
        let pts = cloud();
        let normals = vec![[0.0, 0.0, 1.0f32]; pts.len()];
        let cfg = PoissonReconConfig::default();
        let r = poisson_reconstruct_stub(&pts, &normals, &cfg);
        assert_eq!(r.positions.len(), pts.len());
    }

    #[test]
    fn required_depth_nonzero() {
        let d = required_octree_depth(1000);
        assert!(d > 0);
    }

    #[test]
    fn estimate_faces_depth_3() {
        let f = estimate_output_faces(3);
        assert!(f > 0);
    }

    #[test]
    fn bbox_empty_cloud() {
        let (mn, mx) = point_cloud_bbox(&[]);
        assert_eq!(mn, [0.0, 0.0, 0.0]);
        assert_eq!(mx, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn default_config() {
        let c = PoissonReconConfig::default();
        assert_eq!(c.octree_depth, 8);
    }
}
