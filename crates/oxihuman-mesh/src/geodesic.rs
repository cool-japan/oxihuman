// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Geodesic distance computation on mesh surfaces using Dijkstra's algorithm.
//!
//! Edge weights are the Euclidean distances between vertex positions.

use std::collections::BinaryHeap;

use crate::mesh::MeshBuffers;

// ---------------------------------------------------------------------------
// Ordered distance wrapper (min-heap via reverse ordering)
// ---------------------------------------------------------------------------

#[derive(PartialEq)]
struct Dist(f32);

impl Eq for Dist {}

impl Ord for Dist {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering so that BinaryHeap becomes a min-heap.
        other
            .0
            .partial_cmp(&self.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for Dist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ---------------------------------------------------------------------------
// Public result type
// ---------------------------------------------------------------------------

/// Geodesic distance result from a single source vertex (or multi-source).
#[allow(dead_code)]
pub struct GeodesicResult {
    /// Distance from the source to each vertex. `f32::INFINITY` for unreachable vertices.
    pub distances: Vec<f32>,
    /// Source vertex index (for multi-source, index of the first source).
    pub source: usize,
}

impl GeodesicResult {
    /// Maximum finite distance in the field.
    #[allow(dead_code)]
    pub fn max_distance(&self) -> f32 {
        self.distances
            .iter()
            .copied()
            .filter(|d| d.is_finite())
            .fold(0.0f32, f32::max)
    }

    /// All vertices within a given distance radius (inclusive).
    #[allow(dead_code)]
    pub fn vertices_within(&self, radius: f32) -> Vec<usize> {
        self.distances
            .iter()
            .enumerate()
            .filter(|(_, &d)| d <= radius)
            .map(|(i, _)| i)
            .collect()
    }

    /// Nearest vertex to the source (other than the source itself).
    #[allow(dead_code)]
    pub fn nearest_vertex(&self) -> Option<usize> {
        self.distances
            .iter()
            .enumerate()
            .filter(|&(i, &d)| i != self.source && d.is_finite())
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    /// Reconstruct the approximate shortest path from source to target vertex
    /// using the distance field and edge adjacency (greedy descent on distances).
    ///
    /// Returns a list of vertex indices from source to target (inclusive).
    /// If target is unreachable, returns an empty vec.
    #[allow(dead_code)]
    pub fn path_to(&self, target: usize, mesh: &MeshBuffers) -> Vec<usize> {
        if !self.distances[target].is_finite() {
            return Vec::new();
        }
        if target == self.source {
            return vec![self.source];
        }

        let adj = build_adjacency(mesh);
        let mut path = vec![target];
        let mut current = target;

        // Walk backwards along steepest descent in the distance field.
        loop {
            if current == self.source {
                break;
            }
            let best = adj[current]
                .iter()
                .copied()
                .filter(|&nb| self.distances[nb] < self.distances[current])
                .min_by(|&a, &b| {
                    self.distances[a]
                        .partial_cmp(&self.distances[b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            match best {
                Some(prev) => {
                    path.push(prev);
                    current = prev;
                }
                None => break, // should not happen for reachable vertices
            }
        }

        path.reverse();
        path
    }

    /// Normalize distances to [0, 1] range based on the maximum finite distance.
    #[allow(dead_code)]
    pub fn normalized(&self) -> Vec<f32> {
        let max = self.max_distance();
        if max == 0.0 {
            return vec![0.0; self.distances.len()];
        }
        self.distances
            .iter()
            .map(|&d| if d.is_finite() { d / max } else { 1.0 })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build an adjacency list (vertex -> list of neighbour vertex indices) from
/// the mesh triangle indices.
fn build_adjacency(mesh: &MeshBuffers) -> Vec<Vec<usize>> {
    let n = mesh.vertex_count();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let idx = &mesh.indices;
    let face_count = idx.len() / 3;
    for f in 0..face_count {
        let a = idx[f * 3] as usize;
        let b = idx[f * 3 + 1] as usize;
        let c = idx[f * 3 + 2] as usize;
        for (p, q) in [(a, b), (b, a), (b, c), (c, b), (c, a), (a, c)] {
            if !adj[p].contains(&q) {
                adj[p].push(q);
            }
        }
    }
    adj
}

/// Euclidean distance between two vertex positions.
#[inline]
fn edge_weight(mesh: &MeshBuffers, a: usize, b: usize) -> f32 {
    let pa = mesh.positions[a];
    let pb = mesh.positions[b];
    let dx = pa[0] - pb[0];
    let dy = pa[1] - pb[1];
    let dz = pa[2] - pb[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Core Dijkstra's algorithm from a set of seed entries `(distance, vertex)`.
fn dijkstra_core(mesh: &MeshBuffers, seeds: Vec<(f32, usize)>, n: usize) -> Vec<f32> {
    let adj = build_adjacency(mesh);
    let mut dist = vec![f32::INFINITY; n];
    let mut heap: BinaryHeap<(Dist, usize)> = BinaryHeap::new();

    for (d, v) in seeds {
        dist[v] = d;
        heap.push((Dist(d), v));
    }

    while let Some((Dist(d), u)) = heap.pop() {
        if d > dist[u] {
            continue; // stale entry
        }
        for &nb in &adj[u] {
            let w = edge_weight(mesh, u, nb);
            let nd = d + w;
            if nd < dist[nb] {
                dist[nb] = nd;
                heap.push((Dist(nd), nb));
            }
        }
    }

    dist
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute geodesic distances from a single source vertex using Dijkstra's
/// algorithm. Edge weights are the Euclidean distances between vertex positions.
#[allow(dead_code)]
pub fn geodesic_from_vertex(mesh: &MeshBuffers, source: usize) -> GeodesicResult {
    let n = mesh.vertex_count();
    let distances = dijkstra_core(mesh, vec![(0.0, source)], n);
    GeodesicResult { distances, source }
}

/// Compute geodesic distances from multiple source vertices (multi-source
/// Dijkstra). Each vertex gets the distance to its nearest source.
///
/// The `source` field in the result is set to `sources[0]` (or `0` if the
/// slice is empty).
#[allow(dead_code)]
pub fn geodesic_from_vertices(mesh: &MeshBuffers, sources: &[usize]) -> GeodesicResult {
    let n = mesh.vertex_count();
    let seeds: Vec<(f32, usize)> = sources.iter().map(|&v| (0.0, v)).collect();
    let distances = dijkstra_core(mesh, seeds, n);
    let source = sources.first().copied().unwrap_or(0);
    GeodesicResult { distances, source }
}

/// Compute all-pairs shortest geodesic distances.
///
/// Returns a `Vec<GeodesicResult>`, one per vertex.
///
/// **WARNING**: O(V * (V + E) log V) — only call for small meshes (< 1000
/// vertices).
#[allow(dead_code)]
pub fn geodesic_all_pairs(mesh: &MeshBuffers) -> Vec<GeodesicResult> {
    let n = mesh.vertex_count();
    (0..n).map(|v| geodesic_from_vertex(mesh, v)).collect()
}

/// Find the diameter of the mesh: the pair of vertices with maximum geodesic
/// distance. Returns `(vertex_a, vertex_b, distance)`.
///
/// Returns `(0, 0, 0.0)` for meshes with fewer than 2 vertices.
#[allow(dead_code)]
pub fn mesh_diameter(mesh: &MeshBuffers) -> (usize, usize, f32) {
    let n = mesh.vertex_count();
    if n < 2 {
        return (0, 0, 0.0);
    }

    let all = geodesic_all_pairs(mesh);
    let mut best = (0usize, 0usize, 0.0f32);
    for (a, result) in all.iter().enumerate() {
        for (b, &d) in result.distances.iter().enumerate() {
            if b != a && d.is_finite() && d > best.2 {
                best = (a, b, d);
            }
        }
    }
    best
}

/// Compute a geodesic heat map: normalizes distances to [0, 1] for
/// visualization, mapping:
/// - near (0.0) → blue  `[0, 0, 255]`
/// - mid  (0.5) → green `[0, 255, 0]`
/// - far  (1.0) → red   `[255, 0, 0]`
#[allow(dead_code)]
pub fn geodesic_heat_map(result: &GeodesicResult) -> Vec<[u8; 3]> {
    let normalized = result.normalized();
    normalized
        .iter()
        .map(|&t| {
            // t in [0, 1]
            // Blue -> Green in first half; Green -> Red in second half.
            if t <= 0.5 {
                let s = t * 2.0; // 0..1 over blue->green
                let r = 0u8;
                let g = (s * 255.0).round() as u8;
                let b = ((1.0 - s) * 255.0).round() as u8;
                [r, g, b]
            } else {
                let s = (t - 0.5) * 2.0; // 0..1 over green->red
                let r = (s * 255.0).round() as u8;
                let g = ((1.0 - s) * 255.0).round() as u8;
                let b = 0u8;
                [r, g, b]
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Helper: build a MeshBuffers from raw positions and indices.
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

    /// Single triangle with vertices at (0,0,0), (1,0,0), (0,1,0).
    fn single_triangle() -> MeshBuffers {
        make_mesh(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    /// Two triangles sharing edge (1,2):
    ///   verts: (0,0,0), (1,0,0), (0,1,0), (1,1,0)
    ///   faces: 0-1-2 and 1-3-2
    fn two_triangles() -> MeshBuffers {
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

    /// Disconnected mesh: two separate triangles (verts 0-2 and 3-5).
    fn disconnected_mesh() -> MeshBuffers {
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

    // -----------------------------------------------------------------------

    #[test]
    fn geodesic_single_triangle_source_zero() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        assert_eq!(result.source, 0);
        assert_eq!(result.distances[0], 0.0);
    }

    #[test]
    fn geodesic_single_triangle_distances_positive() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        // Distances to vertices 1 and 2 must be strictly positive.
        assert!(result.distances[1] > 0.0);
        assert!(result.distances[2] > 0.0);
        // Distance to vertex 1 should be 1.0 (along x-axis).
        assert!((result.distances[1] - 1.0).abs() < 1e-5);
        // Distance to vertex 2 should be 1.0 (along y-axis).
        assert!((result.distances[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn geodesic_from_vertices_multi_source_correct() {
        let mesh = two_triangles();
        // Sources at vertices 0 and 3 (opposite corners of the quad).
        let result = geodesic_from_vertices(&mesh, &[0, 3]);
        // Vertex 0 is a source: distance = 0.
        assert_eq!(result.distances[0], 0.0);
        // Vertex 3 is a source: distance = 0.
        assert_eq!(result.distances[3], 0.0);
        // Vertices 1 and 2 should be at most distance 1 from their nearest source.
        assert!(result.distances[1].is_finite());
        assert!(result.distances[2].is_finite());
        assert!(result.distances[1] <= 1.0 + 1e-5);
        assert!(result.distances[2] <= 1.0 + 1e-5);
    }

    #[test]
    fn geodesic_max_distance_positive() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        let max = result.max_distance();
        assert!(max > 0.0);
    }

    #[test]
    fn geodesic_vertices_within_radius() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        // All vertices are within distance 2.0.
        let within2 = result.vertices_within(2.0);
        assert_eq!(within2.len(), 3);
        // Only source is within distance 0.5.
        let within_half = result.vertices_within(0.5);
        assert_eq!(within_half, vec![0]);
    }

    #[test]
    fn geodesic_nearest_vertex_is_adjacent() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        let nearest = result.nearest_vertex();
        // Nearest should be vertex 1 or 2 (both at distance 1.0).
        assert!(nearest == Some(1) || nearest == Some(2));
    }

    #[test]
    fn geodesic_normalized_range_0_1() {
        let mesh = two_triangles();
        let result = geodesic_from_vertex(&mesh, 0);
        let norm = result.normalized();
        for &v in &norm {
            assert!((0.0 - 1e-6..=1.0 + 1e-6).contains(&v), "out of range: {v}");
        }
        // Source vertex should have normalized distance 0.
        assert!((norm[0]).abs() < 1e-6);
    }

    #[test]
    fn geodesic_path_to_adjacent_vertex() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        let path = result.path_to(1, &mesh);
        // Shortest path from 0 to 1 is [0, 1].
        assert_eq!(path, vec![0, 1]);
    }

    #[test]
    fn geodesic_path_to_same_vertex_length_one() {
        let mesh = single_triangle();
        let result = geodesic_from_vertex(&mesh, 0);
        let path = result.path_to(0, &mesh);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], 0);
    }

    #[test]
    fn mesh_diameter_returns_positive_distance() {
        let mesh = two_triangles();
        let (a, b, dist) = mesh_diameter(&mesh);
        assert!(dist > 0.0, "diameter must be positive");
        assert_ne!(a, b, "diameter endpoints must differ");
    }

    #[test]
    fn geodesic_heat_map_length_matches_vertices() {
        let mesh = two_triangles();
        let result = geodesic_from_vertex(&mesh, 0);
        let hmap = geodesic_heat_map(&result);
        assert_eq!(hmap.len(), mesh.vertex_count());
    }

    #[test]
    fn geodesic_all_pairs_small_mesh() {
        let mesh = single_triangle();
        let all = geodesic_all_pairs(&mesh);
        assert_eq!(all.len(), 3);
        // Each result should have source == its vertex index.
        for (i, res) in all.iter().enumerate() {
            assert_eq!(res.source, i);
            assert_eq!(res.distances[i], 0.0);
        }
        // Distance matrix should be symmetric (within floating-point tolerance).
        for i in 0..3 {
            for j in 0..3 {
                let d_ij = all[i].distances[j];
                let d_ji = all[j].distances[i];
                assert!(
                    (d_ij - d_ji).abs() < 1e-5,
                    "asymmetric: d[{i}][{j}]={d_ij}, d[{j}][{i}]={d_ji}"
                );
            }
        }
    }

    #[test]
    fn geodesic_unreachable_vertex_is_infinity() {
        let mesh = disconnected_mesh();
        let result = geodesic_from_vertex(&mesh, 0);
        // Vertices 3, 4, 5 are in a separate component: must be unreachable.
        assert_eq!(result.distances[3], f32::INFINITY);
        assert_eq!(result.distances[4], f32::INFINITY);
        assert_eq!(result.distances[5], f32::INFINITY);
        // Vertices 0, 1, 2 must be finite.
        assert!(result.distances[0].is_finite());
        assert!(result.distances[1].is_finite());
        assert!(result.distances[2].is_finite());
    }
}
