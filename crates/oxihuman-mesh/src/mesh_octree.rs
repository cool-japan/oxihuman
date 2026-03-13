// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Octree spatial index for mesh queries (nearest point, sphere, AABB, ray).
//!
//! Builds a hierarchical axis-aligned octree over a set of vertex positions
//! and triangle indices.  Supports nearest-vertex queries, sphere/AABB range
//! queries, ray-triangle intersection, leaf/node counting, and a simple refit
//! operation for updated vertex positions.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(a, b))
}

#[inline]
fn dist3_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub3(a, b);
    dot3(d, d)
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Configuration for octree construction.
#[allow(dead_code)]
pub struct OctreeConfig {
    /// Maximum number of items in a leaf before splitting.
    pub max_items_per_leaf: usize,
    /// Maximum tree depth.
    pub max_depth: u32,
    /// Padding added to each node's AABB.
    pub aabb_padding: f32,
}

/// A single node in the octree.
#[allow(dead_code)]
pub struct OctreeNode {
    /// Minimum corner of this node's AABB.
    pub min: [f32; 3],
    /// Maximum corner of this node's AABB.
    pub max: [f32; 3],
    /// Indices of vertex positions contained in this leaf (empty for internal nodes).
    pub vertex_indices: Vec<usize>,
    /// Children (8 child nodes for internal, empty for leaves).
    pub children: Vec<OctreeNode>,
    /// Depth of this node (0 = root).
    pub depth: u32,
}

/// Wrapper holding the root of the octree together with a snapshot of vertex
/// positions and triangle indices.
#[allow(dead_code)]
pub struct OctreeQuery {
    /// Root node of the octree.
    pub root: OctreeNode,
    /// Snapshot of vertex positions used when building.
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices (triples), as supplied at build time.
    pub indices: Vec<u32>,
}

/// Statistics about an octree.
#[allow(dead_code)]
pub struct OctreeStats {
    /// Total number of nodes (internal + leaf).
    pub node_count: usize,
    /// Total number of leaf nodes.
    pub leaf_count: usize,
    /// Maximum depth reached.
    pub max_depth: u32,
    /// Total number of vertex references across all leaves.
    pub total_vertex_refs: usize,
}

