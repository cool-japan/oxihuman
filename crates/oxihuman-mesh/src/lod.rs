// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! LOD (Level of Detail) generation via grid-based vertex clustering.
//!
//! Algorithm: divide the bounding box into a uniform grid, merge all vertices
//! within each cell to their centroid, remap face indices, remove degenerate
//! faces (where two or more vertices merged to the same cell).

use crate::mesh::MeshBuffers;

/// LOD level defined by grid resolution.
#[derive(Debug, Clone, Copy)]
pub struct LodLevel {
    /// Target ratio of output vertices to input vertices (0.0–1.0).
    /// Used to pick the grid resolution: higher ratio → finer grid.
    pub ratio: f32,
}

impl LodLevel {
    pub fn new(ratio: f32) -> Self {
        LodLevel {
            ratio: ratio.clamp(0.01, 1.0),
        }
    }

    pub const FULL: LodLevel = LodLevel { ratio: 1.0 };
    pub const HALF: LodLevel = LodLevel { ratio: 0.5 };
    pub const QUARTER: LodLevel = LodLevel { ratio: 0.25 };
    pub const EIGHTH: LodLevel = LodLevel { ratio: 0.125 };
}

/// Generate a simplified mesh using grid-based vertex clustering.
///
/// The grid resolution is chosen so the output has approximately
/// `src.vertex_count() * level.ratio` unique cells.
pub fn generate_lod(src: &MeshBuffers, level: LodLevel) -> MeshBuffers {
    if (level.ratio - 1.0).abs() < 1e-6 || src.positions.is_empty() {
        return src.clone();
    }

    let n = src.positions.len();
    // Grid resolution: cube root of (n * ratio) cells per axis
    let target_cells = (n as f32 * level.ratio).max(1.0);
    let grid_res = (target_cells.cbrt().ceil() as usize).max(2);

    // Compute bounding box
    let mut bb_min = src.positions[0];
    let mut bb_max = src.positions[0];
    for p in &src.positions {
        for i in 0..3 {
            if p[i] < bb_min[i] {
                bb_min[i] = p[i];
            }
            if p[i] > bb_max[i] {
                bb_max[i] = p[i];
            }
        }
    }

    // Avoid degenerate bounding box
    let extent = [
        (bb_max[0] - bb_min[0]).max(1e-8),
        (bb_max[1] - bb_min[1]).max(1e-8),
        (bb_max[2] - bb_min[2]).max(1e-8),
    ];

    let gr = grid_res as f32;

    // Map each original vertex to a grid cell index
    let cell_of: Vec<usize> = src
        .positions
        .iter()
        .map(|p| {
            let ix = ((p[0] - bb_min[0]) / extent[0] * gr).min(gr - 1.0) as usize;
            let iy = ((p[1] - bb_min[1]) / extent[1] * gr).min(gr - 1.0) as usize;
            let iz = ((p[2] - bb_min[2]) / extent[2] * gr).min(gr - 1.0) as usize;
            ix + iy * grid_res + iz * grid_res * grid_res
        })
        .collect();

    // Accumulate vertices per cell for centroid computation
    let total_cells = grid_res * grid_res * grid_res;
    let mut cell_sum_pos = vec![[0.0f32; 3]; total_cells];
    let mut cell_sum_norm = vec![[0.0f32; 3]; total_cells];
    let mut cell_sum_uv = vec![[0.0f32; 2]; total_cells];
    let mut cell_count = vec![0u32; total_cells];

    for (i, &cell) in cell_of.iter().enumerate() {
        let p = src.positions[i];
        cell_sum_pos[cell][0] += p[0];
        cell_sum_pos[cell][1] += p[1];
        cell_sum_pos[cell][2] += p[2];
        if i < src.normals.len() {
            let nrm = src.normals[i];
            cell_sum_norm[cell][0] += nrm[0];
            cell_sum_norm[cell][1] += nrm[1];
            cell_sum_norm[cell][2] += nrm[2];
        }
        if i < src.uvs.len() {
            let uv = src.uvs[i];
            cell_sum_uv[cell][0] += uv[0];
            cell_sum_uv[cell][1] += uv[1];
        }
        cell_count[cell] += 1;
    }

    // Build compacted output vertex list: only cells that were occupied
    let mut cell_to_out: Vec<Option<u32>> = vec![None; total_cells];
    let mut out_positions: Vec<[f32; 3]> = Vec::new();
    let mut out_normals: Vec<[f32; 3]> = Vec::new();
    let mut out_uvs: Vec<[f32; 2]> = Vec::new();

    for (cell, &count) in cell_count.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let idx = out_positions.len() as u32;
        cell_to_out[cell] = Some(idx);
        let c = count as f32;
        out_positions.push([
            cell_sum_pos[cell][0] / c,
            cell_sum_pos[cell][1] / c,
            cell_sum_pos[cell][2] / c,
        ]);
        let nl = (cell_sum_norm[cell][0].powi(2)
            + cell_sum_norm[cell][1].powi(2)
            + cell_sum_norm[cell][2].powi(2))
        .sqrt()
        .max(1e-10);
        out_normals.push([
            cell_sum_norm[cell][0] / nl,
            cell_sum_norm[cell][1] / nl,
            cell_sum_norm[cell][2] / nl,
        ]);
        out_uvs.push([cell_sum_uv[cell][0] / c, cell_sum_uv[cell][1] / c]);
    }

    // Remap faces: skip degenerate triangles (where ≥2 verts map to same cell)
    let mut out_indices: Vec<u32> = Vec::new();
    for tri in src.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= n || i1 >= n || i2 >= n {
            continue;
        }
        let c0 = cell_of[i0];
        let c1 = cell_of[i1];
        let c2 = cell_of[i2];
        if c0 == c1 || c1 == c2 || c0 == c2 {
            continue;
        } // degenerate after merge
        if let (Some(o0), Some(o1), Some(o2)) = (cell_to_out[c0], cell_to_out[c1], cell_to_out[c2])
        {
            out_indices.push(o0);
            out_indices.push(o1);
            out_indices.push(o2);
        }
    }

    MeshBuffers {
        positions: out_positions,
        normals: out_normals,
        tangents: Vec::new(), // tangents need recompute after LOD
        uvs: out_uvs,
        indices: out_indices,
        colors: None,
        has_suit: src.has_suit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshBuffers as MyMesh;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn grid_mesh(n: usize) -> MyMesh {
        // n×n grid of vertices, (n-1)×(n-1)×2 triangles
        let side = n;
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        for row in 0..side {
            for col in 0..side {
                positions.push([col as f32 * 0.1, row as f32 * 0.1, 0.0f32]);
                uvs.push([col as f32 / side as f32, row as f32 / side as f32]);
            }
        }
        for row in 0..side - 1 {
            for col in 0..side - 1 {
                let tl = (row * side + col) as u32;
                let tr = tl + 1;
                let bl = tl + side as u32;
                let br = bl + 1;
                indices.extend_from_slice(&[tl, tr, bl, tr, br, bl]);
            }
        }
        MyMesh::from_morph(MB {
            positions,
            normals: vec![[0.0f32, 0.0, 1.0]; side * side],
            uvs,
            indices,
            has_suit: false,
        })
    }

    #[test]
    fn full_lod_unchanged() {
        let m = grid_mesh(10);
        let lod = generate_lod(&m, LodLevel::FULL);
        assert_eq!(lod.positions.len(), m.positions.len());
        assert_eq!(lod.indices.len(), m.indices.len());
    }

    #[test]
    fn half_lod_reduces_vertices() {
        let m = grid_mesh(20); // 400 verts
        let lod = generate_lod(&m, LodLevel::HALF);
        assert!(
            lod.positions.len() < m.positions.len(),
            "LOD should have fewer verts: got {}",
            lod.positions.len()
        );
    }

    #[test]
    fn quarter_lod_fewer_than_half() {
        let m = grid_mesh(20);
        let half = generate_lod(&m, LodLevel::HALF);
        let quarter = generate_lod(&m, LodLevel::QUARTER);
        assert!(
            quarter.positions.len() <= half.positions.len(),
            "quarter ({}) should be <= half ({})",
            quarter.positions.len(),
            half.positions.len()
        );
    }

    #[test]
    fn lod_no_out_of_bounds_indices() {
        let m = grid_mesh(20);
        let lod = generate_lod(&m, LodLevel::HALF);
        let n = lod.positions.len() as u32;
        for &i in &lod.indices {
            assert!(i < n, "index {} out of bounds (n={})", i, n);
        }
    }

    #[test]
    fn lod_centroids_within_bbox() {
        let m = grid_mesh(20);
        let lod = generate_lod(&m, LodLevel::HALF);
        for p in &lod.positions {
            // Original grid is [0, 1.9] in x and y
            assert!(p[0] >= -0.01 && p[0] <= 2.0, "x {} out of range", p[0]);
            assert!(p[1] >= -0.01 && p[1] <= 2.0, "y {} out of range", p[1]);
        }
    }

    #[test]
    fn empty_mesh_returns_empty() {
        let m = MyMesh::from_morph(MB {
            positions: vec![],
            normals: vec![],
            uvs: vec![],
            indices: vec![],
            has_suit: false,
        });
        let lod = generate_lod(&m, LodLevel::HALF);
        assert!(lod.positions.is_empty());
    }
}
