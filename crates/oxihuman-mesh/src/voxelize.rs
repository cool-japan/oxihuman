// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ---------------------------------------------------------------------------
// VoxelGrid
// ---------------------------------------------------------------------------

/// A 3-D boolean voxel grid (true = solid / surface).
#[derive(Debug, Clone)]
pub struct VoxelGrid {
    /// Flat storage in ZYX order: index = iz*ny*nx + iy*nx + ix
    pub data: Vec<bool>,
    /// Grid dimensions [nx, ny, nz]
    pub dims: [usize; 3],
    /// World-space position of the (0,0,0) voxel centre
    pub origin: [f32; 3],
    /// Uniform side length of every voxel
    pub voxel_size: f32,
}

impl VoxelGrid {
    /// Create an empty (all-false) grid.
    pub fn new(dims: [usize; 3], origin: [f32; 3], voxel_size: f32) -> Self {
        let total = dims[0] * dims[1] * dims[2];
        VoxelGrid {
            data: vec![false; total],
            dims,
            origin,
            voxel_size,
        }
    }

    #[inline]
    fn idx(&self, ix: usize, iy: usize, iz: usize) -> usize {
        iz * self.dims[1] * self.dims[0] + iy * self.dims[0] + ix
    }

    /// Read a single voxel.
    pub fn get(&self, ix: usize, iy: usize, iz: usize) -> bool {
        self.data[self.idx(ix, iy, iz)]
    }

    /// Write a single voxel.
    pub fn set(&mut self, ix: usize, iy: usize, iz: usize, val: bool) {
        let i = self.idx(ix, iy, iz);
        self.data[i] = val;
    }

    /// World-space centre of voxel (ix, iy, iz).
    pub fn world_pos(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        [
            self.origin[0] + ix as f32 * self.voxel_size,
            self.origin[1] + iy as f32 * self.voxel_size,
            self.origin[2] + iz as f32 * self.voxel_size,
        ]
    }

    /// Number of voxels set to `true`.
    pub fn voxel_count(&self) -> usize {
        self.data.iter().filter(|&&v| v).count()
    }

