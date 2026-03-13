// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! LightCluster — clustered forward lighting acceleration structure.

#![allow(dead_code)]

/// A single cluster cell containing indices of lights within it.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LightCluster {
    pub light_indices: Vec<u32>,
}

/// 3-D grid of light clusters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClusterGrid {
    pub nx: u32,
    pub ny: u32,
    pub nz: u32,
    pub clusters: Vec<LightCluster>,
}

/// Create a zeroed `ClusterGrid` with the given tile counts.
#[allow(dead_code)]
pub fn new_cluster_grid(nx: u32, ny: u32, nz: u32) -> ClusterGrid {
    let n = (nx * ny * nz) as usize;
    ClusterGrid { nx, ny, nz, clusters: vec![LightCluster::default(); n] }
}

/// Assign a light index to all clusters that overlap its bounding sphere.
/// `center` is in normalised cluster-grid space [0,1]^3, `radius` in grid units.
#[allow(dead_code)]
pub fn assign_light_to_clusters(grid: &mut ClusterGrid, light_idx: u32, cx: f32, cy: f32, cz: f32, radius: f32) {
    let ix_lo = ((cx - radius) * grid.nx as f32).floor().max(0.0) as u32;
    let iy_lo = ((cy - radius) * grid.ny as f32).floor().max(0.0) as u32;
    let iz_lo = ((cz - radius) * grid.nz as f32).floor().max(0.0) as u32;
    let ix_hi = ((cx + radius) * grid.nx as f32).ceil().min(grid.nx as f32) as u32;
    let iy_hi = ((cy + radius) * grid.ny as f32).ceil().min(grid.ny as f32) as u32;
    let iz_hi = ((cz + radius) * grid.nz as f32).ceil().min(grid.nz as f32) as u32;
    for iz in iz_lo..iz_hi {
        for iy in iy_lo..iy_hi {
            for ix in ix_lo..ix_hi {
                let flat = (iz * grid.ny * grid.nx + iy * grid.nx + ix) as usize;
                if flat < grid.clusters.len() {
                    grid.clusters[flat].light_indices.push(light_idx);
                }
            }
        }
    }
}

/// Return a list of cluster flat indices that cover pixel (px, py) in viewport.
#[allow(dead_code)]
pub fn clusters_at_pixel(grid: &ClusterGrid, px: u32, py: u32, viewport_w: u32, viewport_h: u32) -> Vec<usize> {
    let ix = ((px as f32 / viewport_w.max(1) as f32) * grid.nx as f32) as u32;
    let iy = ((py as f32 / viewport_h.max(1) as f32) * grid.ny as f32) as u32;
    (0..grid.nz)
        .map(|iz| (iz * grid.ny * grid.nx + iy * grid.nx + ix) as usize)
        .filter(|&idx| idx < grid.clusters.len())
        .collect()
}

/// Return the number of lights in cluster at flat index.
#[allow(dead_code)]
pub fn cluster_light_count(grid: &ClusterGrid, flat_index: usize) -> usize {
    grid.clusters.get(flat_index).map(|c| c.light_indices.len()).unwrap_or(0)
}

/// Return the flat cluster index for a 3-D tile coordinate.
#[allow(dead_code)]
pub fn cluster_index_3d(grid: &ClusterGrid, x: u32, y: u32, z: u32) -> usize {
    (z * grid.ny * grid.nx + y * grid.nx + x) as usize
}

/// Return the total number of clusters in the grid.
#[allow(dead_code)]
pub fn cluster_count(grid: &ClusterGrid) -> usize {
    grid.clusters.len()
}

/// Return (nx, ny, nz) dimensions.
#[allow(dead_code)]
pub fn cluster_dimensions(grid: &ClusterGrid) -> (u32, u32, u32) {
    (grid.nx, grid.ny, grid.nz)
}

/// Return the light indices list for cluster at flat index.
#[allow(dead_code)]
pub fn cluster_light_list(grid: &ClusterGrid, flat_index: usize) -> &[u32] {
    grid.clusters.get(flat_index).map(|c| c.light_indices.as_slice()).unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cluster_grid_count() {
        let g = new_cluster_grid(4, 4, 4);
        assert_eq!(cluster_count(&g), 64);
    }

    #[test]
    fn test_cluster_dimensions() {
        let g = new_cluster_grid(8, 6, 4);
        assert_eq!(cluster_dimensions(&g), (8, 6, 4));
    }

    #[test]
    fn test_cluster_index_3d() {
        let g = new_cluster_grid(4, 4, 4);
        assert_eq!(cluster_index_3d(&g, 0, 0, 0), 0);
        assert_eq!(cluster_index_3d(&g, 1, 0, 0), 1);
    }

    #[test]
    fn test_cluster_light_count_empty() {
        let g = new_cluster_grid(2, 2, 2);
        assert_eq!(cluster_light_count(&g, 0), 0);
    }

    #[test]
    fn test_assign_light_to_clusters() {
        let mut g = new_cluster_grid(4, 4, 4);
        assign_light_to_clusters(&mut g, 0, 0.5, 0.5, 0.5, 0.3);
        let total_assignments: usize = g.clusters.iter().map(|c| c.light_indices.len()).sum();
        assert!(total_assignments > 0);
    }

    #[test]
    fn test_cluster_light_list() {
        let g = new_cluster_grid(2, 2, 2);
        assert!(cluster_light_list(&g, 0).is_empty());
    }

    #[test]
    fn test_clusters_at_pixel() {
        let g = new_cluster_grid(4, 4, 4);
        let indices = clusters_at_pixel(&g, 0, 0, 100, 100);
        assert!(!indices.is_empty());
    }

    #[test]
    fn test_cluster_light_count_oob() {
        let g = new_cluster_grid(1, 1, 1);
        assert_eq!(cluster_light_count(&g, 999), 0);
    }
}
