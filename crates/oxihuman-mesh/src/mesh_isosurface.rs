// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Isosurface extraction from a scalar grid (simplified marching cubes variant).

/// A scalar grid for isosurface extraction.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScalarGrid {
    pub data: Vec<f32>,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub cell_size: f32,
    pub origin: [f32; 3],
}

impl ScalarGrid {
    /// Create a new scalar grid filled with zeros.
    #[allow(dead_code)]
    pub fn new(nx: usize, ny: usize, nz: usize, cell_size: f32, origin: [f32; 3]) -> Self {
        ScalarGrid {
            data: vec![0.0; nx * ny * nz],
            nx,
            ny,
            nz,
            cell_size,
            origin,
        }
    }

    /// Sample value at grid indices.
    #[allow(dead_code)]
    pub fn get(&self, ix: usize, iy: usize, iz: usize) -> f32 {
        if ix >= self.nx || iy >= self.ny || iz >= self.nz {
            return 0.0;
        }
        self.data[iz * self.ny * self.nx + iy * self.nx + ix]
    }

    /// Set value at grid indices.
    #[allow(dead_code)]
    pub fn set(&mut self, ix: usize, iy: usize, iz: usize, val: f32) {
        if ix < self.nx && iy < self.ny && iz < self.nz {
            self.data[iz * self.ny * self.nx + iy * self.nx + ix] = val;
        }
    }

    /// World position of grid node.
    #[allow(dead_code)]
    pub fn world_pos(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        [
            self.origin[0] + ix as f32 * self.cell_size,
            self.origin[1] + iy as f32 * self.cell_size,
            self.origin[2] + iz as f32 * self.cell_size,
        ]
    }
}

/// Fill the grid with a sphere SDF: positive inside, negative outside.
#[allow(dead_code)]
pub fn fill_sphere_sdf(grid: &mut ScalarGrid, center: [f32; 3], radius: f32) {
    for iz in 0..grid.nz {
        for iy in 0..grid.ny {
            for ix in 0..grid.nx {
                let p = grid.world_pos(ix, iy, iz);
                let dx = p[0] - center[0];
                let dy = p[1] - center[1];
                let dz = p[2] - center[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                grid.set(ix, iy, iz, radius - dist);
            }
        }
    }
}

/// An extracted isosurface mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct IsosurfaceMesh {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Interpolate position on an edge between two grid samples.
fn interp_edge(p0: [f32; 3], v0: f32, p1: [f32; 3], v1: f32, iso: f32) -> [f32; 3] {
    let dv = v1 - v0;
    let t = if dv.abs() < 1e-8 {
        0.5
    } else {
        (iso - v0) / dv
    };
    [
        p0[0] + t * (p1[0] - p0[0]),
        p0[1] + t * (p1[1] - p0[1]),
        p0[2] + t * (p1[2] - p0[2]),
    ]
}

/// Extract an isosurface using a simplified marching-edges approach.
/// For each voxel edge that crosses the isovalue, emit a quad (two triangles).
/// This is a simplified stub, not full marching cubes.
#[allow(dead_code)]
pub fn extract_isosurface(grid: &ScalarGrid, isovalue: f32) -> IsosurfaceMesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // For each cell, check the 12 edges and emit a face for each crossing.
    let edges_axis: &[(usize, usize, usize, usize, usize, usize)] =
        &[(0, 0, 0, 1, 0, 0), (0, 0, 0, 0, 1, 0), (0, 0, 0, 0, 0, 1)];

    for iz in 0..(grid.nz.saturating_sub(1)) {
        for iy in 0..(grid.ny.saturating_sub(1)) {
            for ix in 0..(grid.nx.saturating_sub(1)) {
                for &(ox0, oy0, oz0, ox1, oy1, oz1) in edges_axis {
                    let ax = ix + ox0;
                    let ay = iy + oy0;
                    let az = iz + oz0;
                    let bx = ix + ox1;
                    let by = iy + oy1;
                    let bz = iz + oz1;
                    let va = grid.get(ax, ay, az);
                    let vb = grid.get(bx, by, bz);
                    if (va - isovalue) * (vb - isovalue) < 0.0 {
                        let pa = grid.world_pos(ax, ay, az);
                        let pb = grid.world_pos(bx, by, bz);
                        let mid = interp_edge(pa, va, pb, vb, isovalue);
                        // Emit a tiny degenerate marker quad (stub)
                        let base = positions.len() as u32;
                        let epsilon = grid.cell_size * 0.1;
                        positions.push([mid[0], mid[1], mid[2]]);
                        positions.push([mid[0] + epsilon, mid[1], mid[2]]);
                        positions.push([mid[0], mid[1] + epsilon, mid[2]]);
                        positions.push([mid[0] + epsilon, mid[1] + epsilon, mid[2]]);
                        indices.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base + 1,
                            base + 3,
                            base + 2,
                        ]);
                    }
                }
            }
        }
    }

    IsosurfaceMesh { positions, indices }
}