    /// Total number of cells (nx * ny * nz).
    pub fn total_cells(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    /// Fraction of cells that are solid.
    pub fn density(&self) -> f32 {
        let total = self.total_cells();
        if total == 0 {
            return 0.0;
        }
        self.voxel_count() as f32 / total as f32
    }
}

// ---------------------------------------------------------------------------
// VoxelizeParams
// ---------------------------------------------------------------------------

/// Parameters controlling the voxelization.
pub struct VoxelizeParams {
    /// Number of voxels along the longest axis of the mesh AABB.
    pub resolution: usize,
    /// When `true`, only the surface shell is marked (no solid fill).
    pub surface_only: bool,
    /// Fractional padding added around the mesh AABB before gridding.
    pub padding: f32,
}

impl Default for VoxelizeParams {
    fn default() -> Self {
        Self {
            resolution: 32,
            surface_only: false,
            padding: 0.05,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper math
// ---------------------------------------------------------------------------

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

// ---------------------------------------------------------------------------
// mesh_bounds
// ---------------------------------------------------------------------------

/// Compute the axis-aligned bounding box of the mesh.
/// Returns `(min_xyz, max_xyz)`.
pub fn mesh_bounds(mesh: &MeshBuffers) -> ([f32; 3], [f32; 3]) {
    if mesh.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut mn = mesh.positions[0];
    let mut mx = mesh.positions[0];
    for p in &mesh.positions {
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mn[2] = mn[2].min(p[2]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
        mx[2] = mx[2].max(p[2]);
    }
    (mn, mx)
}

// ---------------------------------------------------------------------------
// Grid setup helper
// ---------------------------------------------------------------------------

/// Given mesh bounds and params, compute origin, voxel_size, and dims.
fn compute_grid_params(
    mn: [f32; 3],
    mx: [f32; 3],
    params: &VoxelizeParams,
) -> ([f32; 3], f32, [usize; 3]) {
    let extents = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
    let longest = extents[0].max(extents[1]).max(extents[2]).max(1e-6);

    let voxel_size = longest / params.resolution as f32;
    let pad = longest * params.padding;

    let origin = [
        mn[0] - pad + voxel_size * 0.5,
        mn[1] - pad + voxel_size * 0.5,
        mn[2] - pad + voxel_size * 0.5,
    ];

    let dims = [
        (((extents[0] + 2.0 * pad) / voxel_size).ceil() as usize).max(1),
        (((extents[1] + 2.0 * pad) / voxel_size).ceil() as usize).max(1),
        (((extents[2] + 2.0 * pad) / voxel_size).ceil() as usize).max(1),
    ];

    (origin, voxel_size, dims)
}

// ---------------------------------------------------------------------------
// voxelize_surface
// ---------------------------------------------------------------------------

/// Mark all voxels whose centre lies within `voxel_size * 0.5` (perpendicular
/// distance to the triangle plane) AND whose closest-point projection falls
/// inside the triangle (barycentric test).
pub fn voxelize_surface(mesh: &MeshBuffers, params: &VoxelizeParams) -> VoxelGrid {
    let (mn, mx) = mesh_bounds(mesh);
    let (origin, voxel_size, dims) = compute_grid_params(mn, mx, params);
    let mut grid = VoxelGrid::new(dims, origin, voxel_size);

    let threshold = voxel_size * 0.86; // √(3)/2 * voxel_size captures corner hits

    let faces = mesh.indices.len() / 3;
    for f in 0..faces {
        let i0 = mesh.indices[f * 3] as usize;
        let i1 = mesh.indices[f * 3 + 1] as usize;
        let i2 = mesh.indices[f * 3 + 2] as usize;

        if i0 >= mesh.positions.len() || i1 >= mesh.positions.len() || i2 >= mesh.positions.len() {
            continue;
        }

        let v0 = mesh.positions[i0];
        let v1 = mesh.positions[i1];
        let v2 = mesh.positions[i2];

        // Face AABB in voxel indices
        let face_min = [
            v0[0].min(v1[0]).min(v2[0]),
            v0[1].min(v1[1]).min(v2[1]),
            v0[2].min(v1[2]).min(v2[2]),
        ];
        let face_max = [
            v0[0].max(v1[0]).max(v2[0]),
            v0[1].max(v1[1]).max(v2[1]),
            v0[2].max(v1[2]).max(v2[2]),
        ];

        // Convert to voxel index range (with a 1-cell margin)
        let ix_min = ((face_min[0] - origin[0] - threshold) / voxel_size)
            .floor()
            .max(0.0) as usize;
        let iy_min = ((face_min[1] - origin[1] - threshold) / voxel_size)
            .floor()
            .max(0.0) as usize;
        let iz_min = ((face_min[2] - origin[2] - threshold) / voxel_size)
            .floor()
            .max(0.0) as usize;

        let ix_max = ((face_max[0] - origin[0] + threshold) / voxel_size)
            .ceil()
            .min((dims[0] - 1) as f32) as usize;
        let iy_max = ((face_max[1] - origin[1] + threshold) / voxel_size)
            .ceil()
            .min((dims[1] - 1) as f32) as usize;
        let iz_max = ((face_max[2] - origin[2] + threshold) / voxel_size)
            .ceil()
            .min((dims[2] - 1) as f32) as usize;

        // Triangle edge vectors and normal
        let e1 = vec3_sub(v1, v0);
        let e2 = vec3_sub(v2, v0);
        let normal = vec3_cross(e1, e2);
        let normal_len_sq = vec3_dot(normal, normal);

        for iz in iz_min..=iz_max {
            for iy in iy_min..=iy_max {
                for ix in ix_min..=ix_max {
                    let centre = grid.world_pos(ix, iy, iz);

                    // Distance from point to triangle plane
                    if normal_len_sq > 1e-12 {
                        let to_point = vec3_sub(centre, v0);
                        let dist_plane = (vec3_dot(normal, to_point) / normal_len_sq.sqrt()).abs();

                        if dist_plane > threshold {
                            continue;
                        }

                        // Project onto triangle plane, barycentric test
                        let w = vec3_sub(centre, v0);
                        let dot00 = vec3_dot(e2, e2);
                        let dot01 = vec3_dot(e2, e1);
                        let dot02 = vec3_dot(e2, w);
                        let dot11 = vec3_dot(e1, e1);
                        let dot12 = vec3_dot(e1, w);

                        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
                        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
                        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

                        // Allow a small margin to catch edge/vertex-touching voxels
                        let margin = 0.05;
                        if u >= -margin && v >= -margin && u + v <= 1.0 + margin {
                            grid.set(ix, iy, iz, true);
                        }
                    } else {
                        // Degenerate triangle: fall back to point-in-sphere test
                        let d = vec3_len(vec3_sub(centre, v0));
                        if d <= threshold {
                            grid.set(ix, iy, iz, true);
                        }
                    }
                }
            }
        }
    }

    grid
}

// ---------------------------------------------------------------------------
// voxelize_solid  (ray casting along Z)
// ---------------------------------------------------------------------------

/// Ray–triangle intersection returning the Z parameter of the hit (or None).
/// Ray: origin = (cx, cy, -∞), direction = (0, 0, 1).
fn ray_z_triangle_t(cx: f32, cy: f32, v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> Option<f32> {
    let e1 = vec3_sub(v1, v0);
    let e2 = vec3_sub(v2, v0);

    // Möller–Trumbore, direction = (0,0,1)
    // h = cross((0,0,1), e2)  → correct: cross(dir, e2)
    // Actually h = cross(dir, e2), dir=(0,0,1)
    // cross((0,0,1), (ex,ey,ez)) = (0*ez - 1*ey, 1*ex - 0*ez, 0*ey - 0*ex) = (-ey, ex, 0)
    let h = [-e2[1], e2[0], 0.0];
    let a = vec3_dot(e1, h);

    if a.abs() < 1e-10 {
        return None; // parallel
    }

    let f = 1.0 / a;
    let s = [cx - v0[0], cy - v0[1], -v0[2]];
    let u = f * vec3_dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = vec3_cross(s, e1);
    let v = f * q[2]; // dot((0,0,1), q) = q[2]
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    Some(t + v0[2])
}

/// Fill-solid voxelization using Z-axis ray casting and parity counting.
pub fn voxelize_solid(mesh: &MeshBuffers, params: &VoxelizeParams) -> VoxelGrid {
    let (mn, mx) = mesh_bounds(mesh);
    let (origin, voxel_size, dims) = compute_grid_params(mn, mx, params);
    let mut grid = VoxelGrid::new(dims, origin, voxel_size);

    let faces = mesh.indices.len() / 3;

    // Precompute triangle positions
    let mut tris: Vec<([f32; 3], [f32; 3], [f32; 3])> = Vec::with_capacity(faces);
    for f in 0..faces {
        let i0 = mesh.indices[f * 3] as usize;
        let i1 = mesh.indices[f * 3 + 1] as usize;
        let i2 = mesh.indices[f * 3 + 2] as usize;
        if i0 < mesh.positions.len() && i1 < mesh.positions.len() && i2 < mesh.positions.len() {
            tris.push((mesh.positions[i0], mesh.positions[i1], mesh.positions[i2]));
        }
    }

    for iy in 0..dims[1] {
        for ix in 0..dims[0] {
            let cx = origin[0] + ix as f32 * voxel_size;
            let cy = origin[1] + iy as f32 * voxel_size;

            // Collect Z positions of all ray hits
            let mut hits: Vec<f32> = tris
                .iter()
                .filter_map(|&(v0, v1, v2)| ray_z_triangle_t(cx, cy, v0, v1, v2))
                .collect();

            hits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            hits.dedup_by(|a, b| (*a - *b).abs() < voxel_size * 0.01);

            // Parity fill: voxel is inside if an odd number of surfaces are below it
            for iz in 0..dims[2] {
                let cz = origin[2] + iz as f32 * voxel_size;
                let count = hits.iter().filter(|&&hz| hz < cz).count();
                if count % 2 == 1 {
                    grid.set(ix, iy, iz, true);
                }
            }
        }
    }

    grid
}

// ---------------------------------------------------------------------------
// voxelize (dispatch)
// ---------------------------------------------------------------------------

/// Voxelize according to `params.surface_only`:
/// - `false` → solid fill (default)
/// - `true`  → surface only
pub fn voxelize(mesh: &MeshBuffers, params: &VoxelizeParams) -> VoxelGrid {
    if params.surface_only {
        voxelize_surface(mesh, params)
    } else {
        voxelize_solid(mesh, params)
    }
}

// ---------------------------------------------------------------------------
// voxel_to_mesh
// ---------------------------------------------------------------------------

/// Convert every solid voxel into a cube made of 6 quad faces (12 triangles).
/// Normals are recomputed before returning.
pub fn voxel_to_mesh(grid: &VoxelGrid) -> MeshBuffers {
    let hs = grid.voxel_size * 0.5;

    // Offsets for the 8 cube corners relative to voxel centre
    let corners: [[f32; 3]; 8] = [
        [-hs, -hs, -hs], // 0
        [hs, -hs, -hs],  // 1
        [hs, hs, -hs],   // 2
        [-hs, hs, -hs],  // 3
        [-hs, -hs, hs],  // 4
        [hs, -hs, hs],   // 5
        [hs, hs, hs],    // 6
        [-hs, hs, hs],   // 7
    ];

    // 6 faces; each face: 4 vertex indices into `corners`, then 2 triangles
    let face_quads: [[usize; 4]; 6] = [
        [0, 1, 2, 3], // -Z
        [5, 4, 7, 6], // +Z
        [4, 0, 3, 7], // -X
        [1, 5, 6, 2], // +X
        [4, 5, 1, 0], // -Y
        [3, 2, 6, 7], // +Y
    ];

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let uv_face = [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    for iz in 0..grid.dims[2] {
        for iy in 0..grid.dims[1] {
            for ix in 0..grid.dims[0] {
                if !grid.get(ix, iy, iz) {
                    continue;
                }
                let centre = grid.world_pos(ix, iy, iz);

                for quad in &face_quads {
                    let base = positions.len() as u32;
                    for (k, &ci) in quad.iter().enumerate() {
                        let c = corners[ci];
                        positions.push([centre[0] + c[0], centre[1] + c[1], centre[2] + c[2]]);
                        uvs.push(uv_face[k]);
                    }
                    // Two triangles: (0,1,2) and (0,2,3)
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    let n_verts = positions.len();
    let mut buf = MeshBuffers {
        positions,
        normals: vec![[0.0, 1.0, 0.0]; n_verts],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n_verts],
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };

    compute_normals(&mut buf);
    buf
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers;

    fn make_triangle_mesh(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> MeshBuffers {
        MeshBuffers {
            positions: vec![v0, v1, v2],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0]; 3],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    fn make_quad_mesh() -> MeshBuffers {
        // Two triangles forming a 1x1 quad in the XY plane at z=0
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            indices: vec![0, 1, 2, 0, 2, 3],
            colors: None,
            has_suit: false,
        }
    }

    // -----------------------------------------------------------------------
    #[test]
    fn test_voxel_grid_new() {
        let grid = VoxelGrid::new([4, 4, 4], [0.0; 3], 1.0);
        assert_eq!(grid.total_cells(), 64);
        assert_eq!(grid.voxel_count(), 0);
        assert!(!grid.get(0, 0, 0));
    }

    #[test]
    fn test_voxel_grid_get_set() {
        let mut grid = VoxelGrid::new([3, 3, 3], [0.0; 3], 1.0);
        grid.set(1, 2, 0, true);
        assert!(grid.get(1, 2, 0));
        assert!(!grid.get(0, 0, 0));
        assert_eq!(grid.voxel_count(), 1);
    }

    #[test]
    fn test_voxel_grid_world_pos() {
        let grid = VoxelGrid::new([5, 5, 5], [1.0, 2.0, 3.0], 0.5);
        let wp = grid.world_pos(2, 3, 4);
        assert!((wp[0] - 2.0).abs() < 1e-5, "x mismatch {}", wp[0]);
        assert!((wp[1] - 3.5).abs() < 1e-5, "y mismatch {}", wp[1]);
        assert!((wp[2] - 5.0).abs() < 1e-5, "z mismatch {}", wp[2]);
    }

    #[test]
    fn test_voxel_grid_density() {
        let mut grid = VoxelGrid::new([4, 4, 4], [0.0; 3], 1.0);
        assert!((grid.density() - 0.0).abs() < 1e-6);
        // Set 16 of 64 cells → 0.25
        for ix in 0..4 {
            for iy in 0..4 {
                grid.set(ix, iy, 0, true);
            }
        }
        assert!(
            (grid.density() - 0.25).abs() < 1e-5,
            "density={}",
            grid.density()
        );
    }

    #[test]
    fn test_mesh_bounds_triangle() {
        let mesh = make_triangle_mesh([0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 4.0, 5.0]);
        let (mn, mx) = mesh_bounds(&mesh);
        assert!((mn[0] - 0.0).abs() < 1e-6);
        assert!((mx[0] - 3.0).abs() < 1e-6);
        assert!((mx[1] - 4.0).abs() < 1e-6);
        assert!((mx[2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_voxelize_surface_triangle() {
        // A simple triangle in the XY plane
        let mesh = make_triangle_mesh([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]);
        let params = VoxelizeParams {
            resolution: 8,
            surface_only: true,
            padding: 0.1,
        };
        let grid = voxelize_surface(&mesh, &params);
        assert!(
            grid.voxel_count() > 0,
            "expected at least one surface voxel"
        );
        assert!(grid.total_cells() > 0);
    }

    #[test]
    fn test_voxelize_solid_simple() {
        // A closed unit cube (two triangles per face = 12 triangles total)
        // We use a simple quad mesh + a z-slab to test parity logic
        let mesh = make_quad_mesh();
        let params = VoxelizeParams {
            resolution: 8,
            surface_only: false,
            padding: 0.1,
        };
        let grid = voxelize_solid(&mesh, &params);
        // The grid should exist; solid count may be 0 for an open mesh, just verify no crash
        assert!(grid.total_cells() > 0);
    }

    #[test]
    fn test_voxel_to_mesh_single() {
        let mut grid = VoxelGrid::new([1, 1, 1], [0.0; 3], 1.0);
        grid.set(0, 0, 0, true);
        let mesh = voxel_to_mesh(&grid);
        // 6 faces × 4 verts = 24 verts; 6 faces × 2 tris × 3 idx = 36 indices
        assert_eq!(mesh.positions.len(), 24, "verts");
        assert_eq!(mesh.indices.len(), 36, "indices");
    }

    #[test]
    fn test_voxelize_params_default() {
        let p = VoxelizeParams::default();
        assert_eq!(p.resolution, 32);
        assert!(!p.surface_only);
        assert!((p.padding - 0.05).abs() < 1e-6);
    }

    #[test]
    fn test_voxelize_resolution() {
        let mesh = make_triangle_mesh([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let p8 = VoxelizeParams {
            resolution: 8,
            surface_only: true,
            padding: 0.05,
        };
        let p16 = VoxelizeParams {
            resolution: 16,
            surface_only: true,
            padding: 0.05,
        };
        let g8 = voxelize_surface(&mesh, &p8);
        let g16 = voxelize_surface(&mesh, &p16);
        // Higher resolution → more total cells (and generally more surface voxels)
        assert!(g16.total_cells() >= g8.total_cells());
    }

    #[test]
    fn test_voxelize_dispatch_surface_only() {
        let mesh = make_triangle_mesh([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
        let params = VoxelizeParams {
            resolution: 8,
            surface_only: true,
            padding: 0.05,
        };
        let grid = voxelize(&mesh, &params);
        assert!(grid.voxel_count() > 0);
    }

    #[test]
    fn test_voxelize_dispatch_solid() {
        let mesh = make_triangle_mesh([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]);
        let params = VoxelizeParams {
            resolution: 8,
            surface_only: false,
            padding: 0.05,
        };
        let grid = voxelize(&mesh, &params);
        // For an open mesh, just verify it runs without panic and returns a valid grid
        assert!(grid.total_cells() > 0);
    }
}
