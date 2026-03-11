// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! BVH-based broad-phase collision detection.
//!
//! Builds a bounding-volume hierarchy (BVH) bottom-up from axis-aligned
//! bounding boxes (AABBs) and provides efficient overlap queries.

/// Axis-aligned bounding box with an associated object ID.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AabbNode {
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// User-provided object identifier.
    pub id: usize,
    /// Whether this node is still active (not removed).
    pub active: bool,
}

/// An internal node of the BVH.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct BvhInternalNode {
    /// Merged AABB of the subtree.
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    /// Index into `BvhTree::leaves` for leaf nodes, otherwise `usize::MAX`.
    pub leaf_index: usize,
    /// Children in `BvhTree::nodes` (both `usize::MAX` for leaves).
    pub left: usize,
    pub right: usize,
}

/// A BVH tree over a set of AABBs.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhTree {
    /// Leaf AABBs.
    pub leaves: Vec<AabbNode>,
    /// Internal nodes (including leaf wrappers); root is `nodes.last()`.
    nodes: Vec<BvhInternalNode>,
    /// Index of the root node, or `usize::MAX` if the tree is empty.
    root: usize,
}

/// Configuration for the broad-phase system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BroadphaseConfig {
    /// Extra margin added around each AABB to catch nearly-touching pairs.
    pub aabb_margin: f32,
    /// Maximum leaf count per node before forcing a split.
    pub max_leaf_count: usize,
}

/// A pair of object IDs whose AABBs overlap.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionPair {
    pub a: usize,
    pub b: usize,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A list of overlapping object-ID pairs.
pub type PairList = Vec<CollisionPair>;

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default broadphase configuration.
#[allow(dead_code)]
pub fn default_broadphase_config() -> BroadphaseConfig {
    BroadphaseConfig {
        aabb_margin: 0.01,
        max_leaf_count: 1,
    }
}

/// Create an empty BVH tree.
#[allow(dead_code)]
pub fn new_bvh_tree() -> BvhTree {
    BvhTree {
        leaves: Vec::new(),
        nodes: Vec::new(),
        root: usize::MAX,
    }
}

/// Insert an AABB into the tree's leaf list (does **not** rebuild the BVH).
#[allow(dead_code)]
pub fn insert_aabb(tree: &mut BvhTree, min: [f32; 3], max: [f32; 3], id: usize) {
    tree.leaves.push(AabbNode {
        min,
        max,
        id,
        active: true,
    });
}

/// Build (or rebuild) the BVH bottom-up from all active leaves.
#[allow(dead_code)]
pub fn build_bvh(tree: &mut BvhTree) {
    tree.nodes.clear();
    tree.root = usize::MAX;

    let active: Vec<usize> = (0..tree.leaves.len())
        .filter(|&i| tree.leaves[i].active)
        .collect();

    if active.is_empty() {
        return;
    }

    // Create one internal node per leaf.
    let mut node_ids: Vec<usize> = active
        .iter()
        .map(|&li| {
            let leaf = &tree.leaves[li];
            tree.nodes.push(BvhInternalNode {
                aabb_min: leaf.min,
                aabb_max: leaf.max,
                leaf_index: li,
                left: usize::MAX,
                right: usize::MAX,
            });
            tree.nodes.len() - 1
        })
        .collect();

    // Bottom-up pair merge.
    while node_ids.len() > 1 {
        let mut next_level: Vec<usize> = Vec::new();
        let mut i = 0;
        while i < node_ids.len() {
            if i + 1 < node_ids.len() {
                let left = node_ids[i];
                let right = node_ids[i + 1];
                let merged_min = [
                    tree.nodes[left].aabb_min[0].min(tree.nodes[right].aabb_min[0]),
                    tree.nodes[left].aabb_min[1].min(tree.nodes[right].aabb_min[1]),
                    tree.nodes[left].aabb_min[2].min(tree.nodes[right].aabb_min[2]),
                ];
                let merged_max = [
                    tree.nodes[left].aabb_max[0].max(tree.nodes[right].aabb_max[0]),
                    tree.nodes[left].aabb_max[1].max(tree.nodes[right].aabb_max[1]),
                    tree.nodes[left].aabb_max[2].max(tree.nodes[right].aabb_max[2]),
                ];
                tree.nodes.push(BvhInternalNode {
                    aabb_min: merged_min,
                    aabb_max: merged_max,
                    leaf_index: usize::MAX,
                    left,
                    right,
                });
                next_level.push(tree.nodes.len() - 1);
                i += 2;
            } else {
                next_level.push(node_ids[i]);
                i += 1;
            }
        }
        node_ids = next_level;
    }

    tree.root = *node_ids.last().unwrap_or(&usize::MAX);
}

