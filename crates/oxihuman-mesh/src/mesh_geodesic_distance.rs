//! Geodesic distance computation on triangle mesh surfaces using Dijkstra on graph.
//!
//! Builds a weighted graph from mesh edges where edge weights are Euclidean
//! distances between vertices, then runs Dijkstra's algorithm from a source
//! vertex to compute geodesic distances to all other vertices.

use std::collections::BinaryHeap;
use std::cmp::Ordering;

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Configuration for geodesic distance computation.
#[allow(dead_code)]
pub struct GeodesicConfig {
    /// Maximum distance to explore (f32::INFINITY = no limit).
    pub max_distance: f32,
    /// Whether to normalise distances to [0, 1].
    pub normalise: bool,
}

/// Result of geodesic distance computation from a single source vertex.
#[allow(dead_code)]
pub struct GeodesicResult {
    /// Geodesic distance from source to each vertex.
    pub distances: Vec<f32>,
    /// Previous vertex on the shortest path (usize::MAX if none).
    pub previous: Vec<usize>,
    /// The source vertex index.
    pub source: u32,
}

// ---------------------------------------------------------------------------
// Internal priority-queue node
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
struct NodeDist {
    vertex: usize,
    dist: f32,
}

impl Eq for NodeDist {}

impl PartialOrd for NodeDist {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeDist {
    // Min-heap: flip comparison
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .dist
            .partial_cmp(&self.dist)
            .unwrap_or(Ordering::Equal)
    }
}

// ---------------------------------------------------------------------------
// Default config
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn default_geodesic_config() -> GeodesicConfig {
    GeodesicConfig {
        max_distance: f32::INFINITY,
        normalise: false,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Build an adjacency list with edge weights from a triangle index buffer.
fn build_weighted_adjacency(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
) -> Vec<Vec<(usize, f32)>> {
    let n = verts.len();
    let mut adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); n];

    for face in faces {
        let a = face[0] as usize;
        let b = face[1] as usize;
        let c = face[2] as usize;
        if a >= n || b >= n || c >= n {
            continue;
        }
        let pairs = [(a, b), (b, c), (a, c)];
        for (u, v) in pairs {
            let w = dist3(verts[u], verts[v]);
            if !adj[u].iter().any(|&(nb, _)| nb == v) {
                adj[u].push((v, w));
            }
            if !adj[v].iter().any(|&(nb, _)| nb == u) {
                adj[v].push((u, w));
            }
        }
    }
    adj
}

// ---------------------------------------------------------------------------
// Core functions
// ---------------------------------------------------------------------------

/// Compute geodesic distances from `source` to all vertices using Dijkstra.
#[allow(dead_code)]
pub fn compute_geodesic_distances(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    source: u32,
    cfg: &GeodesicConfig,
) -> GeodesicResult {
    let n = verts.len();
    let src = source as usize;
    let mut distances = vec![f32::INFINITY; n];
    let mut previous = vec![usize::MAX; n];

    if src >= n {
        return GeodesicResult {
            distances,
            previous,
            source,
        };
    }

    let adj = build_weighted_adjacency(verts, faces);
    distances[src] = 0.0;

    let mut heap: BinaryHeap<NodeDist> = BinaryHeap::new();
    heap.push(NodeDist { vertex: src, dist: 0.0 });

    while let Some(NodeDist { vertex: u, dist: d }) = heap.pop() {
        if d > distances[u] {
            continue;
        }
        if d > cfg.max_distance {
            break;
        }
        for &(v, w) in &adj[u] {
            let new_dist = d + w;
            if new_dist < distances[v] {
                distances[v] = new_dist;
                previous[v] = u;
                heap.push(NodeDist { vertex: v, dist: new_dist });
            }
        }
    }

    if cfg.normalise {
        let max_d = distances
            .iter()
            .copied()
            .filter(|x| x.is_finite())
            .fold(0.0f32, f32::max);
        if max_d > 1e-10 {
            for d in &mut distances {
                if d.is_finite() {
                    *d /= max_d;
                }
            }
        }
    }

    GeodesicResult {
        distances,
        previous,
        source,
    }
}

/// Reconstruct the shortest path from source to `target` as a list of vertex indices.
#[allow(dead_code)]
pub fn geodesic_path(result: &GeodesicResult, target: u32) -> Vec<u32> {
    let mut path: Vec<u32> = Vec::new();
    let mut cur = target as usize;
    let src = result.source as usize;

    if cur >= result.distances.len() || result.distances[cur].is_infinite() {
        return path;
    }

    loop {
        path.push(cur as u32);
        if cur == src {
            break;
        }
        let prev = result.previous[cur];
        if prev == usize::MAX {
            break;
        }
        cur = prev;
    }
    path.reverse();
    path
}

