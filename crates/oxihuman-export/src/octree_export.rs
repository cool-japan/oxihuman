// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Octree-encoded point cloud export stub.

/// An octree node for point cloud storage.
#[allow(dead_code)]
pub struct OctreeExportNode {
    pub center: [f32; 3],
    pub half_size: f32,
    pub point_indices: Vec<usize>,
    pub children: Vec<usize>,
}

/// Octree point cloud export.
#[allow(dead_code)]
pub struct OctreePointCloudExport {
    pub nodes: Vec<OctreeExportNode>,
    pub points: Vec<[f32; 3]>,
    pub max_depth: u32,
    pub max_points_per_node: usize,
}

/// Create a new octree export.
#[allow(dead_code)]
pub fn new_octree_export(max_depth: u32, max_points_per_node: usize) -> OctreePointCloudExport {
    OctreePointCloudExport {
        nodes: Vec::new(),
        points: Vec::new(),
        max_depth,
        max_points_per_node,
    }
}

/// Add a root node.
#[allow(dead_code)]
pub fn add_octree_root(export: &mut OctreePointCloudExport, center: [f32; 3], half_size: f32) -> usize {
    let idx = export.nodes.len();
    export.nodes.push(OctreeExportNode { center, half_size, point_indices: Vec::new(), children: Vec::new() });
    idx
}

/// Check if a point is inside a node's AABB.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn point_in_node(p: [f32; 3], node: &OctreeExportNode) -> bool {
    for i in 0..3 {
        if (p[i] - node.center[i]).abs() > node.half_size { return false; }
    }
    true
}

/// Point count.
#[allow(dead_code)]
pub fn octree_point_count(export: &OctreePointCloudExport) -> usize {
    export.points.len()
}

/// Node count.
#[allow(dead_code)]
pub fn octree_node_count_export(export: &OctreePointCloudExport) -> usize {
    export.nodes.len()
}

/// Build a simple flat octree for a set of points.
#[allow(dead_code)]
pub fn build_octree_export(points: &[[f32; 3]], max_depth: u32, max_per_node: usize) -> OctreePointCloudExport {
    let mut e = new_octree_export(max_depth, max_per_node);
    e.points = points.to_vec();

    if points.is_empty() {
        return e;
    }

    let mut mn = points[0];
    let mut mx = points[0];
    for &p in points.iter().skip(1) {
        for i in 0..3 {
            if p[i] < mn[i] { mn[i] = p[i]; }
            if p[i] > mx[i] { mx[i] = p[i]; }
        }
    }
    let center = [(mn[0]+mx[0])*0.5, (mn[1]+mx[1])*0.5, (mn[2]+mx[2])*0.5];
    let half = ((mx[0]-mn[0]).max(mx[1]-mn[1]).max(mx[2]-mn[2])) * 0.5 + 1e-4;

    let root = add_octree_root(&mut e, center, half);
    for (i, &p) in points.iter().enumerate() {
        if point_in_node(p, &e.nodes[root]) {
            e.nodes[root].point_indices.push(i);
        }
    }
    e
}

/// Leaf node count (nodes with no children).
#[allow(dead_code)]
pub fn octree_leaf_count_export(export: &OctreePointCloudExport) -> usize {
    export.nodes.iter().filter(|n| n.children.is_empty()).count()
}

/// Export to JSON stub.
#[allow(dead_code)]
pub fn octree_to_json(export: &OctreePointCloudExport) -> String {
    format!(
        "{{\"nodes\":{},\"points\":{},\"max_depth\":{}}}",
        export.nodes.len(), export.points.len(), export.max_depth
    )
}

/// Estimate file size.
#[allow(dead_code)]
pub fn octree_export_size_estimate(point_count: usize, node_count: usize) -> usize {
    point_count * 12 + node_count * 64
}

/// Validate.
#[allow(dead_code)]
pub fn validate_octree_export(export: &OctreePointCloudExport) -> bool {
    !export.nodes.is_empty() && !export.points.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cloud() -> Vec<[f32; 3]> {
        vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,0.5]]
    }

    #[test]
    fn new_empty() {
        let e = new_octree_export(8, 10);
        assert_eq!(octree_point_count(&e), 0);
    }

    #[test]
    fn build_octree_point_count() {
        let pts = cloud();
        let e = build_octree_export(&pts, 4, 8);
        assert_eq!(octree_point_count(&e), 4);
    }

    #[test]
    fn build_octree_has_root() {
        let pts = cloud();
        let e = build_octree_export(&pts, 4, 8);
        assert!(octree_node_count_export(&e) >= 1);
    }

    #[test]
    fn point_in_node_test() {
        let node = OctreeExportNode { center: [0.0,0.0,0.0], half_size: 1.0, point_indices: vec![], children: vec![] };
        assert!(point_in_node([0.5,0.5,0.5], &node));
        assert!(!point_in_node([2.0,0.0,0.0], &node));
    }

    #[test]
    fn leaf_count() {
        let pts = cloud();
        let e = build_octree_export(&pts, 4, 8);
        let lc = octree_leaf_count_export(&e);
        assert!(lc >= 1);
    }

    #[test]
    fn json_contains_fields() {
        let pts = cloud();
        let e = build_octree_export(&pts, 4, 8);
        let j = octree_to_json(&e);
        assert!(j.contains("nodes"));
        assert!(j.contains("points"));
    }

    #[test]
    fn validate_passes() {
        let pts = cloud();
        let e = build_octree_export(&pts, 4, 8);
        assert!(validate_octree_export(&e));
    }

    #[test]
    fn size_estimate() {
        assert!(octree_export_size_estimate(100, 10) > 0);
    }

    #[test]
    fn empty_cloud_no_nodes() {
        let e = build_octree_export(&[], 4, 8);
        assert!(!validate_octree_export(&e));
    }
}
