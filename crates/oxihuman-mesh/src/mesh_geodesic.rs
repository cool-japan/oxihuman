// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Geodesic distance computation on triangle meshes using Dijkstra on edge graph.
//!
//! This module provides single-source and multi-source geodesic distances,
//! path tracing, level-set isolines, Voronoi regions, and Laplacian heat diffusion.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::mesh::MeshBuffers;

// ── Type aliases ───────────────────────────────────────────────────────────────

/// Adjacency list: `adj[v]` holds `(neighbour_vertex, edge_length)` pairs.
#[allow(dead_code)]
pub type EdgeGraph = Vec<Vec<(usize, f32)>>;

/// Per-vertex assignment to the nearest source (Voronoi label).
#[allow(dead_code)]
pub type VoronoiLabels = Vec<usize>;

// ── Config & Result ────────────────────────────────────────────────────────────

/// Configuration for geodesic distance queries.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeodesicConfig {
    /// If true, stop Dijkstra early once every vertex has been visited.
    pub early_stop: bool,
    /// Epsilon used for level-set membership tests.
    pub level_set_eps: f32,
    /// Number of Laplacian smoothing iterations in `geodesic_heat`.
    pub heat_iterations: usize,
    /// Laplacian heat diffusion step size (0 < lambda ≤ 1).
    pub heat_lambda: f32,
}

/// Result of a geodesic distance query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeodesicResult {
    /// Per-vertex distances from the source(s). `f32::INFINITY` = unreachable.
    pub distances: Vec<f32>,
    /// Predecessor vertex for each vertex (used by path tracing). `usize::MAX` = no predecessor.
    pub predecessors: Vec<usize>,
    /// Index of the first source vertex.
    pub source: usize,
}

// ── Default config ─────────────────────────────────────────────────────────────

/// Return a sensible default [`GeodesicConfig`].
#[allow(dead_code)]
pub fn default_geodesic_config() -> GeodesicConfig {
    GeodesicConfig {
        early_stop: false,
        level_set_eps: 0.05,
        heat_iterations: 10,
        heat_lambda: 0.5,
    }
}

// ── Heap entry (min-heap by distance) ─────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
struct HeapNode {
    dist: f32,
    vertex: usize,
}

impl Eq for HeapNode {}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ── Internal helpers ───────────────────────────────────────────────────────────

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Build a weighted edge graph (adjacency list) from triangle mesh indices.
///
/// Each entry `adj[v]` is a list of `(neighbour, euclidean_edge_length)` pairs.
#[allow(dead_code)]
pub fn build_edge_graph(mesh: &MeshBuffers) -> EdgeGraph {
    let n = mesh.vertex_count();
    let mut adj: EdgeGraph = vec![Vec::new(); n];
    let idx = &mesh.indices;
    let face_count = idx.len() / 3;
    for f in 0..face_count {
        let a = idx[f * 3] as usize;
        let b = idx[f * 3 + 1] as usize;
        let c = idx[f * 3 + 2] as usize;
        for (p, q) in [(a, b), (b, a), (b, c), (c, b), (c, a), (a, c)] {
            if !adj[p].iter().any(|&(nb, _)| nb == q) {
                let w = dist3(mesh.positions[p], mesh.positions[q]);
                adj[p].push((q, w));
            }
        }
    }
    adj
}

/// Single-source geodesic distances using Dijkstra on the edge graph.
#[allow(dead_code)]
pub fn geodesic_distances(
    mesh: &MeshBuffers,
    source: usize,
    _cfg: &GeodesicConfig,
) -> GeodesicResult {
    let n = mesh.vertex_count();
    let adj = build_edge_graph(mesh);
    let mut dist = vec![f32::INFINITY; n];
    let mut pred = vec![usize::MAX; n];
    dist[source] = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(HeapNode {
        dist: 0.0,
        vertex: source,
    });

    while let Some(HeapNode { dist: d, vertex: u }) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(nb, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[nb] {
                dist[nb] = nd;
                pred[nb] = u;
                heap.push(HeapNode {
                    dist: nd,
                    vertex: nb,
                });
            }
        }
    }

    GeodesicResult {
        distances: dist,
        predecessors: pred,
        source,
    }
}

