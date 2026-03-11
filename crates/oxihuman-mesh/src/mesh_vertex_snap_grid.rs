#![allow(dead_code)]
//! Grid-based vertex snapping.

/// A snap grid for organizing vertices into cells.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SnapGrid {
    pub cell_size: f32,
    pub cells: std::collections::HashMap<(i32, i32, i32), Vec<usize>>,
}

/// Create a new snap grid with the given cell size.
#[allow(dead_code)]
pub fn new_snap_grid(cell_size: f32) -> SnapGrid {
    SnapGrid {
        cell_size: if cell_size > 0.0 { cell_size } else { 1.0 },
        cells: std::collections::HashMap::new(),
    }
}

/// Snap a vertex to the grid and insert it.
#[allow(dead_code)]
pub fn grid_snap_vertex(grid: &mut SnapGrid, index: usize, pos: [f32; 3]) -> [f32; 3] {
    let cell = grid_cell_index(grid, pos);
    grid.cells.entry(cell).or_default().push(index);
    let cs = grid.cell_size;
    [
        (pos[0] / cs).round() * cs,
        (pos[1] / cs).round() * cs,
        (pos[2] / cs).round() * cs,
    ]
}

/// Get the cell index for a position.
#[allow(dead_code)]
pub fn grid_cell_index(grid: &SnapGrid, pos: [f32; 3]) -> (i32, i32, i32) {
    let cs = grid.cell_size;
    (
        (pos[0] / cs).floor() as i32,
        (pos[1] / cs).floor() as i32,
        (pos[2] / cs).floor() as i32,
    )
}

/// Return the cell size.
#[allow(dead_code)]
pub fn grid_cell_size_sg(grid: &SnapGrid) -> f32 {
    grid.cell_size
}

/// Return grid dimensions as (min_cell, max_cell).
#[allow(dead_code)]
pub fn grid_dimensions_sg(grid: &SnapGrid) -> ((i32, i32, i32), (i32, i32, i32)) {
    if grid.cells.is_empty() {
        return ((0, 0, 0), (0, 0, 0));
    }
    let mut min = (i32::MAX, i32::MAX, i32::MAX);
    let mut max = (i32::MIN, i32::MIN, i32::MIN);
    for k in grid.cells.keys() {
        min.0 = min.0.min(k.0);
        min.1 = min.1.min(k.1);
        min.2 = min.2.min(k.2);
        max.0 = max.0.max(k.0);
        max.1 = max.1.max(k.1);
        max.2 = max.2.max(k.2);
    }
    (min, max)
}

/// Serialize grid info to JSON.
#[allow(dead_code)]
pub fn grid_to_json(grid: &SnapGrid) -> String {
    format!(
        "{{\"cell_size\":{:.4},\"cell_count\":{}}}",
        grid.cell_size,
        grid.cells.len()
    )
}

/// Count vertices in grid.
#[allow(dead_code)]
pub fn grid_vertex_count_sg(grid: &SnapGrid) -> usize {
    grid.cells.values().map(|v| v.len()).sum()
}

/// Clear the grid.
#[allow(dead_code)]
pub fn grid_clear(grid: &mut SnapGrid) {
    grid.cells.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_snap_grid() {
        let g = new_snap_grid(0.5);
        assert!((g.cell_size - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_new_snap_grid_negative() {
        let g = new_snap_grid(-1.0);
        assert!((g.cell_size - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_snap_vertex() {
        let mut g = new_snap_grid(1.0);
        let snapped = grid_snap_vertex(&mut g, 0, [0.3, 0.7, 0.1]);
        assert_eq!(snapped, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_grid_cell_index() {
        let g = new_snap_grid(1.0);
        let cell = grid_cell_index(&g, [1.5, 2.5, 3.5]);
        assert_eq!(cell, (1, 2, 3));
    }

    #[test]
    fn test_grid_cell_size() {
        let g = new_snap_grid(2.0);
        assert!((grid_cell_size_sg(&g) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_grid_dimensions_empty() {
        let g = new_snap_grid(1.0);
        let (min, max) = grid_dimensions_sg(&g);
        assert_eq!(min, (0, 0, 0));
        assert_eq!(max, (0, 0, 0));
    }

    #[test]
    fn test_grid_to_json() {
        let g = new_snap_grid(1.0);
        let j = grid_to_json(&g);
        assert!(j.contains("cell_size"));
    }

    #[test]
    fn test_grid_vertex_count() {
        let mut g = new_snap_grid(1.0);
        grid_snap_vertex(&mut g, 0, [0.0, 0.0, 0.0]);
        grid_snap_vertex(&mut g, 1, [1.0, 0.0, 0.0]);
        assert_eq!(grid_vertex_count_sg(&g), 2);
    }

    #[test]
    fn test_grid_clear() {
        let mut g = new_snap_grid(1.0);
        grid_snap_vertex(&mut g, 0, [0.0, 0.0, 0.0]);
        grid_clear(&mut g);
        assert_eq!(grid_vertex_count_sg(&g), 0);
    }

    #[test]
    fn test_grid_dimensions_nonempty() {
        let mut g = new_snap_grid(1.0);
        grid_snap_vertex(&mut g, 0, [2.0, 3.0, 4.0]);
        let (min, max) = grid_dimensions_sg(&g);
        assert_eq!(min, max);
    }
}
