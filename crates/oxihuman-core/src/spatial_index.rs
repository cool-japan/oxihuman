// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

/// A node in the octree (either internal or leaf).
#[derive(Debug)]
pub struct OctreeNode {
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    /// Indices into `Octree::points` for points in this leaf node.
    pub point_indices: Vec<usize>,
    /// Eight children if this is an internal node.
    pub children: Option<Box<[OctreeNode; 8]>>,
}

/// A spatial octree over a set of 3-D points.
#[derive(Debug)]
pub struct Octree {
    pub root: OctreeNode,
    pub points: Vec<[f32; 3]>,
    pub max_depth: u32,
    pub max_points_per_leaf: usize,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn aabb_contains(min: &[f32; 3], max: &[f32; 3], p: &[f32; 3]) -> bool {
    p[0] >= min[0]
        && p[0] <= max[0]
        && p[1] >= min[1]
        && p[1] <= max[1]
        && p[2] >= min[2]
        && p[2] <= max[2]
}

fn aabb_overlaps_aabb(amin: &[f32; 3], amax: &[f32; 3], bmin: &[f32; 3], bmax: &[f32; 3]) -> bool {
    amin[0] <= bmax[0]
        && amax[0] >= bmin[0]
        && amin[1] <= bmax[1]
        && amax[1] >= bmin[1]
        && amin[2] <= bmax[2]
        && amax[2] >= bmin[2]
}

fn aabb_overlaps_sphere(min: &[f32; 3], max: &[f32; 3], center: &[f32; 3], radius: f32) -> bool {
    let mut dist_sq = 0.0_f32;
    for i in 0..3 {
        let v = center[i].clamp(min[i], max[i]);
        let d = center[i] - v;
        dist_sq += d * d;
    }
    dist_sq <= radius * radius
}

fn aabb_overlaps_ray(
    min: &[f32; 3],
    max: &[f32; 3],
    origin: &[f32; 3],
    inv_dir: &[f32; 3],
    max_dist: f32,
) -> bool {
    let mut tmin = 0.0_f32;
    let mut tmax = max_dist;
    for i in 0..3 {
        if inv_dir[i].is_finite() {
            let t1 = (min[i] - origin[i]) * inv_dir[i];
            let t2 = (max[i] - origin[i]) * inv_dir[i];
            let ta = t1.min(t2);
            let tb = t1.max(t2);
            tmin = tmin.max(ta);
            tmax = tmax.min(tb);
        } else {
            // ray is parallel to this axis
            if origin[i] < min[i] || origin[i] > max[i] {
                return false;
            }
        }
    }
    tmin <= tmax
}

fn dist_sq(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn octant_bounds(
    parent_min: &[f32; 3],
    parent_max: &[f32; 3],
    octant: usize,
) -> ([f32; 3], [f32; 3]) {
    let mid = [
        (parent_min[0] + parent_max[0]) * 0.5,
        (parent_min[1] + parent_max[1]) * 0.5,
        (parent_min[2] + parent_max[2]) * 0.5,
    ];
    let min_x = if octant & 1 != 0 {
        mid[0]
    } else {
        parent_min[0]
    };
    let min_y = if octant & 2 != 0 {
        mid[1]
    } else {
        parent_min[1]
    };
    let min_z = if octant & 4 != 0 {
        mid[2]
    } else {
        parent_min[2]
    };
    let max_x = if octant & 1 != 0 {
        parent_max[0]
    } else {
        mid[0]
    };
    let max_y = if octant & 2 != 0 {
        parent_max[1]
    } else {
        mid[1]
    };
    let max_z = if octant & 4 != 0 {
        parent_max[2]
    } else {
        mid[2]
    };
    ([min_x, min_y, min_z], [max_x, max_y, max_z])
}

// ─── Recursive build ─────────────────────────────────────────────────────────

fn build_node(
    indices: Vec<usize>,
    points: &[[f32; 3]],
    min: [f32; 3],
    max: [f32; 3],
    depth: u32,
    max_depth: u32,
    max_per_leaf: usize,
) -> OctreeNode {
    if depth >= max_depth || indices.len() <= max_per_leaf {
        return OctreeNode {
            bounds_min: min,
            bounds_max: max,
            point_indices: indices,
            children: None,
        };
    }

    let mut child_indices: [Vec<usize>; 8] = Default::default();
    let mid = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];

    for idx in &indices {
        let p = &points[*idx];
        let ox = if p[0] >= mid[0] { 1 } else { 0 };
        let oy = if p[1] >= mid[1] { 2 } else { 0 };
        let oz = if p[2] >= mid[2] { 4 } else { 0 };
        child_indices[ox | oy | oz].push(*idx);
    }

    let children: [OctreeNode; 8] = std::array::from_fn(|i| {
        let (cmin, cmax) = octant_bounds(&min, &max, i);
        build_node(
            std::mem::take(&mut child_indices[i]),
            points,
            cmin,
            cmax,
            depth + 1,
            max_depth,
            max_per_leaf,
        )
    });

    OctreeNode {
        bounds_min: min,
        bounds_max: max,
        point_indices: Vec::new(), // internal nodes hold no points directly
        children: Some(Box::new(children)),
    }
}

// ─── Public build ─────────────────────────────────────────────────────────────

/// Build an octree from a set of 3-D points.
pub fn build_octree(points: &[[f32; 3]], max_depth: u32, max_per_leaf: usize) -> Octree {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in points {
        for i in 0..3 {
            if p[i] < min[i] {
                min[i] = p[i];
            }
            if p[i] > max[i] {
                max[i] = p[i];
            }
        }
    }
    // Add small epsilon to avoid degenerate bounds
    for i in 0..3 {
        max[i] += 1e-4;
        min[i] -= 1e-4;
    }

    let indices: Vec<usize> = (0..points.len()).collect();
    let root = build_node(indices, points, min, max, 0, max_depth, max_per_leaf);
    Octree {
        root,
        points: points.to_vec(),
        max_depth,
        max_points_per_leaf: max_per_leaf,
    }
}

// ─── Queries ─────────────────────────────────────────────────────────────────

fn collect_sphere(
    node: &OctreeNode,
    center: &[f32; 3],
    radius: f32,
    result: &mut Vec<usize>,
    points: &[[f32; 3]],
) {
    if !aabb_overlaps_sphere(&node.bounds_min, &node.bounds_max, center, radius) {
        return;
    }
    if let Some(children) = &node.children {
        for child in children.iter() {
            collect_sphere(child, center, radius, result, points);
        }
    } else {
        let r2 = radius * radius;
        for &idx in &node.point_indices {
            if dist_sq(&points[idx], center) <= r2 {
                result.push(idx);
            }
        }
    }
}

/// Return indices of all points within `radius` of `center`.
pub fn query_sphere(octree: &Octree, center: [f32; 3], radius: f32) -> Vec<usize> {
    let mut result = Vec::new();
    collect_sphere(&octree.root, &center, radius, &mut result, &octree.points);
    result
}

fn collect_aabb(
    node: &OctreeNode,
    min: &[f32; 3],
    max: &[f32; 3],
    result: &mut Vec<usize>,
    points: &[[f32; 3]],
) {
    if !aabb_overlaps_aabb(&node.bounds_min, &node.bounds_max, min, max) {
        return;
    }
    if let Some(children) = &node.children {
        for child in children.iter() {
            collect_aabb(child, min, max, result, points);
        }
    } else {
        for &idx in &node.point_indices {
            if aabb_contains(min, max, &points[idx]) {
                result.push(idx);
            }
        }
    }
}

/// Return indices of all points within the axis-aligned bounding box [min, max].
pub fn query_aabb(octree: &Octree, min: [f32; 3], max: [f32; 3]) -> Vec<usize> {
    let mut result = Vec::new();
    collect_aabb(&octree.root, &min, &max, &mut result, &octree.points);
    result
}

fn collect_nn(
    node: &OctreeNode,
    query: &[f32; 3],
    best_dist_sq: &mut f32,
    best_idx: &mut Option<usize>,
    points: &[[f32; 3]],
) {
    // Prune if closest possible point in this node is farther than current best
    let mut node_dist_sq = 0.0_f32;
    for ((&q, &bmin), &bmax) in query
        .iter()
        .zip(node.bounds_min.iter())
        .zip(node.bounds_max.iter())
    {
        if bmin > bmax {
            return;
        } // inverted/empty bounds
        let v = q.clamp(bmin, bmax);
        let d = q - v;
        node_dist_sq += d * d;
    }
    if node_dist_sq >= *best_dist_sq {
        return;
    }

    if let Some(children) = &node.children {
        for child in children.iter() {
            collect_nn(child, query, best_dist_sq, best_idx, points);
        }
    } else {
        for &idx in &node.point_indices {
            let d2 = dist_sq(&points[idx], query);
            if d2 < *best_dist_sq {
                *best_dist_sq = d2;
                *best_idx = Some(idx);
            }
        }
    }
}

/// Find the nearest neighbour to `query`. Returns (index, distance).
pub fn nearest_neighbor(octree: &Octree, query: [f32; 3]) -> Option<(usize, f32)> {
    let mut best_dist_sq = f32::MAX;
    let mut best_idx = None;
    collect_nn(
        &octree.root,
        &query,
        &mut best_dist_sq,
        &mut best_idx,
        &octree.points,
    );
    best_idx.map(|idx| (idx, best_dist_sq.sqrt()))
}

fn collect_knn(
    node: &OctreeNode,
    query: &[f32; 3],
    heap: &mut Vec<(f32, usize)>, // (dist_sq, idx), max-heap by dist_sq
    k: usize,
    points: &[[f32; 3]],
) {
    let worst = if heap.len() == k { heap[0].0 } else { f32::MAX };
    let mut node_dist_sq = 0.0_f32;
    for ((&q, &bmin), &bmax) in query
        .iter()
        .zip(node.bounds_min.iter())
        .zip(node.bounds_max.iter())
    {
        if bmin > bmax {
            return;
        } // inverted/empty bounds
        let v = q.clamp(bmin, bmax);
        let d = q - v;
        node_dist_sq += d * d;
    }
    if node_dist_sq >= worst {
        return;
    }

    if let Some(children) = &node.children {
        for child in children.iter() {
            collect_knn(child, query, heap, k, points);
        }
    } else {
        for &idx in &node.point_indices {
            let d2 = dist_sq(&points[idx], query);
            let cur_worst = if heap.len() == k { heap[0].0 } else { f32::MAX };
            if d2 < cur_worst {
                if heap.len() == k {
                    // Remove worst
                    heap.remove(0);
                }
                // Insert maintaining max-heap property (simple insertion sort for small k)
                let pos = heap.partition_point(|&(d, _)| d < d2);
                heap.insert(pos, (d2, idx));
            }
        }
    }
}

/// Return the k nearest neighbours sorted by ascending distance.
pub fn k_nearest_neighbors(octree: &Octree, query: [f32; 3], k: usize) -> Vec<(usize, f32)> {
    if k == 0 {
        return Vec::new();
    }
    let mut heap: Vec<(f32, usize)> = Vec::with_capacity(k + 1);
    collect_knn(&octree.root, &query, &mut heap, k, &octree.points);
    heap.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    heap.into_iter().map(|(d2, idx)| (idx, d2.sqrt())).collect()
}

fn node_depth(node: &OctreeNode) -> u32 {
    match &node.children {
        None => 0,
        Some(children) => 1 + children.iter().map(node_depth).max().unwrap_or(0),
    }
}

/// Actual depth of the built octree (0 = single leaf).
pub fn octree_depth(octree: &Octree) -> u32 {
    node_depth(&octree.root)
}

fn count_leaves(node: &OctreeNode) -> usize {
    match &node.children {
        None => 1,
        Some(children) => children.iter().map(count_leaves).sum(),
    }
}

pub fn octree_leaf_count(octree: &Octree) -> usize {
    count_leaves(&octree.root)
}

fn count_points(node: &OctreeNode) -> usize {
    match &node.children {
        None => node.point_indices.len(),
        Some(children) => children.iter().map(count_points).sum(),
    }
}

pub fn octree_point_count(octree: &Octree) -> usize {
    count_points(&octree.root)
}

/// Return (depth, leaf_count, total_points).
pub fn octree_stats(octree: &Octree) -> (u32, usize, usize) {
    (
        octree_depth(octree),
        octree_leaf_count(octree),
        octree_point_count(octree),
    )
}

/// Insert a new point into the octree. Returns the new point's index.
/// Note: rebuilds the affected leaf; simple but correct.
pub fn insert_point(octree: &mut Octree, point: [f32; 3]) -> usize {
    let idx = octree.points.len();
    octree.points.push(point);
    insert_into_node(
        &mut octree.root,
        point,
        idx,
        0,
        octree.max_depth,
        octree.max_points_per_leaf,
        &octree.points.clone(),
    );
    idx
}

fn insert_into_node(
    node: &mut OctreeNode,
    point: [f32; 3],
    idx: usize,
    depth: u32,
    max_depth: u32,
    max_per_leaf: usize,
    points: &[[f32; 3]],
) {
    if !aabb_contains(&node.bounds_min, &node.bounds_max, &point) {
        // Expand bounds to include point
        for ((&pv, bmin), bmax) in point
            .iter()
            .zip(node.bounds_min.iter_mut())
            .zip(node.bounds_max.iter_mut())
        {
            if pv < *bmin {
                *bmin = pv - 1e-4;
            }
            if pv > *bmax {
                *bmax = pv + 1e-4;
            }
        }
    }

    if let Some(children) = &mut node.children {
        // Find the right child
        let mid = [
            (node.bounds_min[0] + node.bounds_max[0]) * 0.5,
            (node.bounds_min[1] + node.bounds_max[1]) * 0.5,
            (node.bounds_min[2] + node.bounds_max[2]) * 0.5,
        ];
        let ox = if point[0] >= mid[0] { 1 } else { 0 };
        let oy = if point[1] >= mid[1] { 2 } else { 0 };
        let oz = if point[2] >= mid[2] { 4 } else { 0 };
        insert_into_node(
            &mut children[ox | oy | oz],
            point,
            idx,
            depth + 1,
            max_depth,
            max_per_leaf,
            points,
        );
    } else {
        node.point_indices.push(idx);
        // Split if over capacity and not at max depth
        if node.point_indices.len() > max_per_leaf && depth < max_depth {
            let all_indices = std::mem::take(&mut node.point_indices);
            let min = node.bounds_min;
            let max = node.bounds_max;
            let mut child_indices: [Vec<usize>; 8] = Default::default();
            let mid = [
                (min[0] + max[0]) * 0.5,
                (min[1] + max[1]) * 0.5,
                (min[2] + max[2]) * 0.5,
            ];
            for i in all_indices {
                let p = &points[i];
                let ox = if p[0] >= mid[0] { 1 } else { 0 };
                let oy = if p[1] >= mid[1] { 2 } else { 0 };
                let oz = if p[2] >= mid[2] { 4 } else { 0 };
                child_indices[ox | oy | oz].push(i);
            }
            let children: [OctreeNode; 8] = std::array::from_fn(|i| {
                let (cmin, cmax) = octant_bounds(&min, &max, i);
                OctreeNode {
                    bounds_min: cmin,
                    bounds_max: cmax,
                    point_indices: std::mem::take(&mut child_indices[i]),
                    children: None,
                }
            });
            node.children = Some(Box::new(children));
        }
    }
}

fn collect_ray(
    node: &OctreeNode,
    origin: &[f32; 3],
    inv_dir: &[f32; 3],
    max_dist: f32,
    result: &mut Vec<usize>,
) {
    if !aabb_overlaps_ray(
        &node.bounds_min,
        &node.bounds_max,
        origin,
        inv_dir,
        max_dist,
    ) {
        return;
    }
    if let Some(children) = &node.children {
        for child in children.iter() {
            collect_ray(child, origin, inv_dir, max_dist, result);
        }
    } else {
        // Return all points in leaf intersected by ray's AABB region
        for &idx in &node.point_indices {
            result.push(idx);
        }
    }
}

/// Return indices of points in leaf nodes intersected by the ray.
pub fn ray_query(
    octree: &Octree,
    origin: [f32; 3],
    direction: [f32; 3],
    max_dist: f32,
) -> Vec<usize> {
    let inv_dir = [
        if direction[0].abs() > 1e-10 {
            1.0 / direction[0]
        } else {
            f32::INFINITY
        },
        if direction[1].abs() > 1e-10 {
            1.0 / direction[1]
        } else {
            f32::INFINITY
        },
        if direction[2].abs() > 1e-10 {
            1.0 / direction[2]
        } else {
            f32::INFINITY
        },
    ];
    let mut result = Vec::new();
    collect_ray(&octree.root, &origin, &inv_dir, max_dist, &mut result);
    result.sort_unstable();
    result.dedup();
    result
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_points(n: usize) -> Vec<[f32; 3]> {
        let mut pts = Vec::new();
        for x in 0..n {
            for y in 0..n {
                for z in 0..n {
                    pts.push([x as f32, y as f32, z as f32]);
                }
            }
        }
        pts
    }

    #[test]
    fn test_build_empty() {
        let oct = build_octree(&[], 4, 8);
        assert_eq!(octree_point_count(&oct), 0);
    }

    #[test]
    fn test_build_single_point() {
        let pts = vec![[1.0, 2.0, 3.0]];
        let oct = build_octree(&pts, 4, 8);
        assert_eq!(octree_point_count(&oct), 1);
    }

    #[test]
    fn test_query_sphere_finds_nearby() {
        let pts = grid_points(5);
        let oct = build_octree(&pts, 4, 8);
        let result = query_sphere(&oct, [0.0, 0.0, 0.0], 1.5);
        assert!(!result.is_empty());
        // All returned points should be within radius
        for idx in &result {
            let p = oct.points[*idx];
            let d = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(d <= 1.5 + 1e-4);
        }
    }

    #[test]
    fn test_query_sphere_excludes_far() {
        let pts = vec![[0.0, 0.0, 0.0], [100.0, 100.0, 100.0]];
        let oct = build_octree(&pts, 4, 4);
        let result = query_sphere(&oct, [0.0, 0.0, 0.0], 1.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0);
    }

    #[test]
    fn test_query_aabb() {
        let pts = grid_points(4);
        let oct = build_octree(&pts, 4, 4);
        let result = query_aabb(&oct, [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        assert!(!result.is_empty());
        for idx in &result {
            let p = oct.points[*idx];
            assert!(p[0] >= 0.0 && p[0] <= 1.0);
            assert!(p[1] >= 0.0 && p[1] <= 1.0);
            assert!(p[2] >= 0.0 && p[2] <= 1.0);
        }
    }

    #[test]
    fn test_nearest_neighbor_exact() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let oct = build_octree(&pts, 4, 4);
        let (idx, dist) = nearest_neighbor(&oct, [1.0, 0.0, 0.0]).unwrap();
        assert_eq!(idx, 1);
        assert!(dist < 1e-5);
    }

    #[test]
    fn test_nearest_neighbor_empty() {
        let oct = build_octree(&[], 4, 4);
        assert!(nearest_neighbor(&oct, [0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn test_k_nearest_neighbors() {
        let pts = grid_points(4);
        let oct = build_octree(&pts, 4, 4);
        let knn = k_nearest_neighbors(&oct, [1.0, 1.0, 1.0], 3);
        assert_eq!(knn.len(), 3);
        // Should be sorted ascending by distance
        for i in 0..knn.len() - 1 {
            assert!(knn[i].1 <= knn[i + 1].1 + 1e-5);
        }
    }

    #[test]
    fn test_octree_depth() {
        let pts = grid_points(3);
        let oct = build_octree(&pts, 4, 2);
        let depth = octree_depth(&oct);
        assert!(depth > 0);
        assert!(depth <= 4);
    }

    #[test]
    fn test_octree_leaf_count_positive() {
        let pts = grid_points(3);
        let oct = build_octree(&pts, 3, 4);
        assert!(octree_leaf_count(&oct) >= 1);
    }

    #[test]
    fn test_octree_point_count_matches() {
        let pts = grid_points(3);
        let oct = build_octree(&pts, 4, 4);
        assert_eq!(octree_point_count(&oct), pts.len());
    }

    #[test]
    fn test_octree_stats() {
        let pts = grid_points(3);
        let oct = build_octree(&pts, 4, 4);
        let (depth, leaves, total) = octree_stats(&oct);
        assert_eq!(total, pts.len());
        assert!(leaves >= 1);
        assert!(depth <= 4);
    }

    #[test]
    fn test_insert_point() {
        let pts = vec![[0.0, 0.0, 0.0]];
        let mut oct = build_octree(&pts, 4, 4);
        let new_idx = insert_point(&mut oct, [5.0, 5.0, 5.0]);
        assert_eq!(new_idx, 1);
        assert_eq!(octree_point_count(&oct), 2);
    }

    #[test]
    fn test_ray_query() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 5.0, 0.0], // far off axis
        ];
        let oct = build_octree(&pts, 4, 4);
        let result = ray_query(&oct, [0.0, 0.0, -1.0], [0.0, 0.0, 1.0], 100.0);
        // At least the point at z=0 should be in some intersected leaf
        assert!(!result.is_empty());
    }

    #[test]
    fn test_k_nearest_zero_k() {
        let pts = grid_points(3);
        let oct = build_octree(&pts, 4, 4);
        let knn = k_nearest_neighbors(&oct, [0.0, 0.0, 0.0], 0);
        assert!(knn.is_empty());
    }
}
