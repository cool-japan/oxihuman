// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Voxel grid for collision detection and SDF approximation.
//!
//! Voxels are stored in a flat `Vec<Voxel>` addressed by `(ix, iy, iz)` in
//! X-major order: `index = ix + nx * (iy + ny * iz)`.

/// Configuration for a voxel grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxelConfig {
    /// Voxel cell size (same in all axes).
    pub cell_size: f32,
    /// Grid origin (world-space minimum corner).
    pub origin: [f32; 3],
    /// Grid dimensions in voxels: `[nx, ny, nz]`.
    pub dimensions: [usize; 3],
}

/// A single voxel cell.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Voxel {
    /// Whether this cell is occupied by mesh geometry.
    pub occupied: bool,
    /// Approximate signed distance to the nearest surface (negative = inside).
    pub sdf: f32,
}

/// A voxel grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxelGrid {
    pub config: VoxelConfig,
    /// Flat storage: `data[ix + nx * (iy + ny * iz)]`.
    pub data: Vec<Voxel>,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// Voxel grid index triple.
pub type VoxelIndex = [usize; 3];

/// World-space position.
pub type WorldPos = [f32; 3];

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default voxel configuration (1 m cells, 16³ grid at origin).
#[allow(dead_code)]
pub fn default_voxel_config() -> VoxelConfig {
    VoxelConfig {
        cell_size: 1.0,
        origin: [0.0; 3],
        dimensions: [16, 16, 16],
    }
}

/// Construct an empty voxel grid from a configuration.
#[allow(dead_code)]
pub fn new_voxel_grid(config: VoxelConfig) -> VoxelGrid {
    let n = config.dimensions[0] * config.dimensions[1] * config.dimensions[2];
    VoxelGrid {
        config,
        data: vec![Voxel::default(); n],
    }
}

/// Set a voxel at grid index `[ix, iy, iz]`.
///
/// Returns `false` if the index is out of bounds.
#[allow(dead_code)]
pub fn set_voxel(grid: &mut VoxelGrid, idx: VoxelIndex, voxel: Voxel) -> bool {
    if let Some(flat) = flat_index(&grid.config, idx) {
        grid.data[flat] = voxel;
        true
    } else {
        false
    }
}

/// Get the voxel at grid index `[ix, iy, iz]`, or `None` if out of bounds.
#[allow(dead_code)]
pub fn get_voxel(grid: &VoxelGrid, idx: VoxelIndex) -> Option<Voxel> {
    flat_index(&grid.config, idx).map(|f| grid.data[f])
}

/// Voxelize a triangle mesh (given flat positions and indices) into the grid.
///
/// A voxel is marked occupied if any triangle centre falls within its cell.
/// This is a fast conservative approximation.
#[allow(dead_code)]
pub fn voxelize_mesh(grid: &mut VoxelGrid, positions: &[f32], indices: &[usize]) {
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let a = indices[f * 3];
        let b = indices[f * 3 + 1];
        let c = indices[f * 3 + 2];
        let cx = (positions[a * 3] + positions[b * 3] + positions[c * 3]) / 3.0;
        let cy = (positions[a * 3 + 1] + positions[b * 3 + 1] + positions[c * 3 + 1]) / 3.0;
        let cz = (positions[a * 3 + 2] + positions[b * 3 + 2] + positions[c * 3 + 2]) / 3.0;
        if let Some(idx) = world_to_grid(grid, [cx, cy, cz]) {
            if let Some(flat) = flat_index(&grid.config, idx) {
                grid.data[flat].occupied = true;
            }
        }
    }
}

/// Return the total number of voxel cells in the grid.
#[allow(dead_code)]
pub fn voxel_count(grid: &VoxelGrid) -> usize {
    grid.data.len()
}

/// Return the number of occupied voxels.
#[allow(dead_code)]
pub fn occupied_count(grid: &VoxelGrid) -> usize {
    grid.data.iter().filter(|v| v.occupied).count()
}

/// Convert a voxel grid index to the world-space centre of that voxel.
#[allow(dead_code)]
pub fn grid_to_world(grid: &VoxelGrid, idx: VoxelIndex) -> WorldPos {
    let s = grid.config.cell_size;
    let o = grid.config.origin;
    [
        o[0] + (idx[0] as f32 + 0.5) * s,
        o[1] + (idx[1] as f32 + 0.5) * s,
        o[2] + (idx[2] as f32 + 0.5) * s,
    ]
}

/// Convert a world-space position to the voxel grid index, or `None` if outside
/// the grid bounds.
#[allow(dead_code)]
pub fn world_to_grid(grid: &VoxelGrid, pos: WorldPos) -> Option<VoxelIndex> {
    let s = grid.config.cell_size;
    let o = grid.config.origin;
    let dim = grid.config.dimensions;
    let raw = [
        ((pos[0] - o[0]) / s).floor() as i64,
        ((pos[1] - o[1]) / s).floor() as i64,
        ((pos[2] - o[2]) / s).floor() as i64,
    ];
    if raw[0] < 0
        || raw[1] < 0
        || raw[2] < 0
        || raw[0] >= dim[0] as i64
        || raw[1] >= dim[1] as i64
        || raw[2] >= dim[2] as i64
    {
        return None;
    }
    Some([raw[0] as usize, raw[1] as usize, raw[2] as usize])
}

