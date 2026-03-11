// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Build a coarse voxel grid from mesh vertices for spatial queries.

/// A coarse grid cell key.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCell {
    pub ix: i32,
    pub iy: i32,
    pub iz: i32,
}

/// The coarse grid mapping cells to vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CoarseGrid {
    pub cells: std::collections::HashMap<GridCell, Vec<usize>>,
    pub cell_size: f32,
}

/// Map a 3-D position to a grid cell.
#[allow(dead_code)]
pub fn position_to_grid_cell(p: [f32; 3], cell_size: f32) -> GridCell {
    GridCell {
        ix: (p[0] / cell_size).floor() as i32,
        iy: (p[1] / cell_size).floor() as i32,
        iz: (p[2] / cell_size).floor() as i32,
    }
}

/// Build a coarse grid from mesh positions.
#[allow(dead_code)]
pub fn build_coarse_grid(positions: &[[f32; 3]], cell_size: f32) -> CoarseGrid {
    let mut cells: std::collections::HashMap<GridCell, Vec<usize>> =
        std::collections::HashMap::new();
    let cs = cell_size.max(1e-9);
    for (i, &p) in positions.iter().enumerate() {
        let key = position_to_grid_cell(p, cs);
        cells.entry(key).or_default().push(i);
    }
    CoarseGrid {
        cells,
        cell_size: cs,
    }
}

/// Number of occupied cells.
#[allow(dead_code)]
pub fn occupied_cell_count(grid: &CoarseGrid) -> usize {
    grid.cells.len()
}

/// Total vertices indexed in the grid.
#[allow(dead_code)]
pub fn total_indexed_vertices(grid: &CoarseGrid) -> usize {
    grid.cells.values().map(|v| v.len()).sum()
}

/// Get vertices in a specific cell.
#[allow(dead_code)]
pub fn vertices_in_cell_cg(grid: &CoarseGrid, cell: GridCell) -> &[usize] {
    grid.cells.get(&cell).map(|v| v.as_slice()).unwrap_or(&[])
}

/// Find all vertices within a given radius using the coarse grid.
#[allow(dead_code)]
pub fn find_nearby_vertices(
    grid: &CoarseGrid,
    positions: &[[f32; 3]],
    query: [f32; 3],
    radius: f32,
) -> Vec<usize> {
    let r_cells = (radius / grid.cell_size).ceil() as i32 + 1;
    let center_cell = position_to_grid_cell(query, grid.cell_size);
    let r2 = radius * radius;
    let mut result = Vec::new();
    for dx in -r_cells..=r_cells {
        for dy in -r_cells..=r_cells {
            for dz in -r_cells..=r_cells {
                let cell = GridCell {
                    ix: center_cell.ix + dx,
                    iy: center_cell.iy + dy,
                    iz: center_cell.iz + dz,
                };
                for &vi in vertices_in_cell_cg(grid, cell) {
                    if vi < positions.len() {
                        let p = positions[vi];
                        let d2 = (p[0] - query[0]).powi(2)
                            + (p[1] - query[1]).powi(2)
                            + (p[2] - query[2]).powi(2);
                        if d2 <= r2 {
                            result.push(vi);
                        }
                    }
                }
            }
        }
    }
    result.sort_unstable();
    result.dedup();
    result
}

/// Serialize coarse grid stats to JSON.
#[allow(dead_code)]
pub fn coarse_grid_to_json(grid: &CoarseGrid) -> String {
    format!(
        "{{\"cells\":{},\"cell_size\":{:.4}}}",
        occupied_cell_count(grid),
        grid.cell_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn five_points() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        ]
    }

    #[test]
    fn test_position_to_cell() {
        let cell = position_to_grid_cell([0.5, 0.5, 0.5], 1.0);
        assert_eq!(
            cell,
            GridCell {
                ix: 0,
                iy: 0,
                iz: 0
            }
        );
    }

    #[test]
    fn test_build_coarse_grid_occupied() {
        let pts = five_points();
        let grid = build_coarse_grid(&pts, 0.5);
        assert!(occupied_cell_count(&grid) > 0);
    }

    #[test]
    fn test_total_indexed_vertices() {
        let pts = five_points();
        let grid = build_coarse_grid(&pts, 0.5);
        assert_eq!(total_indexed_vertices(&grid), pts.len());
    }

    #[test]
    fn test_find_nearby_vertices() {
        let pts = five_points();
        let grid = build_coarse_grid(&pts, 0.5);
        let near = find_nearby_vertices(&grid, &pts, [0.0, 0.0, 0.0], 0.2);
        assert!(near.contains(&0));
        assert!(near.contains(&1));
    }

    #[test]
    fn test_find_nearby_excludes_distant() {
        let pts = five_points();
        let grid = build_coarse_grid(&pts, 0.5);
        let near = find_nearby_vertices(&grid, &pts, [0.0, 0.0, 0.0], 0.2);
        assert!(!near.contains(&4));
    }

    #[test]
    fn test_empty_grid() {
        let grid = build_coarse_grid(&[], 1.0);
        assert_eq!(occupied_cell_count(&grid), 0);
    }

    #[test]
    fn test_coarse_grid_to_json() {
        let pts = five_points();
        let grid = build_coarse_grid(&pts, 1.0);
        let j = coarse_grid_to_json(&grid);
        assert!(j.contains("cell_size"));
    }

    #[test]
    fn test_vertices_in_cell_empty() {
        let grid = build_coarse_grid(&[], 1.0);
        let v = vertices_in_cell_cg(
            &grid,
            GridCell {
                ix: 0,
                iy: 0,
                iz: 0,
            },
        );
        assert!(v.is_empty());
    }

    #[test]
    fn test_negative_coords() {
        let pts = vec![[-1.0, -1.0, -1.0]];
        let grid = build_coarse_grid(&pts, 1.0);
        assert_eq!(occupied_cell_count(&grid), 1);
    }

    #[test]
    fn test_find_nearby_empty() {
        let grid = build_coarse_grid(&[], 1.0);
        let near = find_nearby_vertices(&grid, &[], [0.0, 0.0, 0.0], 1.0);
        assert!(near.is_empty());
    }
}
