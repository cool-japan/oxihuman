// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Geodesic Voronoi diagram on a mesh surface.
//!
//! Given a set of seed vertices, this module partitions all mesh vertices into
//! Voronoi regions via a Dijkstra-like propagation of geodesic distances
//! (approximated by summing Euclidean edge lengths).  Each vertex is assigned
//! to the seed from which the shortest geodesic path reaches it first.

#![allow(dead_code)]

/// Configuration for the surface Voronoi computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoronoiSurfConfig {
    /// If `true`, edge weights are the Euclidean length of the edge.
    /// If `false`, all edges are weight 1 (hop distance).
    pub use_euclidean_weights: bool,
}

/// Returns a sensible default [`VoronoiSurfConfig`].
#[allow(dead_code)]
pub fn default_voronoi_surf_config() -> VoronoiSurfConfig {
    VoronoiSurfConfig { use_euclidean_weights: true }
}

/// One Voronoi region: the set of vertex indices closest to `seed`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoronoiRegion {
    /// The seed vertex index.
    pub seed: usize,
    /// All vertex indices in this region (including the seed).
    pub vertices: Vec<usize>,
}

/// Full result of a Voronoi surface computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoronoiSurfResult {
    /// One entry per mesh vertex: which seed owns it (`usize::MAX` = unreachable).
    pub owner: Vec<usize>,
    /// Shortest geodesic distance from the owning seed to each vertex.
    pub distance: Vec<f32>,
    /// Regions indexed by seed order.
    pub regions: Vec<VoronoiRegion>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_adj(vertex_count: usize, indices: &[u32]) -> Vec<Vec<(usize, f32)>> {
    let mut adj: Vec<Vec<(usize, f32)>> = vec![vec![]; vertex_count];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let base = t * 3;
        for k in 0..3 {
            let a = indices[base + k] as usize;
            let b = indices[base + (k + 1) % 3] as usize;
            adj[a].push((b, 1.0));
            adj[b].push((a, 1.0));
        }
    }
    adj
}

fn build_adj_weighted(
    vertex_count: usize,
    verts: &[[f32; 3]],
    indices: &[u32],
) -> Vec<Vec<(usize, f32)>> {
    let mut adj: Vec<Vec<(usize, f32)>> = vec![vec![]; vertex_count];
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let base = t * 3;
        for k in 0..3 {
            let ai = indices[base + k] as usize;
            let bi = indices[base + (k + 1) % 3] as usize;
            let a = verts[ai];
            let b = verts[bi];
            let dx = b[0]-a[0]; let dy = b[1]-a[1]; let dz = b[2]-a[2];
            let w = (dx*dx + dy*dy + dz*dz).sqrt();
            adj[ai].push((bi, w));
            adj[bi].push((ai, w));
        }
    }
    adj
}

