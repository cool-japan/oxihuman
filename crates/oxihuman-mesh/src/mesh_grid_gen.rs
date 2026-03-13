// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct GridParams3d {
    pub nx: u32,
    pub ny: u32,
    pub nz: u32,
    pub spacing: f32,
}

pub fn new_grid_params_3d(nx: u32, ny: u32, nz: u32, spacing: f32) -> GridParams3d {
    GridParams3d {
        nx,
        ny,
        nz,
        spacing,
    }
}

pub fn grid_vertex(p: &GridParams3d, ix: u32, iy: u32, iz: u32) -> [f32; 3] {
    [
        ix as f32 * p.spacing,
        iy as f32 * p.spacing,
        iz as f32 * p.spacing,
    ]
}

pub fn grid_vertex_count(p: &GridParams3d) -> usize {
    ((p.nx + 1) * (p.ny + 1) * (p.nz + 1)) as usize
}

pub fn grid_edge_count(p: &GridParams3d) -> usize {
    let nx = p.nx as usize;
    let ny = p.ny as usize;
    let nz = p.nz as usize;
    nx * (ny + 1) * (nz + 1) + (nx + 1) * ny * (nz + 1) + (nx + 1) * (ny + 1) * nz
}

pub fn grid_cell_count(p: &GridParams3d) -> usize {
    (p.nx * p.ny * p.nz) as usize
}

pub fn grid_bounds(p: &GridParams3d) -> ([f32; 3], [f32; 3]) {
    (
        [0.0, 0.0, 0.0],
        [
            p.nx as f32 * p.spacing,
            p.ny as f32 * p.spacing,
            p.nz as f32 * p.spacing,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        /* construction */
        let g = new_grid_params_3d(4, 4, 4, 1.0);
        assert_eq!(g.nx, 4);
    }

    #[test]
    fn test_grid_vertex_count() {
        /* (nx+1)^3 for uniform grid */
        let g = new_grid_params_3d(3, 3, 3, 1.0);
        assert_eq!(grid_vertex_count(&g), 64);
    }

    #[test]
    fn test_grid_cell_count() {
        /* nx*ny*nz */
        let g = new_grid_params_3d(2, 3, 4, 1.0);
        assert_eq!(grid_cell_count(&g), 24);
    }

    #[test]
    fn test_grid_vertex_position() {
        /* vertex at (1,2,3) with spacing=2 */
        let g = new_grid_params_3d(4, 4, 4, 2.0);
        let v = grid_vertex(&g, 1, 2, 3);
        assert!((v[0] - 2.0).abs() < 1e-5);
        assert!((v[1] - 4.0).abs() < 1e-5);
        assert!((v[2] - 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_grid_bounds() {
        /* bounds */
        let g = new_grid_params_3d(4, 4, 4, 1.0);
        let (min, max) = grid_bounds(&g);
        assert!((max[0] - 4.0).abs() < 1e-5);
        let _ = min;
    }

    #[test]
    fn test_grid_edge_count_positive() {
        /* positive */
        let g = new_grid_params_3d(2, 2, 2, 1.0);
        assert!(grid_edge_count(&g) > 0);
    }
}