/// Multi-source geodesic distances: each vertex gets its distance to the nearest source.
#[allow(dead_code)]
pub fn geodesic_distances_multi(
    mesh: &MeshBuffers,
    sources: &[usize],
    cfg: &GeodesicConfig,
) -> GeodesicResult {
    let n = mesh.vertex_count();
    let adj = build_edge_graph(mesh);
    let mut dist = vec![f32::INFINITY; n];
    let mut pred = vec![usize::MAX; n];

    let mut heap = BinaryHeap::new();
    for &s in sources {
        if s < n {
            dist[s] = 0.0;
            pred[s] = s;
            heap.push(HeapNode {
                dist: 0.0,
                vertex: s,
            });
        }
    }

    while let Some(HeapNode { dist: d, vertex: u }) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(nb, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[nb] {
                dist[nb] = nd;
                pred[nb] = u;
                heap.push(HeapNode {
                    dist: nd,
                    vertex: nb,
                });
            }
        }
    }

    let _ = cfg; // used through build_edge_graph implicitly
    let source = sources.first().copied().unwrap_or(0);
    GeodesicResult {
        distances: dist,
        predecessors: pred,
        source,
    }
}

/// Return the vertex with the maximum geodesic distance from the source.
///
/// Returns `(vertex_index, distance)`. Skips `f32::INFINITY` entries.
#[allow(dead_code)]
pub fn farthest_point(result: &GeodesicResult) -> (usize, f32) {
    result
        .distances
        .iter()
        .enumerate()
        .filter(|(_, &d)| d.is_finite())
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(i, &d)| (i, d))
        .unwrap_or((0, 0.0))
}

/// Trace the shortest path from `source` to `target` using the predecessor map.
///
/// Returns vertex indices from source to target (inclusive), or empty vec if unreachable.
#[allow(dead_code)]
pub fn geodesic_path(result: &GeodesicResult, target: usize) -> Vec<usize> {
    if !result.distances[target].is_finite() {
        return Vec::new();
    }
    let source = result.source;
    if target == source {
        return vec![source];
    }
    let mut path = Vec::new();
    let mut cur = target;
    let max_steps = result.distances.len() + 1;
    for _ in 0..max_steps {
        path.push(cur);
        if cur == source {
            break;
        }
        let p = result.predecessors[cur];
        if p == usize::MAX || p == cur {
            // unreachable or self-loop guard
            if cur != source {
                return Vec::new();
            }
            break;
        }
        cur = p;
    }
    path.reverse();
    path
}

/// Approximate geodesic diameter using the two-source method.
///
/// Returns `(vertex_a, vertex_b, diameter_distance)`.
#[allow(dead_code)]
pub fn geodesic_diameter(mesh: &MeshBuffers, cfg: &GeodesicConfig) -> (usize, usize, f32) {
    let n = mesh.vertex_count();
    if n < 2 {
        return (0, 0, 0.0);
    }
    // Pass 1: farthest from vertex 0
    let r1 = geodesic_distances(mesh, 0, cfg);
    let (a, _) = farthest_point(&r1);
    // Pass 2: farthest from a
    let r2 = geodesic_distances(mesh, a, cfg);
    let (b, d) = farthest_point(&r2);
    (a, b, d)
}

/// Collect vertices whose geodesic distance is within `[target_dist - eps, target_dist + eps]`.
#[allow(dead_code)]
pub fn level_set_isolines(result: &GeodesicResult, target_dist: f32, eps: f32) -> Vec<usize> {
    result
        .distances
        .iter()
        .enumerate()
        .filter(|(_, &d)| d.is_finite() && (d - target_dist).abs() <= eps)
        .map(|(i, _)| i)
        .collect()
}

/// Normalize geodesic distances to `[0, 1]` based on the maximum finite distance.
#[allow(dead_code)]
pub fn normalize_geodesic(result: &GeodesicResult) -> Vec<f32> {
    let max = result
        .distances
        .iter()
        .copied()
        .filter(|d| d.is_finite())
        .fold(0.0f32, f32::max);
    if max == 0.0 {
        return vec![0.0; result.distances.len()];
    }
    result
        .distances
        .iter()
        .map(|&d| if d.is_finite() { d / max } else { 1.0 })
        .collect()
}

