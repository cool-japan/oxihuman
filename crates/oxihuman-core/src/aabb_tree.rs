#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 2D AABB tree for spatial queries.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AabbNode2 {
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub id: u32,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AabbTree2 {
    pub nodes: Vec<AabbNode2>,
}

#[allow(dead_code)]
pub fn new_aabb_tree2() -> AabbTree2 {
    AabbTree2 { nodes: Vec::new() }
}

#[allow(dead_code)]
pub fn aabb2_insert(tree: &mut AabbTree2, min: [f32; 2], max: [f32; 2], id: u32) -> usize {
    let idx = tree.nodes.len();
    tree.nodes.push(AabbNode2 {
        min,
        max,
        id,
        left: None,
        right: None,
    });
    idx
}

#[allow(dead_code)]
pub fn aabb2_query_point(tree: &AabbTree2, pt: [f32; 2]) -> Vec<u32> {
    tree.nodes
        .iter()
        .filter(|n| {
            (n.min[0]..=n.max[0]).contains(&pt[0]) && (n.min[1]..=n.max[1]).contains(&pt[1])
        })
        .map(|n| n.id)
        .collect()
}

#[allow(dead_code)]
pub fn aabb2_overlaps(
    a_min: [f32; 2],
    a_max: [f32; 2],
    b_min: [f32; 2],
    b_max: [f32; 2],
) -> bool {
    a_min[0] <= b_max[0]
        && a_max[0] >= b_min[0]
        && a_min[1] <= b_max[1]
        && a_max[1] >= b_min[1]
}

#[allow(dead_code)]
pub fn aabb2_node_count(tree: &AabbTree2) -> usize {
    tree.nodes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tree_empty() {
        let tree = new_aabb_tree2();
        assert_eq!(aabb2_node_count(&tree), 0);
    }

    #[test]
    fn test_insert_returns_index() {
        let mut tree = new_aabb_tree2();
        let idx = aabb2_insert(&mut tree, [0.0, 0.0], [1.0, 1.0], 42);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_node_count_after_insert() {
        let mut tree = new_aabb_tree2();
        aabb2_insert(&mut tree, [0.0, 0.0], [1.0, 1.0], 1);
        aabb2_insert(&mut tree, [2.0, 2.0], [3.0, 3.0], 2);
        assert_eq!(aabb2_node_count(&tree), 2);
    }

    #[test]
    fn test_query_point_hit() {
        let mut tree = new_aabb_tree2();
        aabb2_insert(&mut tree, [0.0, 0.0], [2.0, 2.0], 7);
        let hits = aabb2_query_point(&tree, [1.0, 1.0]);
        assert!(hits.contains(&7));
    }

    #[test]
    fn test_query_point_miss() {
        let mut tree = new_aabb_tree2();
        aabb2_insert(&mut tree, [0.0, 0.0], [1.0, 1.0], 5);
        let hits = aabb2_query_point(&tree, [5.0, 5.0]);
        assert!(hits.is_empty());
    }

    #[test]
    fn test_query_point_boundary() {
        let mut tree = new_aabb_tree2();
        aabb2_insert(&mut tree, [0.0, 0.0], [1.0, 1.0], 3);
        let hits = aabb2_query_point(&tree, [1.0, 1.0]);
        assert!(hits.contains(&3));
    }

    #[test]
    fn test_overlaps_true() {
        assert!(aabb2_overlaps([0.0, 0.0], [2.0, 2.0], [1.0, 1.0], [3.0, 3.0]));
    }

    #[test]
    fn test_overlaps_false() {
        assert!(!aabb2_overlaps([0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]));
    }

    #[test]
    fn test_overlaps_touching() {
        assert!(aabb2_overlaps([0.0, 0.0], [1.0, 1.0], [1.0, 0.0], [2.0, 1.0]));
    }

    #[test]
    fn test_multiple_hits() {
        let mut tree = new_aabb_tree2();
        aabb2_insert(&mut tree, [0.0, 0.0], [3.0, 3.0], 10);
        aabb2_insert(&mut tree, [1.0, 1.0], [4.0, 4.0], 11);
        aabb2_insert(&mut tree, [5.0, 5.0], [6.0, 6.0], 12);
        let hits = aabb2_query_point(&tree, [2.0, 2.0]);
        assert_eq!(hits.len(), 2);
        assert!(hits.contains(&10));
        assert!(hits.contains(&11));
    }
}