/// Return whether the voxel at `idx` is occupied.
#[allow(dead_code)]
pub fn is_occupied(grid: &VoxelGrid, idx: VoxelIndex) -> bool {
    get_voxel(grid, idx).map(|v| v.occupied).unwrap_or(false)
}

/// Compute an approximate SDF for all voxels using a brute-force sweep.
///
/// For each voxel, the SDF is set to the distance to the nearest occupied
/// voxel centre (negative if the voxel itself is occupied).
#[allow(dead_code)]
pub fn compute_sdf(grid: &mut VoxelGrid) {
    let nx = grid.config.dimensions[0];
    let ny = grid.config.dimensions[1];
    let nz = grid.config.dimensions[2];
    let s = grid.config.cell_size;
    let o = grid.config.origin;

    // Collect occupied voxel world positions.
    let mut occupied_positions: Vec<[f32; 3]> = Vec::new();
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let flat = ix + nx * (iy + ny * iz);
                if grid.data[flat].occupied {
                    occupied_positions.push([
                        o[0] + (ix as f32 + 0.5) * s,
                        o[1] + (iy as f32 + 0.5) * s,
                        o[2] + (iz as f32 + 0.5) * s,
                    ]);
                }
            }
        }
    }

    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let flat = ix + nx * (iy + ny * iz);
                let wp = [
                    o[0] + (ix as f32 + 0.5) * s,
                    o[1] + (iy as f32 + 0.5) * s,
                    o[2] + (iz as f32 + 0.5) * s,
                ];
                let min_dist = occupied_positions
                    .iter()
                    .map(|op| {
                        let dx = wp[0] - op[0];
                        let dy = wp[1] - op[1];
                        let dz = wp[2] - op[2];
                        (dx * dx + dy * dy + dz * dz).sqrt()
                    })
                    .fold(f32::MAX, f32::min);
                let sign = if grid.data[flat].occupied { -1.0 } else { 1.0 };
                grid.data[flat].sdf = sign * min_dist;
            }
        }
    }
}

/// Return the world-space AABB of the entire grid as `[min_x,min_y,min_z, max_x,max_y,max_z]`.
#[allow(dead_code)]
pub fn voxel_grid_bounds(grid: &VoxelGrid) -> [f32; 6] {
    let o = grid.config.origin;
    let s = grid.config.cell_size;
    let d = grid.config.dimensions;
    [
        o[0],
        o[1],
        o[2],
        o[0] + d[0] as f32 * s,
        o[1] + d[1] as f32 * s,
        o[2] + d[2] as f32 * s,
    ]
}

/// Stub for marching-cubes iso-surface extraction.
///
/// Returns the number of potential surface cells (occupied neighbours of empty
/// cells) as a proxy metric.
#[allow(dead_code)]
pub fn marching_cubes_stub(grid: &VoxelGrid) -> usize {
    let nx = grid.config.dimensions[0];
    let ny = grid.config.dimensions[1];
    let nz = grid.config.dimensions[2];
    let mut surface = 0usize;
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let flat = ix + nx * (iy + ny * iz);
                if !grid.data[flat].occupied {
                    // Check 6-neighbours.
                    let has_occ_neighbour = [
                        ix.checked_sub(1).map(|x| x + nx * (iy + ny * iz)),
                        if ix + 1 < nx { Some(ix + 1 + nx * (iy + ny * iz)) } else { None },
                        iy.checked_sub(1).map(|y| ix + nx * (y + ny * iz)),
                        if iy + 1 < ny { Some(ix + nx * (iy + 1 + ny * iz)) } else { None },
                        iz.checked_sub(1).map(|z| ix + nx * (iy + ny * z)),
                        if iz + 1 < nz { Some(ix + nx * (iy + ny * (iz + 1))) } else { None },
                    ]
                    .iter()
                    .any(|nb| nb.map(|f| grid.data[f].occupied).unwrap_or(false));
                    if has_occ_neighbour {
                        surface += 1;
                    }
                }
            }
        }
    }
    surface
}