/// Assign each vertex to its nearest source (geodesic Voronoi).
///
/// `sources` must be non-empty. Returns per-vertex source index (into `sources` slice).
#[allow(dead_code)]
pub fn geodesic_voronoi(
    mesh: &MeshBuffers,
    sources: &[usize],
    cfg: &GeodesicConfig,
) -> VoronoiLabels {
    let n = mesh.vertex_count();
    if sources.is_empty() {
        return vec![0; n];
    }
    // Run multi-source Dijkstra and track which source each vertex came from.
    let adj = build_edge_graph(mesh);
    let mut dist = vec![f32::INFINITY; n];
    let mut label = vec![0usize; n];

    let mut heap = BinaryHeap::new();
    for (si, &sv) in sources.iter().enumerate() {
        if sv < n {
            dist[sv] = 0.0;
            label[sv] = si;
            heap.push(HeapNode {
                dist: 0.0,
                vertex: sv,
            });
        }
    }

    let _ = cfg;
    while let Some(HeapNode { dist: d, vertex: u }) = heap.pop() {
        if d > dist[u] {
            continue;
        }
        for &(nb, w) in &adj[u] {
            let nd = d + w;
            if nd < dist[nb] {
                dist[nb] = nd;
                label[nb] = label[u];
                heap.push(HeapNode {
                    dist: nd,
                    vertex: nb,
                });
            }
        }
    }
    label
}

/// Smooth geodesic distances via a Laplacian heat-diffusion pass.
///
/// Applies `cfg.heat_iterations` steps of weighted averaging.
#[allow(dead_code)]
pub fn geodesic_heat(
    mesh: &MeshBuffers,
    result: &GeodesicResult,
    cfg: &GeodesicConfig,
) -> Vec<f32> {
    let adj = build_edge_graph(mesh);
    let n = mesh.vertex_count();
    let lambda = cfg.heat_lambda.clamp(0.0, 1.0);

    // Replace infinities with a large finite sentinel for smoothing
    let max_finite = result
        .distances
        .iter()
        .copied()
        .filter(|d| d.is_finite())
        .fold(0.0f32, f32::max);
    let sentinel = max_finite * 2.0;

    let mut signal: Vec<f32> = result
        .distances
        .iter()
        .map(|&d| if d.is_finite() { d } else { sentinel })
        .collect();

    for _ in 0..cfg.heat_iterations {
        let prev = signal.clone();
        for v in 0..n {
            if adj[v].is_empty() {
                continue;
            }
            let sum: f32 = adj[v].iter().map(|&(nb, _)| prev[nb]).sum();
            let avg = sum / adj[v].len() as f32;
            signal[v] = prev[v] * (1.0 - lambda) + avg * lambda;
        }
    }
    signal
}

