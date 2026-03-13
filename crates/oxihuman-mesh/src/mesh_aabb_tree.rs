// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! AABB tree (bounding volume hierarchy) for accelerated ray-triangle
//! intersection queries on triangle meshes.
//!
//! The tree is built bottom-up by recursively splitting the list of triangle
//! indices along the longest axis of the combined bounding box.  Each leaf
//! node stores a single triangle index; internal nodes store child indices.

#![allow(dead_code)]

/// Configuration for the AABB tree builder.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AabbTreeConfig {
    /// Maximum number of triangles per leaf node (currently always 1).
    pub max_leaf_tris: usize,
    /// Maximum allowed tree depth (to guard against degenerate inputs).
    pub max_depth: usize,
}

/// Returns a sensible default [`AabbTreeConfig`].
#[allow(dead_code)]
pub fn default_aabb_tree_config() -> AabbTreeConfig {
    AabbTreeConfig {
        max_leaf_tris: 1,
        max_depth: 64,
    }
}

/// A 3-D axis-aligned bounding box used internally by the tree.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AabbBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AabbBox {
    fn new_empty() -> Self {
        AabbBox {
            min: [f32::MAX; 3],
            max: [f32::MIN; 3],
        }
    }

    fn expand_point(&mut self, p: [f32; 3]) {
        if p[0] < self.min[0] { self.min[0] = p[0]; }
        if p[0] > self.max[0] { self.max[0] = p[0]; }
        if p[1] < self.min[1] { self.min[1] = p[1]; }
        if p[1] > self.max[1] { self.max[1] = p[1]; }
        if p[2] < self.min[2] { self.min[2] = p[2]; }
        if p[2] > self.max[2] { self.max[2] = p[2]; }
    }

    fn expand_box(&mut self, other: &AabbBox) {
        if other.min[0] < self.min[0] { self.min[0] = other.min[0]; }
        if other.max[0] > self.max[0] { self.max[0] = other.max[0]; }
        if other.min[1] < self.min[1] { self.min[1] = other.min[1]; }
        if other.max[1] > self.max[1] { self.max[1] = other.max[1]; }
        if other.min[2] < self.min[2] { self.min[2] = other.min[2]; }
        if other.max[2] > self.max[2] { self.max[2] = other.max[2]; }
    }

    fn longest_axis(&self) -> usize {
        let dx = self.max[0] - self.min[0];
        let dy = self.max[1] - self.min[1];
        let dz = self.max[2] - self.min[2];
        if dx >= dy && dx >= dz { 0 }
        else if dy >= dz { 1 }
        else { 2 }
    }

    /// Slab-test ray-AABB intersection.
    /// Returns the near `t` if the ray hits, otherwise `None`.
    fn ray_intersect(&self, origin: [f32; 3], inv_dir: [f32; 3]) -> Option<f32> {
        let mut t_min = 0.0_f32;
        let mut t_max = f32::MAX;

        for axis in 0..3 {
            let t1 = (self.min[axis] - origin[axis]) * inv_dir[axis];
            let t2 = (self.max[axis] - origin[axis]) * inv_dir[axis];
            let (lo, hi) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
            t_min = t_min.max(lo);
            t_max = t_max.min(hi);
        }
        if t_max >= t_min && t_max >= 0.0 { Some(t_min) } else { None }
    }
}

/// A single node in the AABB tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AabbNode {
    /// Axis-aligned bounding box for this node.
    pub aabb: AabbBox,
    /// For leaf nodes: the triangle index.  For internal nodes: `usize::MAX`.
    pub tri_index: usize,
    /// Left child index into [`AabbTree::nodes`].  `usize::MAX` if leaf.
    pub left: usize,
    /// Right child index into [`AabbTree::nodes`].  `usize::MAX` if leaf.
    pub right: usize,
}

impl AabbNode {
    fn is_leaf(&self) -> bool {
        self.tri_index != usize::MAX
    }
}