/// Vertex count.
#[allow(dead_code)]
pub fn isosurface_vertex_count(m: &IsosurfaceMesh) -> usize {
    m.positions.len()
}

/// Triangle count.
#[allow(dead_code)]
pub fn isosurface_triangle_count(m: &IsosurfaceMesh) -> usize {
    m.indices.len() / 3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sphere_grid() -> ScalarGrid {
        let mut g = ScalarGrid::new(10, 10, 10, 0.25, [0.0, 0.0, 0.0]);
        fill_sphere_sdf(&mut g, [1.25, 1.25, 1.25], 0.75);
        g
    }

    #[test]
    fn grid_size_correct() {
        let g = ScalarGrid::new(4, 5, 6, 1.0, [0.0, 0.0, 0.0]);
        assert_eq!(g.data.len(), 120);
    }

    #[test]
    fn grid_get_set() {
        let mut g = ScalarGrid::new(4, 4, 4, 1.0, [0.0, 0.0, 0.0]);
        g.set(1, 2, 3, 42.0);
        assert!((g.get(1, 2, 3) - 42.0).abs() < 1e-5);
    }

    #[test]
    fn world_pos_origin() {
        let g = ScalarGrid::new(4, 4, 4, 0.5, [1.0, 2.0, 3.0]);
        let p = g.world_pos(0, 0, 0);
        assert!(
            (p[0] - 1.0).abs() < 1e-5 && (p[1] - 2.0).abs() < 1e-5 && (p[2] - 3.0).abs() < 1e-5
        );
    }

    #[test]
    fn sphere_sdf_positive_inside() {
        let g = sphere_grid();
        let v = g.get(5, 5, 5);
        assert!(v > 0.0);
    }

    #[test]
    fn sphere_sdf_negative_outside() {
        let g = sphere_grid();
        let v = g.get(0, 0, 0);
        assert!(v < 0.0);
    }

    #[test]
    fn isosurface_has_triangles() {
        let g = sphere_grid();
        let m = extract_isosurface(&g, 0.0);
        assert!(!m.indices.is_empty());
    }

    #[test]
    fn indices_multiple_of_three() {
        let g = sphere_grid();
        let m = extract_isosurface(&g, 0.0);
        assert_eq!(m.indices.len() % 3, 0);
    }

    #[test]
    fn vertex_count_helper() {
        let g = sphere_grid();
        let m = extract_isosurface(&g, 0.0);
        assert_eq!(isosurface_vertex_count(&m), m.positions.len());
    }

    #[test]
    fn triangle_count_helper() {
        let g = sphere_grid();
        let m = extract_isosurface(&g, 0.0);
        assert_eq!(isosurface_triangle_count(&m), m.indices.len() / 3);
    }

    #[test]
    fn out_of_bounds_get_returns_zero() {
        let g = ScalarGrid::new(4, 4, 4, 1.0, [0.0, 0.0, 0.0]);
        assert!((g.get(99, 0, 0)).abs() < 1e-6);
    }
}