/// Clear all voxels (reset to default / unoccupied).
#[allow(dead_code)]
pub fn clear_voxels(grid: &mut VoxelGrid) {
    for v in grid.data.iter_mut() {
        *v = Voxel::default();
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn flat_index(config: &VoxelConfig, idx: VoxelIndex) -> Option<usize> {
    let [nx, ny, nz] = config.dimensions;
    let [ix, iy, iz] = idx;
    if ix >= nx || iy >= ny || iz >= nz {
        return None;
    }
    Some(ix + nx * (iy + ny * iz))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn small_grid() -> VoxelGrid {
        new_voxel_grid(VoxelConfig {
            cell_size: 1.0,
            origin: [0.0; 3],
            dimensions: [4, 4, 4],
        })
    }

    #[test]
    fn test_default_voxel_config() {
        let cfg = default_voxel_config();
        assert_eq!(cfg.dimensions, [16, 16, 16]);
        assert!(cfg.cell_size > 0.0);
    }

    #[test]
    fn test_new_voxel_grid_total_count() {
        let grid = small_grid();
        assert_eq!(voxel_count(&grid), 64);
    }

    #[test]
    fn test_set_and_get_voxel() {
        let mut grid = small_grid();
        let v = Voxel { occupied: true, sdf: -1.0 };
        assert!(set_voxel(&mut grid, [1, 2, 3], v));
        let got = get_voxel(&grid, [1, 2, 3]).unwrap();
        assert!(got.occupied);
        assert!((got.sdf + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_voxel_out_of_bounds() {
        let mut grid = small_grid();
        let v = Voxel { occupied: true, sdf: 0.0 };
        assert!(!set_voxel(&mut grid, [10, 0, 0], v));
    }

    #[test]
    fn test_get_voxel_out_of_bounds() {
        let grid = small_grid();
        assert!(get_voxel(&grid, [99, 0, 0]).is_none());
    }

    #[test]
    fn test_occupied_count_initially_zero() {
        let grid = small_grid();
        assert_eq!(occupied_count(&grid), 0);
    }

    #[test]
    fn test_occupied_count_after_set() {
        let mut grid = small_grid();
        set_voxel(&mut grid, [0, 0, 0], Voxel { occupied: true, sdf: 0.0 });
        set_voxel(&mut grid, [1, 1, 1], Voxel { occupied: true, sdf: 0.0 });
        assert_eq!(occupied_count(&grid), 2);
    }

    #[test]
    fn test_grid_to_world_origin() {
        let grid = small_grid();
        let wp = grid_to_world(&grid, [0, 0, 0]);
        assert!((wp[0] - 0.5).abs() < 1e-5);
        assert!((wp[1] - 0.5).abs() < 1e-5);
        assert!((wp[2] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_world_to_grid_centre() {
        let grid = small_grid();
        let idx = world_to_grid(&grid, [1.5, 2.5, 0.5]).unwrap();
        assert_eq!(idx, [1, 2, 0]);
    }

    #[test]
    fn test_world_to_grid_out_of_bounds() {
        let grid = small_grid();
        assert!(world_to_grid(&grid, [100.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_is_occupied_false() {
        let grid = small_grid();
        assert!(!is_occupied(&grid, [0, 0, 0]));
    }

    #[test]
    fn test_is_occupied_true() {
        let mut grid = small_grid();
        set_voxel(&mut grid, [2, 2, 2], Voxel { occupied: true, sdf: 0.0 });
        assert!(is_occupied(&grid, [2, 2, 2]));
    }

    #[test]
    fn test_voxelize_mesh_single_triangle() {
        let mut grid = small_grid();
        // Triangle centred at ~(1.5, 1.5, 0)  → voxel [1,1,0]
        let pos = vec![
            1.0, 1.0, 0.0,
            2.0, 1.0, 0.0,
            1.5, 2.0, 0.0,
        ];
        let idx = vec![0, 1, 2];
        voxelize_mesh(&mut grid, &pos, &idx);
        assert!(occupied_count(&grid) >= 1);
    }

    #[test]
    fn test_voxel_grid_bounds() {
        let grid = small_grid();
        let bb = voxel_grid_bounds(&grid);
        assert!((bb[0]).abs() < 1e-5); // min_x = 0
        assert!((bb[3] - 4.0).abs() < 1e-5); // max_x = 4
    }

    #[test]
    fn test_compute_sdf_occupied_cell_negative() {
        let mut grid = small_grid();
        set_voxel(&mut grid, [0, 0, 0], Voxel { occupied: true, sdf: 0.0 });
        compute_sdf(&mut grid);
        let v = get_voxel(&grid, [0, 0, 0]).unwrap();
        assert!(v.sdf <= 0.0);
    }

    #[test]
    fn test_compute_sdf_empty_cell_positive() {
        let mut grid = small_grid();
        set_voxel(&mut grid, [0, 0, 0], Voxel { occupied: true, sdf: 0.0 });
        compute_sdf(&mut grid);
        let v = get_voxel(&grid, [3, 3, 3]).unwrap();
        assert!(v.sdf > 0.0);
    }

    #[test]
    fn test_marching_cubes_stub_no_occupied() {
        let grid = small_grid();
        // No occupied voxels → no surface cells.
        assert_eq!(marching_cubes_stub(&grid), 0);
    }

    #[test]
    fn test_marching_cubes_stub_one_occupied() {
        let mut grid = small_grid();
        set_voxel(&mut grid, [1, 1, 1], Voxel { occupied: true, sdf: 0.0 });
        // Adjacent empty voxels are surface candidates.
        assert!(marching_cubes_stub(&grid) > 0);
    }

    #[test]
    fn test_clear_voxels() {
        let mut grid = small_grid();
        set_voxel(&mut grid, [0, 0, 0], Voxel { occupied: true, sdf: -1.0 });
        clear_voxels(&mut grid);
        assert_eq!(occupied_count(&grid), 0);
    }
}
