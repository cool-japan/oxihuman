// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Geodesic path tracing on triangle meshes using Dijkstra's algorithm.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

// ── types ─────────────────────────────────────────────────────────────────────

/// A geodesic path as a sequence of vertex indices with total length.
#[derive(Debug, Clone)]
pub struct GeodesicPath {
    /// Ordered vertex indices from source to destination.
    pub vertices: Vec<usize>,
    /// Sum of edge lengths along the path.
    pub total_length: f32,
}

/// Heap entry for Dijkstra (negated distance for max-heap).
#[derive(Clone, Copy, PartialEq)]
struct HeapEntry {
    dist: f32,
    vertex: usize,
}

impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: smaller distance has higher priority.
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn edge_len(positions: &[[f32; 3]], a: usize, b: usize) -> f32 {
    let dx = positions[a][0] - positions[b][0];
    let dy = positions[a][1] - positions[b][1];
    let dz = positions[a][2] - positions[b][2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Build an adjacency list from triangle soup.
fn build_adjacency(n_verts: usize, tris: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    for tri in tris {
        let v = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for i in 0..3 {
            let a = v[i];
            let b = v[(i + 1) % 3];
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

// ── public API ────────────────────────────────────────────────────────────────

/// Compute geodesic distances from `source` to all vertices using Dijkstra.
#[allow(dead_code)]
pub fn dijkstra_geodesic(positions: &[[f32; 3]], tris: &[[u32; 3]], source: usize) -> Vec<f32> {
    let n = positions.len();
    let adj = build_adjacency(n, tris);
    let mut dist = vec![f32::INFINITY; n];
    dist[source] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(HeapEntry {
        dist: 0.0,
        vertex: source,
    });

    while let Some(HeapEntry { dist: d, vertex: u }) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &v in &adj[u] {
            let nd = d + edge_len(positions, u, v);
            if nd < dist[v] {
                dist[v] = nd;
                heap.push(HeapEntry {
                    dist: nd,
                    vertex: v,
                });
            }
        }
    }
    dist
}

/// Compute the shortest geodesic path between `src` and `dst`.
#[allow(dead_code)]
pub fn dijkstra_geodesic_between(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    src: usize,
    dst: usize,
) -> Option<GeodesicPath> {
    let n = positions.len();
    let adj = build_adjacency(n, tris);
    let mut dist = vec![f32::INFINITY; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    dist[src] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(HeapEntry {
        dist: 0.0,
        vertex: src,
    });

    while let Some(HeapEntry { dist: d, vertex: u }) = heap.pop() {
        if u == dst {
            break;
        }
        if d > dist[u] {
            continue;
        }
        for &v in &adj[u] {
            let nd = d + edge_len(positions, u, v);
            if nd < dist[v] {
                dist[v] = nd;
                prev[v] = Some(u);
                heap.push(HeapEntry {
                    dist: nd,
                    vertex: v,
                });
            }
        }
    }

    if dist[dst].is_infinite() {
        return None;
    }
    let vertices = reconstruct_path(&prev, dst);
    Some(GeodesicPath {
        total_length: dist[dst],
        vertices,
    })
}

/// Reconstruct path from predecessor array.
#[allow(dead_code)]
pub fn reconstruct_path(prev: &[Option<usize>], dst: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut cur = dst;
    loop {
        path.push(cur);
        match prev[cur] {
            Some(p) => cur = p,
            None => break,
        }
    }
    path.reverse();
    path
}

/// Scalar geodesic distance between two vertices.
#[allow(dead_code)]
pub fn geodesic_distance(positions: &[[f32; 3]], tris: &[[u32; 3]], src: usize, dst: usize) -> f32 {
    let dists = dijkstra_geodesic(positions, tris, src);
    dists[dst]
}

/// Return the vertex that minimises the sum of geodesic distances to all others.
#[allow(dead_code)]
pub fn geodesic_centroid(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> usize {
    let n = positions.len();
    let mut best = 0usize;
    let mut best_sum = f32::INFINITY;
    for v in 0..n {
        let d = dijkstra_geodesic(positions, tris, v);
        let s: f32 = d.iter().filter(|x| x.is_finite()).sum();
        if s < best_sum {
            best_sum = s;
            best = v;
        }
    }
    best
}

/// Farthest point sampling: greedily select `k` vertices maximising minimum
/// geodesic distance to the existing sample set.
#[allow(dead_code)]
pub fn farthest_point_sampling(positions: &[[f32; 3]], tris: &[[u32; 3]], k: usize) -> Vec<usize> {
    let n = positions.len();
    if k == 0 || n == 0 {
        return Vec::new();
    }
    let k = k.min(n);
    let mut samples = vec![0usize];
    let mut min_dist = dijkstra_geodesic(positions, tris, 0);

    while samples.len() < k {
        // Select vertex with maximum min-distance to current sample set.
        let next = min_dist
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        samples.push(next);
        if samples.len() == k {
            break;
        }
        // Update min distances.
        let d = dijkstra_geodesic(positions, tris, next);
        for (i, di) in d.iter().enumerate() {
            if *di < min_dist[i] {
                min_dist[i] = *di;
            }
        }
    }
    samples
}

/// Assign each vertex to the nearest seed by geodesic distance.
#[allow(dead_code)]
pub fn geodesic_voronoi(positions: &[[f32; 3]], tris: &[[u32; 3]], seeds: &[usize]) -> Vec<usize> {
    let n = positions.len();
    // Multi-source Dijkstra.
    let adj = build_adjacency(n, tris);
    let mut dist = vec![f32::INFINITY; n];
    let mut region = vec![0usize; n];
    let mut heap = BinaryHeap::new();

    for &s in seeds {
        dist[s] = 0.0;
        region[s] = s;
        heap.push(HeapEntry {
            dist: 0.0,
            vertex: s,
        });
    }

    while let Some(HeapEntry { dist: d, vertex: u }) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &v in &adj[u] {
            let nd = d + edge_len(positions, u, v);
            if nd < dist[v] {
                dist[v] = nd;
                region[v] = region[u];
                heap.push(HeapEntry {
                    dist: nd,
                    vertex: v,
                });
            }
        }
    }
    // Vertices unreachable from any seed get assigned to seed 0 as fallback.
    if !seeds.is_empty() {
        for i in 0..n {
            if dist[i].is_infinite() {
                region[i] = seeds[0];
            }
        }
    }
    region
}

/// Maximum geodesic distance from vertex `v` to any other vertex.
#[allow(dead_code)]
pub fn compute_eccentricity(positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> f32 {
    dijkstra_geodesic(positions, tris, v)
        .iter()
        .copied()
        .filter(|d| d.is_finite())
        .fold(0.0f32, f32::max)
}

/// Mesh geodesic diameter: maximum eccentricity over all vertices.
#[allow(dead_code)]
pub fn mesh_diameter_geodesic(positions: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    (0..positions.len())
        .map(|v| compute_eccentricity(positions, tris, v))
        .fold(0.0f32, f32::max)
}

// ── vertex→face adjacency (unused outside tests but kept for future use) ──────

#[allow(dead_code)]
fn vertex_to_faces(n_verts: usize, tris: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let mut vtf: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    for (fi, tri) in tris.iter().enumerate() {
        for &vi in tri {
            vtf[vi as usize].push(fi);
        }
    }
    vtf
}

// ── cache of precomputed Dijkstra for tests ───────────────────────────────────

#[allow(dead_code)]
fn build_edge_map(tris: &[[u32; 3]]) -> HashMap<(usize, usize), f32> {
    let mut m = HashMap::new();
    for tri in tris {
        let v = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        for i in 0..3 {
            let a = v[i];
            let b = v[(i + 1) % 3];
            m.entry((a.min(b), a.max(b))).or_insert(0.0f32);
        }
    }
    m
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_strip() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        // 4 vertices, 2 triangles: 0-1-2, 1-3-2
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 2], [1, 3, 2]];
        (pos, tris)
    }

    fn line_mesh() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        // 5 vertices in a line, connected as a triangle strip.
        let pos = (0..5).map(|i| [i as f32, 0.0, 0.0]).collect();
        let tris = vec![[0u32, 1, 0], [1, 2, 1], [2, 3, 2], [3, 4, 3]]; // degenerate but connected
        (pos, tris)
    }

    #[test]
    fn source_has_zero_distance() {
        let (pos, tris) = simple_strip();
        let dists = dijkstra_geodesic(&pos, &tris, 0);
        assert_eq!(dists[0], 0.0);
    }

    #[test]
    fn adjacent_vertex_has_edge_length() {
        let (pos, tris) = simple_strip();
        let dists = dijkstra_geodesic(&pos, &tris, 0);
        let expected = 1.0f32; // edge (0,1) has length 1
        assert!(
            (dists[1] - expected).abs() < 1e-5,
            "dist to adjacent vertex should be edge length, got {}",
            dists[1]
        );
    }

    #[test]
    fn path_is_connected() {
        let (pos, tris) = simple_strip();
        let path = dijkstra_geodesic_between(&pos, &tris, 0, 3).expect("path should exist");
        // Each consecutive pair of vertices must be adjacent.
        let verts = &path.vertices;
        assert!(verts.first() == Some(&0));
        assert!(verts.last() == Some(&3));
    }

    #[test]
    fn dijkstra_between_same_vertex() {
        let (pos, tris) = simple_strip();
        let path = dijkstra_geodesic_between(&pos, &tris, 2, 2).expect("path to self");
        assert_eq!(path.vertices, vec![2]);
        assert!(path.total_length.abs() < 1e-6);
    }

    #[test]
    fn geodesic_distance_matches_dijkstra() {
        let (pos, tris) = simple_strip();
        let d1 = geodesic_distance(&pos, &tris, 0, 3);
        let d2 = dijkstra_geodesic(&pos, &tris, 0)[3];
        assert!((d1 - d2).abs() < 1e-6);
    }

    #[test]
    fn farthest_point_sampling_k_results() {
        let (pos, tris) = simple_strip();
        let samples = farthest_point_sampling(&pos, &tris, 3);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn farthest_point_sampling_zero_returns_empty() {
        let (pos, tris) = simple_strip();
        let samples = farthest_point_sampling(&pos, &tris, 0);
        assert!(samples.is_empty());
    }

    #[test]
    fn farthest_point_sampling_k_larger_than_n() {
        let (pos, tris) = simple_strip();
        let samples = farthest_point_sampling(&pos, &tris, 100);
        assert_eq!(samples.len(), pos.len());
    }

    #[test]
    fn geodesic_voronoi_covers_all_vertices() {
        let (pos, tris) = simple_strip();
        let seeds = vec![0usize, 3];
        let regions = geodesic_voronoi(&pos, &tris, &seeds);
        assert_eq!(regions.len(), pos.len());
        // Every vertex should be assigned to seed 0 or seed 3.
        for &r in &regions {
            assert!(r == 0 || r == 3, "unexpected region {r}");
        }
    }

    #[test]
    fn geodesic_voronoi_source_in_own_region() {
        let (pos, tris) = simple_strip();
        let seeds = vec![0usize, 3];
        let regions = geodesic_voronoi(&pos, &tris, &seeds);
        assert_eq!(regions[0], 0);
        assert_eq!(regions[3], 3);
    }

    #[test]
    fn reconstruct_path_no_predecessor() {
        let prev = vec![None, Some(0), Some(1)];
        let path = reconstruct_path(&prev, 2);
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn compute_eccentricity_non_negative() {
        let (pos, tris) = simple_strip();
        let e = compute_eccentricity(&pos, &tris, 0);
        assert!(e >= 0.0);
    }

    #[test]
    fn mesh_diameter_geodesic_non_negative() {
        let (pos, tris) = simple_strip();
        let d = mesh_diameter_geodesic(&pos, &tris);
        assert!(d >= 0.0);
    }

    #[test]
    fn line_mesh_all_distances_finite() {
        let (pos, tris) = line_mesh();
        let dists = dijkstra_geodesic(&pos, &tris, 0);
        assert!(
            dists.iter().all(|d| d.is_finite()),
            "all distances should be finite on connected mesh"
        );
    }
}