/// Return the number of vertices reachable from the geodesic result.
#[allow(dead_code)]
pub fn geodesic_vertex_count(result: &GeodesicResult) -> usize {
    result.distances.iter().filter(|d| d.is_finite()).count()
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    fn make_mesh(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> MeshBuffers {
        let n = positions.len();
        MeshBuffers::from_morph(MB {
            positions,
            normals: vec![[0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices,
            has_suit: false,
        })
    }

    fn single_tri() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    fn two_tris() -> MeshBuffers {
        make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 1, 3, 2],
        )
    }

    fn disconnected() -> MeshBuffers {
        make_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![0, 1, 2, 3, 4, 5],
        )
    }

    fn cfg() -> GeodesicConfig {
        default_geodesic_config()
    }

    #[test]
    fn test_default_config() {
        let c = default_geodesic_config();
        assert!(!c.early_stop);
        assert!(c.level_set_eps > 0.0);
    }

    #[test]
    fn test_build_edge_graph_single_tri() {
        let mesh = single_tri();
        let adj = build_edge_graph(&mesh);
        assert_eq!(adj.len(), 3);
        // Each vertex has 2 neighbours in a single triangle
        assert_eq!(adj[0].len(), 2);
    }

    #[test]
    fn test_geodesic_distances_source_zero() {
        let mesh = single_tri();
        let r = geodesic_distances(&mesh, 0, &cfg());
        assert_eq!(r.distances[0], 0.0);
        assert_eq!(r.source, 0);
    }

    #[test]
    fn test_geodesic_distances_correct_lengths() {
        let mesh = single_tri();
        let r = geodesic_distances(&mesh, 0, &cfg());
        assert!((r.distances[1] - 1.0).abs() < 1e-5);
        assert!((r.distances[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_geodesic_distances_multi_all_sources_zero() {
        let mesh = two_tris();
        let r = geodesic_distances_multi(&mesh, &[0, 3], &cfg());
        assert_eq!(r.distances[0], 0.0);
        assert_eq!(r.distances[3], 0.0);
    }

    #[test]
    fn test_farthest_point_nonzero_distance() {
        let mesh = two_tris();
        let r = geodesic_distances(&mesh, 0, &cfg());
        let (v, d) = farthest_point(&r);
        assert!(d > 0.0);
        assert_ne!(v, 0);
    }

    #[test]
    fn test_geodesic_path_adjacent() {
        let mesh = single_tri();
        let r = geodesic_distances(&mesh, 0, &cfg());
        let path = geodesic_path(&r, 1);
        assert_eq!(path.first(), Some(&0));
        assert_eq!(path.last(), Some(&1));
    }

    #[test]
    fn test_geodesic_path_self() {
        let mesh = single_tri();
        let r = geodesic_distances(&mesh, 0, &cfg());
        let path = geodesic_path(&r, 0);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_geodesic_path_unreachable_empty() {
        let mesh = disconnected();
        let r = geodesic_distances(&mesh, 0, &cfg());
        let path = geodesic_path(&r, 3);
        assert!(path.is_empty());
    }

    #[test]
    fn test_geodesic_diameter_positive() {
        let mesh = two_tris();
        let (a, b, d) = geodesic_diameter(&mesh, &cfg());
        assert!(d > 0.0);
        assert_ne!(a, b);
    }

    #[test]
    fn test_level_set_isolines() {
        let mesh = single_tri();
        let r = geodesic_distances(&mesh, 0, &cfg());
        // level set at distance 1.0, eps = 0.1 should catch vertices 1 and 2
        let iso = level_set_isolines(&r, 1.0, 0.1);
        assert!(iso.contains(&1));
        assert!(iso.contains(&2));
    }

    #[test]
    fn test_normalize_geodesic_range() {
        let mesh = two_tris();
        let r = geodesic_distances(&mesh, 0, &cfg());
        let n = normalize_geodesic(&r);
        for &v in &n {
            assert!((0.0..=1.0 + 1e-6).contains(&v));
        }
        assert!((n[0]).abs() < 1e-6); // source = 0
    }

    #[test]
    fn test_geodesic_voronoi_two_sources() {
        let mesh = two_tris();
        let labels = geodesic_voronoi(&mesh, &[0, 3], &cfg());
        assert_eq!(labels.len(), 4);
        assert_eq!(labels[0], 0); // vertex 0 -> source 0
        assert_eq!(labels[3], 1); // vertex 3 -> source 1
    }

    #[test]
    fn test_geodesic_heat_length_preserved() {
        let mesh = two_tris();
        let r = geodesic_distances(&mesh, 0, &cfg());
        let heat = geodesic_heat(&mesh, &r, &cfg());
        assert_eq!(heat.len(), mesh.vertex_count());
    }

    #[test]
    fn test_geodesic_vertex_count_connected() {
        let mesh = single_tri();
        let r = geodesic_distances(&mesh, 0, &cfg());
        assert_eq!(geodesic_vertex_count(&r), 3);
    }

    #[test]
    fn test_geodesic_vertex_count_disconnected() {
        let mesh = disconnected();
        let r = geodesic_distances(&mesh, 0, &cfg());
        // Only 3 of 6 vertices are reachable
        assert_eq!(geodesic_vertex_count(&r), 3);
    }

    #[test]
    fn test_multi_source_middle_vertices_finite() {
        let mesh = two_tris();
        let r = geodesic_distances_multi(&mesh, &[0, 3], &cfg());
        assert!(r.distances[1].is_finite());
        assert!(r.distances[2].is_finite());
    }
}
