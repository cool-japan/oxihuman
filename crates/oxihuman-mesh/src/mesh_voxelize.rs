// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Voxelize a triangle mesh into a 3D boolean occupancy grid.

/// Parameters for voxelization.
#[allow(dead_code)]
pub struct VoxelizeV2Params {
    pub resolution: [usize; 3],
    pub min_corner: [f32; 3],
    pub voxel_size: f32,
}

/// A 3D boolean voxel grid.
#[allow(dead_code)]
pub struct VoxelGridV2 {
    pub data: Vec<bool>,
    pub resolution: [usize; 3],
    pub min_corner: [f32; 3],
    pub voxel_size: f32,
}

#[allow(dead_code)]
impl VoxelGridV2 {
    pub fn new(params: &VoxelizeV2Params) -> Self {
        let total = params.resolution[0] * params.resolution[1] * params.resolution[2];
        Self {
            data: vec![false; total],
            resolution: params.resolution,
            min_corner: params.min_corner,
            voxel_size: params.voxel_size,
        }
    }

    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.resolution[0] * (y + self.resolution[1] * z)
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        let i = self.index(x, y, z);
        self.data.get(i).copied().unwrap_or(false)
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, val: bool) {
        let i = self.index(x, y, z);
        if i < self.data.len() {
            self.data[i] = val;
        }
    }

    pub fn occupied_count(&self) -> usize {
        self.data.iter().filter(|&&v| v).count()
    }

    pub fn voxel_center(&self, x: usize, y: usize, z: usize) -> [f32; 3] {
        let s = self.voxel_size;
        let h = s * 0.5;
        [
            self.min_corner[0] + x as f32 * s + h,
            self.min_corner[1] + y as f32 * s + h,
            self.min_corner[2] + z as f32 * s + h,
        ]
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Check if a voxel (given its AABB min-corner) overlaps a triangle using SAT.
fn voxel_triangle_overlap(
    v_min: [f32; 3],
    vox: f32,
    p0: [f32; 3],
    p1: [f32; 3],
    p2: [f32; 3],
) -> bool {
    let v_cen = [
        v_min[0] + vox * 0.5,
        v_min[1] + vox * 0.5,
        v_min[2] + vox * 0.5,
    ];
    let half = vox * 0.5;
    let verts = [sub3(p0, v_cen), sub3(p1, v_cen), sub3(p2, v_cen)];
    let axes_box: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for ax in &axes_box {
        let d0 = dot3(verts[0], *ax);
        let d1 = dot3(verts[1], *ax);
        let d2 = dot3(verts[2], *ax);
        let t_min = d0.min(d1).min(d2);
        let t_max = d0.max(d1).max(d2);
        if t_min > half || t_max < -half {
            return false;
        }
    }
    let e0 = sub3(p1, p0);
    let e1 = sub3(p2, p1);
    let e2 = sub3(p0, p2);
    let tri_normal = cross3(e0, e1);
    let n_dot = dot3(tri_normal, v_cen);
    let rad = half * (tri_normal[0].abs() + tri_normal[1].abs() + tri_normal[2].abs());
    let d_tri = dot3(tri_normal, p0);
    if (d_tri - n_dot).abs() > rad {
        return false;
    }
    let edges = [e0, e1, e2];
    for edge in &edges {
        for ax in &axes_box {
            let sep = cross3(*edge, *ax);
            if sep[0].abs() < 1e-8 && sep[1].abs() < 1e-8 && sep[2].abs() < 1e-8 {
                continue;
            }
            let d0 = dot3(verts[0], sep);
            let d1 = dot3(verts[1], sep);
            let d2 = dot3(verts[2], sep);
            let t_min = d0.min(d1).min(d2);
            let t_max = d0.max(d1).max(d2);
            let r = half * (sep[0].abs() + sep[1].abs() + sep[2].abs());
            if t_min > r || t_max < -r {
                return false;
            }
        }
    }
    true
}

/// Voxelize a triangle mesh surface into a boolean grid.
#[allow(dead_code)]
pub fn voxelize_surface_v2(
    positions: &[[f32; 3]],
    indices: &[u32],
    params: &VoxelizeV2Params,
) -> VoxelGridV2 {
    let mut grid = VoxelGridV2::new(params);
    let s = params.voxel_size;
    let [rx, ry, rz] = params.resolution;
    let n_tri = indices.len() / 3;
    for t in 0..n_tri {
        let p0 = positions[indices[t * 3] as usize];
        let p1 = positions[indices[t * 3 + 1] as usize];
        let p2 = positions[indices[t * 3 + 2] as usize];
        let x_min = p0[0].min(p1[0]).min(p2[0]);
        let x_max = p0[0].max(p1[0]).max(p2[0]);
        let y_min = p0[1].min(p1[1]).min(p2[1]);
        let y_max = p0[1].max(p1[1]).max(p2[1]);
        let z_min = p0[2].min(p1[2]).min(p2[2]);
        let z_max = p0[2].max(p1[2]).max(p2[2]);
        let ix_lo = (((x_min - params.min_corner[0]) / s).floor() as isize).max(0) as usize;
        let ix_hi = (((x_max - params.min_corner[0]) / s).ceil() as usize).min(rx - 1);
        let iy_lo = (((y_min - params.min_corner[1]) / s).floor() as isize).max(0) as usize;
        let iy_hi = (((y_max - params.min_corner[1]) / s).ceil() as usize).min(ry - 1);
        let iz_lo = (((z_min - params.min_corner[2]) / s).floor() as isize).max(0) as usize;
        let iz_hi = (((z_max - params.min_corner[2]) / s).ceil() as usize).min(rz - 1);
        for iz in iz_lo..=iz_hi {
            for iy in iy_lo..=iy_hi {
                for ix in ix_lo..=ix_hi {
                    let v_min = [
                        params.min_corner[0] + ix as f32 * s,
                        params.min_corner[1] + iy as f32 * s,
                        params.min_corner[2] + iz as f32 * s,
                    ];
                    if voxel_triangle_overlap(v_min, s, p0, p1, p2) {
                        grid.set(ix, iy, iz, true);
                    }
                }
            }
        }
    }
    grid
}

/// Total number of voxels in the grid.
#[allow(dead_code)]
pub fn total_voxel_count(grid: &VoxelGridV2) -> usize {
    grid.data.len()
}

/// Build a default VoxelizeV2Params around a bounding box.
#[allow(dead_code)]
pub fn default_voxelize_v2_params(
    min_corner: [f32; 3],
    extent: f32,
    res: usize,
) -> VoxelizeV2Params {
    VoxelizeV2Params {
        resolution: [res; 3],
        min_corner,
        voxel_size: extent / res as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 5, 1, 0, 4, 5, 2, 6, 3, 3, 6, 7, 1, 6, 2, 1, 5,
            6, 0, 3, 7, 0, 7, 4,
        ];
        (positions, indices)
    }

    #[test]
    fn grid_new_empty() {
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 4);
        let grid = VoxelGridV2::new(&params);
        assert_eq!(grid.occupied_count(), 0);
    }

    #[test]
    fn grid_set_get() {
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 4);
        let mut grid = VoxelGridV2::new(&params);
        grid.set(1, 2, 3, true);
        assert!(grid.get(1, 2, 3));
        assert!(!grid.get(0, 0, 0));
    }

    #[test]
    fn voxel_center_correct() {
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 4);
        let grid = VoxelGridV2::new(&params);
        let c = grid.voxel_center(0, 0, 0);
        assert!((c[0] - 0.125).abs() < 1e-5);
    }

    #[test]
    fn total_voxel_count_correct() {
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 4);
        let grid = VoxelGridV2::new(&params);
        assert_eq!(total_voxel_count(&grid), 64);
    }

    #[test]
    fn voxelize_cube_surface_nonempty() {
        let (positions, indices) = unit_cube_mesh();
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 8);
        let grid = voxelize_surface_v2(&positions, &indices, &params);
        assert!(grid.occupied_count() > 0);
    }

    #[test]
    fn voxelize_single_triangle() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices: Vec<u32> = vec![0, 1, 2];
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 4);
        let grid = voxelize_surface_v2(&positions, &indices, &params);
        assert!(grid.occupied_count() > 0);
    }

    #[test]
    fn default_params_resolution() {
        let params = default_voxelize_v2_params([-1.0; 3], 2.0, 10);
        assert_eq!(params.resolution, [10, 10, 10]);
        assert!((params.voxel_size - 0.2).abs() < 1e-5);
    }

    #[test]
    fn voxel_triangle_overlap_touching() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [0.5, 0.0, 0.0];
        let p2 = [0.0, 0.5, 0.0];
        let v_min = [0.0, 0.0, 0.0];
        assert!(voxel_triangle_overlap(v_min, 0.5, p0, p1, p2));
    }

    #[test]
    fn voxel_triangle_overlap_far_away() {
        let p0 = [10.0, 10.0, 10.0];
        let p1 = [11.0, 10.0, 10.0];
        let p2 = [10.0, 11.0, 10.0];
        let v_min = [0.0, 0.0, 0.0];
        assert!(!voxel_triangle_overlap(v_min, 0.5, p0, p1, p2));
    }

    #[test]
    fn index_mapping_consistent() {
        let params = default_voxelize_v2_params([0.0; 3], 1.0, 3);
        let grid = VoxelGridV2::new(&params);
        assert_eq!(grid.index(2, 2, 2), 26);
    }
}
