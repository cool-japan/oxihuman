// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

/// A node in a BVH tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhNode {
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub leaf_id: Option<u32>,
}

/// A bounding volume hierarchy tree.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct BvhTree {
    pub nodes: Vec<BvhNode>,
}

/// Create an empty BVH tree.
#[allow(dead_code)]
pub fn new_bvh_tree() -> BvhTree {
    BvhTree::default()
}

/// Check if an AABB contains a point.
#[allow(dead_code)]
pub fn aabb_contains_point(min: [f32; 3], max: [f32; 3], pt: [f32; 3]) -> bool {
    pt[0] >= min[0] && pt[0] <= max[0]
        && pt[1] >= min[1] && pt[1] <= max[1]
        && pt[2] >= min[2] && pt[2] <= max[2]
}

fn aabbs_overlap(min_a: [f32; 3], max_a: [f32; 3], min_b: [f32; 3], max_b: [f32; 3]) -> bool {
    min_a[0] <= max_b[0] && max_a[0] >= min_b[0]
        && min_a[1] <= max_b[1] && max_a[1] >= min_b[1]
        && min_a[2] <= max_b[2] && max_a[2] >= min_b[2]
}

fn merge_aabb(min_a: [f32; 3], max_a: [f32; 3], min_b: [f32; 3], max_b: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    (
        [min_a[0].min(min_b[0]), min_a[1].min(min_b[1]), min_a[2].min(min_b[2])],
        [max_a[0].max(max_b[0]), max_a[1].max(max_b[1]), max_a[2].max(max_b[2])],
    )
}

/// Insert a leaf node with the given AABB and ID.
/// This is a simple flat insert (no rebalancing). Returns the leaf node index.
#[allow(dead_code)]
pub fn bvh_insert_leaf(tree: &mut BvhTree, aabb_min: [f32; 3], aabb_max: [f32; 3], id: u32) -> usize {
    let leaf_idx = tree.nodes.len();
    tree.nodes.push(BvhNode {
        aabb_min,
        aabb_max,
        left: None,
        right: None,
        leaf_id: Some(id),
    });

    // If this is the first node, it is both the root and the leaf
    if leaf_idx == 0 {
        return leaf_idx;
    }

    // Find the last internal node (or root) and attach via parent
    // Simple strategy: group pairs of leaves under a new internal node
    if tree.nodes.len().is_multiple_of(2) {
        let prev_idx = leaf_idx - 1;
        let prev_min = tree.nodes[prev_idx].aabb_min;
        let prev_max = tree.nodes[prev_idx].aabb_max;
        let (merged_min, merged_max) = merge_aabb(aabb_min, aabb_max, prev_min, prev_max);
        let internal_idx = tree.nodes.len();
        tree.nodes.push(BvhNode {
            aabb_min: merged_min,
            aabb_max: merged_max,
            left: Some(prev_idx),
            right: Some(leaf_idx),
            leaf_id: None,
        });
        return internal_idx;
    }

    leaf_idx
}

/// Query the BVH tree for all leaf IDs whose AABBs contain the given point.
#[allow(dead_code)]
pub fn bvh_query_point(tree: &BvhTree, pt: [f32; 3]) -> Vec<u32> {
    let mut result = Vec::new();
    for node in &tree.nodes {
        if let Some(id) = node.leaf_id {
            if aabb_contains_point(node.aabb_min, node.aabb_max, pt) {
                result.push(id);
            }
        }
    }
    result
}

/// Get the number of nodes (leaves + internal) in the BVH tree.
#[allow(dead_code)]
pub fn bvh_node_count(tree: &BvhTree) -> usize {
    tree.nodes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree_empty() {
        let tree = new_bvh_tree();
        assert_eq!(bvh_node_count(&tree), 0);
    }

    #[test]
    fn insert_one_leaf() {
        let mut tree = new_bvh_tree();
        bvh_insert_leaf(&mut tree, [0.0; 3], [1.0; 3], 42);
        assert!(bvh_node_count(&tree) >= 1);
    }

    #[test]
    fn aabb_contains_point_inside() {
        assert!(aabb_contains_point([0.0; 3], [1.0; 3], [0.5, 0.5, 0.5]));
    }

    #[test]
    fn aabb_contains_point_outside() {
        assert!(!aabb_contains_point([0.0; 3], [1.0; 3], [2.0, 0.5, 0.5]));
    }

    #[test]
    fn aabb_contains_point_on_boundary() {
        assert!(aabb_contains_point([0.0; 3], [1.0; 3], [1.0, 1.0, 1.0]));
    }

    #[test]
    fn query_finds_leaf() {
        let mut tree = new_bvh_tree();
        bvh_insert_leaf(&mut tree, [0.0; 3], [2.0; 3], 7);
        let results = bvh_query_point(&tree, [1.0, 1.0, 1.0]);
        assert!(results.contains(&7));
    }

    #[test]
    fn query_no_hit() {
        let mut tree = new_bvh_tree();
        bvh_insert_leaf(&mut tree, [0.0; 3], [1.0; 3], 5);
        let results = bvh_query_point(&tree, [5.0, 5.0, 5.0]);
        assert!(results.is_empty());
    }

    #[test]
    fn multiple_leaves_query() {
        let mut tree = new_bvh_tree();
        bvh_insert_leaf(&mut tree, [0.0; 3], [2.0; 3], 1);
        bvh_insert_leaf(&mut tree, [1.0; 3], [3.0; 3], 2);
        // Point [1.5, 1.5, 1.5] is in both
        let results = bvh_query_point(&tree, [1.5, 1.5, 1.5]);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
    }

    #[test]
    fn node_count_increases_on_insert() {
        let mut tree = new_bvh_tree();
        let c0 = bvh_node_count(&tree);
        bvh_insert_leaf(&mut tree, [0.0; 3], [1.0; 3], 0);
        let c1 = bvh_node_count(&tree);
        assert!(c1 > c0);
    }

    #[test]
    fn aabb_min_corner_contained() {
        assert!(aabb_contains_point([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [1.0, 2.0, 3.0]));
    }
}