/// Dijkstra with multiple sources.  Returns `(owner, distance)` per vertex.
fn multi_source_dijkstra(
    adj: &[Vec<(usize, f32)>],
    seeds: &[usize],
) -> (Vec<usize>, Vec<f32>) {
    let n = adj.len();
    let mut owner = vec![usize::MAX; n];
    let mut dist = vec![f32::MAX; n];
    // Binary heap: (neg_dist_bits, vertex).  We use ordered_float trick manually.
    use std::collections::BinaryHeap;
    use std::cmp::Reverse;

    // (distance_bits as u32 for Ord, vertex, owner)
    let mut heap: BinaryHeap<Reverse<(u32, usize, usize)>> = BinaryHeap::new();

    for &s in seeds {
        if s < n {
            dist[s] = 0.0;
            owner[s] = s;
            heap.push(Reverse((0u32, s, s)));
        }
    }

    while let Some(Reverse((d_bits, v, o))) = heap.pop() {
        let d = f32::from_bits(d_bits);
        if d > dist[v] { continue; }
        for &(nb, w) in &adj[v] {
            let nd = d + w;
            if nd < dist[nb] {
                dist[nb] = nd;
                owner[nb] = o;
                heap.push(Reverse((nd.to_bits(), nb, o)));
            }
        }
    }
    (owner, dist)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the geodesic Voronoi diagram on a mesh surface.
///
/// * `verts`   – vertex positions (used for Euclidean edge weights).
/// * `indices` – triangle index buffer.
/// * `seeds`   – vertex indices to use as Voronoi seeds.
#[allow(dead_code)]
pub fn compute_voronoi_surf(
    verts: &[[f32; 3]],
    indices: &[u32],
    seeds: &[usize],
    config: VoronoiSurfConfig,
) -> VoronoiSurfResult {
    let n = verts.len();
    if n == 0 || seeds.is_empty() {
        return VoronoiSurfResult {
            owner: vec![],
            distance: vec![],
            regions: vec![],
        };
    }
    let adj = if config.use_euclidean_weights {
        build_adj_weighted(n, verts, indices)
    } else {
        build_adj(n, indices)
    };
    let (owner, distance) = multi_source_dijkstra(&adj, seeds);

    // Build regions.
    use std::collections::HashMap;
    let mut seed_idx: HashMap<usize, usize> = HashMap::new();
    for (i, &s) in seeds.iter().enumerate() {
        seed_idx.insert(s, i);
    }
    let mut regions: Vec<VoronoiRegion> = seeds.iter().map(|&s| VoronoiRegion { seed: s, vertices: vec![] }).collect();
    for (v, &o) in owner.iter().enumerate() {
        if o != usize::MAX {
            if let Some(&ri) = seed_idx.get(&o) {
                regions[ri].vertices.push(v);
            }
        }
    }
    VoronoiSurfResult { owner, distance, regions }
}

/// Number of Voronoi regions.
#[allow(dead_code)]
pub fn voronoi_region_count(result: &VoronoiSurfResult) -> usize {
    result.regions.len()
}

/// Seed that owns vertex `v` (`usize::MAX` if unreachable).
#[allow(dead_code)]
pub fn voronoi_region_for_vertex(result: &VoronoiSurfResult, v: usize) -> usize {
    result.owner.get(v).copied().unwrap_or(usize::MAX)
}

/// Slice of seed vertex indices.
#[allow(dead_code)]
pub fn voronoi_seed_vertices(result: &VoronoiSurfResult) -> Vec<usize> {
    result.regions.iter().map(|r| r.seed).collect()
}

/// Number of vertices in region `i`.
#[allow(dead_code)]
pub fn voronoi_region_size(result: &VoronoiSurfResult, i: usize) -> usize {
    result.regions.get(i).map_or(0, |r| r.vertices.len())
}

/// Returns pairs `(v0, v1)` where `v0` and `v1` belong to different Voronoi regions.
/// These are the boundary edges of the Voronoi diagram.
#[allow(dead_code)]
pub fn voronoi_boundary_edges(result: &VoronoiSurfResult, indices: &[u32]) -> Vec<(usize, usize)> {
    let tri_count = indices.len() / 3;
    let mut edges = Vec::new();
    for t in 0..tri_count {
        let base = t * 3;
        for k in 0..3 {
            let a = indices[base + k] as usize;
            let b = indices[base + (k + 1) % 3] as usize;
            let oa = result.owner.get(a).copied().unwrap_or(usize::MAX);
            let ob = result.owner.get(b).copied().unwrap_or(usize::MAX);
            if oa != ob {
                edges.push((a, b));
            }
        }
    }
    edges.sort();
    edges.dedup();
    edges
}

/// Serialize the result to a simple JSON string.
#[allow(dead_code)]
pub fn voronoi_surf_to_json(result: &VoronoiSurfResult) -> String {
    let seeds: Vec<String> = result.regions.iter().map(|r| r.seed.to_string()).collect();
    let sizes: Vec<String> = result.regions.iter().map(|r| r.vertices.len().to_string()).collect();
    format!(
        "{{\"region_count\":{},\"seeds\":[{}],\"sizes\":[{}]}}",
        result.regions.len(),
        seeds.join(","),
        sizes.join(",")
    )
}

/// Region index with the most vertices.
#[allow(dead_code)]
pub fn voronoi_largest_region(result: &VoronoiSurfResult) -> Option<usize> {
    result.regions.iter().enumerate()
        .max_by_key(|(_, r)| r.vertices.len())
        .map(|(i, _)| i)
}

/// Region index with the fewest vertices.
#[allow(dead_code)]
pub fn voronoi_smallest_region(result: &VoronoiSurfResult) -> Option<usize> {
    result.regions.iter().enumerate()
        .min_by_key(|(_, r)| r.vertices.len())
        .map(|(i, _)| i)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn line_mesh(n: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
        // n vertices in a line, connected by (n-1) degenerate "triangles"
        // Actually we create a strip of real triangles.
        // For simplicity use a 1-D graph: pair of vertices per "edge".
        let verts: Vec<[f32; 3]> = (0..n).map(|i| [i as f32, 0.0, 0.0]).collect();
        let mut indices = Vec::new();
        // Dummy triangles that share edges along the line (degenerate Y).
        for i in 0..n.saturating_sub(2) {
            indices.push(i as u32);
            indices.push((i + 1) as u32);
            indices.push((i + 2) as u32);
        }
        (verts, indices)
    }

    fn small_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        (verts, indices)
    }

    #[test]
    fn test_empty_input() {
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&[], &[], &[0], cfg);
        assert_eq!(voronoi_region_count(&r), 0);
    }

    #[test]
    fn test_no_seeds() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[], cfg);
        assert_eq!(voronoi_region_count(&r), 0);
    }

    #[test]
    fn test_single_seed_owns_all() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[0], cfg);
        assert_eq!(voronoi_region_count(&r), 1);
        assert_eq!(voronoi_region_size(&r, 0), verts.len());
    }

    #[test]
    fn test_two_seeds_partition() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[0, 2], cfg);
        assert_eq!(voronoi_region_count(&r), 2);
        let total = voronoi_region_size(&r, 0) + voronoi_region_size(&r, 1);
        assert_eq!(total, verts.len());
    }

    #[test]
    fn test_seed_vertices() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[1, 3], cfg);
        let seeds = voronoi_seed_vertices(&r);
        assert!(seeds.contains(&1));
        assert!(seeds.contains(&3));
    }

    #[test]
    fn test_region_for_vertex_seed_itself() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[0, 2], cfg);
        // Seed 0 should own itself.
        assert_eq!(voronoi_region_for_vertex(&r, 0), 0);
        // Seed 2 should own itself.
        assert_eq!(voronoi_region_for_vertex(&r, 2), 2);
    }

    #[test]
    fn test_boundary_edges() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[0, 2], cfg);
        let be = voronoi_boundary_edges(&r, &indices);
        // There should be at least one boundary edge.
        assert!(!be.is_empty());
    }

    #[test]
    fn test_largest_smallest_region() {
        let (verts, indices) = line_mesh(10);
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[0, 9], cfg);
        let lg = voronoi_largest_region(&r);
        let sm = voronoi_smallest_region(&r);
        assert!(lg.is_some());
        assert!(sm.is_some());
    }

    #[test]
    fn test_json_output() {
        let (verts, indices) = small_quad();
        let cfg = default_voronoi_surf_config();
        let r = compute_voronoi_surf(&verts, &indices, &[0], cfg);
        let json = voronoi_surf_to_json(&r);
        assert!(json.contains("region_count"));
        assert!(json.contains("seeds"));
    }

    #[test]
    fn test_hop_distance() {
        let (verts, indices) = small_quad();
        let cfg = VoronoiSurfConfig { use_euclidean_weights: false };
        let r = compute_voronoi_surf(&verts, &indices, &[0], cfg);
        // All vertices reachable.
        assert_eq!(voronoi_region_size(&r, 0), verts.len());
    }
}