/// Return all overlapping AABB pairs among active leaves.
#[allow(dead_code)]
pub fn query_overlapping_pairs(tree: &BvhTree) -> PairList {
    let active: Vec<usize> = (0..tree.leaves.len())
        .filter(|&i| tree.leaves[i].active)
        .collect();
    let mut pairs = Vec::new();
    for i in 0..active.len() {
        for j in (i + 1)..active.len() {
            let a = &tree.leaves[active[i]];
            let b = &tree.leaves[active[j]];
            if aabbs_overlap(a.min, a.max, b.min, b.max) {
                pairs.push(CollisionPair { a: a.id, b: b.id });
            }
        }
    }
    pairs
}

/// Return all active leaves whose AABBs overlap with the query AABB.
#[allow(dead_code)]
pub fn query_single(tree: &BvhTree, min: [f32; 3], max: [f32; 3]) -> Vec<usize> {
    tree.leaves
        .iter()
        .filter(|l| l.active && aabbs_overlap(l.min, l.max, min, max))
        .map(|l| l.id)
        .collect()
}

/// Return the depth of the BVH (0 = empty).
#[allow(dead_code)]
pub fn bvh_depth(tree: &BvhTree) -> usize {
    if tree.root == usize::MAX {
        return 0;
    }
    node_depth(&tree.nodes, tree.root)
}

/// Return the total number of internal nodes in the BVH.
#[allow(dead_code)]
pub fn bvh_node_count(tree: &BvhTree) -> usize {
    tree.nodes.len()
}

/// Rebuild the BVH (alias for `build_bvh`).
#[allow(dead_code)]
pub fn rebuild_bvh(tree: &mut BvhTree) {
    build_bvh(tree);
}

/// Clear all leaves and internal nodes.
#[allow(dead_code)]
pub fn clear_bvh(tree: &mut BvhTree) {
    tree.leaves.clear();
    tree.nodes.clear();
    tree.root = usize::MAX;
}

/// Return the AABB of the root node (entire scene bounds).
///
/// Returns `[[0;3];2]` if the tree is empty.
#[allow(dead_code)]
pub fn bvh_bounds(tree: &BvhTree) -> [[f32; 3]; 2] {
    if tree.root == usize::MAX || tree.nodes.is_empty() {
        return [[0.0; 3]; 2];
    }
    let root = &tree.nodes[tree.root];
    [root.aabb_min, root.aabb_max]
}

/// Return the number of overlapping pairs found from `query_overlapping_pairs`.
#[allow(dead_code)]
pub fn pair_count(pairs: &PairList) -> usize {
    pairs.len()
}

