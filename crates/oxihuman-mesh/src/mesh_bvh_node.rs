// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! BVH node construction and traversal for triangle meshes.

/// Axis-aligned bounding box for BVH.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct BvhNodeAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// A single BVH node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BvhNode2 {
    pub aabb: BvhNodeAabb,
    /// Leaf face indices, empty if internal.
    pub face_indices: Vec<usize>,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

/// Compute AABB for a list of triangle indices.
#[allow(dead_code)]
pub fn triangle_aabb(positions: &[[f32; 3]], indices: &[u32], face: usize) -> BvhNodeAabb {
    let base = face * 3;
    let i0 = indices[base] as usize;
    let i1 = indices[base + 1] as usize;
    let i2 = indices[base + 2] as usize;
    let verts = [positions[i0], positions[i1], positions[i2]];
    let mut mn = verts[0];
    let mut mx = verts[0];
    for &v in &verts {
        for k in 0..3 {
            if v[k] < mn[k] {
                mn[k] = v[k];
            }
            if v[k] > mx[k] {
                mx[k] = v[k];
            }
        }
    }
    BvhNodeAabb { min: mn, max: mx }
}

/// Merge two AABBs.
#[allow(dead_code)]
pub fn merge_node_aabb(a: BvhNodeAabb, b: BvhNodeAabb) -> BvhNodeAabb {
    let mut mn = a.min;
    let mut mx = a.max;
    for k in 0..3 {
        if b.min[k] < mn[k] {
            mn[k] = b.min[k];
        }
        if b.max[k] > mx[k] {
            mx[k] = b.max[k];
        }
    }
    BvhNodeAabb { min: mn, max: mx }
}

/// Compute AABB surface area (SAH heuristic).
#[allow(dead_code)]
pub fn aabb_surface_area(aabb: BvhNodeAabb) -> f32 {
    let e = [
        aabb.max[0] - aabb.min[0],
        aabb.max[1] - aabb.min[1],
        aabb.max[2] - aabb.min[2],
    ];
    2.0 * (e[0] * e[1] + e[1] * e[2] + e[0] * e[2])
}

/// Check if a ray hits an AABB (slab method).
#[allow(dead_code)]
pub fn ray_aabb_hit(origin: [f32; 3], inv_dir: [f32; 3], aabb: BvhNodeAabb) -> bool {
    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    for k in 0..3 {
        let t0 = (aabb.min[k] - origin[k]) * inv_dir[k];
        let t1 = (aabb.max[k] - origin[k]) * inv_dir[k];
        let (lo, hi) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
        t_min = t_min.max(lo);
        t_max = t_max.min(hi);
        if t_max < t_min {
            return false;
        }
    }
    t_max >= 0.0
}

/// Build a flat BVH from triangle centroids using a simple median split.
#[allow(dead_code)]
pub fn build_bvh_nodes(positions: &[[f32; 3]], indices: &[u32]) -> Vec<BvhNode2> {
    let tri_count = indices.len() / 3;
    let face_list: Vec<usize> = (0..tri_count).collect();
    let mut nodes = Vec::new();
    build_recursive(positions, indices, &face_list, &mut nodes);
    nodes
}

