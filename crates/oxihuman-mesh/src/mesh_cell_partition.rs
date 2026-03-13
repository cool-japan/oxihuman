// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Grid-based cell partition for spatial queries on mesh vertices.

/// A spatial cell partition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CellPartition {
    pub cell_size: f32,
    pub min_corner: [f32; 3],
    pub dims: [usize; 3],
    pub cells: Vec<Vec<usize>>,
}

/// Compute grid cell index from position.
#[allow(dead_code)]
pub fn position_to_cell(
    pos: [f32; 3],
    min_corner: [f32; 3],
    cell_size: f32,
    dims: [usize; 3],
) -> Option<usize> {
    let ix = ((pos[0] - min_corner[0]) / cell_size) as isize;
    let iy = ((pos[1] - min_corner[1]) / cell_size) as isize;
    let iz = ((pos[2] - min_corner[2]) / cell_size) as isize;
    if ix < 0 || iy < 0 || iz < 0 {
        return None;
    }
    let (ux, uy, uz) = (ix as usize, iy as usize, iz as usize);
    if ux >= dims[0] || uy >= dims[1] || uz >= dims[2] {
        return None;
    }
    Some(ux + uy * dims[0] + uz * dims[0] * dims[1])
}

/// Build cell partition from positions.
#[allow(dead_code)]
pub fn build_cell_partition(positions: &[[f32; 3]], cell_size: f32) -> CellPartition {
    if positions.is_empty() || cell_size <= 0.0 {
        return CellPartition {
            cell_size: cell_size.max(1.0),
            min_corner: [0.0; 3],
            dims: [0; 3],
            cells: vec![],
        };
    }
    let mut min_c = [f32::MAX; 3];
    let mut max_c = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            min_c[i] = min_c[i].min(p[i]);
            max_c[i] = max_c[i].max(p[i]);
        }
    }
    let dims = [
        ((max_c[0] - min_c[0]) / cell_size).ceil() as usize + 1,
        ((max_c[1] - min_c[1]) / cell_size).ceil() as usize + 1,
        ((max_c[2] - min_c[2]) / cell_size).ceil() as usize + 1,
    ];
    let total = dims[0] * dims[1] * dims[2];
    let mut cells = vec![Vec::new(); total];
    for (vi, p) in positions.iter().enumerate() {
        if let Some(ci) = position_to_cell(*p, min_c, cell_size, dims) {
            cells[ci].push(vi);
        }
    }
    CellPartition {
        cell_size,
        min_corner: min_c,
        dims,
        cells,
    }
}

/// Total number of cells.
#[allow(dead_code)]
pub fn total_cells(cp: &CellPartition) -> usize {
    cp.cells.len()
}

/// Number of non-empty cells.
#[allow(dead_code)]
pub fn occupied_cells(cp: &CellPartition) -> usize {
    cp.cells.iter().filter(|c| !c.is_empty()).count()
}

/// Query all vertex indices in a cell.
#[allow(dead_code)]
pub fn vertices_in_cell(cp: &CellPartition, cell_idx: usize) -> &[usize] {
    cp.cells.get(cell_idx).map_or(&[], |v| v.as_slice())
}

/// Max occupancy across all cells.
#[allow(dead_code)]
pub fn max_cell_occupancy(cp: &CellPartition) -> usize {
    cp.cells.iter().map(|c| c.len()).max().unwrap_or(0)
}

/// Average occupancy of non-empty cells.
#[allow(dead_code)]
pub fn avg_occupancy(cp: &CellPartition) -> f32 {
    let occ = occupied_cells(cp);
    if occ == 0 {
        return 0.0;
    }
    let total: usize = cp.cells.iter().map(|c| c.len()).sum();
    total as f32 / occ as f32
}

/// Export to JSON.
#[allow(dead_code)]
pub fn cell_partition_to_json(cp: &CellPartition) -> String {
    format!(
        "{{\"total_cells\":{},\"occupied\":{},\"cell_size\":{:.4}}}",
        total_cells(cp),
        occupied_cells(cp),
        cp.cell_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_empty() {
        let cp = build_cell_partition(&[], 1.0);
        assert_eq!(total_cells(&cp), 0);
    }

    #[test]
    fn test_build_single() {
        let cp = build_cell_partition(&[[0.0, 0.0, 0.0]], 1.0);
        assert!(occupied_cells(&cp) > 0);
    }

    #[test]
    fn test_position_to_cell_valid() {
        let ci = position_to_cell([0.5, 0.5, 0.5], [0.0; 3], 1.0, [2, 2, 2]);
        assert!(ci.is_some());
    }

    #[test]
    fn test_position_to_cell_oob() {
        let ci = position_to_cell([5.0, 0.0, 0.0], [0.0; 3], 1.0, [2, 2, 2]);
        assert!(ci.is_none());
    }

    #[test]
    fn test_vertices_in_cell() {
        let cp = build_cell_partition(&[[0.0, 0.0, 0.0], [0.1, 0.1, 0.1]], 1.0);
        let total_verts: usize = cp.cells.iter().map(|c| c.len()).sum();
        assert_eq!(total_verts, 2);
    }

    #[test]
    fn test_max_occupancy() {
        let cp = build_cell_partition(&[[0.0; 3], [0.01; 3]], 1.0);
        assert!(max_cell_occupancy(&cp) >= 1);
    }

    #[test]
    fn test_avg_occupancy() {
        let cp = build_cell_partition(&[[0.0; 3]], 1.0);
        assert!(avg_occupancy(&cp) > 0.0);
    }

    #[test]
    fn test_to_json() {
        let cp = build_cell_partition(&[[0.0; 3]], 1.0);
        let j = cell_partition_to_json(&cp);
        assert!(j.contains("\"cell_size\""));
    }

    #[test]
    fn test_negative_cell_size() {
        let cp = build_cell_partition(&[[0.0; 3]], -1.0);
        assert_eq!(total_cells(&cp), 0);
    }

    #[test]
    fn test_multiple_cells() {
        let positions = vec![[0.0; 3], [5.0, 5.0, 5.0]];
        let cp = build_cell_partition(&positions, 1.0);
        assert!(occupied_cells(&cp) >= 2);
    }
}