/// Result of a ray–mesh intersection query.
#[allow(dead_code)]
pub struct RayHit {
    /// Triangle index (into the original `indices` triples) that was hit.
    pub triangle_index: usize,
    /// Distance along the ray to the hit point.
    pub t: f32,
    /// World-space hit position.
    pub position: [f32; 3],
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// A pair of AABB corners returned by `octree_bounds`.
pub type AabbBounds = ([f32; 3], [f32; 3]);

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Return a default `OctreeConfig`.
#[allow(dead_code)]
pub fn default_octree_config() -> OctreeConfig {
    OctreeConfig {
        max_items_per_leaf: 8,
        max_depth: 12,
        aabb_padding: 1e-4,
    }
}

// ---------------------------------------------------------------------------
// AABB helpers
// ---------------------------------------------------------------------------

fn aabb_contains(min: [f32; 3], max: [f32; 3], p: [f32; 3]) -> bool {
    p[0] >= min[0]
        && p[0] <= max[0]
        && p[1] >= min[1]
        && p[1] <= max[1]
        && p[2] >= min[2]
        && p[2] <= max[2]
}

fn aabb_overlaps_sphere(min: [f32; 3], max: [f32; 3], center: [f32; 3], r: f32) -> bool {
    let cx = center[0].clamp(min[0], max[0]);
    let cy = center[1].clamp(min[1], max[1]);
    let cz = center[2].clamp(min[2], max[2]);
    dist3_sq([cx, cy, cz], center) <= r * r
}

fn aabb_overlaps_aabb(a_min: [f32; 3], a_max: [f32; 3], b_min: [f32; 3], b_max: [f32; 3]) -> bool {
    a_min[0] <= b_max[0]
        && a_max[0] >= b_min[0]
        && a_min[1] <= b_max[1]
        && a_max[1] >= b_min[1]
        && a_min[2] <= b_max[2]
        && a_max[2] >= b_min[2]
}

fn aabb_center(min: [f32; 3], max: [f32; 3]) -> [f32; 3] {
    [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ]
}

/// Compute a tight AABB for a slice of positions.
fn compute_aabb(positions: &[[f32; 3]], indices: &[usize]) -> ([f32; 3], [f32; 3]) {
    if indices.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let first = positions[indices[0]];
    let mut mn = first;
    let mut mx = first;
    for &i in &indices[1..] {
        let p = positions[i];
        mn[0] = mn[0].min(p[0]);
        mn[1] = mn[1].min(p[1]);
        mn[2] = mn[2].min(p[2]);
        mx[0] = mx[0].max(p[0]);
        mx[1] = mx[1].max(p[1]);
        mx[2] = mx[2].max(p[2]);
    }
    (mn, mx)
}

// ---------------------------------------------------------------------------
// Build
// ---------------------------------------------------------------------------

fn build_node(
    positions: &[[f32; 3]],
    vertex_indices: Vec<usize>,
    min: [f32; 3],
    max: [f32; 3],
    depth: u32,
    cfg: &OctreeConfig,
) -> OctreeNode {
    let pad = cfg.aabb_padding;
    let min_p = [min[0] - pad, min[1] - pad, min[2] - pad];
    let max_p = [max[0] + pad, max[1] + pad, max[2] + pad];

    if vertex_indices.len() <= cfg.max_items_per_leaf || depth >= cfg.max_depth {
        return OctreeNode {
            min: min_p,
            max: max_p,
            vertex_indices,
            children: Vec::new(),
            depth,
        };
    }

    let center = aabb_center(min, max);
    // Partition vertices into 8 octants.
    let mut buckets: [Vec<usize>; 8] = Default::default();
    for idx in vertex_indices {
        let p = positions[idx];
        let xi = if p[0] >= center[0] { 1 } else { 0 };
        let yi = if p[1] >= center[1] { 1 } else { 0 };
        let zi = if p[2] >= center[2] { 1 } else { 0 };
        buckets[xi + yi * 2 + zi * 4].push(idx);
    }

    let child_bounds: [([f32; 3], [f32; 3]); 8] = [
        (min, center),
        ([center[0], min[1], min[2]], [max[0], center[1], center[2]]),
        ([min[0], center[1], min[2]], [center[0], max[1], center[2]]),
        ([center[0], center[1], min[2]], [max[0], max[1], center[2]]),
        ([min[0], min[1], center[2]], [center[0], center[1], max[2]]),
        ([center[0], min[1], center[2]], [max[0], center[1], max[2]]),
        ([min[0], center[1], center[2]], [center[0], max[1], max[2]]),
        (center, max),
    ];

    let mut children = Vec::new();
    for (i, bucket) in buckets.into_iter().enumerate() {
        if !bucket.is_empty() {
            let (cmin, cmax) = child_bounds[i];
            children.push(build_node(positions, bucket, cmin, cmax, depth + 1, cfg));
        }
    }

    OctreeNode {
        min: min_p,
        max: max_p,
        vertex_indices: Vec::new(),
        children,
        depth,
    }
}

/// Build an octree from a set of vertex positions and triangle indices.
#[allow(dead_code)]
pub fn build_octree(positions: &[[f32; 3]], indices: &[u32], cfg: &OctreeConfig) -> OctreeQuery {
    let all_indices: Vec<usize> = (0..positions.len()).collect();
    let (mn, mx) = compute_aabb(positions, &all_indices);
    let root = build_node(positions, all_indices, mn, mx, 0, cfg);
    OctreeQuery {
        root,
        positions: positions.to_vec(),
        indices: indices.to_vec(),
    }
}

// ---------------------------------------------------------------------------
// Traversal helpers
// ---------------------------------------------------------------------------

fn node_depth_max(node: &OctreeNode) -> u32 {
    if node.children.is_empty() {
        node.depth
    } else {
        node.children
            .iter()
            .map(node_depth_max)
            .max()
            .unwrap_or(node.depth)
    }
}

fn count_nodes(node: &OctreeNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

fn count_leaves(node: &OctreeNode) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(count_leaves).sum()
    }
}

