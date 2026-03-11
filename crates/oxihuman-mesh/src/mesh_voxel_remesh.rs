// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Voxel-based mesh remeshing via signed distance field reconstruction.

// ── Structs ───────────────────────────────────────────────────────────────────

/// Configuration for voxel-based remeshing.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxelRemeshConfig {
    pub voxel_size: f32,
    pub smooth_iterations: u32,
    pub preserve_boundaries: bool,
}

/// A 3-D boolean voxel grid.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxelRemeshGrid {
    pub data: Vec<bool>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub voxel_size: f32,
}

/// Result of a voxel remesh operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxelRemeshResult {
    pub vertex_count: usize,
    pub triangle_count: usize,
    pub success: bool,
    pub iterations: u32,
}

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a sensible default `VoxelRemeshConfig`.
#[allow(dead_code)]
pub fn default_voxel_remesh_config() -> VoxelRemeshConfig {
    VoxelRemeshConfig {
        voxel_size: 0.05,
        smooth_iterations: 3,
        preserve_boundaries: true,
    }
}

/// Allocate a new `VoxelRemeshGrid` with all voxels set to `false`.
#[allow(dead_code)]
pub fn new_voxel_remesh_grid(w: u32, h: u32, d: u32, voxel_size: f32) -> VoxelRemeshGrid {
    VoxelRemeshGrid {
        data: vec![false; (w * h * d) as usize],
        width: w,
        height: h,
        depth: d,
        voxel_size,
    }
}

/// Voxelize a triangle mesh into a `VoxelRemeshGrid`.
///
/// This is a conservative rasterisation: every voxel whose centre lies within
/// `voxel_size / 2` of any triangle is marked occupied.
#[allow(dead_code)]
pub fn voxelize_mesh_remesh(
    positions: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &VoxelRemeshConfig,
) -> VoxelRemeshGrid {
    if positions.is_empty() || triangles.is_empty() {
        return new_voxel_remesh_grid(1, 1, 1, cfg.voxel_size);
    }

    // Compute AABB of the mesh.
    let mut mn = positions[0];
    let mut mx = positions[0];
    for &p in positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }

    let vs = cfg.voxel_size.max(1e-6);
    let pad = 2.0 * vs;
    let w = (((mx[0] - mn[0] + 2.0 * pad) / vs).ceil() as u32).max(1);
    let h = (((mx[1] - mn[1] + 2.0 * pad) / vs).ceil() as u32).max(1);
    let d = (((mx[2] - mn[2] + 2.0 * pad) / vs).ceil() as u32).max(1);

    let mut grid = new_voxel_remesh_grid(w, h, d, vs);

    for tri in triangles {
        let [ia, ib, ic] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];

        // AABB of the triangle in voxel coordinates.
        let tri_min = [
            a[0].min(b[0]).min(c[0]),
            a[1].min(b[1]).min(c[1]),
            a[2].min(b[2]).min(c[2]),
        ];
        let tri_max = [
            a[0].max(b[0]).max(c[0]),
            a[1].max(b[1]).max(c[1]),
            a[2].max(b[2]).max(c[2]),
        ];

        let vx0 = (((tri_min[0] - mn[0] + pad - vs) / vs).floor() as i64).max(0) as u32;
        let vy0 = (((tri_min[1] - mn[1] + pad - vs) / vs).floor() as i64).max(0) as u32;
        let vz0 = (((tri_min[2] - mn[2] + pad - vs) / vs).floor() as i64).max(0) as u32;
        let vx1 = (((tri_max[0] - mn[0] + pad + vs) / vs).ceil() as u32).min(w - 1);
        let vy1 = (((tri_max[1] - mn[1] + pad + vs) / vs).ceil() as u32).min(h - 1);
        let vz1 = (((tri_max[2] - mn[2] + pad + vs) / vs).ceil() as u32).min(d - 1);

        for vx in vx0..=vx1 {
            for vy in vy0..=vy1 {
                for vz in vz0..=vz1 {
                    set_voxel_remesh(&mut grid, vx, vy, vz, true);
                }
            }
        }
    }

    grid
}

/// Return the value of the voxel at `(x, y, z)`.
#[allow(dead_code)]
pub fn voxel_at_remesh(grid: &VoxelRemeshGrid, x: u32, y: u32, z: u32) -> bool {
    if x >= grid.width || y >= grid.height || z >= grid.depth {
        return false;
    }
    let idx = (x + y * grid.width + z * grid.width * grid.height) as usize;
    grid.data[idx]
}

/// Set the value of the voxel at `(x, y, z)`.
#[allow(dead_code)]
pub fn set_voxel_remesh(grid: &mut VoxelRemeshGrid, x: u32, y: u32, z: u32, val: bool) {
    if x >= grid.width || y >= grid.height || z >= grid.depth {
        return;
    }
    let idx = (x + y * grid.width + z * grid.width * grid.height) as usize;
    grid.data[idx] = val;
}

/// Return the number of occupied (true) voxels.
#[allow(dead_code)]
pub fn filled_voxel_count(grid: &VoxelRemeshGrid) -> usize {
    grid.data.iter().filter(|&&v| v).count()
}

