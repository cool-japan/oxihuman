// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! BVH-based broad-phase collision detection.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl BvhAabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Enclosing AABB of two AABBs.
    pub fn union(&self, other: &BvhAabb) -> BvhAabb {
        BvhAabb {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn centroid(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhNode {
    pub aabb: BvhAabb,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub objects: Vec<usize>,
}

impl BvhNode {
    pub fn leaf(aabb: BvhAabb, objects: Vec<usize>) -> Self {
        Self {
            aabb,
            left: None,
            right: None,
            objects,
        }
    }

    pub fn internal(aabb: BvhAabb, left: usize, right: usize) -> Self {
        Self {
            aabb,
            left: Some(left),
            right: Some(right),
            objects: Vec::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhTree {
    pub nodes: Vec<BvhNode>,
    pub object_aabbs: Vec<BvhAabb>,
}

impl BvhTree {
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            object_aabbs: Vec::new(),
        }
    }
}

// ─── Build ────────────────────────────────────────────────────────────────────

/// Build a BVH tree from a slice of AABBs using top-down median splitting.
pub fn build_bvh(aabbs: &[BvhAabb]) -> BvhTree {
    if aabbs.is_empty() {
        return BvhTree::empty();
    }
    let mut nodes: Vec<BvhNode> = Vec::new();
    let indices: Vec<usize> = (0..aabbs.len()).collect();
    build_recursive(aabbs, &indices, &mut nodes);
    BvhTree {
        nodes,
        object_aabbs: aabbs.to_vec(),
    }
}

fn build_recursive(aabbs: &[BvhAabb], indices: &[usize], nodes: &mut Vec<BvhNode>) -> usize {
    // Compute enclosing AABB
    let mut enclosing = aabbs[indices[0]].clone();
    for &i in &indices[1..] {
        enclosing = enclosing.union(&aabbs[i]);
    }

    if indices.len() <= 4 {
        // leaf node
        let node_idx = nodes.len();
        nodes.push(BvhNode::leaf(enclosing, indices.to_vec()));
        return node_idx;
    }

    // Choose longest axis
    let extent = [
        enclosing.max[0] - enclosing.min[0],
        enclosing.max[1] - enclosing.min[1],
        enclosing.max[2] - enclosing.min[2],
    ];
    let axis = if extent[0] >= extent[1] && extent[0] >= extent[2] {
        0
    } else if extent[1] >= extent[2] {
        1
    } else {
        2
    };

    // Sort by centroid along chosen axis
    let mut sorted = indices.to_vec();
    sorted.sort_by(|&a, &b| {
        aabbs[a].centroid()[axis]
            .partial_cmp(&aabbs[b].centroid()[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mid = sorted.len() / 2;
    let (left_indices, right_indices) = sorted.split_at(mid);

    // Reserve placeholder for internal node
    let node_idx = nodes.len();
    nodes.push(BvhNode::leaf(enclosing.clone(), Vec::new())); // placeholder

    let left_idx = build_recursive(aabbs, left_indices, nodes);
    let right_idx = build_recursive(aabbs, right_indices, nodes);

    nodes[node_idx] = BvhNode::internal(enclosing, left_idx, right_idx);
    node_idx
}

// ─── Query ────────────────────────────────────────────────────────────────────

/// Return indices of objects whose AABBs overlap the query AABB.
pub fn query_aabb(tree: &BvhTree, query: &BvhAabb) -> Vec<usize> {
    if tree.nodes.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    query_aabb_recursive(tree, 0, query, &mut result);
    result.sort_unstable();
    result.dedup();
    result
}

fn query_aabb_recursive(tree: &BvhTree, node_idx: usize, query: &BvhAabb, result: &mut Vec<usize>) {
    let node = &tree.nodes[node_idx];
    if !aabb_overlap(&node.aabb, query) {
        return;
    }
    if node.is_leaf() {
        for &obj in &node.objects {
            if aabb_overlap(&tree.object_aabbs[obj], query) {
                result.push(obj);
            }
        }
        return;
    }
    if let Some(left) = node.left {
        query_aabb_recursive(tree, left, query, result);
    }
    if let Some(right) = node.right {
        query_aabb_recursive(tree, right, query, result);
    }
}

/// Return indices of objects whose AABBs are hit by an infinite ray.
pub fn query_ray(tree: &BvhTree, origin: [f32; 3], dir: [f32; 3]) -> Vec<usize> {
    if tree.nodes.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    query_ray_recursive(tree, 0, origin, dir, &mut result);
    result.sort_unstable();
    result.dedup();
    result
}

fn ray_vs_aabb(aabb: &BvhAabb, origin: [f32; 3], dir: [f32; 3]) -> bool {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    for i in 0..3 {
        if dir[i].abs() < 1e-8 {
            if origin[i] < aabb.min[i] || origin[i] > aabb.max[i] {
                return false;
            }
        } else {
            let inv = 1.0 / dir[i];
            let t1 = (aabb.min[i] - origin[i]) * inv;
            let t2 = (aabb.max[i] - origin[i]) * inv;
            tmin = tmin.max(t1.min(t2));
            tmax = tmax.min(t1.max(t2));
            if tmax < tmin {
                return false;
            }
        }
    }
    tmax >= 0.0
}

fn query_ray_recursive(
    tree: &BvhTree,
    node_idx: usize,
    origin: [f32; 3],
    dir: [f32; 3],
    result: &mut Vec<usize>,
) {
    let node = &tree.nodes[node_idx];
    if !ray_vs_aabb(&node.aabb, origin, dir) {
        return;
    }
    if node.is_leaf() {
        for &obj in &node.objects {
            if ray_vs_aabb(&tree.object_aabbs[obj], origin, dir) {
                result.push(obj);
            }
        }
        return;
    }
    if let Some(left) = node.left {
        query_ray_recursive(tree, left, origin, dir, result);
    }
    if let Some(right) = node.right {
        query_ray_recursive(tree, right, origin, dir, result);
    }
}

// ─── Primitives ───────────────────────────────────────────────────────────────

/// Test if two AABBs overlap.
pub fn aabb_overlap(a: &BvhAabb, b: &BvhAabb) -> bool {
    a.min[0] <= b.max[0]
        && a.max[0] >= b.min[0]
        && a.min[1] <= b.max[1]
        && a.max[1] >= b.min[1]
        && a.min[2] <= b.max[2]
        && a.max[2] >= b.min[2]
}

/// Build AABB from sphere center + radius.
pub fn aabb_from_sphere(center: [f32; 3], radius: f32) -> BvhAabb {
    BvhAabb {
        min: [center[0] - radius, center[1] - radius, center[2] - radius],
        max: [center[0] + radius, center[1] + radius, center[2] + radius],
    }
}

/// Build AABB from a capsule (two endpoints + radius).
pub fn aabb_from_capsule(a: [f32; 3], b: [f32; 3], radius: f32) -> BvhAabb {
    BvhAabb {
        min: [
            a[0].min(b[0]) - radius,
            a[1].min(b[1]) - radius,
            a[2].min(b[2]) - radius,
        ],
        max: [
            a[0].max(b[0]) + radius,
            a[1].max(b[1]) + radius,
            a[2].max(b[2]) + radius,
        ],
    }
}

/// Inflate an AABB by a uniform margin on all sides.
pub fn aabb_expand(aabb: &BvhAabb, margin: f32) -> BvhAabb {
    BvhAabb {
        min: [
            aabb.min[0] - margin,
            aabb.min[1] - margin,
            aabb.min[2] - margin,
        ],
        max: [
            aabb.max[0] + margin,
            aabb.max[1] + margin,
            aabb.max[2] + margin,
        ],
    }
}

/// Surface area of an AABB (2*(wh + hd + dw)).
pub fn aabb_surface_area(aabb: &BvhAabb) -> f32 {
    let w = (aabb.max[0] - aabb.min[0]).max(0.0);
    let h = (aabb.max[1] - aabb.min[1]).max(0.0);
    let d = (aabb.max[2] - aabb.min[2]).max(0.0);
    2.0 * (w * h + h * d + d * w)
}

/// Brute-force all-pairs overlap for small sets.
pub fn compute_all_pair_overlaps(aabbs: &[BvhAabb]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..aabbs.len() {
        for j in (i + 1)..aabbs.len() {
            if aabb_overlap(&aabbs[i], &aabbs[j]) {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

/// Validate that BVH query gives the same result as brute force for a query AABB.
pub fn bvh_vs_brute_force(aabbs: &[BvhAabb], query: &BvhAabb) -> bool {
    let tree = build_bvh(aabbs);
    let bvh_result = query_aabb(&tree, query);

    let mut brute: Vec<usize> = aabbs
        .iter()
        .enumerate()
        .filter(|(_, a)| aabb_overlap(a, query))
        .map(|(i, _)| i)
        .collect();
    brute.sort_unstable();

    bvh_result == brute
}

// ─── Tests ───────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn unit_aabb(cx: f32, cy: f32, cz: f32) -> BvhAabb {
        BvhAabb::new(
            [cx - 0.5, cy - 0.5, cz - 0.5],
            [cx + 0.5, cy + 0.5, cz + 0.5],
        )
    }

    #[test]
    fn aabb_overlap_touching_returns_true() {
        let a = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = BvhAabb::new([1.0, 0.0, 0.0], [2.0, 1.0, 1.0]);
        assert!(aabb_overlap(&a, &b));
    }

    #[test]
    fn aabb_overlap_separated_returns_false() {
        let a = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = BvhAabb::new([2.0, 0.0, 0.0], [3.0, 1.0, 1.0]);
        assert!(!aabb_overlap(&a, &b));
    }

    #[test]
    fn aabb_overlap_nested_returns_true() {
        let a = BvhAabb::new([-5.0, -5.0, -5.0], [5.0, 5.0, 5.0]);
        let b = BvhAabb::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        assert!(aabb_overlap(&a, &b));
    }

    #[test]
    fn aabb_overlap_y_separated_returns_false() {
        let a = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = BvhAabb::new([0.0, 2.0, 0.0], [1.0, 3.0, 1.0]);
        assert!(!aabb_overlap(&a, &b));
    }

    #[test]
    fn aabb_from_sphere_correct_bounds() {
        let aabb = aabb_from_sphere([1.0, 2.0, 3.0], 1.0);
        assert!((aabb.min[0] - 0.0).abs() < 1e-5);
        assert!((aabb.max[0] - 2.0).abs() < 1e-5);
        assert!((aabb.min[1] - 1.0).abs() < 1e-5);
        assert!((aabb.max[2] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn aabb_from_capsule_correct_bounds() {
        let aabb = aabb_from_capsule([0.0, 0.0, 0.0], [0.0, 4.0, 0.0], 1.0);
        assert!((aabb.min[1] - -1.0).abs() < 1e-5);
        assert!((aabb.max[1] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn aabb_expand_increases_size() {
        let aabb = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let expanded = aabb_expand(&aabb, 0.5);
        assert!((expanded.min[0] - -0.5).abs() < 1e-5);
        assert!((expanded.max[0] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn aabb_surface_area_unit_cube() {
        let aabb = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!((aabb_surface_area(&aabb) - 6.0).abs() < 1e-5);
    }

    #[test]
    fn build_bvh_no_panic_empty() {
        let tree = build_bvh(&[]);
        assert!(tree.nodes.is_empty());
    }

    #[test]
    fn build_bvh_no_panic_single() {
        let aabbs = vec![unit_aabb(0.0, 0.0, 0.0)];
        let tree = build_bvh(&aabbs);
        assert!(!tree.nodes.is_empty());
    }

    #[test]
    fn build_bvh_no_panic_many() {
        let aabbs: Vec<BvhAabb> = (0..20)
            .map(|i| unit_aabb(i as f32 * 2.0, 0.0, 0.0))
            .collect();
        let tree = build_bvh(&aabbs);
        assert!(!tree.nodes.is_empty());
    }

    #[test]
    fn query_aabb_empty_tree_returns_empty() {
        let tree = BvhTree::empty();
        let query = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(query_aabb(&tree, &query).is_empty());
    }

    #[test]
    fn query_aabb_no_overlap_returns_empty() {
        let aabbs = vec![unit_aabb(100.0, 100.0, 100.0)];
        let tree = build_bvh(&aabbs);
        let query = BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(query_aabb(&tree, &query).is_empty());
    }

    #[test]
    fn query_aabb_finds_overlapping() {
        let aabbs: Vec<BvhAabb> = (0..10)
            .map(|i| unit_aabb(i as f32 * 10.0, 0.0, 0.0))
            .collect();
        let tree = build_bvh(&aabbs);
        // Query around object 0
        let query = BvhAabb::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let result = query_aabb(&tree, &query);
        assert!(result.contains(&0));
    }

    #[test]
    fn bvh_matches_brute_force() {
        let aabbs: Vec<BvhAabb> = (0..15)
            .map(|i| unit_aabb(i as f32 * 3.0, 0.0, 0.0))
            .collect();
        let query = BvhAabb::new([1.0, -1.0, -1.0], [8.0, 1.0, 1.0]);
        assert!(bvh_vs_brute_force(&aabbs, &query));
    }

    #[test]
    fn compute_all_pair_overlaps_none_for_separated() {
        let aabbs = vec![
            BvhAabb::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
            BvhAabb::new([5.0, 0.0, 0.0], [6.0, 1.0, 1.0]),
        ];
        let pairs = compute_all_pair_overlaps(&aabbs);
        assert!(pairs.is_empty());
    }

    #[test]
    fn compute_all_pair_overlaps_finds_pair() {
        let aabbs = vec![
            BvhAabb::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
            BvhAabb::new([1.0, 0.0, 0.0], [3.0, 2.0, 2.0]),
        ];
        let pairs = compute_all_pair_overlaps(&aabbs);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], (0, 1));
    }
}
