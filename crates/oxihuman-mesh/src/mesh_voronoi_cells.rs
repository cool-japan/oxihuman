#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Voronoi cell decomposition on mesh surface (Lloyd's relaxation stub).

#[allow(dead_code)]
pub struct VoronoiCell {
    pub seed_idx: usize,
    pub vertex_indices: Vec<usize>,
}

#[allow(dead_code)]
pub fn init_voronoi_seeds(n_verts: usize, n_seeds: usize) -> Vec<usize> {
    if n_verts == 0 || n_seeds == 0 {
        return vec![];
    }
    let count = n_seeds.min(n_verts);
    let step = if count > 1 { n_verts / count } else { 1 };
    (0..count).map(|i| (i * step).min(n_verts - 1)).collect()
}

#[allow(dead_code)]
pub fn assign_voronoi_cells(verts: &[[f32; 3]], seeds: &[usize]) -> Vec<VoronoiCell> {
    if seeds.is_empty() || verts.is_empty() {
        return vec![];
    }
    let mut cells: Vec<VoronoiCell> = seeds
        .iter()
        .map(|&s| VoronoiCell { seed_idx: s, vertex_indices: vec![] })
        .collect();
    for (vi, v) in verts.iter().enumerate() {
        let mut best = 0usize;
        let mut best_d = f32::MAX;
        for (ci, &si) in seeds.iter().enumerate() {
            let s = verts[si];
            let d = dist3(*v, s);
            if d < best_d {
                best_d = d;
                best = ci;
            }
        }
        cells[best].vertex_indices.push(vi);
    }
    cells
}

#[allow(dead_code)]
pub fn cell_centroid(cell: &VoronoiCell, verts: &[[f32; 3]]) -> [f32; 3] {
    if cell.vertex_indices.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for &vi in &cell.vertex_indices {
        cx += verts[vi][0];
        cy += verts[vi][1];
        cz += verts[vi][2];
    }
    let n = cell.vertex_indices.len() as f32;
    [cx / n, cy / n, cz / n]
}

#[allow(dead_code)]
pub fn voronoi_area(cell: &VoronoiCell, _verts: &[[f32; 3]]) -> f32 {
    cell.vertex_indices.len() as f32
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn init_seeds_count() {
        let seeds = init_voronoi_seeds(10, 3);
        assert_eq!(seeds.len(), 3);
    }

    #[test]
    fn init_seeds_empty_verts() {
        let seeds = init_voronoi_seeds(0, 3);
        assert!(seeds.is_empty());
    }

    #[test]
    fn init_seeds_more_than_verts() {
        let seeds = init_voronoi_seeds(2, 10);
        assert_eq!(seeds.len(), 2);
    }

    #[test]
    fn assign_empty_seeds() {
        let verts = unit_verts();
        let cells = assign_voronoi_cells(&verts, &[]);
        assert!(cells.is_empty());
    }

    #[test]
    fn assign_single_seed_gets_all_verts() {
        let verts = unit_verts();
        let cells = assign_voronoi_cells(&verts, &[0]);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].vertex_indices.len(), 4);
    }

    #[test]
    fn cell_centroid_single() {
        let verts = vec![[4.0, 2.0, 1.0]];
        let cell = VoronoiCell { seed_idx: 0, vertex_indices: vec![0] };
        let c = cell_centroid(&cell, &verts);
        assert!((c[0] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn cell_centroid_empty() {
        let verts = unit_verts();
        let cell = VoronoiCell { seed_idx: 0, vertex_indices: vec![] };
        let c = cell_centroid(&cell, &verts);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn voronoi_area_returns_vert_count() {
        let verts = unit_verts();
        let cell = VoronoiCell { seed_idx: 0, vertex_indices: vec![0, 1, 2] };
        assert!((voronoi_area(&cell, &verts) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn assign_two_seeds_partitions_verts() {
        let verts = unit_verts();
        let seeds = vec![0, 3];
        let cells = assign_voronoi_cells(&verts, &seeds);
        assert_eq!(cells.len(), 2);
        let total: usize = cells.iter().map(|c| c.vertex_indices.len()).sum();
        assert_eq!(total, 4);
    }
}
