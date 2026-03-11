//! Volume density / voxel grid export.
#![allow(dead_code)]

/// A 3D voxel grid for density export.
#[allow(dead_code)]
pub struct VoxelGrid2 {
    pub density: Vec<f32>,
    pub dims: [usize; 3],
}

/// Volume density export wrapper.
#[allow(dead_code)]
pub struct VolumeDensityExport2 {
    pub grid: VoxelGrid2,
}

/// Create a new voxel grid.
#[allow(dead_code)]
pub fn new_voxel_grid2(dims: [usize; 3]) -> VoxelGrid2 {
    let total = dims[0] * dims[1] * dims[2];
    VoxelGrid2 { density: vec![0.0; total], dims }
}

fn idx3(dims: [usize;3], x: usize, y: usize, z: usize) -> Option<usize> {
    if x >= dims[0] || y >= dims[1] || z >= dims[2] { None }
    else { Some(x + y * dims[0] + z * dims[0] * dims[1]) }
}

/// Set density at (x, y, z).
#[allow(dead_code)]
pub fn set_density2(grid: &mut VoxelGrid2, x: usize, y: usize, z: usize, v: f32) {
    if let Some(i) = idx3(grid.dims, x, y, z) { grid.density[i] = v.max(0.0); }
}

/// Get density at (x, y, z).
#[allow(dead_code)]
pub fn get_density2(grid: &VoxelGrid2, x: usize, y: usize, z: usize) -> f32 {
    idx3(grid.dims, x, y, z).map(|i| grid.density[i]).unwrap_or(0.0)
}

/// Export volume as raw bytes (f32 LE).
#[allow(dead_code)]
pub fn export_volume2_raw(grid: &VoxelGrid2) -> Vec<u8> {
    grid.density.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// Get total voxel count.
#[allow(dead_code)]
pub fn voxel2_count(grid: &VoxelGrid2) -> usize { grid.density.len() }

/// Get min density.
#[allow(dead_code)]
pub fn density2_min(grid: &VoxelGrid2) -> f32 {
    grid.density.iter().copied().fold(f32::MAX, f32::min)
}

/// Get max density.
#[allow(dead_code)]
pub fn density2_max(grid: &VoxelGrid2) -> f32 {
    grid.density.iter().copied().fold(f32::MIN, f32::max)
}

/// Get grid dimensions.
#[allow(dead_code)]
pub fn grid2_dimensions(grid: &VoxelGrid2) -> [usize; 3] { grid.dims }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_voxel_grid_size() {
        let g = new_voxel_grid2([4, 4, 4]);
        assert_eq!(voxel2_count(&g), 64);
    }

    #[test]
    fn test_set_get_density() {
        let mut g = new_voxel_grid2([4, 4, 4]);
        set_density2(&mut g, 1, 2, 3, 0.75);
        assert!((get_density2(&g, 1, 2, 3) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_get_density_oob() {
        let g = new_voxel_grid2([4, 4, 4]);
        assert!((get_density2(&g, 100, 0, 0)).abs() < 1e-5);
    }

    #[test]
    fn test_export_raw_bytes() {
        let g = new_voxel_grid2([2, 2, 2]);
        let b = export_volume2_raw(&g);
        assert_eq!(b.len(), 8 * 4);
    }

    #[test]
    fn test_density_min_zero() {
        let g = new_voxel_grid2([2, 2, 1]);
        assert!((density2_min(&g)).abs() < 1e-5);
    }

    #[test]
    fn test_density_max_after_set() {
        let mut g = new_voxel_grid2([2, 2, 1]);
        set_density2(&mut g, 0, 0, 0, 1.0);
        assert!((density2_max(&g) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_grid_dimensions() {
        let g = new_voxel_grid2([3, 4, 5]);
        let d = grid2_dimensions(&g);
        assert_eq!(d, [3, 4, 5]);
    }

    #[test]
    fn test_volume_density_export2_struct() {
        let g = new_voxel_grid2([2, 2, 2]);
        let ve = VolumeDensityExport2 { grid: g };
        assert_eq!(ve.grid.dims[0], 2);
    }

    #[test]
    fn test_set_density_clamped_neg() {
        let mut g = new_voxel_grid2([2, 2, 1]);
        set_density2(&mut g, 0, 0, 0, -1.0);
        assert!((get_density2(&g, 0, 0, 0)).abs() < 1e-5);
    }
}
