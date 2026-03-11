// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
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

/// Extract a fluid surface from a scalar density field.
/// Returns an empty mesh stub (no real MC in this stub).
pub fn extract_fluid_surface(
    _density: &[f32],
    _dims: [usize; 3],
    params: &FluidSurfaceParams,
) -> FluidSurface {
    let _ = params;
    FluidSurface {
        positions: Vec::new(),
        indices: Vec::new(),
        normals: Vec::new(),
    }
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
}