fn count_vertex_refs(node: &OctreeNode) -> usize {
    node.vertex_indices.len() + node.children.iter().map(count_vertex_refs).sum::<usize>()
}

// ---------------------------------------------------------------------------
// Public query functions
// ---------------------------------------------------------------------------

/// Return the maximum depth of the octree.
#[allow(dead_code)]
pub fn octree_depth(oq: &OctreeQuery) -> u32 {
    node_depth_max(&oq.root)
}

/// Return the total number of nodes in the octree.
#[allow(dead_code)]
pub fn octree_node_count(oq: &OctreeQuery) -> usize {
    count_nodes(&oq.root)
}

/// Return the number of leaf nodes in the octree.
#[allow(dead_code)]
pub fn octree_leaf_count(oq: &OctreeQuery) -> usize {
    count_leaves(&oq.root)
}

/// Return the AABB (min, max) of the root node.
#[allow(dead_code)]
pub fn octree_bounds(oq: &OctreeQuery) -> AabbBounds {
    (oq.root.min, oq.root.max)
}

/// Return statistics about the octree.
#[allow(dead_code)]
pub fn octree_stats(oq: &OctreeQuery) -> OctreeStats {
    OctreeStats {
        node_count: count_nodes(&oq.root),
        leaf_count: count_leaves(&oq.root),
        max_depth: node_depth_max(&oq.root),
        total_vertex_refs: count_vertex_refs(&oq.root),
    }
}

/// Find the index of the nearest vertex to `query` among all positions.
///
/// Returns `None` if there are no vertices.
#[allow(dead_code)]
pub fn query_nearest_point(oq: &OctreeQuery, query: [f32; 3]) -> Option<usize> {
    let positions = &oq.positions;
    if positions.is_empty() {
        return None;
    }
    let mut best_idx = 0usize;
    let mut best_dist = f32::MAX;
    nearest_in_node(&oq.root, positions, query, &mut best_idx, &mut best_dist);
    Some(best_idx)
}

fn nearest_in_node(
    node: &OctreeNode,
    positions: &[[f32; 3]],
    query: [f32; 3],
    best_idx: &mut usize,
    best_dist: &mut f32,
) {
    // Prune: if the closest possible point in this AABB is farther than current best, skip.
    let cx = query[0].clamp(node.min[0], node.max[0]);
    let cy = query[1].clamp(node.min[1], node.max[1]);
    let cz = query[2].clamp(node.min[2], node.max[2]);
    if dist3_sq([cx, cy, cz], query) >= *best_dist * *best_dist {
        return;
    }

    for &vi in &node.vertex_indices {
        let d = dist3(positions[vi], query);
        if d < *best_dist {
            *best_dist = d;
            *best_idx = vi;
        }
    }
    for child in &node.children {
        nearest_in_node(child, positions, query, best_idx, best_dist);
    }
}

/// Find all vertex indices within a sphere `(center, radius)`.
#[allow(dead_code)]
pub fn query_sphere(oq: &OctreeQuery, center: [f32; 3], radius: f32) -> Vec<usize> {
    let mut result = Vec::new();
    sphere_in_node(&oq.root, &oq.positions, center, radius, &mut result);
    result
}

fn sphere_in_node(
    node: &OctreeNode,
    positions: &[[f32; 3]],
    center: [f32; 3],
    radius: f32,
    result: &mut Vec<usize>,
) {
    if !aabb_overlaps_sphere(node.min, node.max, center, radius) {
        return;
    }
    for &vi in &node.vertex_indices {
        if dist3(positions[vi], center) <= radius {
            result.push(vi);
        }
    }
    for child in &node.children {
        sphere_in_node(child, positions, center, radius, result);
    }
}

