// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fluid surface mesh (marching-cubes wrapper stub).

/// Parameters for fluid surface extraction.
#[derive(Debug, Clone)]
pub struct FluidSurfaceParams {
    /// Iso-level threshold.
    pub iso_level: f32,
    /// Grid cell size.
    pub cell_size: f32,
    /// Smoothing iterations after extraction.
    pub smooth_iters: u32,
}

impl Default for FluidSurfaceParams {
    fn default() -> Self {
        Self {
            iso_level: 0.5,
            cell_size: 0.05,
            smooth_iters: 2,
        }
    }
}

/// Result of fluid surface extraction.
#[derive(Debug, Clone)]
pub struct FluidSurface {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
}

impl FluidSurface {
    /// Number of triangles in the extracted surface.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }
}

/// Extract a fluid surface from a scalar density field using marching cubes,
/// followed by Laplacian smoothing and per-face normal accumulation.
pub fn extract_fluid_surface(
    density: &[f32],
    dims: [usize; 3],
    params: &FluidSurfaceParams,
) -> FluidSurface {
    let expected = dims[0] * dims[1] * dims[2];
    if density.is_empty() || density.len() != expected {
        return FluidSurface {
            positions: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
        };
    }

    use crate::marching_cubes::{marching_cubes, ScalarField};

    let origin = [0.0f32; 3];
    let spacing = [params.cell_size; 3];
    let mut field = ScalarField::new(dims, origin, spacing);
    // Copy density into field; ScalarField uses row-major [x + nx*(y + ny*z)] layout.
    field.data.copy_from_slice(density);

    let mesh = marching_cubes(&field, params.iso_level);

    if mesh.positions.is_empty() {
        return FluidSurface {
            positions: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
        };
    }

    let mut positions = mesh.positions;
    let indices = mesh.indices;

    for _ in 0..params.smooth_iters {
        positions = laplacian_smooth(&positions, &indices);
    }

    let normals = compute_vertex_normals(&positions, &indices);

    FluidSurface {
        positions,
        indices,
        normals,
    }
}

/// Build a vertex-to-neighbours map from the triangle index list and average
/// each vertex towards its connected neighbours (one Laplacian pass).
fn laplacian_smooth(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    if n == 0 || indices.is_empty() {
        return positions.to_vec();
    }

    // Accumulate neighbour sums and counts.
    let mut sum = vec![[0.0f32; 3]; n];
    let mut count = vec![0u32; n];

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let a = indices[t * 3] as usize;
        let b = indices[t * 3 + 1] as usize;
        let c = indices[t * 3 + 2] as usize;
        for &(u, v) in &[(a, b), (a, c), (b, a), (b, c), (c, a), (c, b)] {
            if u < n && v < n {
                sum[u][0] += positions[v][0];
                sum[u][1] += positions[v][1];
                sum[u][2] += positions[v][2];
                count[u] += 1;
            }
        }
    }

    let mut out = positions.to_vec();
    for i in 0..n {
        if count[i] > 0 {
            let inv = 1.0 / count[i] as f32;
            out[i] = [sum[i][0] * inv, sum[i][1] * inv, sum[i][2] * inv];
        }
    }
    out
}

/// Accumulate face normals into each vertex and normalise.
fn compute_vertex_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let n = positions.len();
    let mut normals = vec![[0.0f32; 3]; n];

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= n || ib >= n || ic >= n {
            continue;
        }
        let pa = positions[ia];
        let pb = positions[ib];
        let pc = positions[ic];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let nx = ab[1] * ac[2] - ab[2] * ac[1];
        let ny = ab[2] * ac[0] - ab[0] * ac[2];
        let nz = ab[0] * ac[1] - ab[1] * ac[0];
        for &idx in &[ia, ib, ic] {
            normals[idx][0] += nx;
            normals[idx][1] += ny;
            normals[idx][2] += nz;
        }
    }

    for norm in &mut normals {
        let len = (norm[0] * norm[0] + norm[1] * norm[1] + norm[2] * norm[2]).sqrt();
        if len > 1e-10 {
            norm[0] /= len;
            norm[1] /= len;
            norm[2] /= len;
        }
    }
    normals
}

