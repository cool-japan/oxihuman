//! Structured grid mesh generation (extended).
#![allow(dead_code)]

/// A structured grid mesh.
#[allow(dead_code)]
pub struct GridMesh2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
    pub rows: usize,
    pub cols: usize,
}

/// Generate a flat grid mesh.
#[allow(dead_code)]
pub fn gen_grid2(rows: usize, cols: usize, width: f32, height: f32) -> GridMesh2 {
    let rows = rows.max(1);
    let cols = cols.max(1);
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    for r in 0..=rows {
        for c in 0..=cols {
            let x = (c as f32 / cols as f32) * width - width * 0.5;
            let y = (r as f32 / rows as f32) * height - height * 0.5;
            positions.push([x, y, 0.0]);
            uvs.push([c as f32 / cols as f32, r as f32 / rows as f32]);
        }
    }
    let stride = cols + 1;
    for r in 0..rows {
        for c in 0..cols {
            let i00 = (r * stride + c) as u32;
            let i10 = (r * stride + c + 1) as u32;
            let i01 = ((r+1) * stride + c) as u32;
            let i11 = ((r+1) * stride + c + 1) as u32;
            indices.extend_from_slice(&[i00, i10, i11, i00, i11, i01]);
        }
    }
    GridMesh2 { positions, indices, uvs, rows, cols }
}

/// Generate UV coordinates for the grid.
#[allow(dead_code)]
pub fn gen_grid2_uv(rows: usize, cols: usize) -> Vec<[f32; 2]> {
    let rows = rows.max(1); let cols = cols.max(1);
    let mut uvs = Vec::new();
    for r in 0..=rows {
        for c in 0..=cols {
            uvs.push([c as f32 / cols as f32, r as f32 / rows as f32]);
        }
    }
    uvs
}

/// Get the vertex position at grid coordinate (r, c).
#[allow(dead_code)]
pub fn grid2_vertex_at(grid: &GridMesh2, r: usize, c: usize) -> Option<[f32; 3]> {
    let idx = r * (grid.cols + 1) + c;
    grid.positions.get(idx).copied()
}

/// Get the number of faces in the grid.
#[allow(dead_code)]
pub fn grid2_face_count(rows: usize, cols: usize) -> usize {
    rows.max(1) * cols.max(1) * 2
}

/// Get the vertex count for the grid.
#[allow(dead_code)]
pub fn grid2_vertex_count(rows: usize, cols: usize) -> usize {
    (rows.max(1) + 1) * (cols.max(1) + 1)
}

/// Convert grid to a flat triangle list (already in indices).
#[allow(dead_code)]
pub fn grid2_to_triangles(grid: &GridMesh2) -> Vec<[f32; 3]> {
    let mut tris = Vec::new();
    let tris_count = grid.indices.len() / 3;
    for t in 0..tris_count {
        let i0 = grid.indices[t*3] as usize;
        let i1 = grid.indices[t*3+1] as usize;
        let i2 = grid.indices[t*3+2] as usize;
        if i0 < grid.positions.len() && i1 < grid.positions.len() && i2 < grid.positions.len() {
            tris.extend_from_slice(&[grid.positions[i0], grid.positions[i1], grid.positions[i2]]);
        }
    }
    tris
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_grid2_vertex_count() {
        let g = gen_grid2(3, 4, 1.0, 1.0);
        assert_eq!(g.positions.len(), grid2_vertex_count(3, 4));
    }

    #[test]
    fn test_gen_grid2_face_count() {
        let g = gen_grid2(2, 3, 1.0, 1.0);
        assert_eq!(g.indices.len() / 3, grid2_face_count(2, 3));
    }

    #[test]
    fn test_gen_grid2_uv_count() {
        let uvs = gen_grid2_uv(2, 3);
        assert_eq!(uvs.len(), grid2_vertex_count(2, 3));
    }

    #[test]
    fn test_grid2_vertex_at_valid() {
        let g = gen_grid2(3, 4, 2.0, 2.0);
        let v = grid2_vertex_at(&g, 0, 0);
        assert!(v.is_some());
    }

    #[test]
    fn test_grid2_vertex_at_oob() {
        let g = gen_grid2(2, 2, 1.0, 1.0);
        let v = grid2_vertex_at(&g, 100, 100);
        assert!(v.is_none());
    }

    #[test]
    fn test_grid2_face_count_formula() {
        assert_eq!(grid2_face_count(4, 5), 40);
    }

    #[test]
    fn test_grid2_vertex_count_formula() {
        assert_eq!(grid2_vertex_count(3, 4), 20);
    }

    #[test]
    fn test_grid2_to_triangles_count() {
        let g = gen_grid2(2, 2, 1.0, 1.0);
        let tris = grid2_to_triangles(&g);
        assert_eq!(tris.len(), g.indices.len());
    }

    #[test]
    fn test_gen_grid2_uv_range() {
        let uvs = gen_grid2_uv(2, 2);
        for uv in &uvs {
            assert!((0.0..=1.0).contains(&uv[0]));
            assert!((0.0..=1.0).contains(&uv[1]));
        }
    }
}