/// Find all vertex indices within an axis-aligned bounding box `(qmin, qmax)`.
#[allow(dead_code)]
pub fn query_aabb(oq: &OctreeQuery, qmin: [f32; 3], qmax: [f32; 3]) -> Vec<usize> {
    let mut result = Vec::new();
    aabb_in_node(&oq.root, &oq.positions, qmin, qmax, &mut result);
    result
}

fn aabb_in_node(
    node: &OctreeNode,
    positions: &[[f32; 3]],
    qmin: [f32; 3],
    qmax: [f32; 3],
    result: &mut Vec<usize>,
) {
    if !aabb_overlaps_aabb(node.min, node.max, qmin, qmax) {
        return;
    }
    for &vi in &node.vertex_indices {
        if aabb_contains(qmin, qmax, positions[vi]) {
            result.push(vi);
        }
    }
    for child in &node.children {
        aabb_in_node(child, positions, qmin, qmax, result);
    }
}

/// Möller–Trumbore ray–triangle intersection test.
///
/// Returns `Some(t)` if the ray hits, or `None`.
fn ray_triangle(
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let h = cross3(ray_dir, edge2);
    let a = dot3(edge1, h);
    if a.abs() < 1e-9 {
        return None;
    }
    let f = 1.0 / a;
    let s = sub3(ray_origin, v0);
    let u = f * dot3(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross3(s, edge1);
    let v = f * dot3(ray_dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot3(edge2, q);
    if t < 1e-6 {
        None
    } else {
        Some(t)
    }
}

/// Find the closest triangle intersection for a ray.
///
/// `ray_dir` need not be normalised.
#[allow(dead_code)]
pub fn ray_intersect_octree(
    oq: &OctreeQuery,
    ray_origin: [f32; 3],
    ray_dir: [f32; 3],
) -> Option<RayHit> {
    let positions = &oq.positions;
    let indices = &oq.indices;
    let tri_count = indices.len() / 3;
    let mut best: Option<RayHit> = None;

    for ti in 0..tri_count {
        let i0 = indices[ti * 3] as usize;
        let i1 = indices[ti * 3 + 1] as usize;
        let i2 = indices[ti * 3 + 2] as usize;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }
        if let Some(t) = ray_triangle(
            ray_origin,
            ray_dir,
            positions[i0],
            positions[i1],
            positions[i2],
        ) {
            let better = best.as_ref().is_none_or(|b| t < b.t);
            if better {
                let pos = add3(ray_origin, scale3(ray_dir, t));
                best = Some(RayHit {
                    triangle_index: ti,
                    t,
                    position: pos,
                });
            }
        }
    }
    best
}

/// Refit the octree bounding boxes to account for updated vertex positions.
///
/// Rebuilds the entire tree with the same configuration.
#[allow(dead_code)]
pub fn refit_octree(oq: &mut OctreeQuery, new_positions: &[[f32; 3]], cfg: &OctreeConfig) {
    oq.positions = new_positions.to_vec();
    let all_indices: Vec<usize> = (0..new_positions.len()).collect();
    let (mn, mx) = compute_aabb(new_positions, &all_indices);
    oq.root = build_node(new_positions, all_indices, mn, mx, 0, cfg);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ]
    }

    fn simple_indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2, 0, 1, 4]
    }

    fn build_simple() -> OctreeQuery {
        let pos = simple_positions();
        let idx = simple_indices();
        let cfg = default_octree_config();
        build_octree(&pos, &idx, &cfg)
    }

    #[test]
    fn test_default_octree_config() {
        let cfg = default_octree_config();
        assert_eq!(cfg.max_items_per_leaf, 8);
        assert_eq!(cfg.max_depth, 12);
    }

    #[test]
    fn test_build_octree_non_empty() {
        let oq = build_simple();
        assert!(!oq.positions.is_empty());
    }

    #[test]
    fn test_octree_node_count_positive() {
        let oq = build_simple();
        assert!(octree_node_count(&oq) >= 1);
    }

    #[test]
    fn test_octree_leaf_count_positive() {
        let oq = build_simple();
        assert!(octree_leaf_count(&oq) >= 1);
    }

    #[test]
    fn test_octree_depth_at_least_zero() {
        let oq = build_simple();
        let d = octree_depth(&oq);
        assert!(d < 20);
    }

    #[test]
    fn test_octree_bounds_contains_all() {
        let oq = build_simple();
        let (mn, mx) = octree_bounds(&oq);
        for p in &oq.positions {
            assert!(p[0] >= mn[0] - 1e-3 && p[0] <= mx[0] + 1e-3);
            assert!(p[1] >= mn[1] - 1e-3 && p[1] <= mx[1] + 1e-3);
            assert!(p[2] >= mn[2] - 1e-3 && p[2] <= mx[2] + 1e-3);
        }
    }

    #[test]
    fn test_query_nearest_origin() {
        let oq = build_simple();
        let idx = query_nearest_point(&oq, [0.0, 0.0, 0.0]).expect("should succeed");
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_query_nearest_far_corner() {
        let oq = build_simple();
        let idx = query_nearest_point(&oq, [1.0, 1.0, 0.0]).expect("should succeed");
        assert_eq!(oq.positions[idx], [1.0, 1.0, 0.0]);
    }

    #[test]
    fn test_query_sphere_all() {
        let oq = build_simple();
        let hits = query_sphere(&oq, [0.5, 0.5, 0.5], 10.0);
        assert_eq!(hits.len(), oq.positions.len());
    }

    #[test]
    fn test_query_sphere_none() {
        let oq = build_simple();
        let hits = query_sphere(&oq, [100.0, 100.0, 100.0], 0.1);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_query_aabb_all() {
        let oq = build_simple();
        let hits = query_aabb(&oq, [-10.0; 3], [10.0; 3]);
        assert_eq!(hits.len(), oq.positions.len());
    }

    #[test]
    fn test_query_aabb_single() {
        let oq = build_simple();
        let hits = query_aabb(&oq, [-0.01; 3], [0.01, 0.01, 0.01]);
        assert!(hits.contains(&0));
    }

    #[test]
    fn test_ray_intersect_hit() {
        let oq = build_simple();
        // Ray from above looking down at the first triangle (0,1,2).
        let origin = [0.25, 0.25, 5.0];
        let dir = [0.0, 0.0, -1.0];
        let hit = ray_intersect_octree(&oq, origin, dir);
        assert!(hit.is_some(), "expected a hit");
        assert!(hit.expect("should succeed").t > 0.0);
    }

    #[test]
    fn test_ray_intersect_miss() {
        let oq = build_simple();
        let origin = [10.0, 10.0, 5.0];
        let dir = [0.0, 0.0, -1.0];
        let hit = ray_intersect_octree(&oq, origin, dir);
        assert!(hit.is_none());
    }

    #[test]
    fn test_octree_stats_fields() {
        let oq = build_simple();
        let stats = octree_stats(&oq);
        assert!(stats.node_count >= stats.leaf_count);
        assert!(stats.total_vertex_refs > 0);
    }

    #[test]
    fn test_refit_octree() {
        let mut oq = build_simple();
        let cfg = default_octree_config();
        let new_pos: Vec<[f32; 3]> = oq
            .positions
            .iter()
            .map(|&p| [p[0] + 5.0, p[1], p[2]])
            .collect();
        refit_octree(&mut oq, &new_pos, &cfg);
        let (mn, _mx) = octree_bounds(&oq);
        assert!(mn[0] > 4.0, "bounds should have shifted: mn.x={}", mn[0]);
    }

    #[test]
    fn test_empty_positions_nearest_none() {
        let oq = build_octree(&[], &[], &default_octree_config());
        let result = query_nearest_point(&oq, [0.0, 0.0, 0.0]);
        assert!(result.is_none());
    }
}
