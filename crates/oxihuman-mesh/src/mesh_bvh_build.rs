#![allow(dead_code)]
//! BVH tree construction for triangle meshes.

/// A BVH node.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BvhNode {
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub triangle_index: Option<usize>,
}

/// A BVH tree.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BvhTree {
    pub nodes: Vec<BvhNode>,
}

fn tri_aabb(positions: &[[f32; 3]], tri: &[u32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let mn = [
        a[0].min(b[0]).min(c[0]),
        a[1].min(b[1]).min(c[1]),
        a[2].min(b[2]).min(c[2]),
    ];
    let mx = [
        a[0].max(b[0]).max(c[0]),
        a[1].max(b[1]).max(c[1]),
        a[2].max(b[2]).max(c[2]),
    ];
    (mn, mx)
}

fn tri_centroid(positions: &[[f32; 3]], tri: &[u32; 3]) -> [f32; 3] {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    [
        (a[0] + b[0] + c[0]) / 3.0,
        (a[1] + b[1] + c[1]) / 3.0,
        (a[2] + b[2] + c[2]) / 3.0,
    ]
}

/// Build a BVH tree from positions and triangle indices.
#[allow(dead_code)]
pub fn build_bvh_tree(positions: &[[f32; 3]], indices: &[[u32; 3]]) -> BvhTree {
    let mut tree = BvhTree { nodes: Vec::new() };
    if indices.is_empty() {
        return tree;
    }
    let mut tri_indices: Vec<usize> = (0..indices.len()).collect();
    build_recursive(positions, indices, &mut tri_indices, &mut tree);
    tree
}

fn build_recursive(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    tri_indices: &mut [usize],
    tree: &mut BvhTree,
) -> usize {
    if tri_indices.len() == 1 {
        let ti = tri_indices[0];
        let (mn, mx) = tri_aabb(positions, &indices[ti]);
        let node = BvhNode {
            aabb_min: mn,
            aabb_max: mx,
            left: None,
            right: None,
            triangle_index: Some(ti),
        };
        tree.nodes.push(node);
        return tree.nodes.len() - 1;
    }

    // Compute overall AABB
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for &ti in tri_indices.iter() {
        let (tmn, tmx) = tri_aabb(positions, &indices[ti]);
        for j in 0..3 {
            mn[j] = mn[j].min(tmn[j]);
            mx[j] = mx[j].max(tmx[j]);
        }
    }

    // Split along longest axis
    let extent = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
    let axis = if extent[0] >= extent[1] && extent[0] >= extent[2] {
        0
    } else if extent[1] >= extent[2] {
        1
    } else {
        2
    };

    tri_indices.sort_by(|&a, &b| {
        let ca = tri_centroid(positions, &indices[a]);
        let cb = tri_centroid(positions, &indices[b]);
        ca[axis].partial_cmp(&cb[axis]).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mid = tri_indices.len() / 2;
    let (left_slice, right_slice) = tri_indices.split_at_mut(mid);

    let left = build_recursive(positions, indices, left_slice, tree);
    let right = build_recursive(positions, indices, right_slice, tree);

    let node = BvhNode {
        aabb_min: mn,
        aabb_max: mx,
        left: Some(left),
        right: Some(right),
        triangle_index: None,
    };
    tree.nodes.push(node);
    tree.nodes.len() - 1
}

/// Return total node count.
#[allow(dead_code)]
pub fn bvh_node_count(tree: &BvhTree) -> usize {
    tree.nodes.len()
}

/// Return leaf count.
#[allow(dead_code)]
pub fn bvh_leaf_count(tree: &BvhTree) -> usize {
    tree.nodes.iter().filter(|n| n.triangle_index.is_some()).count()
}

/// Return tree depth.
#[allow(dead_code)]
pub fn bvh_depth(tree: &BvhTree) -> usize {
    if tree.nodes.is_empty() {
        return 0;
    }
    fn depth_at(nodes: &[BvhNode], idx: usize) -> usize {
        let node = &nodes[idx];
        let ld = node.left.map_or(0, |l| depth_at(nodes, l));
        let rd = node.right.map_or(0, |r| depth_at(nodes, r));
        1 + ld.max(rd)
    }
    depth_at(&tree.nodes, tree.nodes.len() - 1)
}

/// Traverse the tree calling a visitor on each node.
#[allow(dead_code)]
pub fn bvh_traverse(tree: &BvhTree, visitor: &mut dyn FnMut(usize, &BvhNode)) {
    if tree.nodes.is_empty() {
        return;
    }
    fn visit(nodes: &[BvhNode], idx: usize, visitor: &mut dyn FnMut(usize, &BvhNode)) {
        visitor(idx, &nodes[idx]);
        if let Some(l) = nodes[idx].left {
            visit(nodes, l, visitor);
        }
        if let Some(r) = nodes[idx].right {
            visit(nodes, r, visitor);
        }
    }
    visit(&tree.nodes, tree.nodes.len() - 1, visitor);
}

/// Simple ray-AABB intersection test, returns closest triangle index or None.
#[allow(dead_code)]
pub fn bvh_ray_intersect(
    tree: &BvhTree,
    origin: [f32; 3],
    direction: [f32; 3],
) -> Option<usize> {
    if tree.nodes.is_empty() {
        return None;
    }
    let inv_dir = [
        if direction[0].abs() > 1e-10 { 1.0 / direction[0] } else { 1e10 },
        if direction[1].abs() > 1e-10 { 1.0 / direction[1] } else { 1e10 },
        if direction[2].abs() > 1e-10 { 1.0 / direction[2] } else { 1e10 },
    ];

    fn search(
        nodes: &[BvhNode],
        idx: usize,
        origin: &[f32; 3],
        inv_dir: &[f32; 3],
    ) -> Option<usize> {
        let node = &nodes[idx];
        if !ray_aabb_hit(origin, inv_dir, &node.aabb_min, &node.aabb_max) {
            return None;
        }
        if let Some(ti) = node.triangle_index {
            return Some(ti);
        }
        let l = node.left.and_then(|l| search(nodes, l, origin, inv_dir));
        if l.is_some() {
            return l;
        }
        node.right.and_then(|r| search(nodes, r, origin, inv_dir))
    }

    fn ray_aabb_hit(origin: &[f32; 3], inv_dir: &[f32; 3], mn: &[f32; 3], mx: &[f32; 3]) -> bool {
        let mut tmin = f32::MIN;
        let mut tmax = f32::MAX;
        for i in 0..3 {
            let t1 = (mn[i] - origin[i]) * inv_dir[i];
            let t2 = (mx[i] - origin[i]) * inv_dir[i];
            tmin = tmin.max(t1.min(t2));
            tmax = tmax.min(t1.max(t2));
        }
        tmax >= tmin.max(0.0)
    }

    search(&tree.nodes, tree.nodes.len() - 1, &origin, &inv_dir)
}

/// Query all triangles whose AABB overlaps a given AABB.
#[allow(dead_code)]
pub fn bvh_aabb_query(tree: &BvhTree, query_min: [f32; 3], query_max: [f32; 3]) -> Vec<usize> {
    let mut results = Vec::new();
    if tree.nodes.is_empty() {
        return results;
    }

    fn overlaps(amn: &[f32; 3], amx: &[f32; 3], bmn: &[f32; 3], bmx: &[f32; 3]) -> bool {
        amn[0] <= bmx[0] && amx[0] >= bmn[0]
            && amn[1] <= bmx[1] && amx[1] >= bmn[1]
            && amn[2] <= bmx[2] && amx[2] >= bmn[2]
    }

    fn collect(
        nodes: &[BvhNode],
        idx: usize,
        qmn: &[f32; 3],
        qmx: &[f32; 3],
        results: &mut Vec<usize>,
    ) {
        let node = &nodes[idx];
        if !overlaps(&node.aabb_min, &node.aabb_max, qmn, qmx) {
            return;
        }
        if let Some(ti) = node.triangle_index {
            results.push(ti);
            return;
        }
        if let Some(l) = node.left {
            collect(nodes, l, qmn, qmx, results);
        }
        if let Some(r) = node.right {
            collect(nodes, r, qmn, qmx, results);
        }
    }

    collect(&tree.nodes, tree.nodes.len() - 1, &query_min, &query_max, &mut results);
    results
}

/// Find the closest triangle to a point (by centroid distance).
#[allow(dead_code)]
pub fn bvh_closest_triangle(
    tree: &BvhTree,
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    point: [f32; 3],
) -> Option<usize> {
    let leaves: Vec<usize> = tree
        .nodes
        .iter()
        .filter_map(|n| n.triangle_index)
        .collect();
    if leaves.is_empty() {
        return None;
    }
    let mut best = f32::MAX;
    let mut best_idx = 0;
    for &ti in &leaves {
        let c = tri_centroid(positions, &indices[ti]);
        let dx = c[0] - point[0];
        let dy = c[1] - point[1];
        let dz = c[2] - point[2];
        let d = dx * dx + dy * dy + dz * dz;
        if d < best {
            best = d;
            best_idx = ti;
        }
    }
    Some(best_idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (
            vec![
                [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [2.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        )
    }

    #[test]
    fn test_build_bvh_tree() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        assert!(bvh_node_count(&tree) > 0);
    }

    #[test]
    fn test_bvh_leaf_count() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        assert_eq!(bvh_leaf_count(&tree), 2);
    }

    #[test]
    fn test_bvh_depth() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        assert!(bvh_depth(&tree) >= 1);
    }

    #[test]
    fn test_bvh_traverse() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        let mut count = 0;
        bvh_traverse(&tree, &mut |_, _| count += 1);
        assert_eq!(count, bvh_node_count(&tree));
    }

    #[test]
    fn test_bvh_ray_intersect() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        let result = bvh_ray_intersect(&tree, [0.3, 0.3, -1.0], [0.0, 0.0, 1.0]);
        assert!(result.is_some());
    }

    #[test]
    fn test_bvh_aabb_query() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        let results = bvh_aabb_query(&tree, [-1.0; 3], [4.0, 2.0, 1.0]);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_bvh_closest_triangle() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        let closest = bvh_closest_triangle(&tree, &p, &i, [0.0, 0.0, 0.0]);
        assert!(closest.is_some());
    }

    #[test]
    fn test_empty_bvh() {
        let tree = build_bvh_tree(&[], &[]);
        assert_eq!(bvh_node_count(&tree), 0);
        assert_eq!(bvh_depth(&tree), 0);
    }

    #[test]
    fn test_single_triangle_bvh() {
        let p = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let i = vec![[0, 1, 2]];
        let tree = build_bvh_tree(&p, &i);
        assert_eq!(bvh_leaf_count(&tree), 1);
    }

    #[test]
    fn test_bvh_ray_miss() {
        let (p, i) = sample();
        let tree = build_bvh_tree(&p, &i);
        let result = bvh_ray_intersect(&tree, [100.0, 100.0, 100.0], [0.0, 0.0, 1.0]);
        assert!(result.is_none());
    }
}