fn build_recursive(
    positions: &[[f32; 3]],
    indices: &[u32],
    face_list: &[usize],
    nodes: &mut Vec<BvhNode2>,
) -> usize {
    let idx = nodes.len();
    if face_list.is_empty() {
        nodes.push(BvhNode2 {
            aabb: BvhNodeAabb {
                min: [0.0; 3],
                max: [0.0; 3],
            },
            face_indices: vec![],
            left: None,
            right: None,
        });
        return idx;
    }
    // compute merged AABB
    let aabb = face_list.iter().fold(
        triangle_aabb(positions, indices, face_list[0]),
        |acc, &f| merge_node_aabb(acc, triangle_aabb(positions, indices, f)),
    );
    if face_list.len() <= 4 {
        nodes.push(BvhNode2 {
            aabb,
            face_indices: face_list.to_vec(),
            left: None,
            right: None,
        });
        return idx;
    }
    // split along longest axis by median
    let size = [
        aabb.max[0] - aabb.min[0],
        aabb.max[1] - aabb.min[1],
        aabb.max[2] - aabb.min[2],
    ];
    let axis = if size[0] >= size[1] && size[0] >= size[2] {
        0
    } else if size[1] >= size[2] {
        1
    } else {
        2
    };
    let mut sorted = face_list.to_vec();
    sorted.sort_by(|&a, &b| {
        let ca = triangle_centroid_axis(positions, indices, a, axis);
        let cb = triangle_centroid_axis(positions, indices, b, axis);
        ca.partial_cmp(&cb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = sorted.len() / 2;
    nodes.push(BvhNode2 {
        aabb,
        face_indices: vec![],
        left: None,
        right: None,
    });
    let left = build_recursive(positions, indices, &sorted[..mid], nodes);
    let right = build_recursive(positions, indices, &sorted[mid..], nodes);
    nodes[idx].left = Some(left);
    nodes[idx].right = Some(right);
    idx
}

fn triangle_centroid_axis(
    positions: &[[f32; 3]],
    indices: &[u32],
    face: usize,
    axis: usize,
) -> f32 {
    let base = face * 3;
    let i0 = indices[base] as usize;
    let i1 = indices[base + 1] as usize;
    let i2 = indices[base + 2] as usize;
    (positions[i0][axis] + positions[i1][axis] + positions[i2][axis]) / 3.0
}

/// Count nodes in the BVH.
#[allow(dead_code)]
pub fn bvh_node_count(nodes: &[BvhNode2]) -> usize {
    nodes.len()
}

/// Count leaf nodes.
#[allow(dead_code)]
pub fn bvh_leaf_count(nodes: &[BvhNode2]) -> usize {
    nodes
        .iter()
        .filter(|n| n.left.is_none() && n.right.is_none())
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.5, 1.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        (pos, idx)
    }

    #[test]
    fn test_triangle_aabb_valid() {
        let (pos, idx) = two_tris();
        let aabb = triangle_aabb(&pos, &idx, 0);
        assert!(aabb.max[0] >= aabb.min[0]);
    }

    #[test]
    fn test_merge_node_aabb() {
        let a = BvhNodeAabb {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        };
        let b = BvhNodeAabb {
            min: [-1.0, -1.0, -1.0],
            max: [0.5, 0.5, 0.5],
        };
        let m = merge_node_aabb(a, b);
        assert!((m.min[0] - (-1.0)).abs() < 1e-6);
        assert!((m.max[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_surface_area_positive() {
        let aabb = BvhNodeAabb {
            min: [0.0; 3],
            max: [1.0, 2.0, 3.0],
        };
        assert!(aabb_surface_area(aabb) > 0.0);
    }

    #[test]
    fn test_ray_aabb_hit_true() {
        let aabb = BvhNodeAabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        let origin = [0.5, 0.5, -1.0];
        let inv_dir = [f32::INFINITY, f32::INFINITY, 1.0];
        assert!(ray_aabb_hit(origin, inv_dir, aabb));
    }

    #[test]
    fn test_ray_aabb_hit_miss() {
        let aabb = BvhNodeAabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        let origin = [5.0, 0.5, -1.0];
        let inv_dir = [0.0, 0.0, 1.0];
        assert!(!ray_aabb_hit(origin, inv_dir, aabb));
    }

    #[test]
    fn test_build_bvh_node_count_positive() {
        let (pos, idx) = two_tris();
        let nodes = build_bvh_nodes(&pos, &idx);
        assert!(bvh_node_count(&nodes) > 0);
    }

    #[test]
    fn test_bvh_leaf_count_positive() {
        let (pos, idx) = two_tris();
        let nodes = build_bvh_nodes(&pos, &idx);
        assert!(bvh_leaf_count(&nodes) > 0);
    }

    #[test]
    fn test_build_bvh_empty() {
        let nodes = build_bvh_nodes(&[], &[]);
        assert_eq!(bvh_node_count(&nodes), 1);
    }

    #[test]
    fn test_aabb_surface_area_unit_cube() {
        let aabb = BvhNodeAabb {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        assert!((aabb_surface_area(aabb) - 6.0).abs() < 1e-5);
    }
}