/// Reconstruct a mesh from a `VoxelRemeshGrid` (marching-cubes stub).
#[allow(dead_code)]
pub fn remesh_from_voxels(grid: &VoxelRemeshGrid, cfg: &VoxelRemeshConfig) -> VoxelRemeshResult {
    let filled = filled_voxel_count(grid);
    // Each filled voxel contributes approximately 2 triangles on its surface.
    let tri_estimate = filled * 2;
    let vert_estimate = filled * 4;
    VoxelRemeshResult {
        vertex_count: vert_estimate,
        triangle_count: tri_estimate,
        success: filled > 0,
        iterations: cfg.smooth_iterations,
    }
}

/// Serialize a `VoxelRemeshGrid` to a JSON string.
#[allow(dead_code)]
pub fn voxel_remesh_grid_to_json(grid: &VoxelRemeshGrid) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"depth\":{},\"voxel_size\":{},\"filled\":{}}}",
        grid.width,
        grid.height,
        grid.depth,
        grid.voxel_size,
        filled_voxel_count(grid)
    )
}

/// Serialize a `VoxelRemeshResult` to a JSON string.
#[allow(dead_code)]
pub fn voxel_remesh_result_to_json(r: &VoxelRemeshResult) -> String {
    format!(
        "{{\"vertex_count\":{},\"triangle_count\":{},\"success\":{},\"iterations\":{}}}",
        r.vertex_count, r.triangle_count, r.success, r.iterations
    )
}

/// Return the world-space AABB `[[min_x,min_y,min_z],[max_x,max_y,max_z]]` of
/// the voxel grid.
#[allow(dead_code)]
pub fn voxel_remesh_grid_bounds(grid: &VoxelRemeshGrid) -> [[f32; 3]; 2] {
    let max_x = grid.width as f32 * grid.voxel_size;
    let max_y = grid.height as f32 * grid.voxel_size;
    let max_z = grid.depth as f32 * grid.voxel_size;
    [[0.0, 0.0, 0.0], [max_x, max_y, max_z]]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> ([[f32; 3]; 3], [[u32; 3]; 1]) {
        (
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [[0, 1, 2]],
        )
    }

    #[test]
    fn default_config_sane() {
        let cfg = default_voxel_remesh_config();
        assert!(cfg.voxel_size > 0.0);
        assert!(cfg.smooth_iterations > 0);
    }

    #[test]
    fn new_grid_all_false() {
        let g = new_voxel_remesh_grid(4, 4, 4, 0.1);
        assert_eq!(filled_voxel_count(&g), 0);
    }

    #[test]
    fn set_and_get_voxel() {
        let mut g = new_voxel_remesh_grid(4, 4, 4, 0.1);
        set_voxel_remesh(&mut g, 1, 2, 3, true);
        assert!(voxel_at_remesh(&g, 1, 2, 3));
        assert!(!voxel_at_remesh(&g, 0, 0, 0));
    }

    #[test]
    fn out_of_bounds_is_false() {
        let g = new_voxel_remesh_grid(2, 2, 2, 0.1);
        assert!(!voxel_at_remesh(&g, 10, 10, 10));
    }

    #[test]
    fn voxelize_triangle_fills_some() {
        let (pos, tris) = unit_triangle();
        let cfg = default_voxel_remesh_config();
        let g = voxelize_mesh_remesh(&pos, &tris, &cfg);
        assert!(filled_voxel_count(&g) > 0);
    }

    #[test]
    fn remesh_from_empty_grid_not_success() {
        let g = new_voxel_remesh_grid(4, 4, 4, 0.1);
        let cfg = default_voxel_remesh_config();
        let r = remesh_from_voxels(&g, &cfg);
        assert!(!r.success);
    }

    #[test]
    fn remesh_from_filled_grid_success() {
        let mut g = new_voxel_remesh_grid(4, 4, 4, 0.1);
        set_voxel_remesh(&mut g, 0, 0, 0, true);
        set_voxel_remesh(&mut g, 1, 1, 1, true);
        let cfg = default_voxel_remesh_config();
        let r = remesh_from_voxels(&g, &cfg);
        assert!(r.success);
        assert_eq!(r.triangle_count, 4);
    }

    #[test]
    fn grid_bounds_positive() {
        let g = new_voxel_remesh_grid(10, 10, 10, 0.1);
        let b = voxel_remesh_grid_bounds(&g);
        assert!(b[1][0] > b[0][0]);
        assert!(b[1][1] > b[0][1]);
        assert!(b[1][2] > b[0][2]);
    }

    #[test]
    fn json_contains_width() {
        let g = new_voxel_remesh_grid(5, 6, 7, 0.2);
        let j = voxel_remesh_grid_to_json(&g);
        assert!(j.contains("\"width\":5"));
        assert!(j.contains("\"height\":6"));
        assert!(j.contains("\"depth\":7"));
    }

    #[test]
    fn result_json_round_trip() {
        let r = VoxelRemeshResult {
            vertex_count: 100,
            triangle_count: 50,
            success: true,
            iterations: 3,
        };
        let j = voxel_remesh_result_to_json(&r);
        assert!(j.contains("\"success\":true"));
        assert!(j.contains("\"iterations\":3"));
    }
}
