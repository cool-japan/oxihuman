// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Octree spatial index for fast 3-D point queries.

/// Axis-aligned bounding box used by the octree.
#[derive(Debug, Clone)]
pub struct OctreeAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl OctreeAabb {
    /// Create a new AABB from explicit min/max corners.
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    /// Centre of the AABB.
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Per-axis half-extents.
    pub fn half_size(&self) -> [f32; 3] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
            (self.max[2] - self.min[2]) * 0.5,
        ]
    }

    /// Returns `true` when `p` lies inside (inclusive of the boundary).
    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        p[0] >= self.min[0]
            && p[0] <= self.max[0]
            && p[1] >= self.min[1]
            && p[1] <= self.max[1]
            && p[2] >= self.min[2]
            && p[2] <= self.max[2]
    }

    /// Expand the AABB in-place so that it contains `p`.
    pub fn expand(&mut self, p: [f32; 3]) {
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 {
            if p[i] < self.min[i] {
                self.min[i] = p[i];
            }
            if p[i] > self.max[i] {
                self.max[i] = p[i];
            }
        }
    }

    /// Compute a tight AABB from a slice of points.
    /// Returns `None` when `points` is empty.
    pub fn from_points(points: &[[f32; 3]]) -> Option<Self> {
        let first = points.first()?;
        let mut aabb = Self::new(*first, *first);
        for &p in points.iter().skip(1) {
            aabb.expand(p);
        }
        Some(aabb)
    }

    /// Squared distance from `p` to the nearest point on or inside the AABB.
    /// Returns `0.0` when `p` is inside.
    pub fn sq_dist_to_point(&self, p: [f32; 3]) -> f32 {
        let mut sq = 0.0_f32;
        #[allow(clippy::needless_range_loop)]
        for i in 0..3 {
            let v = if p[i] < self.min[i] {
                p[i] - self.min[i]
            } else if p[i] > self.max[i] {
                p[i] - self.max[i]
            } else {
                0.0
            };
            sq += v * v;
        }
        sq
    }

    /// Split into 8 child AABBs (octants) sharing the centre as a corner.
    pub fn octants(&self) -> [OctreeAabb; 8] {
        let c = self.center();
        // Bit 0 = X side, bit 1 = Y side, bit 2 = Z side.
        // 0 = lower half, 1 = upper half.
        [
            OctreeAabb::new([self.min[0], self.min[1], self.min[2]], [c[0], c[1], c[2]]),
            OctreeAabb::new([c[0], self.min[1], self.min[2]], [self.max[0], c[1], c[2]]),
            OctreeAabb::new([self.min[0], c[1], self.min[2]], [c[0], self.max[1], c[2]]),
            OctreeAabb::new([c[0], c[1], self.min[2]], [self.max[0], self.max[1], c[2]]),
            OctreeAabb::new([self.min[0], self.min[1], c[2]], [c[0], c[1], self.max[2]]),
            OctreeAabb::new([c[0], self.min[1], c[2]], [self.max[0], c[1], self.max[2]]),
            OctreeAabb::new([self.min[0], c[1], c[2]], [c[0], self.max[1], self.max[2]]),
            OctreeAabb::new([c[0], c[1], c[2]], [self.max[0], self.max[1], self.max[2]]),
        ]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal tree nodes
// ─────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
enum OctreeNode {
    Leaf {
        /// `(original_index, position)` pairs.
        points: Vec<(usize, [f32; 3])>,
    },
    Internal {
        children: Box<[Option<Box<OctreeNode>>; 8]>,
    },
}

impl OctreeNode {
    fn build(
        points: Vec<(usize, [f32; 3])>,
        bounds: &OctreeAabb,
        depth: usize,
        max_depth: usize,
        max_leaf_size: usize,
    ) -> Self {
        if points.len() <= max_leaf_size || depth >= max_depth {
            return OctreeNode::Leaf { points };
        }

        let octants = bounds.octants();

        // Distribute points into 8 buckets.
        let mut buckets: [Vec<(usize, [f32; 3])>; 8] = Default::default();
        for pt in points {
            let idx = octant_index(pt.1, bounds);
            buckets[idx].push(pt);
        }

        // Build child nodes only for non-empty buckets.
        let children: [Option<Box<OctreeNode>>; 8] = {
            let mut arr: [Option<Box<OctreeNode>>; 8] = Default::default();
            for (i, bucket) in buckets.into_iter().enumerate() {
                if !bucket.is_empty() {
                    arr[i] = Some(Box::new(OctreeNode::build(
                        bucket,
                        &octants[i],
                        depth + 1,
                        max_depth,
                        max_leaf_size,
                    )));
                }
            }
            arr
        };

        OctreeNode::Internal {
            children: Box::new(children),
        }
    }

    fn count(&self) -> usize {
        match self {
            OctreeNode::Leaf { points } => points.len(),
            OctreeNode::Internal { children } => children.iter().flatten().map(|c| c.count()).sum(),
        }
    }

    fn max_depth(&self) -> usize {
        match self {
            OctreeNode::Leaf { .. } => 0,
            OctreeNode::Internal { children } => children
                .iter()
                .flatten()
                .map(|c| c.max_depth() + 1)
                .max()
                .unwrap_or(0),
        }
    }

    /// Branch-and-bound nearest-neighbour search.
    fn nearest(&self, query: [f32; 3], bounds: &OctreeAabb, best: &mut Option<(usize, f32)>) {
        // If the AABB is farther than our current best, prune.
        if let Some((_, best_sq)) = *best {
            if bounds.sq_dist_to_point(query) >= best_sq {
                return;
            }
        }

        match self {
            OctreeNode::Leaf { points } => {
                for &(idx, pos) in points {
                    let sq = sq_dist(query, pos);
                    if best.is_none_or(|(_, b)| sq < b) {
                        *best = Some((idx, sq));
                    }
                }
            }
            OctreeNode::Internal { children } => {
                let octants = bounds.octants();
                // Visit the octant that contains `query` first for faster pruning.
                let preferred = octant_index(query, bounds);

                let visit_order: [usize; 8] = {
                    let mut order = [0usize; 8];
                    order[0] = preferred;
                    let mut k = 1;
                    for i in 0..8 {
                        if i != preferred {
                            order[k] = i;
                            k += 1;
                        }
                    }
                    order
                };

                for &i in &visit_order {
                    if let Some(child) = &children[i] {
                        child.nearest(query, &octants[i], best);
                    }
                }
            }
        }
    }

    /// Radius search: collect all points within `radius_sq` (squared).
    fn radius_search(
        &self,
        query: [f32; 3],
        radius_sq: f32,
        bounds: &OctreeAabb,
        results: &mut Vec<(usize, f32)>,
    ) {
        // Prune if AABB is entirely outside the sphere.
        if bounds.sq_dist_to_point(query) > radius_sq {
            return;
        }

        match self {
            OctreeNode::Leaf { points } => {
                for &(idx, pos) in points {
                    let sq = sq_dist(query, pos);
                    if sq <= radius_sq {
                        results.push((idx, sq));
                    }
                }
            }
            OctreeNode::Internal { children } => {
                let octants = bounds.octants();
                for (i, child_opt) in children.iter().enumerate() {
                    if let Some(child) = child_opt {
                        child.radius_search(query, radius_sq, &octants[i], results);
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

#[inline]
fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Return the octant index (0..8) for `p` relative to `bounds`.
/// Bit 0 = X, bit 1 = Y, bit 2 = Z; 1 means upper half.
#[inline]
fn octant_index(p: [f32; 3], bounds: &OctreeAabb) -> usize {
    let c = bounds.center();
    let xi = usize::from(p[0] >= c[0]);
    let yi = usize::from(p[1] >= c[1]);
    let zi = usize::from(p[2] >= c[2]);
    xi | (yi << 1) | (zi << 2)
}

// ─────────────────────────────────────────────────────────────────────────────
// Public Octree
// ─────────────────────────────────────────────────────────────────────────────

/// An octree for spatial indexing of 3-D points.
pub struct Octree {
    bounds: OctreeAabb,
    root: OctreeNode,
    max_depth: usize,
    max_leaf_size: usize,
}

impl Octree {
    /// Build an octree from a slice of positions.
    ///
    /// * `max_depth`     – maximum subdivision depth (e.g. `8`)
    /// * `max_leaf_size` – maximum points per leaf before splitting (e.g. `16`)
    pub fn build(points: &[[f32; 3]], max_depth: usize, max_leaf_size: usize) -> Self {
        let bounds = OctreeAabb::from_points(points).unwrap_or_else(|| {
            // Degenerate: no points – give unit cube so tree is still valid.
            OctreeAabb::new([0.0; 3], [1.0; 3])
        });

        // Add a tiny epsilon so that points exactly on the max boundary are
        // still inside the root AABB.
        let mut inflated = bounds.clone();
        for i in 0..3 {
            inflated.max[i] += 1e-6 * (inflated.max[i].abs() + 1.0);
        }

        let indexed: Vec<(usize, [f32; 3])> = points.iter().copied().enumerate().collect();

        let root = OctreeNode::build(indexed, &inflated, 0, max_depth, max_leaf_size);

        Self {
            bounds: inflated,
            root,
            max_depth,
            max_leaf_size,
        }
    }

    /// Find the index of the nearest point to `query`.
    /// Returns `(index, squared_distance)`, or `None` if the tree is empty.
    pub fn nearest(&self, query: [f32; 3]) -> Option<(usize, f32)> {
        let mut best: Option<(usize, f32)> = None;
        self.root.nearest(query, &self.bounds, &mut best);
        best
    }

    /// Find all points within `radius` of `query`.
    /// Returns `Vec<(index, squared_distance)>`.
    pub fn radius_search(&self, query: [f32; 3], radius: f32) -> Vec<(usize, f32)> {
        let mut results = Vec::new();
        self.root
            .radius_search(query, radius * radius, &self.bounds, &mut results);
        results
    }

    /// Total number of points stored in the tree.
    pub fn len(&self) -> usize {
        self.root.count()
    }

    /// Returns `true` if the tree contains no points.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Maximum depth of the tree (number of levels minus one).
    pub fn depth(&self) -> usize {
        self.root.max_depth()
    }

    /// Accessor for the root AABB (used in tests).
    #[allow(dead_code)]
    pub fn bounds(&self) -> &OctreeAabb {
        &self.bounds
    }

    /// Expose stored parameters (used in tests).
    #[allow(dead_code)]
    pub fn max_depth_param(&self) -> usize {
        self.max_depth
    }

    /// Expose stored parameters (used in tests).
    #[allow(dead_code)]
    pub fn max_leaf_size_param(&self) -> usize {
        self.max_leaf_size
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Brute-force nearest neighbour: returns (index, sq_dist).
    fn brute_nearest(points: &[[f32; 3]], query: [f32; 3]) -> Option<(usize, f32)> {
        points
            .iter()
            .copied()
            .enumerate()
            .map(|(i, p)| {
                let d = sq_dist(query, p);
                (i, d)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).expect("should succeed"))
    }

    fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        dx * dx + dy * dy + dz * dz
    }

    // ── OctreeAabb tests ─────────────────────────────────────────────────────

    #[test]
    fn octree_aabb_contains_point_inside() {
        let aabb = OctreeAabb::new([0.0; 3], [1.0; 3]);
        assert!(aabb.contains_point([0.5, 0.5, 0.5]));
        assert!(aabb.contains_point([0.0, 0.0, 0.0])); // boundary
        assert!(aabb.contains_point([1.0, 1.0, 1.0])); // boundary
    }

    #[test]
    fn octree_aabb_not_contains_point_outside() {
        let aabb = OctreeAabb::new([0.0; 3], [1.0; 3]);
        assert!(!aabb.contains_point([1.5, 0.5, 0.5]));
        assert!(!aabb.contains_point([-0.1, 0.5, 0.5]));
        assert!(!aabb.contains_point([0.5, 2.0, 0.5]));
    }

    #[test]
    fn octree_aabb_from_points_correct_bounds() {
        let pts = vec![[-1.0_f32, 0.0, 2.0], [3.0, -4.0, 1.0], [0.0, 5.0, -3.0]];
        let aabb = OctreeAabb::from_points(&pts).expect("should succeed");
        assert!((aabb.min[0] - -1.0).abs() < 1e-6);
        assert!((aabb.min[1] - -4.0).abs() < 1e-6);
        assert!((aabb.min[2] - -3.0).abs() < 1e-6);
        assert!((aabb.max[0] - 3.0).abs() < 1e-6);
        assert!((aabb.max[1] - 5.0).abs() < 1e-6);
        assert!((aabb.max[2] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn octree_aabb_octants_count_is_8() {
        let aabb = OctreeAabb::new([0.0; 3], [2.0; 3]);
        let octants = aabb.octants();
        assert_eq!(octants.len(), 8);
    }

    // ── Octree build/query tests ──────────────────────────────────────────────

    #[test]
    fn octree_build_empty_still_works() {
        let tree = Octree::build(&[], 8, 16);
        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
        assert!(tree.nearest([0.0; 3]).is_none());
        assert!(tree.radius_search([0.0; 3], 1.0).is_empty());
    }

    #[test]
    fn octree_build_single_point() {
        let pts = vec![[1.0_f32, 2.0, 3.0]];
        let tree = Octree::build(&pts, 8, 16);
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    fn octree_nearest_single_point_returns_it() {
        let pts = vec![[1.0_f32, 2.0, 3.0]];
        let tree = Octree::build(&pts, 8, 16);
        let (idx, sq) = tree.nearest([0.0, 0.0, 0.0]).expect("should succeed");
        assert_eq!(idx, 0);
        // sq_dist([0,0,0], [1,2,3]) = 1+4+9 = 14
        assert!((sq - 14.0).abs() < 1e-4, "sq={sq}");
    }

    #[test]
    fn octree_nearest_returns_closest() {
        let pts = vec![[0.0_f32, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 0.0, 0.0]];
        let tree = Octree::build(&pts, 8, 16);
        // Query near index 1
        let (idx, _) = tree.nearest([9.9, 0.0, 0.0]).expect("should succeed");
        assert_eq!(idx, 1);
    }

    #[test]
    fn octree_nearest_among_many() {
        // 100 deterministic pseudo-random points
        let pts: Vec<[f32; 3]> = (0..100_u32)
            .map(|i| {
                let x = ((i * 1_664_525 + 1_013_904_223) % 1000) as f32 / 100.0;
                let y = ((i * 22_695_477 + 1) % 1000) as f32 / 100.0;
                let z = ((i * 6_364_136 + 1_442_695) % 1000) as f32 / 100.0;
                [x, y, z]
            })
            .collect();

        let tree = Octree::build(&pts, 8, 8);
        let queries: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [5.0, 5.0, 5.0],
            [9.9, 9.9, 9.9],
            [3.0, 7.0, 1.5],
        ];

        for q in queries {
            let (tree_idx, tree_sq) = tree.nearest(q).expect("should succeed");
            let (bf_idx, bf_sq) = brute_nearest(&pts, q).expect("should succeed");
            // The squared distances must be equal (not just the indices, in case
            // of ties).
            assert!(
                (tree_sq - bf_sq).abs() < 1e-4,
                "query={q:?}: tree sq={tree_sq} (idx={tree_idx}) vs brute sq={bf_sq} (idx={bf_idx})"
            );
        }
    }

    #[test]
    fn octree_radius_search_finds_nearby() {
        let pts = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
        ];
        let tree = Octree::build(&pts, 8, 16);
        let results = tree.radius_search([0.0, 0.0, 0.0], 1.5);
        let mut idxs: Vec<usize> = results.iter().map(|&(i, _)| i).collect();
        idxs.sort_unstable();
        assert_eq!(idxs, vec![0, 1]);
    }

    #[test]
    fn octree_radius_search_excludes_far() {
        let pts = vec![[100.0_f32, 0.0, 0.0], [200.0, 0.0, 0.0]];
        let tree = Octree::build(&pts, 8, 16);
        let results = tree.radius_search([0.0, 0.0, 0.0], 1.0);
        assert!(results.is_empty());
    }

    #[test]
    fn octree_len_matches_input() {
        let pts: Vec<[f32; 3]> = (0..42).map(|i| [i as f32, 0.0, 0.0]).collect();
        let tree = Octree::build(&pts, 8, 16);
        assert_eq!(tree.len(), 42);
    }

    #[test]
    fn octree_build_many_points() {
        // 1000 deterministic points; verify nearest matches brute-force.
        let pts: Vec<[f32; 3]> = (0..1000_u64)
            .map(|i| {
                let x = ((i * 1_664_525 + 1_013_904_223) % 1000) as f32;
                let y = ((i * 22_695_477 + 1) % 1000) as f32;
                let z = ((i * 6_364_136 + 1_442_695) % 1000) as f32;
                [x, y, z]
            })
            .collect();

        let tree = Octree::build(&pts, 10, 16);

        let queries: Vec<[f32; 3]> =
            vec![[0.0, 0.0, 0.0], [500.0, 500.0, 500.0], [999.0, 1.0, 500.0]];

        for q in queries {
            let (tree_idx, tree_sq) = tree.nearest(q).expect("should succeed");
            let (bf_idx, bf_sq) = brute_nearest(&pts, q).expect("should succeed");
            assert!(
                (tree_sq - bf_sq).abs() < 1e-2,
                "query={q:?}: tree sq={tree_sq} (idx={tree_idx}) vs brute sq={bf_sq} (idx={bf_idx})"
            );
        }
    }

    #[test]
    fn octree_sq_dist_to_aabb_zero_when_inside() {
        let aabb = OctreeAabb::new([0.0; 3], [10.0; 3]);
        assert_eq!(aabb.sq_dist_to_point([5.0, 5.0, 5.0]), 0.0);
        assert_eq!(aabb.sq_dist_to_point([0.0, 0.0, 0.0]), 0.0); // corner
        assert_eq!(aabb.sq_dist_to_point([10.0, 10.0, 10.0]), 0.0); // corner
    }
}
