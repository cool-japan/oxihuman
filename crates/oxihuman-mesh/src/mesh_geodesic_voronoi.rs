// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Geodesic Voronoi partitioning on a triangle mesh.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A Voronoi cell on the mesh surface.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoVoronoiCell {
    pub seed: usize,
    pub vertices: Vec<usize>,
}

/// Result of geodesic Voronoi computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoVoronoiResult {
    pub labels: Vec<usize>,
    pub distances: Vec<f32>,
    pub cells: Vec<GeoVoronoiCell>,
}

#[derive(Debug)]
struct State {
    cost: u32,
    vertex: usize,
}

impl Eq for State {}
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Build adjacency from triangle indices.
#[allow(dead_code)]
pub fn build_adjacency(vertex_count: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let vs = [
            indices[t * 3] as usize,
            indices[t * 3 + 1] as usize,
            indices[t * 3 + 2] as usize,
        ];
        for k in 0..3 {
            let a = vs[k];
            let b = vs[(k + 1) % 3];
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }
    adj
}

/// Edge weight (Euclidean distance).
#[allow(dead_code)]
fn edge_weight(positions: &[[f32; 3]], a: usize, b: usize) -> f32 {
    let pa = positions[a];
    let pb = positions[b];
    let dx = pa[0] - pb[0];
    let dy = pa[1] - pb[1];
    let dz = pa[2] - pb[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Compute geodesic Voronoi regions from multiple seed vertices.
#[allow(dead_code)]
pub fn compute_geodesic_voronoi(
    positions: &[[f32; 3]],
    indices: &[u32],
    seeds: &[usize],
) -> GeoVoronoiResult {
    let n = positions.len();
    if n == 0 || seeds.is_empty() {
        return GeoVoronoiResult {
            labels: vec![],
            distances: vec![],
            cells: vec![],
        };
    }
    let adj = build_adjacency(n, indices);
    let mut dist = vec![f32::MAX; n];
    let mut labels = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();

    for (si, &seed) in seeds.iter().enumerate() {
        if seed < n {
            dist[seed] = 0.0;
            labels[seed] = si;
            heap.push(State {
                cost: 0,
                vertex: seed,
            });
        }
    }

    while let Some(State { cost, vertex }) = heap.pop() {
        let d = cost as f32 * 1e-6;
        if d > dist[vertex] + 1e-6 {
            continue;
        }
        for &nb in &adj[vertex] {
            let new_d = dist[vertex] + edge_weight(positions, vertex, nb);
            if new_d < dist[nb] {
                dist[nb] = new_d;
                labels[nb] = labels[vertex];
                heap.push(State {
                    cost: (new_d * 1e6) as u32,
                    vertex: nb,
                });
            }
        }
    }

    let mut cells: Vec<GeoVoronoiCell> = seeds
        .iter()
        .map(|&s| GeoVoronoiCell {
            seed: s,
            vertices: Vec::new(),
        })
        .collect();
    for (vi, &label) in labels.iter().enumerate() {
        if label < cells.len() {
            cells[label].vertices.push(vi);
        }
    }

    GeoVoronoiResult {
        labels,
        distances: dist,
        cells,
    }
}

/// Number of Voronoi cells.
#[allow(dead_code)]
pub fn voronoi_cell_count(result: &GeoVoronoiResult) -> usize {
    result.cells.len()
}

/// Get label for a vertex.
#[allow(dead_code)]
pub fn vertex_label(result: &GeoVoronoiResult, vertex: usize) -> Option<usize> {
    result
        .labels
        .get(vertex)
        .copied()
        .filter(|&l| l != usize::MAX)
}

/// Largest cell vertex count.
#[allow(dead_code)]
pub fn largest_cell_size(result: &GeoVoronoiResult) -> usize {
    result
        .cells
        .iter()
        .map(|c| c.vertices.len())
        .max()
        .unwrap_or(0)
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn voronoi_result_to_json(result: &GeoVoronoiResult) -> String {
    format!("{{\"cell_count\":{}}}", result.cells.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 3, 1, 2, 4, 1, 4, 3];
        (pos, idx)
    }

    #[test]
    fn test_build_adjacency() {
        let (pos, idx) = line_mesh();
        let adj = build_adjacency(pos.len(), &idx);
        assert!(!adj[0].is_empty());
    }

    #[test]
    fn test_single_seed() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0]);
        assert_eq!(voronoi_cell_count(&result), 1);
    }

    #[test]
    fn test_two_seeds() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0, 2]);
        assert_eq!(voronoi_cell_count(&result), 2);
    }

    #[test]
    fn test_vertex_label() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0]);
        assert_eq!(vertex_label(&result, 0), Some(0));
    }

    #[test]
    fn test_largest_cell() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0]);
        assert_eq!(largest_cell_size(&result), pos.len());
    }

    #[test]
    fn test_empty_mesh() {
        let result = compute_geodesic_voronoi(&[], &[], &[]);
        assert_eq!(voronoi_cell_count(&result), 0);
    }

    #[test]
    fn test_empty_seeds() {
        let pos = vec![[0.0; 3]];
        let result = compute_geodesic_voronoi(&pos, &[], &[]);
        assert_eq!(voronoi_cell_count(&result), 0);
    }

    #[test]
    fn test_voronoi_result_to_json() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0, 2]);
        let json = voronoi_result_to_json(&result);
        assert!(json.contains("\"cell_count\":2"));
    }

    #[test]
    fn test_seed_distance_zero() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0]);
        assert!((result.distances[0]).abs() < 1e-6);
    }

    #[test]
    fn test_all_vertices_labeled() {
        let (pos, idx) = line_mesh();
        let result = compute_geodesic_voronoi(&pos, &idx, &[0]);
        for vi in 0..pos.len() {
            assert!(vertex_label(&result, vi).is_some());
        }
    }
}