/// Estimate the memory footprint (bytes) of a fluid surface grid.
pub fn grid_memory_bytes(dims: [usize; 3]) -> usize {
    dims[0] * dims[1] * dims[2] * std::mem::size_of::<f32>()
}

/// Validate that dims are non-zero.
pub fn validate_dims(dims: [usize; 3]) -> bool {
    dims.iter().all(|&d| d > 0)
}

/// Compute the world-space extent of a grid cell.
pub fn grid_extent(dims: [usize; 3], cell_size: f32) -> [f32; 3] {
    [
        dims[0] as f32 * cell_size,
        dims[1] as f32 * cell_size,
        dims[2] as f32 * cell_size,
    ]
}

/// Clamp the iso-level to a valid range `[0, 1]`.
pub fn clamp_iso_level(level: f32) -> f32 {
    level.clamp(0.0, 1.0)
}

/// Estimate the number of surface triangles from voxel count.
pub fn estimate_surface_triangles(active_voxels: usize) -> usize {
    active_voxels * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_iso_level() {
        /* iso level should be 0.5 */
        let p = FluidSurfaceParams::default();
        assert!((p.iso_level - 0.5).abs() < 1e-6);
    }

    #[test]
    fn extract_stub_empty() {
        /* stub returns empty mesh */
        let s = extract_fluid_surface(&[], [4, 4, 4], &FluidSurfaceParams::default());
        assert_eq!(s.triangle_count(), 0);
    }

    #[test]
    fn grid_memory_nonzero() {
        /* 4x4x4 grid should have memory */
        assert!(grid_memory_bytes([4, 4, 4]) > 0);
    }

    #[test]
    fn validate_dims_ok() {
        /* all positive dims are valid */
        assert!(validate_dims([8, 8, 8]));
    }

    #[test]
    fn validate_dims_zero() {
        /* zero dim is invalid */
        assert!(!validate_dims([0, 8, 8]));
    }

    #[test]
    fn grid_extent_correct() {
        /* 4x4x4 at 0.1 → 0.4 each axis */
        let e = grid_extent([4, 4, 4], 0.1);
        assert!((e[0] - 0.4).abs() < 1e-5);
    }

    #[test]
    fn clamp_iso_above_one() {
        /* clamped to 1.0 */
        assert!((clamp_iso_level(2.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clamp_iso_below_zero() {
        /* clamped to 0.0 */
        assert!((clamp_iso_level(-0.3)).abs() < 1e-6);
    }

    #[test]
    fn estimate_triangles() {
        /* 100 voxels → 200 triangles */
        assert_eq!(estimate_surface_triangles(100), 200);
    }

    #[test]
    fn fluid_surface_vertex_count() {
        /* vertex_count returns positions.len() */
        let s = FluidSurface {
            positions: vec![[0.0; 3]; 6],
            indices: vec![0, 1, 2, 3, 4, 5],
            normals: vec![[0.0, 1.0, 0.0]; 6],
        };
        assert_eq!(s.vertex_count(), 6);
    }

    #[test]
    fn sphere_density_produces_non_empty_surface() {
        // Build a 20x20x20 grid whose density is 1 inside a sphere of radius 7
        // centred at (10, 10, 10), and 0 outside.
        let n: usize = 20;
        let total = n * n * n;
        let mut density = vec![0.0f32; total];
        let cx = 10.0f32;
        let cy = 10.0f32;
        let cz = 10.0f32;
        let r = 7.0f32;
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dz = z as f32 - cz;
                    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                    let idx = x + n * (y + n * z);
                    density[idx] = if dist < r { 1.0 } else { 0.0 };
                }
            }
        }
        let params = FluidSurfaceParams {
            iso_level: 0.5,
            cell_size: 0.1,
            smooth_iters: 0,
        };
        let surface = extract_fluid_surface(&density, [n, n, n], &params);
        assert!(
            !surface.positions.is_empty(),
            "sphere density must produce non-empty surface"
        );
        assert!(
            !surface.indices.is_empty(),
            "sphere density must produce non-empty indices"
        );
    }
}