/// Mark a node with the given `id` as inactive (effectively removed).
///
/// Returns `true` if at least one leaf was deactivated.
#[allow(dead_code)]
pub fn remove_node(tree: &mut BvhTree, id: usize) -> bool {
    let mut found = false;
    for leaf in tree.leaves.iter_mut() {
        if leaf.id == id && leaf.active {
            leaf.active = false;
            found = true;
        }
    }
    found
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn aabbs_overlap(amin: [f32; 3], amax: [f32; 3], bmin: [f32; 3], bmax: [f32; 3]) -> bool {
    amin[0] <= bmax[0]
        && amax[0] >= bmin[0]
        && amin[1] <= bmax[1]
        && amax[1] >= bmin[1]
        && amin[2] <= bmax[2]
        && amax[2] >= bmin[2]
}

fn node_depth(nodes: &[BvhInternalNode], idx: usize) -> usize {
    if idx == usize::MAX || idx >= nodes.len() {
        return 0;
    }
    let node = &nodes[idx];
    if node.left == usize::MAX && node.right == usize::MAX {
        return 1;
    }
    1 + node_depth(nodes, node.left).max(node_depth(nodes, node.right))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_aabb(id: usize, offset: f32) -> (usize, [f32; 3], [f32; 3]) {
        (id, [offset, offset, offset], [offset + 1.0, offset + 1.0, offset + 1.0])
    }

    #[test]
    fn test_default_broadphase_config() {
        let cfg = default_broadphase_config();
        assert!(cfg.aabb_margin >= 0.0);
        assert!(cfg.max_leaf_count >= 1);
    }

    #[test]
    fn test_new_bvh_tree_empty() {
        let tree = new_bvh_tree();
        assert_eq!(bvh_node_count(&tree), 0);
        assert_eq!(bvh_depth(&tree), 0);
    }

    #[test]
    fn test_insert_aabb_increments_leaves() {
        let mut tree = new_bvh_tree();
        let (id, mn, mx) = unit_aabb(0, 0.0);
        insert_aabb(&mut tree, mn, mx, id);
        assert_eq!(tree.leaves.len(), 1);
    }

    #[test]
    fn test_build_bvh_single_leaf() {
        let mut tree = new_bvh_tree();
        let (id, mn, mx) = unit_aabb(42, 0.0);
        insert_aabb(&mut tree, mn, mx, id);
        build_bvh(&mut tree);
        assert_eq!(bvh_node_count(&tree), 1);
        assert_eq!(bvh_depth(&tree), 1);
    }

    #[test]
    fn test_build_bvh_two_leaves() {
        let mut tree = new_bvh_tree();
        let (id0, mn0, mx0) = unit_aabb(0, 0.0);
        let (id1, mn1, mx1) = unit_aabb(1, 10.0);
        insert_aabb(&mut tree, mn0, mx0, id0);
        insert_aabb(&mut tree, mn1, mx1, id1);
        build_bvh(&mut tree);
        assert!(bvh_node_count(&tree) >= 2);
        assert!(bvh_depth(&tree) >= 1);
    }

    #[test]
    fn test_query_overlapping_pairs_no_overlap() {
        let mut tree = new_bvh_tree();
        let (id0, mn0, mx0) = unit_aabb(0, 0.0);
        let (id1, mn1, mx1) = unit_aabb(1, 10.0);
        insert_aabb(&mut tree, mn0, mx0, id0);
        insert_aabb(&mut tree, mn1, mx1, id1);
        let pairs = query_overlapping_pairs(&tree);
        assert_eq!(pair_count(&pairs), 0);
    }

    #[test]
    fn test_query_overlapping_pairs_with_overlap() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [2.0; 3], 0);
        insert_aabb(&mut tree, [1.0; 3], [3.0; 3], 1);
        let pairs = query_overlapping_pairs(&tree);
        assert_eq!(pair_count(&pairs), 1);
        assert_eq!(pairs[0].a, 0);
        assert_eq!(pairs[0].b, 1);
    }

    #[test]
    fn test_query_single_found() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [1.0; 3], 7);
        let hits = query_single(&tree, [0.5; 3], [1.5; 3]);
        assert!(hits.contains(&7));
    }

    #[test]
    fn test_query_single_not_found() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [1.0; 3], 7);
        let hits = query_single(&tree, [5.0; 3], [6.0; 3]);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_bvh_bounds_empty() {
        let tree = new_bvh_tree();
        let b = bvh_bounds(&tree);
        assert_eq!(b[0], [0.0; 3]);
    }

    #[test]
    fn test_bvh_bounds_non_empty() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [5.0; 3], 0);
        insert_aabb(&mut tree, [-2.0; 3], [3.0; 3], 1);
        build_bvh(&mut tree);
        let b = bvh_bounds(&tree);
        assert!(b[0][0] <= -2.0 + 1e-5);
        assert!(b[1][0] >= 5.0 - 1e-5);
    }

    #[test]
    fn test_rebuild_bvh() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [1.0; 3], 0);
        rebuild_bvh(&mut tree);
        assert!(bvh_node_count(&tree) > 0);
    }

    #[test]
    fn test_clear_bvh() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [1.0; 3], 0);
        build_bvh(&mut tree);
        clear_bvh(&mut tree);
        assert_eq!(bvh_node_count(&tree), 0);
        assert_eq!(tree.leaves.len(), 0);
    }

    #[test]
    fn test_remove_node_deactivates_leaf() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [1.0; 3], 99);
        assert!(remove_node(&mut tree, 99));
        assert!(!tree.leaves[0].active);
    }

    #[test]
    fn test_remove_node_missing() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [1.0; 3], 1);
        assert!(!remove_node(&mut tree, 999));
    }

    #[test]
    fn test_remove_node_excluded_from_pairs() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [2.0; 3], 0);
        insert_aabb(&mut tree, [1.0; 3], [3.0; 3], 1);
        remove_node(&mut tree, 1);
        let pairs = query_overlapping_pairs(&tree);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_pair_count() {
        let pairs: PairList = vec![
            CollisionPair { a: 0, b: 1 },
            CollisionPair { a: 2, b: 3 },
        ];
        assert_eq!(pair_count(&pairs), 2);
    }

    #[test]
    fn test_three_overlapping_aabbs() {
        let mut tree = new_bvh_tree();
        insert_aabb(&mut tree, [0.0; 3], [2.0; 3], 0);
        insert_aabb(&mut tree, [1.0; 3], [3.0; 3], 1);
        insert_aabb(&mut tree, [1.5; 3], [2.5; 3], 2);
        let pairs = query_overlapping_pairs(&tree);
        // All three pairs overlap.
        assert_eq!(pair_count(&pairs), 3);
    }
}
