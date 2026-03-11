// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Bounding box tree (AABB tree) for spatial queries on mesh faces.

/// An axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BboxNode {
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Index of the face, or usize::MAX for internal nodes.
    pub face_index: usize,
    pub left: usize,
    pub right: usize,
}

/// AABB tree structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BboxTree {
    pub nodes: Vec<BboxNode>,
    pub root: usize,
}

/// Compute AABB for a single triangle.
#[allow(dead_code)]
pub fn triangle_bbox(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let min = [
        v0[0].min(v1[0]).min(v2[0]),
        v0[1].min(v1[1]).min(v2[1]),
        v0[2].min(v1[2]).min(v2[2]),
    ];
    let max = [
        v0[0].max(v1[0]).max(v2[0]),
        v0[1].max(v1[1]).max(v2[1]),
        v0[2].max(v1[2]).max(v2[2]),
    ];
    (min, max)
}

/// Merge two AABBs.
#[allow(dead_code)]
pub fn merge_bbox(a: ([f32; 3], [f32; 3]), b: ([f32; 3], [f32; 3])) -> ([f32; 3], [f32; 3]) {
    let min = [a.0[0].min(b.0[0]), a.0[1].min(b.0[1]), a.0[2].min(b.0[2])];
    let max = [a.1[0].max(b.1[0]), a.1[1].max(b.1[1]), a.1[2].max(b.1[2])];
    (min, max)
}

/// Build a simple AABB tree from mesh triangles (flat list, no recursive).
#[allow(dead_code)]
pub fn build_bbox_tree(positions: &[[f32; 3]], indices: &[u32]) -> BboxTree {
    let tri_count = indices.len() / 3;
    let mut nodes = Vec::with_capacity(tri_count);

    for t in 0..tri_count {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let (min, max) = triangle_bbox(positions[i0], positions[i1], positions[i2]);
        nodes.push(BboxNode {
            min,
            max,
            face_index: t,
            left: usize::MAX,
            right: usize::MAX,
        });
    }

    let root = if nodes.is_empty() { usize::MAX } else { 0 };
    BboxTree { nodes, root }
}

/// Check if a point is inside an AABB.
#[allow(dead_code)]
pub fn point_in_bbox(p: [f32; 3], min: [f32; 3], max: [f32; 3]) -> bool {
    p[0] >= min[0]
        && p[0] <= max[0]
        && p[1] >= min[1]
        && p[1] <= max[1]
        && p[2] >= min[2]
        && p[2] <= max[2]
}

/// Check if two AABBs overlap.
#[allow(dead_code)]
pub fn bbox_overlap(a_min: [f32; 3], a_max: [f32; 3], b_min: [f32; 3], b_max: [f32; 3]) -> bool {
    a_min[0] <= b_max[0]
        && a_max[0] >= b_min[0]
        && a_min[1] <= b_max[1]
        && a_max[1] >= b_min[1]
        && a_min[2] <= b_max[2]
        && a_max[2] >= b_min[2]
}

/// Node count in tree.
#[allow(dead_code)]
pub fn node_count(tree: &BboxTree) -> usize {
    tree.nodes.len()
}

/// Compute volume of an AABB.
#[allow(dead_code)]
pub fn bbox_volume(min: [f32; 3], max: [f32; 3]) -> f32 {
    (max[0] - min[0]) * (max[1] - min[1]) * (max[2] - min[2])
}

/// Convert tree stats to JSON.
#[allow(dead_code)]
pub fn bbox_tree_to_json(tree: &BboxTree) -> String {
    format!("{{\"node_count\":{}}}", tree.nodes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_bbox() {
        let (min, max) = triangle_bbox([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((min[0]).abs() < 1e-9);
        assert!((max[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_merge_bbox() {
        let a = ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = ([-1.0, -1.0, -1.0], [0.5, 0.5, 0.5]);
        let (min, max) = merge_bbox(a, b);
        assert!((min[0] - (-1.0)).abs() < 1e-6);
        assert!((max[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_build_tree() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let tree = build_bbox_tree(&pos, &idx);
        assert_eq!(node_count(&tree), 1);
    }

    #[test]
    fn test_build_empty() {
        let tree = build_bbox_tree(&[], &[]);
        assert_eq!(node_count(&tree), 0);
        assert_eq!(tree.root, usize::MAX);
    }

    #[test]
    fn test_point_in_bbox() {
        assert!(point_in_bbox(
            [0.5, 0.5, 0.5],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0]
        ));
        assert!(!point_in_bbox(
            [1.5, 0.5, 0.5],
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0]
        ));
    }

    #[test]
    fn test_bbox_overlap() {
        assert!(bbox_overlap(
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.5, 0.5, 0.5],
            [1.5, 1.5, 1.5]
        ));
        assert!(!bbox_overlap(
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0]
        ));
    }

    #[test]
    fn test_bbox_volume() {
        let v = bbox_volume([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        assert!((v - 24.0).abs() < 1e-6);
    }

    #[test]
    fn test_node_face_index() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let tree = build_bbox_tree(&pos, &idx);
        assert_eq!(tree.nodes[0].face_index, 0);
    }

    #[test]
    fn test_bbox_tree_to_json() {
        let tree = BboxTree {
            nodes: vec![],
            root: usize::MAX,
        };
        let j = bbox_tree_to_json(&tree);
        assert!(j.contains("\"node_count\":0"));
    }

    #[test]
    fn test_multi_triangle_tree() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 1, 3, 2];
        let tree = build_bbox_tree(&pos, &idx);
        assert_eq!(node_count(&tree), 2);
    }
}