/// An AABB tree (BVH) built over a triangle mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AabbTree {
    /// All nodes in the tree (root is at index 0).
    pub nodes: Vec<AabbNode>,
    /// Configuration used during construction.
    pub config: AabbTreeConfig,
    /// Triangle count in the source mesh.
    pub tri_count: usize,
}

// ---------------------------------------------------------------------------
// Construction helpers
// ---------------------------------------------------------------------------

fn tri_aabb(verts: &[[f32; 3]], indices: &[u32], tri: usize) -> AabbBox {
    let mut b = AabbBox::new_empty();
    let base = tri * 3;
    b.expand_point(verts[indices[base] as usize]);
    b.expand_point(verts[indices[base + 1] as usize]);
    b.expand_point(verts[indices[base + 2] as usize]);
    b
}

fn tri_centroid(verts: &[[f32; 3]], indices: &[u32], tri: usize) -> [f32; 3] {
    let base = tri * 3;
    let v0 = verts[indices[base] as usize];
    let v1 = verts[indices[base + 1] as usize];
    let v2 = verts[indices[base + 2] as usize];
    [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ]
}

#[allow(clippy::too_many_arguments)]
fn build_recursive(
    nodes: &mut Vec<AabbNode>,
    verts: &[[f32; 3]],
    indices: &[u32],
    tri_list: &mut Vec<usize>,
    start: usize,
    end: usize,
    depth: usize,
    max_depth: usize,
) -> usize {
    // Compute combined AABB.
    let mut aabb = AabbBox::new_empty();
    for &tri in &tri_list[start..end] {
        aabb.expand_box(&tri_aabb(verts, indices, tri));
    }

    let count = end - start;
    if count == 1 || depth >= max_depth {
        // Leaf node.
        let tri = tri_list[start];
        let node = AabbNode { aabb, tri_index: tri, left: usize::MAX, right: usize::MAX };
        let idx = nodes.len();
        nodes.push(node);
        return idx;
    }

    // Split along longest axis by centroid median.
    let axis = aabb.longest_axis();
    tri_list[start..end].sort_by(|&a, &b| {
        let ca = tri_centroid(verts, indices, a)[axis];
        let cb = tri_centroid(verts, indices, b)[axis];
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mid = start + count / 2;

    // Reserve slot for this internal node before recursing.
    let node_idx = nodes.len();
    nodes.push(AabbNode { aabb, tri_index: usize::MAX, left: usize::MAX, right: usize::MAX });

    let left = build_recursive(nodes, verts, indices, tri_list, start, mid, depth + 1, max_depth);
    let right = build_recursive(nodes, verts, indices, tri_list, mid, end, depth + 1, max_depth);

    nodes[node_idx].left = left;
    nodes[node_idx].right = right;
    node_idx
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build an AABB tree from a triangle mesh.
///
/// * `verts`   – flat array of vertex positions.
/// * `indices` – flat array of triangle vertex indices (length = `3 * tri_count`).
#[allow(dead_code)]
pub fn build_aabb_tree(
    verts: &[[f32; 3]],
    indices: &[u32],
    config: AabbTreeConfig,
) -> AabbTree {
    let tri_count = indices.len() / 3;
    let mut nodes: Vec<AabbNode> = Vec::with_capacity(tri_count * 2);
    if tri_count == 0 {
        return AabbTree { nodes, config, tri_count: 0 };
    }
    let mut tri_list: Vec<usize> = (0..tri_count).collect();
    let max_depth = config.max_depth;
    build_recursive(&mut nodes, verts, indices, &mut tri_list, 0, tri_count, 0, max_depth);
    AabbTree { nodes, config, tri_count }
}

/// Returns the total number of nodes in the tree.
#[allow(dead_code)]
pub fn aabb_tree_node_count(tree: &AabbTree) -> usize {
    tree.nodes.len()
}

/// Returns the number of leaf nodes (one per triangle).
#[allow(dead_code)]
pub fn aabb_tree_leaf_count(tree: &AabbTree) -> usize {
    tree.nodes.iter().filter(|n| n.is_leaf()).count()
}

/// Returns the depth of the tree (maximum root-to-leaf path length).
#[allow(dead_code)]
pub fn aabb_tree_depth(tree: &AabbTree) -> usize {
    if tree.nodes.is_empty() { return 0; }
    fn depth_from(nodes: &[AabbNode], idx: usize) -> usize {
        if nodes[idx].is_leaf() { return 1; }
        let ld = if nodes[idx].left != usize::MAX { depth_from(nodes, nodes[idx].left) } else { 0 };
        let rd = if nodes[idx].right != usize::MAX { depth_from(nodes, nodes[idx].right) } else { 0 };
        1 + ld.max(rd)
    }
    depth_from(&tree.nodes, 0)
}

/// Returns the AABB of the root node (i.e., the whole mesh).
#[allow(dead_code)]
pub fn aabb_tree_aabb(tree: &AabbTree) -> Option<AabbBox> {
    tree.nodes.first().map(|n| n.aabb)
}

/// Returns `true` if the tree contains no triangles.
#[allow(dead_code)]
pub fn aabb_tree_is_empty(tree: &AabbTree) -> bool {
    tree.tri_count == 0
}

/// Möller–Trumbore ray-triangle intersection test.
/// Returns the ray parameter `t` if the ray hits, otherwise `None`.
fn ray_triangle(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let e1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
    let e2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
    let h = cross(dir, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-8 { return None; }
    let f = 1.0 / a;
    let s = [origin[0]-v0[0], origin[1]-v0[1], origin[2]-v0[2]];
    let u = f * dot(s, h);
    if !(0.0..=1.0).contains(&u) { return None; }
    let q = cross(s, e1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 { return None; }
    let t = f * dot(e2, q);
    if t > 1e-8 { Some(t) } else { None }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1]*b[2] - a[2]*b[1],
        a[2]*b[0] - a[0]*b[2],
        a[0]*b[1] - a[1]*b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

/// Cast a ray against the tree.  Returns `(triangle_index, t)` for the
/// closest hit, or `None` if no intersection.
#[allow(dead_code)]
pub fn aabb_tree_ray_cast(
    tree: &AabbTree,
    verts: &[[f32; 3]],
    indices: &[u32],
    origin: [f32; 3],
    dir: [f32; 3],
) -> Option<(usize, f32)> {
    if tree.nodes.is_empty() { return None; }
    let inv_dir = [
        if dir[0].abs() > 1e-12 { 1.0 / dir[0] } else { f32::MAX },
        if dir[1].abs() > 1e-12 { 1.0 / dir[1] } else { f32::MAX },
        if dir[2].abs() > 1e-12 { 1.0 / dir[2] } else { f32::MAX },
    ];
    let mut best: Option<(usize, f32)> = None;
    let mut stack = vec![0usize];
    while let Some(idx) = stack.pop() {
        let node = &tree.nodes[idx];
        if node.aabb.ray_intersect(origin, inv_dir).is_none() { continue; }
        if node.is_leaf() {
            let tri = node.tri_index;
            let base = tri * 3;
            let v0 = verts[indices[base] as usize];
            let v1 = verts[indices[base+1] as usize];
            let v2 = verts[indices[base+2] as usize];
            if let Some(t) = ray_triangle(origin, dir, v0, v1, v2) {
                let update = match best { None => true, Some((_, bt)) => t < bt };
                if update { best = Some((tri, t)); }
            }
        } else {
            if node.left != usize::MAX { stack.push(node.left); }
            if node.right != usize::MAX { stack.push(node.right); }
        }
    }
    best
}

/// Return all triangle indices whose AABB overlaps the query box.
#[allow(dead_code)]
pub fn aabb_query_triangles(tree: &AabbTree, query: AabbBox) -> Vec<usize> {
    let mut result = Vec::new();
    if tree.nodes.is_empty() { return result; }
    let mut stack = vec![0usize];
    while let Some(idx) = stack.pop() {
        let node = &tree.nodes[idx];
        // Test overlap using explicit axis checks.
        let overlaps =
            node.aabb.max[0] >= query.min[0] && node.aabb.min[0] <= query.max[0] &&
            node.aabb.max[1] >= query.min[1] && node.aabb.min[1] <= query.max[1] &&
            node.aabb.max[2] >= query.min[2] && node.aabb.min[2] <= query.max[2];
        if !overlaps { continue; }
        if node.is_leaf() {
            result.push(node.tri_index);
        } else {
            if node.left != usize::MAX { stack.push(node.left); }
            if node.right != usize::MAX { stack.push(node.right); }
        }
    }
    result
}

/// Find the closest point on any triangle to the query point.
/// Returns `(triangle_index, closest_point, distance)`.
#[allow(dead_code)]
pub fn aabb_tree_closest_point(
    tree: &AabbTree,
    verts: &[[f32; 3]],
    indices: &[u32],
    query: [f32; 3],
) -> Option<(usize, [f32; 3], f32)> {
    if tree.nodes.is_empty() { return None; }
    let mut best: Option<(usize, [f32; 3], f32)> = None;
    let mut stack = vec![0usize];
    while let Some(idx) = stack.pop() {
        let node = &tree.nodes[idx];
        // Prune if AABB is farther than current best.
        let cx = query[0].clamp(node.aabb.min[0], node.aabb.max[0]);
        let cy = query[1].clamp(node.aabb.min[1], node.aabb.max[1]);
        let cz = query[2].clamp(node.aabb.min[2], node.aabb.max[2]);
        let aabb_dist = {
            let dx = query[0] - cx; let dy = query[1] - cy; let dz = query[2] - cz;
            (dx*dx + dy*dy + dz*dz).sqrt()
        };
        if let Some((_, _, bd)) = best {
            if aabb_dist >= bd { continue; }
        }
        if node.is_leaf() {
            let tri = node.tri_index;
            let base = tri * 3;
            let v0 = verts[indices[base] as usize];
            let v1 = verts[indices[base+1] as usize];
            let v2 = verts[indices[base+2] as usize];
            let cp = closest_point_on_tri(query, v0, v1, v2);
            let d = {
                let dx = cp[0]-query[0]; let dy = cp[1]-query[1]; let dz = cp[2]-query[2];
                (dx*dx + dy*dy + dz*dz).sqrt()
            };
            let update = match best { None => true, Some((_, _, bd)) => d < bd };
            if update { best = Some((tri, cp, d)); }
        } else {
            if node.left != usize::MAX { stack.push(node.left); }
            if node.right != usize::MAX { stack.push(node.right); }
        }
    }
    best
}

/// Closest point on a triangle (v0, v1, v2) to point p.
fn closest_point_on_tri(p: [f32; 3], v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let ab = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
    let ac = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
    let ap = [p[0]-v0[0], p[1]-v0[1], p[2]-v0[2]];

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 { return v0; }

    let bp = [p[0]-v1[0], p[1]-v1[1], p[2]-v1[2]];
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 { return v1; }

    let cp2 = [p[0]-v2[0], p[1]-v2[1], p[2]-v2[2]];
    let d5 = dot(ab, cp2);
    let d6 = dot(ac, cp2);
    if d6 >= 0.0 && d5 <= d6 { return v2; }

    let vc = d1*d4 - d3*d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [v0[0]+v*ab[0], v0[1]+v*ab[1], v0[2]+v*ab[2]];
    }

    let vb = d5*d2 - d1*d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [v0[0]+w*ac[0], v0[1]+w*ac[1], v0[2]+w*ac[2]];
    }

    let va = d3*d6 - d5*d4;
    if va <= 0.0 && (d4-d3) >= 0.0 && (d5-d6) >= 0.0 {
        let w = (d4-d3) / ((d4-d3) + (d5-d6));
        let bc = [v2[0]-v1[0], v2[1]-v1[1], v2[2]-v1[2]];
        return [v1[0]+w*bc[0], v1[1]+w*bc[1], v1[2]+w*bc[2]];
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        v0[0] + v*ab[0] + w*ac[0],
        v0[1] + v*ab[1] + w*ac[1],
        v0[2] + v*ab[2] + w*ac[2],
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_quad_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        // Two triangles forming a unit square in XY plane.
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        (verts, indices)
    }

    #[test]
    fn test_build_empty() {
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&[], &[], cfg);
        assert!(aabb_tree_is_empty(&tree));
        assert_eq!(aabb_tree_node_count(&tree), 0);
        assert_eq!(aabb_tree_leaf_count(&tree), 0);
        assert_eq!(aabb_tree_depth(&tree), 0);
        assert!(aabb_tree_aabb(&tree).is_none());
    }

    #[test]
    fn test_build_two_tris() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        assert!(!aabb_tree_is_empty(&tree));
        assert_eq!(aabb_tree_leaf_count(&tree), 2);
        assert!(aabb_tree_node_count(&tree) >= 2);
    }

    #[test]
    fn test_aabb_covers_mesh() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        let b = aabb_tree_aabb(&tree).expect("should succeed");
        assert!(b.min[0] <= 0.0 && b.max[0] >= 1.0);
        assert!(b.min[1] <= 0.0 && b.max[1] >= 1.0);
    }

    #[test]
    fn test_ray_cast_hit() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        let origin = [0.5, 0.5, 1.0];
        let dir = [0.0, 0.0, -1.0];
        let hit = aabb_tree_ray_cast(&tree, &verts, &indices, origin, dir);
        assert!(hit.is_some(), "expected a hit");
        let (_, t) = hit.expect("should succeed");
        assert!((t - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_ray_cast_miss() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        let origin = [5.0, 5.0, 1.0];
        let dir = [0.0, 0.0, -1.0];
        let hit = aabb_tree_ray_cast(&tree, &verts, &indices, origin, dir);
        assert!(hit.is_none());
    }

    #[test]
    fn test_ray_cast_empty_tree() {
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&[], &[], cfg);
        let hit = aabb_tree_ray_cast(&tree, &[], &[], [0.0;3], [0.0,0.0,1.0]);
        assert!(hit.is_none());
    }

    #[test]
    fn test_aabb_query_triangles() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        // Query box overlaps the whole mesh.
        let query = AabbBox { min: [-1.0,-1.0,-1.0], max: [2.0,2.0,2.0] };
        let hits = aabb_query_triangles(&tree, query);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn test_aabb_query_no_overlap() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        let query = AabbBox { min: [10.0,10.0,10.0], max: [11.0,11.0,11.0] };
        let hits = aabb_query_triangles(&tree, query);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_closest_point_above_center() {
        let (verts, indices) = unit_quad_mesh();
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        let query = [0.5, 0.5, 2.0];
        let result = aabb_tree_closest_point(&tree, &verts, &indices, query);
        assert!(result.is_some());
        let (_, cp, dist) = result.expect("should succeed");
        // Closest point should be directly below at z=0.
        assert!((cp[2]).abs() < 1e-4);
        assert!((dist - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_depth_single_tri() {
        let verts = vec![[0.0f32,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]];
        let indices = vec![0u32,1,2];
        let cfg = default_aabb_tree_config();
        let tree = build_aabb_tree(&verts, &indices, cfg);
        assert_eq!(aabb_tree_depth(&tree), 1);
        assert_eq!(aabb_tree_leaf_count(&tree), 1);
    }
}
