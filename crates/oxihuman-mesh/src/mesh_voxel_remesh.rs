// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Voxel-based mesh remeshing via signed distance field reconstruction.

use crate::marching_cubes::{marching_cubes, ScalarField};

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
    /// Extracted vertex positions (world-space XYZ).
    pub vertices: Vec<[f32; 3]>,
    /// Triangle index list (flat, groups of 3).
    pub triangles: Vec<u32>,
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

/// Apply one pass of Laplacian smoothing to a set of vertex positions given a
/// flat triangle index list.  Each vertex is moved toward the mean of its
/// one-ring neighbours.  The result is written back to `positions` in-place.
fn laplacian_smooth_pass(positions: &mut [[f32; 3]], indices: &[u32]) {
    if positions.is_empty() || indices.len() < 3 {
        return;
    }
    let n = positions.len();
    let mut sum = vec![[0.0f32; 3]; n];
    let mut count = vec![0u32; n];

    let mut t = 0;
    while t + 2 < indices.len() {
        let [ia, ib, ic] = [
            indices[t] as usize,
            indices[t + 1] as usize,
            indices[t + 2] as usize,
        ];
        if ia < n && ib < n && ic < n {
            // Each vertex accumulates its two triangle-neighbours
            for &(src, dst) in &[(ib, ia), (ic, ia), (ia, ib), (ic, ib), (ia, ic), (ib, ic)] {
                for k in 0..3 {
                    sum[dst][k] += positions[src][k];
                }
                count[dst] += 1;
            }
        }
        t += 3;
    }

    for i in 0..n {
        if count[i] > 0 {
            let inv = 1.0 / count[i] as f32;
            positions[i][0] = sum[i][0] * inv;
            positions[i][1] = sum[i][1] * inv;
            positions[i][2] = sum[i][2] * inv;
        }
    }
}

/// Reconstruct a mesh from a `VoxelRemeshGrid` using Marching Cubes.
///
/// The boolean voxel occupancy is converted to a signed scalar field:
/// occupied cells get `-1.0` (inside) and empty cells get `+1.0` (outside).
/// Marching Cubes extracts the zero-crossing surface.  Optional Laplacian
/// smoothing passes are applied according to `cfg.smooth_iterations`.
#[allow(dead_code)]
pub fn remesh_from_voxels(grid: &VoxelRemeshGrid, cfg: &VoxelRemeshConfig) -> VoxelRemeshResult {
    let filled = filled_voxel_count(grid);
    if filled == 0 {
        return VoxelRemeshResult {
            vertex_count: 0,
            triangle_count: 0,
            success: false,
            iterations: cfg.smooth_iterations,
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
    }

    let nx = grid.width as usize;
    let ny = grid.height as usize;
    let nz = grid.depth as usize;

    // Build a ScalarField: occupied → -1 (inside), empty → +1 (outside).
    let mut field = ScalarField::new(
        [nx, ny, nz],
        [0.0f32; 3],
        [grid.voxel_size; 3],
    );
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let linear = ix + iy * nx + iz * nx * ny;
                let val = if grid.data[linear] { -1.0f32 } else { 1.0f32 };
                field.set(ix, iy, iz, val);
            }
        }
    }

    // Run Marching Cubes at isovalue 0.0.
    let mesh_buffers = marching_cubes(&field, 0.0);
    let mut positions = mesh_buffers.positions;
    let indices = mesh_buffers.indices;

    // Apply Laplacian smoothing.
    for _ in 0..cfg.smooth_iterations {
        laplacian_smooth_pass(&mut positions, &indices);
    }

    let vertex_count = positions.len();
    let triangle_count = indices.len() / 3;
    let success = vertex_count > 0 && triangle_count > 0;

    VoxelRemeshResult {
        vertex_count,
        triangle_count,
        success,
        iterations: cfg.smooth_iterations,
        vertices: positions,
        triangles: indices,
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
        assert!(r.triangle_count > 0, "marching cubes must produce triangles");
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
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
        let j = voxel_remesh_result_to_json(&r);
        assert!(j.contains("\"success\":true"));
        assert!(j.contains("\"iterations\":3"));
    }

    /// Build a small sphere mesh and round-trip through voxelise → remesh.
    /// The real marching cubes path must produce a non-trivial mesh.
    #[test]
    fn remesh_from_voxels_produces_triangles() {
        use std::f32::consts::PI;
        // Icosphere-like sphere mesh: unit sphere, ~100 surface triangles.
        let n_lat = 8usize;
        let n_lon = 8usize;
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut triangles: Vec<[u32; 3]> = Vec::new();

        // Generate latitude/longitude grid vertices on the unit sphere.
        for i in 0..=n_lat {
            let phi = PI * i as f32 / n_lat as f32; // 0 .. π
            for j in 0..=n_lon {
                let theta = 2.0 * PI * j as f32 / n_lon as f32; // 0 .. 2π
                positions.push([
                    phi.sin() * theta.cos(),
                    phi.cos(),
                    phi.sin() * theta.sin(),
                ]);
            }
        }
        let stride = n_lon + 1;
        for i in 0..n_lat {
            for j in 0..n_lon {
                let a = (i * stride + j) as u32;
                let b = (i * stride + j + 1) as u32;
                let c = ((i + 1) * stride + j) as u32;
                let d = ((i + 1) * stride + j + 1) as u32;
                triangles.push([a, b, c]);
                triangles.push([b, d, c]);
            }
        }

        let cfg = VoxelRemeshConfig {
            voxel_size: 0.3,
            smooth_iterations: 1,
            preserve_boundaries: false,
        };
        let grid = voxelize_mesh_remesh(&positions, &triangles, &cfg);
        let result = remesh_from_voxels(&grid, &cfg);

        assert!(result.success, "remesh of a sphere voxel grid must succeed");
        assert!(
            result.vertex_count > 4,
            "remesh must produce more than 4 vertices, got {}",
            result.vertex_count
        );
        assert!(
            result.triangle_count > 0,
            "remesh must produce triangles, got {}",
            result.triangle_count
        );
        assert_eq!(
            result.vertices.len(),
            result.vertex_count,
            "vertices vec length must match vertex_count"
        );
        assert_eq!(
            result.triangles.len(),
            result.triangle_count * 3,
            "triangles vec length must be triangle_count * 3"
        );
    }
}
