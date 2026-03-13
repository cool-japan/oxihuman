// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0 / #![allow(dead_code)]

/// Grid-based spatial sampling of mesh surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GridSample {
    pub resolution: [usize; 3],
    pub origin: [f32; 3],
    pub cell_size: f32,
    pub occupied: Vec<bool>,
}

#[allow(dead_code)]
impl GridSample {
    /// Create a grid from mesh AABB.
    pub fn from_positions(positions: &[[f32; 3]], cell_size: f32) -> Self {
        if positions.is_empty() || cell_size <= 0.0 {
            return Self { resolution: [0, 0, 0], origin: [0.0, 0.0, 0.0], cell_size, occupied: Vec::new() };
        }
        let mut min = positions[0];
        let mut max = positions[0];
        for p in positions {
            for i in 0..3 {
                if p[i] < min[i] { min[i] = p[i]; }
                if p[i] > max[i] { max[i] = p[i]; }
            }
        }
        let res = [
            ((max[0] - min[0]) / cell_size).ceil() as usize + 1,
            ((max[1] - min[1]) / cell_size).ceil() as usize + 1,
            ((max[2] - min[2]) / cell_size).ceil() as usize + 1,
        ];
        let total = res[0] * res[1] * res[2];
        let mut occupied = vec![false; total];
        for p in positions {
            let ix = ((p[0] - min[0]) / cell_size) as usize;
            let iy = ((p[1] - min[1]) / cell_size) as usize;
            let iz = ((p[2] - min[2]) / cell_size) as usize;
            let idx = iz * res[1] * res[0] + iy * res[0] + ix;
            if idx < total {
                occupied[idx] = true;
            }
        }
        Self { resolution: res, origin: min, cell_size, occupied }
    }

    /// Total cells.
    pub fn total_cells(&self) -> usize {
        self.occupied.len()
    }

    /// Count occupied cells.
    pub fn occupied_count(&self) -> usize {
        self.occupied.iter().filter(|&&o| o).count()
    }

    /// Occupancy ratio.
    pub fn occupancy_ratio(&self) -> f32 {
        if self.occupied.is_empty() { return 0.0; }
        self.occupied_count() as f32 / self.total_cells() as f32
    }

    /// Check if a cell is occupied.
    pub fn is_occupied(&self, ix: usize, iy: usize, iz: usize) -> bool {
        let idx = iz * self.resolution[1] * self.resolution[0] + iy * self.resolution[0] + ix;
        idx < self.occupied.len() && self.occupied[idx]
    }

    /// Get cell center position.
    pub fn cell_center(&self, ix: usize, iy: usize, iz: usize) -> [f32; 3] {
        [
            self.origin[0] + (ix as f32 + 0.5) * self.cell_size,
            self.origin[1] + (iy as f32 + 0.5) * self.cell_size,
            self.origin[2] + (iz as f32 + 0.5) * self.cell_size,
        ]
    }
}

/// Sample points on grid cell centers that are occupied.
#[allow(dead_code)]
pub fn sample_occupied_centers(grid: &GridSample) -> Vec<[f32; 3]> {
    let mut centers = Vec::new();
    for iz in 0..grid.resolution[2] {
        for iy in 0..grid.resolution[1] {
            for ix in 0..grid.resolution[0] {
                if grid.is_occupied(ix, iy, iz) {
                    centers.push(grid.cell_center(ix, iy, iz));
                }
            }
        }
    }
    centers
}

/// Serialize grid stats to JSON.
#[allow(dead_code)]
pub fn grid_sample_to_json(grid: &GridSample) -> String {
    format!(
        "{{\"resolution\":[{},{},{}],\"total\":{},\"occupied\":{},\"ratio\":{}}}",
        grid.resolution[0], grid.resolution[1], grid.resolution[2],
        grid.total_cells(), grid.occupied_count(), grid.occupancy_ratio()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0,0.0,0.0],[1.0,0.0,0.0],[1.0,1.0,0.0],[0.0,1.0,0.0],
            [0.0,0.0,1.0],[1.0,0.0,1.0],[1.0,1.0,1.0],[0.0,1.0,1.0],
        ]
    }

    #[test]
    fn test_create_grid() {
        let grid = GridSample::from_positions(&cube_verts(), 0.5);
        assert!(grid.total_cells() > 0);
    }

    #[test]
    fn test_occupied_count() {
        let grid = GridSample::from_positions(&cube_verts(), 0.5);
        assert!(grid.occupied_count() > 0);
    }

    #[test]
    fn test_occupancy_ratio() {
        let grid = GridSample::from_positions(&cube_verts(), 0.5);
        let r = grid.occupancy_ratio();
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn test_cell_center() {
        let grid = GridSample::from_positions(&cube_verts(), 1.0);
        let c = grid.cell_center(0, 0, 0);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_sample_centers() {
        let grid = GridSample::from_positions(&cube_verts(), 0.5);
        let centers = sample_occupied_centers(&grid);
        assert!(!centers.is_empty());
    }

    #[test]
    fn test_empty() {
        let grid = GridSample::from_positions(&[], 1.0);
        assert_eq!(grid.total_cells(), 0);
    }

    #[test]
    fn test_single_point() {
        let grid = GridSample::from_positions(&[[0.0, 0.0, 0.0]], 1.0);
        assert!(grid.occupied_count() > 0);
    }

    #[test]
    fn test_to_json() {
        let grid = GridSample::from_positions(&cube_verts(), 0.5);
        let json = grid_sample_to_json(&grid);
        assert!(json.contains("resolution"));
    }

    #[test]
    fn test_large_cell() {
        let grid = GridSample::from_positions(&cube_verts(), 10.0);
        assert!(grid.total_cells() > 0);
    }

    #[test]
    fn test_is_occupied_oob() {
        let grid = GridSample::from_positions(&cube_verts(), 1.0);
        assert!(!grid.is_occupied(999, 999, 999));
    }
}
