// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Octree-based voxel structure for a mesh.

/// Axis-aligned bounding box.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct VoxAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// A single node in the voxel octree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxOctNode {
    pub bounds: VoxAabb,
    /// Whether this node contains geometry.
    pub occupied: bool,
    /// Child indices (up to 8); empty vec = leaf.
    pub children: Vec<usize>,
    pub depth: u32,
}

/// An octree voxel structure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VoxOctree {
    pub nodes: Vec<VoxOctNode>,
    pub max_depth: u32,
}

/// Compute the AABB of a set of positions.
#[allow(dead_code)]
pub fn compute_aabb_vo(positions: &[[f32; 3]]) -> Option<VoxAabb> {
    if positions.is_empty() {
        return None;
    }
    let mut mn = positions[0];
    let mut mx = positions[0];
    for &p in positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    Some(VoxAabb { min: mn, max: mx })
}

/// Split an AABB into 8 children.
#[allow(dead_code)]
pub fn split_aabb(aabb: VoxAabb) -> [VoxAabb; 8] {
    let mid = [
        (aabb.min[0] + aabb.max[0]) * 0.5,
        (aabb.min[1] + aabb.max[1]) * 0.5,
        (aabb.min[2] + aabb.max[2]) * 0.5,
    ];
    let mut children = [VoxAabb {
        min: [0.0; 3],
        max: [0.0; 3],
    }; 8];
    #[allow(clippy::needless_range_loop)]
    for i in 0..8usize {
        let xi = (i & 1) != 0;
        let yi = (i & 2) != 0;
        let zi = (i & 4) != 0;
        children[i].min[0] = if xi { mid[0] } else { aabb.min[0] };
        children[i].min[1] = if yi { mid[1] } else { aabb.min[1] };
        children[i].min[2] = if zi { mid[2] } else { aabb.min[2] };
        children[i].max[0] = if xi { aabb.max[0] } else { mid[0] };
        children[i].max[1] = if yi { aabb.max[1] } else { mid[1] };
        children[i].max[2] = if zi { aabb.max[2] } else { mid[2] };
    }
    children
}

/// Check if a point is inside an AABB.
#[allow(dead_code)]
pub fn point_in_aabb(p: [f32; 3], aabb: VoxAabb) -> bool {
    (0..3).all(|k| p[k] >= aabb.min[k] && p[k] <= aabb.max[k])
}

/// Build an octree by recursively subdividing until `max_depth`.
#[allow(dead_code)]
pub fn build_vox_octree(positions: &[[f32; 3]], max_depth: u32) -> VoxOctree {
    let Some(root_aabb) = compute_aabb_vo(positions) else {
        return VoxOctree {
            nodes: vec![],
            max_depth,
        };
    };
    let mut nodes: Vec<VoxOctNode> = Vec::new();
    build_node(&mut nodes, positions, root_aabb, 0, max_depth);
    VoxOctree { nodes, max_depth }
}

fn build_node(
    nodes: &mut Vec<VoxOctNode>,
    positions: &[[f32; 3]],
    bounds: VoxAabb,
    depth: u32,
    max_depth: u32,
) -> usize {
    let occupied = positions.iter().any(|&p| point_in_aabb(p, bounds));
    let idx = nodes.len();
    nodes.push(VoxOctNode {
        bounds,
        occupied,
        children: Vec::new(),
        depth,
    });
    if depth < max_depth && occupied {
        let children_aabb = split_aabb(bounds);
        let mut child_indices = Vec::new();
        for child_aabb in &children_aabb {
            let ci = build_node(nodes, positions, *child_aabb, depth + 1, max_depth);
            child_indices.push(ci);
        }
        nodes[idx].children = child_indices;
    }
    idx
}

/// Count occupied leaf nodes.
#[allow(dead_code)]
pub fn occupied_leaf_count(tree: &VoxOctree) -> usize {
    tree.nodes
        .iter()
        .filter(|n| n.children.is_empty() && n.occupied)
        .count()
}

/// Count total nodes.
#[allow(dead_code)]
pub fn node_count_vo(tree: &VoxOctree) -> usize {
    tree.nodes.len()
}

/// Serialise tree metadata to JSON.
#[allow(dead_code)]
pub fn vox_octree_to_json(tree: &VoxOctree) -> String {
    format!(
        "{{\"node_count\":{},\"max_depth\":{},\"occupied_leaves\":{}}}",
        node_count_vo(tree),
        tree.max_depth,
        occupied_leaf_count(tree)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn four_points() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn test_compute_aabb() {
        let pts = four_points();
        let aabb = compute_aabb_vo(&pts).unwrap();
        assert!((aabb.max[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_aabb_empty() {
        assert!(compute_aabb_vo(&[]).is_none());
    }

    #[test]
    fn test_split_aabb_eight_children() {
        let aabb = VoxAabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        let children = split_aabb(aabb);
        assert_eq!(children.len(), 8);
    }

    #[test]
    fn test_point_in_aabb_inside() {
        let aabb = VoxAabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        assert!(point_in_aabb([0.5, 0.5, 0.5], aabb));
    }

    #[test]
    fn test_point_in_aabb_outside() {
        let aabb = VoxAabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        assert!(!point_in_aabb([2.0, 0.5, 0.5], aabb));
    }

    #[test]
    fn test_build_octree_nonempty() {
        let pts = four_points();
        let tree = build_vox_octree(&pts, 2);
        assert!(node_count_vo(&tree) > 0);
    }

    #[test]
    fn test_occupied_leaf_count_positive() {
        let pts = four_points();
        let tree = build_vox_octree(&pts, 2);
        assert!(occupied_leaf_count(&tree) > 0);
    }

    #[test]
    fn test_empty_input_tree() {
        let tree = build_vox_octree(&[], 3);
        assert_eq!(node_count_vo(&tree), 0);
    }

    #[test]
    fn test_vox_octree_to_json() {
        let pts = four_points();
        let tree = build_vox_octree(&pts, 1);
        let j = vox_octree_to_json(&tree);
        assert!(j.contains("node_count"));
    }

    #[test]
    fn test_split_aabb_coverage() {
        let aabb = VoxAabb {
            min: [0.0; 3],
            max: [2.0; 3],
        };
        let ch = split_aabb(aabb);
        // all children have max >= min
        for c in &ch {
            for k in 0..3 {
                assert!(c.max[k] >= c.min[k]);
            }
        }
    }
}