/// Return all vertices whose geodesic distance is ≤ `threshold`.
#[allow(dead_code)]
pub fn geodesic_iso_contour(result: &GeodesicResult, threshold: f32) -> Vec<u32> {
    result
        .distances
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d <= threshold)
        .map(|(i, _)| i as u32)
        .collect()
}

/// Farthest-point sampling: greedily select `n` vertices that are mutually far apart.
#[allow(dead_code)]
pub fn farthest_point_sampling(
    verts: &[[f32; 3]],
    faces: &[[u32; 3]],
    n: usize,
) -> Vec<u32> {
    let v_count = verts.len();
    if v_count == 0 || n == 0 {
        return Vec::new();
    }

    let cfg = GeodesicConfig {
        max_distance: f32::INFINITY,
        normalise: false,
    };

    let mut selected: Vec<u32> = Vec::with_capacity(n);
    let mut min_dists = vec![f32::INFINITY; v_count];

    // Start from vertex 0.
    selected.push(0);

    for _ in 1..n {
        let last = selected[selected.len() - 1];
        let result = compute_geodesic_distances(verts, faces, last, &cfg);

        // Update min distances.
        for (v, &d) in result.distances.iter().enumerate() {
            if d < min_dists[v] {
                min_dists[v] = d;
            }
        }

        // Pick vertex with largest min-distance.
        let farthest = min_dists
            .iter()
            .enumerate()
            .max_by(|(_, &da), (_, &db)| da.partial_cmp(&db).unwrap_or(Ordering::Equal))
            .map(|(i, _)| i as u32)
            .unwrap_or(0);

        selected.push(farthest);
    }

    selected
}

/// Assign each vertex to the nearest source vertex (Voronoi-style labels).
/// Returns a label vector of length `result.distances.len()`.
#[allow(dead_code)]
pub fn geodesic_voronoi_regions(result: &GeodesicResult) -> Vec<u32> {
    // With a single source result, every reachable vertex maps to source=0.
    result
        .distances
        .iter()
        .map(|&d| if d.is_finite() { 0u32 } else { u32::MAX })
        .collect()
}

/// Compute the average geodesic distance from source to all reachable vertices.
#[allow(dead_code)]
pub fn average_geodesic_distance(result: &GeodesicResult) -> f32 {
    let finite: Vec<f32> = result
        .distances
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    if finite.is_empty() {
        return 0.0;
    }
    finite.iter().sum::<f32>() / finite.len() as f32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Simple 4-vertex tetrahedron-like mesh.
    fn quad_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_faces() -> Vec<[u32; 3]> {
        vec![[0, 1, 2], [0, 2, 3]]
    }

    #[test]
    fn test_source_has_zero_distance() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        assert_eq!(result.distances[0], 0.0);
    }

    #[test]
    fn test_all_vertices_reachable() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        for (i, &d) in result.distances.iter().enumerate() {
            assert!(
                d.is_finite(),
                "vertex {i} is not reachable, distance = {d}"
            );
        }
    }

    #[test]
    fn test_geodesic_path_reaches_source() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        let path = geodesic_path(&result, 3);
        assert!(!path.is_empty());
        assert_eq!(*path.first().unwrap(), 0, "path must start at source");
        assert_eq!(*path.last().unwrap(), 3, "path must end at target");
    }

    #[test]
    fn test_geodesic_iso_contour_includes_source() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        let contour = geodesic_iso_contour(&result, 0.0);
        assert!(contour.contains(&0), "source must be in iso-contour at threshold 0");
    }

    #[test]
    fn test_farthest_point_sampling_count() {
        let verts = quad_verts();
        let faces = quad_faces();
        let samples = farthest_point_sampling(&verts, &faces, 3);
        assert_eq!(samples.len(), 3);
    }

    #[test]
    fn test_average_geodesic_distance_positive() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        let avg = average_geodesic_distance(&result);
        assert!(avg > 0.0, "average geodesic distance should be positive");
    }

    #[test]
    fn test_geodesic_voronoi_regions_all_zero_for_single_source() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        let labels = geodesic_voronoi_regions(&result);
        assert_eq!(labels.len(), verts.len());
        for &l in &labels {
            assert_eq!(l, 0, "all vertices should map to source region 0");
        }
    }

    #[test]
    fn test_normalise_config_clamps_to_one() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = GeodesicConfig {
            max_distance: f32::INFINITY,
            normalise: true,
        };
        let result = compute_geodesic_distances(&verts, &faces, 0, &cfg);
        for &d in &result.distances {
            if d.is_finite() {
                assert!(d <= 1.0 + 1e-6, "normalised distance > 1.0: {d}");
            }
        }
    }

    #[test]
    fn test_invalid_source_returns_infinities() {
        let verts = quad_verts();
        let faces = quad_faces();
        let cfg = default_geodesic_config();
        let result = compute_geodesic_distances(&verts, &faces, 100, &cfg);
        for &d in &result.distances {
            assert!(d.is_infinite(), "expected all infinite for invalid source");
        }
    }
}
