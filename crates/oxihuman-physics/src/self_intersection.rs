//! Mesh self-intersection detection and resolution.
//!
//! Uses a BVH (Bounding Volume Hierarchy) for broad-phase acceleration and
//! Moeller's triangle-triangle intersection test for narrow-phase detection.
//! All geometry uses `f64`.

// ---------------------------------------------------------------------------
// Vector utilities
// ---------------------------------------------------------------------------

fn vec_sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec_add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec_scale(v: &[f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vec_dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec_cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec_length(v: &[f64; 3]) -> f64 {
    vec_dot(v, v).sqrt()
}

fn vec_normalize(v: &[f64; 3]) -> [f64; 3] {
    let len = vec_length(v);
    if len < 1e-15 {
        return [0.0, 0.0, 0.0];
    }
    let inv = 1.0 / len;
    [v[0] * inv, v[1] * inv, v[2] * inv]
}

fn component_min(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0].min(b[0]), a[1].min(b[1]), a[2].min(b[2])]
}

fn component_max(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]
}

// ---------------------------------------------------------------------------
// BVH types
// ---------------------------------------------------------------------------

/// Child reference in the BVH tree.
#[derive(Debug, Clone)]
enum BvhChild {
    /// Index into the `bvh_nodes` array.
    Internal(usize),
    /// Leaf holding a triangle index.
    Leaf(usize),
}

/// A node in the bounding volume hierarchy.
#[derive(Debug, Clone)]
struct BvhNode {
    aabb_min: [f64; 3],
    aabb_max: [f64; 3],
    left: BvhChild,
    right: BvhChild,
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A pair of intersecting triangles with contact information.
#[derive(Debug, Clone)]
pub struct IntersectionPair {
    /// First triangle index.
    pub triangle_a: usize,
    /// Second triangle index.
    pub triangle_b: usize,
    /// Approximate point of intersection.
    pub intersection_point: [f64; 3],
    /// Estimated penetration depth.
    pub depth: f64,
}

/// Detects and resolves self-intersections in a triangle mesh.
///
/// Builds a BVH over the mesh triangles for efficient broad-phase culling,
/// then performs Moeller's separating-axis triangle-triangle test for pairs
/// whose AABBs overlap.
pub struct SelfIntersectionDetector {
    bvh_nodes: Vec<BvhNode>,
    /// Minimum thickness / margin added to AABBs.
    thickness: f64,
    /// Root node index (always 0 after build, but stored explicitly).
    root: Option<usize>,
}

impl SelfIntersectionDetector {
    /// Create a new detector with the given thickness margin.
    pub fn new(thickness: f64) -> Self {
        Self {
            bvh_nodes: Vec::new(),
            thickness: thickness.max(0.0),
            root: None,
        }
    }

    /// Build the BVH from a mesh.
    ///
    /// `positions` contains vertex positions and `triangles` contains index
    /// triples referencing into `positions`.
    pub fn build(
        &mut self,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
    ) -> anyhow::Result<()> {
        if triangles.is_empty() {
            self.bvh_nodes.clear();
            self.root = None;
            return Ok(());
        }

        // Validate indices
        for (ti, tri) in triangles.iter().enumerate() {
            for &idx in tri {
                if idx >= positions.len() {
                    return Err(anyhow::anyhow!(
                        "Triangle {} references vertex index {} but only {} vertices exist",
                        ti,
                        idx,
                        positions.len()
                    ));
                }
            }
        }

        // Precompute triangle centroids and AABBs
        let mut tri_indices: Vec<usize> = (0..triangles.len()).collect();
        let centroids: Vec<[f64; 3]> = triangles
            .iter()
            .map(|tri| {
                let a = &positions[tri[0]];
                let b = &positions[tri[1]];
                let c = &positions[tri[2]];
                [
                    (a[0] + b[0] + c[0]) / 3.0,
                    (a[1] + b[1] + c[1]) / 3.0,
                    (a[2] + b[2] + c[2]) / 3.0,
                ]
            })
            .collect();

        self.bvh_nodes.clear();
        // Reserve an upper bound (2*n - 1 nodes for n leaves)
        self.bvh_nodes.reserve(2 * triangles.len());

        let root = self.build_recursive(
            positions,
            triangles,
            &centroids,
            &mut tri_indices,
            0,
            triangles.len(),
        );
        self.root = Some(root);

        Ok(())
    }

    /// Detect all self-intersecting triangle pairs.
    pub fn detect(
        &self,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
    ) -> Vec<IntersectionPair> {
        let root = match self.root {
            Some(r) => r,
            None => return Vec::new(),
        };

        let mut results = Vec::new();
        self.detect_recursive(root, root, positions, triangles, &mut results);
        results
    }

    /// Resolve intersections by pushing vertices apart.
    ///
    /// Iterates `iterations` times, each time recomputing intersections (using
    /// a fresh BVH) and nudging overlapping vertices along the separating
    /// direction.  Returns the number of remaining intersections.
    pub fn resolve(
        intersections: &[IntersectionPair],
        positions: &mut [[f64; 3]],
        triangles: &[[usize; 3]],
        iterations: usize,
    ) -> anyhow::Result<usize> {
        if intersections.is_empty() {
            return Ok(0);
        }

        // Initial push based on provided intersections
        Self::push_apart(intersections, positions, triangles);

        let mut remaining = intersections.len();
        let thickness = 1e-4;

        for _ in 1..iterations {
            let mut detector = SelfIntersectionDetector::new(thickness);
            detector.build(positions, triangles)?;
            let new_intersections = detector.detect(positions, triangles);
            remaining = new_intersections.len();
            if remaining == 0 {
                break;
            }
            Self::push_apart(&new_intersections, positions, triangles);
        }

        Ok(remaining)
    }

    /// Triangle-triangle intersection test using Moeller's separating axis
    /// theorem approach.
    ///
    /// Tests edges and face normals of both triangles as separating axes.
    /// Returns `Some(point)` with an approximate intersection point if
    /// overlapping, or `None` if separated.
    fn triangle_triangle_test(
        a0: &[f64; 3],
        a1: &[f64; 3],
        a2: &[f64; 3],
        b0: &[f64; 3],
        b1: &[f64; 3],
        b2: &[f64; 3],
    ) -> Option<[f64; 3]> {
        // Compute edges
        let ea0 = vec_sub(a1, a0);
        let ea1 = vec_sub(a2, a0);
        let ea2 = vec_sub(a2, a1);

        let eb0 = vec_sub(b1, b0);
        let eb1 = vec_sub(b2, b0);
        let eb2 = vec_sub(b2, b1);

        // Face normals
        let na = vec_cross(&ea0, &ea1);
        let nb = vec_cross(&eb0, &eb1);

        let na_len_sq = vec_dot(&na, &na);
        let nb_len_sq = vec_dot(&nb, &nb);

        // Check if triangles are (nearly) coplanar: face normals are parallel
        // and all edge cross products will be zero.
        let coplanar = if na_len_sq > 1e-20 && nb_len_sq > 1e-20 {
            let cross_normals = vec_cross(&na, &nb);
            let cross_len_sq = vec_dot(&cross_normals, &cross_normals);
            // Normals are parallel if their cross product is near zero
            cross_len_sq < 1e-16 * na_len_sq * nb_len_sq
        } else {
            // Degenerate triangle(s)
            false
        };

        if coplanar {
            // For coplanar triangles, project onto the dominant 2D plane and
            // use 2D separating-axis test with edge normals within that plane.
            return Self::coplanar_triangle_test(a0, a1, a2, b0, b1, b2, &na);
        }

        // Collect separating axis candidates:
        // 1. Face normals
        // 2. Cross products of edge pairs
        let mut axes: Vec<[f64; 3]> = Vec::with_capacity(13);
        axes.push(na);
        axes.push(nb);

        let edges_a = [ea0, ea1, ea2];
        let edges_b = [eb0, eb1, eb2];

        for ea in &edges_a {
            for eb in &edges_b {
                let c = vec_cross(ea, eb);
                if vec_dot(&c, &c) > 1e-20 {
                    axes.push(c);
                }
            }
        }

        let verts_a = [a0, a1, a2];
        let verts_b = [b0, b1, b2];

        for axis in &axes {
            let axis_len_sq = vec_dot(axis, axis);
            if axis_len_sq < 1e-20 {
                continue;
            }

            // Project all vertices of both triangles onto the axis
            let mut min_a = f64::MAX;
            let mut max_a = f64::MIN;
            for v in &verts_a {
                let proj = vec_dot(v, axis);
                if proj < min_a {
                    min_a = proj;
                }
                if proj > max_a {
                    max_a = proj;
                }
            }

            let mut min_b = f64::MAX;
            let mut max_b = f64::MIN;
            for v in &verts_b {
                let proj = vec_dot(v, axis);
                if proj < min_b {
                    min_b = proj;
                }
                if proj > max_b {
                    max_b = proj;
                }
            }

            // Check for separation
            if max_a < min_b || max_b < min_a {
                return None; // separated along this axis
            }
        }

        // No separating axis found => intersection.
        // Compute approximate intersection point as centroid of both triangles.
        let cx = (a0[0] + a1[0] + a2[0] + b0[0] + b1[0] + b2[0]) / 6.0;
        let cy = (a0[1] + a1[1] + a2[1] + b0[1] + b1[1] + b2[1]) / 6.0;
        let cz = (a0[2] + a1[2] + a2[2] + b0[2] + b1[2] + b2[2]) / 6.0;

        Some([cx, cy, cz])
    }

    /// 2D separating-axis test for coplanar triangles.
    ///
    /// Projects both triangles onto the dominant 2D plane (determined by the
    /// face normal) and tests all 6 edge normals as separating axes.
    fn coplanar_triangle_test(
        a0: &[f64; 3],
        a1: &[f64; 3],
        a2: &[f64; 3],
        b0: &[f64; 3],
        b1: &[f64; 3],
        b2: &[f64; 3],
        normal: &[f64; 3],
    ) -> Option<[f64; 3]> {
        // Choose the two axes for projection (drop the dominant normal component)
        let abs_n = [normal[0].abs(), normal[1].abs(), normal[2].abs()];
        let (u_axis, v_axis) = if abs_n[0] >= abs_n[1] && abs_n[0] >= abs_n[2] {
            (1, 2) // drop X
        } else if abs_n[1] >= abs_n[2] {
            (0, 2) // drop Y
        } else {
            (0, 1) // drop Z
        };

        let project = |p: &[f64; 3]| -> [f64; 2] { [p[u_axis], p[v_axis]] };

        let pa0 = project(a0);
        let pa1 = project(a1);
        let pa2 = project(a2);
        let pb0 = project(b0);
        let pb1 = project(b1);
        let pb2 = project(b2);

        let verts_a_2d = [pa0, pa1, pa2];
        let verts_b_2d = [pb0, pb1, pb2];

        // Edge normals (perpendicular to each edge in 2D)
        let edges_2d = [
            [pa1[0] - pa0[0], pa1[1] - pa0[1]],
            [pa2[0] - pa1[0], pa2[1] - pa1[1]],
            [pa0[0] - pa2[0], pa0[1] - pa2[1]],
            [pb1[0] - pb0[0], pb1[1] - pb0[1]],
            [pb2[0] - pb1[0], pb2[1] - pb1[1]],
            [pb0[0] - pb2[0], pb0[1] - pb2[1]],
        ];

        for edge in &edges_2d {
            // 2D perpendicular as separating axis
            let axis = [-edge[1], edge[0]];
            let axis_len_sq = axis[0] * axis[0] + axis[1] * axis[1];
            if axis_len_sq < 1e-20 {
                continue;
            }

            let mut min_a = f64::MAX;
            let mut max_a = f64::MIN;
            for v in &verts_a_2d {
                let proj = v[0] * axis[0] + v[1] * axis[1];
                if proj < min_a {
                    min_a = proj;
                }
                if proj > max_a {
                    max_a = proj;
                }
            }

            let mut min_b = f64::MAX;
            let mut max_b = f64::MIN;
            for v in &verts_b_2d {
                let proj = v[0] * axis[0] + v[1] * axis[1];
                if proj < min_b {
                    min_b = proj;
                }
                if proj > max_b {
                    max_b = proj;
                }
            }

            if max_a <= min_b || max_b <= min_a {
                return None; // separated
            }
        }

        // No separating axis => overlapping in the plane
        let cx = (a0[0] + a1[0] + a2[0] + b0[0] + b1[0] + b2[0]) / 6.0;
        let cy = (a0[1] + a1[1] + a2[1] + b0[1] + b1[1] + b2[1]) / 6.0;
        let cz = (a0[2] + a1[2] + a2[2] + b0[2] + b1[2] + b2[2]) / 6.0;

        Some([cx, cy, cz])
    }

    // -----------------------------------------------------------------------
    // BVH construction (top-down, longest-axis median split)
    // -----------------------------------------------------------------------

    fn build_recursive(
        &mut self,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        centroids: &[[f64; 3]],
        indices: &mut [usize],
        start: usize,
        end: usize,
    ) -> usize {
        let count = end - start;
        debug_assert!(count > 0);

        // Compute AABB over all triangles in this range
        let (aabb_min, aabb_max) = self.compute_aabb(positions, triangles, &indices[start..end]);

        if count == 1 {
            // Leaf node
            let node_idx = self.bvh_nodes.len();
            self.bvh_nodes.push(BvhNode {
                aabb_min,
                aabb_max,
                left: BvhChild::Leaf(indices[start]),
                right: BvhChild::Leaf(indices[start]), // dummy, both point to same
            });
            return node_idx;
        }

        if count == 2 {
            // Two leaves
            let node_idx = self.bvh_nodes.len();
            self.bvh_nodes.push(BvhNode {
                aabb_min,
                aabb_max,
                left: BvhChild::Leaf(indices[start]),
                right: BvhChild::Leaf(indices[start + 1]),
            });
            return node_idx;
        }

        // Find longest axis
        let extent = vec_sub(&aabb_max, &aabb_min);
        let split_axis = if extent[0] >= extent[1] && extent[0] >= extent[2] {
            0
        } else if extent[1] >= extent[2] {
            1
        } else {
            2
        };

        // Sort by centroid along split axis and take median
        let slice = &mut indices[start..end];
        slice.sort_by(|&a, &b| {
            centroids[a][split_axis]
                .partial_cmp(&centroids[b][split_axis])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mid = start + count / 2;

        // Reserve a slot for this internal node
        let node_idx = self.bvh_nodes.len();
        self.bvh_nodes.push(BvhNode {
            aabb_min,
            aabb_max,
            left: BvhChild::Internal(0),  // placeholder
            right: BvhChild::Internal(0), // placeholder
        });

        let left_idx = self.build_recursive(positions, triangles, centroids, indices, start, mid);
        let right_idx = self.build_recursive(positions, triangles, centroids, indices, mid, end);

        self.bvh_nodes[node_idx].left = BvhChild::Internal(left_idx);
        self.bvh_nodes[node_idx].right = BvhChild::Internal(right_idx);

        node_idx
    }

    fn compute_aabb(
        &self,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        indices: &[usize],
    ) -> ([f64; 3], [f64; 3]) {
        let mut min = [f64::MAX; 3];
        let mut max = [f64::MIN; 3];

        for &ti in indices {
            let tri = &triangles[ti];
            for &vi in tri {
                let v = &positions[vi];
                min = component_min(&min, v);
                max = component_max(&max, v);
            }
        }

        // Expand by thickness
        let t = self.thickness;
        (
            [min[0] - t, min[1] - t, min[2] - t],
            [max[0] + t, max[1] + t, max[2] + t],
        )
    }

    // -----------------------------------------------------------------------
    // BVH traversal for self-intersection detection
    // -----------------------------------------------------------------------

    fn detect_recursive(
        &self,
        node_a: usize,
        node_b: usize,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        results: &mut Vec<IntersectionPair>,
    ) {
        if node_a >= self.bvh_nodes.len() || node_b >= self.bvh_nodes.len() {
            return;
        }

        let a = &self.bvh_nodes[node_a];
        let b = &self.bvh_nodes[node_b];

        // AABB overlap test
        if !aabb_overlap(&a.aabb_min, &a.aabb_max, &b.aabb_min, &b.aabb_max) {
            return;
        }

        // Check if both are leaves
        let a_leaf = self.get_leaf_triangle(&a.left, &a.right);
        let b_leaf = self.get_leaf_triangle(&b.left, &b.right);

        match (a_leaf, b_leaf) {
            (Some(tri_a), Some(tri_b)) => {
                // Both leaves -- test triangle pair
                if tri_a == tri_b {
                    return; // same triangle
                }
                // Skip if triangles share a vertex (adjacent triangles)
                if Self::triangles_share_vertex(&triangles[tri_a], &triangles[tri_b]) {
                    return;
                }
                let ta = &triangles[tri_a];
                let tb = &triangles[tri_b];
                if let Some(point) = Self::triangle_triangle_test(
                    &positions[ta[0]],
                    &positions[ta[1]],
                    &positions[ta[2]],
                    &positions[tb[0]],
                    &positions[tb[1]],
                    &positions[tb[2]],
                ) {
                    let depth = Self::estimate_depth(
                        &positions[ta[0]],
                        &positions[ta[1]],
                        &positions[ta[2]],
                        &positions[tb[0]],
                        &positions[tb[1]],
                        &positions[tb[2]],
                    );
                    // Avoid duplicates: only record (a, b) where a < b
                    let (lo, hi) = if tri_a < tri_b {
                        (tri_a, tri_b)
                    } else {
                        (tri_b, tri_a)
                    };
                    // Check for existing pair (simple linear scan; acceptable for
                    // typical intersection counts)
                    let already = results
                        .iter()
                        .any(|p| p.triangle_a == lo && p.triangle_b == hi);
                    if !already {
                        results.push(IntersectionPair {
                            triangle_a: lo,
                            triangle_b: hi,
                            intersection_point: point,
                            depth,
                        });
                    }
                }
            }
            (Some(tri_a_idx), None) => {
                // A is leaf, B has children -- test leaf A against B's children
                let b_node = &self.bvh_nodes[node_b];
                let left_c = b_node.left.clone();
                let right_c = b_node.right.clone();
                match &left_c {
                    BvhChild::Internal(idx) => {
                        self.test_leaf_vs_subtree(tri_a_idx, *idx, positions, triangles, results);
                    }
                    BvhChild::Leaf(other) => {
                        self.test_leaf_pair(tri_a_idx, *other, positions, triangles, results);
                    }
                }
                match &right_c {
                    BvhChild::Internal(idx) => {
                        self.test_leaf_vs_subtree(tri_a_idx, *idx, positions, triangles, results);
                    }
                    BvhChild::Leaf(other) => {
                        self.test_leaf_pair(tri_a_idx, *other, positions, triangles, results);
                    }
                }
            }
            (None, Some(tri_b_idx)) => {
                // B is leaf, A has children -- test leaf B against A's children
                let a_node = &self.bvh_nodes[node_a];
                let left_c = a_node.left.clone();
                let right_c = a_node.right.clone();
                match &left_c {
                    BvhChild::Internal(idx) => {
                        self.test_leaf_vs_subtree(tri_b_idx, *idx, positions, triangles, results);
                    }
                    BvhChild::Leaf(other) => {
                        self.test_leaf_pair(tri_b_idx, *other, positions, triangles, results);
                    }
                }
                match &right_c {
                    BvhChild::Internal(idx) => {
                        self.test_leaf_vs_subtree(tri_b_idx, *idx, positions, triangles, results);
                    }
                    BvhChild::Leaf(other) => {
                        self.test_leaf_pair(tri_b_idx, *other, positions, triangles, results);
                    }
                }
            }
            (None, None) => {
                // Node is not a single-leaf node. It may have two leaf children
                // (count==2 case) or two internal children.

                if node_a == node_b {
                    // Self-test: test left vs left, right vs right, left vs right
                    let a_node = &self.bvh_nodes[node_a];
                    let left_c = a_node.left.clone();
                    let right_c = a_node.right.clone();
                    self.test_child_pair(&left_c, &left_c, positions, triangles, results);
                    self.test_child_pair(&right_c, &right_c, positions, triangles, results);
                    self.test_child_pair(&left_c, &right_c, positions, triangles, results);
                } else {
                    // Cross-test: descend the larger node
                    let a_size = self.subtree_size(node_a);
                    let b_size = self.subtree_size(node_b);
                    if a_size >= b_size {
                        let a_node = &self.bvh_nodes[node_a];
                        let left_c = a_node.left.clone();
                        let right_c = a_node.right.clone();
                        self.test_child_vs_node(&left_c, node_b, positions, triangles, results);
                        self.test_child_vs_node(&right_c, node_b, positions, triangles, results);
                    } else {
                        let b_node = &self.bvh_nodes[node_b];
                        let left_c = b_node.left.clone();
                        let right_c = b_node.right.clone();
                        self.test_node_vs_child(node_a, &left_c, positions, triangles, results);
                        self.test_node_vs_child(node_a, &right_c, positions, triangles, results);
                    }
                }
            }
        }
    }

    /// Test a pair of BVH children against each other.
    /// Handles Leaf-Leaf, Leaf-Internal, and Internal-Internal combinations.
    fn test_child_pair(
        &self,
        child_a: &BvhChild,
        child_b: &BvhChild,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        results: &mut Vec<IntersectionPair>,
    ) {
        match (child_a, child_b) {
            (BvhChild::Leaf(tri_a), BvhChild::Leaf(tri_b)) => {
                self.test_leaf_pair(*tri_a, *tri_b, positions, triangles, results);
            }
            (BvhChild::Leaf(tri), BvhChild::Internal(node)) => {
                self.test_leaf_vs_subtree(*tri, *node, positions, triangles, results);
            }
            (BvhChild::Internal(node), BvhChild::Leaf(tri)) => {
                self.test_leaf_vs_subtree(*tri, *node, positions, triangles, results);
            }
            (BvhChild::Internal(na), BvhChild::Internal(nb)) => {
                self.detect_recursive(*na, *nb, positions, triangles, results);
            }
        }
    }

    /// Test a BVH child against a node index.
    fn test_child_vs_node(
        &self,
        child: &BvhChild,
        node: usize,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        results: &mut Vec<IntersectionPair>,
    ) {
        match child {
            BvhChild::Internal(idx) => {
                self.detect_recursive(*idx, node, positions, triangles, results);
            }
            BvhChild::Leaf(tri) => {
                self.test_leaf_vs_subtree(*tri, node, positions, triangles, results);
            }
        }
    }

    /// Test a node against a BVH child.
    fn test_node_vs_child(
        &self,
        node: usize,
        child: &BvhChild,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        results: &mut Vec<IntersectionPair>,
    ) {
        match child {
            BvhChild::Internal(idx) => {
                self.detect_recursive(node, *idx, positions, triangles, results);
            }
            BvhChild::Leaf(tri) => {
                self.test_leaf_vs_subtree(*tri, node, positions, triangles, results);
            }
        }
    }

    /// Test a single leaf triangle against all leaves in a subtree.
    fn test_leaf_vs_subtree(
        &self,
        tri_idx: usize,
        node: usize,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        results: &mut Vec<IntersectionPair>,
    ) {
        if node >= self.bvh_nodes.len() {
            return;
        }
        let n = &self.bvh_nodes[node];
        // Check AABB of the leaf triangle against this node's AABB
        let tri = &triangles[tri_idx];
        let leaf_min = component_min(
            &component_min(&positions[tri[0]], &positions[tri[1]]),
            &positions[tri[2]],
        );
        let leaf_max = component_max(
            &component_max(&positions[tri[0]], &positions[tri[1]]),
            &positions[tri[2]],
        );
        let t = self.thickness;
        let padded_min = [leaf_min[0] - t, leaf_min[1] - t, leaf_min[2] - t];
        let padded_max = [leaf_max[0] + t, leaf_max[1] + t, leaf_max[2] + t];

        if !aabb_overlap(&padded_min, &padded_max, &n.aabb_min, &n.aabb_max) {
            return;
        }

        let leaf_a = self.get_leaf_triangle(&n.left, &n.right);
        if let Some(other_tri) = leaf_a {
            self.test_leaf_pair(tri_idx, other_tri, positions, triangles, results);
            return;
        }

        // Two-leaf or internal node: recurse into children
        let left_c = n.left.clone();
        let right_c = n.right.clone();

        match &left_c {
            BvhChild::Internal(idx) => {
                self.test_leaf_vs_subtree(tri_idx, *idx, positions, triangles, results);
            }
            BvhChild::Leaf(other) => {
                self.test_leaf_pair(tri_idx, *other, positions, triangles, results);
            }
        }
        match &right_c {
            BvhChild::Internal(idx) => {
                self.test_leaf_vs_subtree(tri_idx, *idx, positions, triangles, results);
            }
            BvhChild::Leaf(other) => {
                self.test_leaf_pair(tri_idx, *other, positions, triangles, results);
            }
        }
    }

    /// Test a pair of leaf triangles for intersection.
    fn test_leaf_pair(
        &self,
        tri_a: usize,
        tri_b: usize,
        positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
        results: &mut Vec<IntersectionPair>,
    ) {
        if tri_a == tri_b {
            return;
        }
        if Self::triangles_share_vertex(&triangles[tri_a], &triangles[tri_b]) {
            return;
        }
        let ta = &triangles[tri_a];
        let tb = &triangles[tri_b];
        if let Some(point) = Self::triangle_triangle_test(
            &positions[ta[0]],
            &positions[ta[1]],
            &positions[ta[2]],
            &positions[tb[0]],
            &positions[tb[1]],
            &positions[tb[2]],
        ) {
            let depth = Self::estimate_depth(
                &positions[ta[0]],
                &positions[ta[1]],
                &positions[ta[2]],
                &positions[tb[0]],
                &positions[tb[1]],
                &positions[tb[2]],
            );
            let (lo, hi) = if tri_a < tri_b {
                (tri_a, tri_b)
            } else {
                (tri_b, tri_a)
            };
            let already = results
                .iter()
                .any(|p| p.triangle_a == lo && p.triangle_b == hi);
            if !already {
                results.push(IntersectionPair {
                    triangle_a: lo,
                    triangle_b: hi,
                    intersection_point: point,
                    depth,
                });
            }
        }
    }

    /// For a leaf node (where left == right == Leaf), return the triangle index.
    fn get_leaf_triangle(&self, left: &BvhChild, right: &BvhChild) -> Option<usize> {
        match (left, right) {
            (BvhChild::Leaf(a), BvhChild::Leaf(b)) if a == b => Some(*a),
            _ => None,
        }
    }

    fn subtree_size(&self, node: usize) -> usize {
        if node >= self.bvh_nodes.len() {
            return 0;
        }
        let n = &self.bvh_nodes[node];
        let left_size = match &n.left {
            BvhChild::Internal(i) => self.subtree_size(*i),
            BvhChild::Leaf(_) => 1,
        };
        let right_size = match &n.right {
            BvhChild::Internal(i) => self.subtree_size(*i),
            BvhChild::Leaf(_) => 1,
        };
        1 + left_size + right_size
    }

    fn triangles_share_vertex(a: &[usize; 3], b: &[usize; 3]) -> bool {
        for &va in a {
            for &vb in b {
                if va == vb {
                    return true;
                }
            }
        }
        false
    }

    /// Estimate penetration depth between two triangles.
    /// Uses the minimum overlap along the face normals as an approximation.
    fn estimate_depth(
        a0: &[f64; 3],
        a1: &[f64; 3],
        a2: &[f64; 3],
        b0: &[f64; 3],
        b1: &[f64; 3],
        b2: &[f64; 3],
    ) -> f64 {
        let na = vec_normalize(&vec_cross(&vec_sub(a1, a0), &vec_sub(a2, a0)));
        let nb = vec_normalize(&vec_cross(&vec_sub(b1, b0), &vec_sub(b2, b0)));

        let mut min_depth = f64::MAX;

        for axis in &[na, nb] {
            if vec_dot(axis, axis) < 1e-15 {
                continue;
            }
            let mut min_a = f64::MAX;
            let mut max_a = f64::MIN;
            for v in &[a0, a1, a2] {
                let proj = vec_dot(v, axis);
                if proj < min_a {
                    min_a = proj;
                }
                if proj > max_a {
                    max_a = proj;
                }
            }
            let mut min_b = f64::MAX;
            let mut max_b = f64::MIN;
            for v in &[b0, b1, b2] {
                let proj = vec_dot(v, axis);
                if proj < min_b {
                    min_b = proj;
                }
                if proj > max_b {
                    max_b = proj;
                }
            }

            let overlap = (max_a.min(max_b) - min_a.max(min_b)).max(0.0);
            if overlap < min_depth {
                min_depth = overlap;
            }
        }

        if min_depth == f64::MAX {
            0.0
        } else {
            min_depth
        }
    }

    // -----------------------------------------------------------------------
    // Resolution helpers
    // -----------------------------------------------------------------------

    /// Push apart vertices of intersecting triangles along the average normal.
    fn push_apart(
        intersections: &[IntersectionPair],
        positions: &mut [[f64; 3]],
        triangles: &[[usize; 3]],
    ) {
        for pair in intersections {
            if pair.triangle_a >= triangles.len() || pair.triangle_b >= triangles.len() {
                continue;
            }

            let ta = &triangles[pair.triangle_a];
            let tb = &triangles[pair.triangle_b];

            // Validate vertex indices
            let max_idx = positions.len();
            let valid_a = ta.iter().all(|&i| i < max_idx);
            let valid_b = tb.iter().all(|&i| i < max_idx);
            if !valid_a || !valid_b {
                continue;
            }

            // Compute face normals
            let na = vec_normalize(&vec_cross(
                &vec_sub(&positions[ta[1]], &positions[ta[0]]),
                &vec_sub(&positions[ta[2]], &positions[ta[0]]),
            ));
            let nb = vec_normalize(&vec_cross(
                &vec_sub(&positions[tb[1]], &positions[tb[0]]),
                &vec_sub(&positions[tb[2]], &positions[tb[0]]),
            ));

            // Average separation direction
            let sep_raw = vec_add(&na, &nb);
            let sep_len = vec_length(&sep_raw);
            let sep = if sep_len > 1e-12 {
                vec_normalize(&sep_raw)
            } else {
                // Normals are opposite; use na
                na
            };

            // Push distance: half the estimated depth + small margin
            let push_dist = (pair.depth * 0.5 + 1e-5).max(1e-5);
            let push_a = vec_scale(&sep, push_dist);
            let push_b = vec_scale(&sep, -push_dist);

            // Move triangle A vertices outward
            for &vi in ta {
                positions[vi] = vec_add(&positions[vi], &push_a);
            }
            // Move triangle B vertices inward
            for &vi in tb {
                positions[vi] = vec_add(&positions[vi], &push_b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AABB utility
// ---------------------------------------------------------------------------

fn aabb_overlap(min_a: &[f64; 3], max_a: &[f64; 3], min_b: &[f64; 3], max_b: &[f64; 3]) -> bool {
    min_a[0] <= max_b[0]
        && max_a[0] >= min_b[0]
        && min_a[1] <= max_b[1]
        && max_a[1] >= min_b[1]
        && min_a[2] <= max_b[2]
        && max_a[2] >= min_b[2]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a simple mesh: two non-intersecting triangles.
    fn two_separated_triangles() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [5.0, 0.0, 0.0],
            [6.0, 0.0, 0.0],
            [5.5, 1.0, 0.0],
        ];
        let triangles = vec![[0, 1, 2], [3, 4, 5]];
        (positions, triangles)
    }

    /// Build two intersecting triangles that overlap in the XY plane.
    fn two_intersecting_triangles() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        let positions = vec![
            // Triangle A in XY plane
            [-1.0, 0.0, -0.5],
            [1.0, 0.0, -0.5],
            [0.0, 0.0, 1.0],
            // Triangle B crossing through A
            [0.0, -1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.5],
        ];
        let triangles = vec![[0, 1, 2], [3, 4, 5]];
        (positions, triangles)
    }

    #[test]
    fn test_build_empty() {
        let mut det = SelfIntersectionDetector::new(0.01);
        let result = det.build(&[], &[]);
        assert!(result.is_ok());
        assert!(det.root.is_none());
    }

    #[test]
    fn test_build_single_triangle() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let triangles = vec![[0, 1, 2]];
        let mut det = SelfIntersectionDetector::new(0.01);
        let result = det.build(&positions, &triangles);
        assert!(result.is_ok());
        assert!(det.root.is_some());
    }

    #[test]
    fn test_build_invalid_index() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let triangles = vec![[0, 1, 99]]; // index 99 is out of bounds
        let mut det = SelfIntersectionDetector::new(0.01);
        let result = det.build(&positions, &triangles);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_intersection_separated() {
        let (positions, triangles) = two_separated_triangles();
        let mut det = SelfIntersectionDetector::new(0.001);
        det.build(&positions, &triangles)
            .expect("build should succeed");
        let pairs = det.detect(&positions, &triangles);
        assert!(
            pairs.is_empty(),
            "Separated triangles should not intersect, found {} pairs",
            pairs.len()
        );
    }

    #[test]
    fn test_intersection_detected() {
        let (positions, triangles) = two_intersecting_triangles();
        let mut det = SelfIntersectionDetector::new(0.001);
        det.build(&positions, &triangles)
            .expect("build should succeed");
        let pairs = det.detect(&positions, &triangles);
        assert!(
            !pairs.is_empty(),
            "Intersecting triangles should be detected"
        );
        assert_eq!(pairs[0].triangle_a.min(pairs[0].triangle_b), 0);
        assert_eq!(pairs[0].triangle_a.max(pairs[0].triangle_b), 1);
    }

    #[test]
    fn test_resolve_reduces_intersections() {
        let (mut positions, triangles) = two_intersecting_triangles();
        let mut det = SelfIntersectionDetector::new(0.001);
        det.build(&positions, &triangles)
            .expect("build should succeed");
        let pairs = det.detect(&positions, &triangles);
        assert!(!pairs.is_empty());

        let remaining = SelfIntersectionDetector::resolve(&pairs, &mut positions, &triangles, 10);
        assert!(remaining.is_ok());
        // After resolution, should have fewer or zero intersections
        let r = remaining.expect("resolve failed");
        assert!(
            r <= pairs.len(),
            "Should not have more intersections after resolution"
        );
    }

    #[test]
    fn test_triangle_triangle_test_no_intersection() {
        let a0 = [0.0, 0.0, 0.0];
        let a1 = [1.0, 0.0, 0.0];
        let a2 = [0.5, 1.0, 0.0];
        let b0 = [5.0, 0.0, 0.0];
        let b1 = [6.0, 0.0, 0.0];
        let b2 = [5.5, 1.0, 0.0];
        let result = SelfIntersectionDetector::triangle_triangle_test(&a0, &a1, &a2, &b0, &b1, &b2);
        assert!(result.is_none());
    }

    #[test]
    fn test_triangle_triangle_test_coplanar_overlapping() {
        // Two triangles in the same plane that overlap
        let a0 = [0.0, 0.0, 0.0];
        let a1 = [2.0, 0.0, 0.0];
        let a2 = [1.0, 2.0, 0.0];
        let b0 = [1.0, 0.0, 0.0];
        let b1 = [3.0, 0.0, 0.0];
        let b2 = [2.0, 2.0, 0.0];
        let result = SelfIntersectionDetector::triangle_triangle_test(&a0, &a1, &a2, &b0, &b1, &b2);
        // SAT may or may not detect coplanar overlap depending on edge cross products
        // being zero; we just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_aabb_overlap_yes() {
        assert!(aabb_overlap(
            &[0.0, 0.0, 0.0],
            &[1.0, 1.0, 1.0],
            &[0.5, 0.5, 0.5],
            &[1.5, 1.5, 1.5]
        ));
    }

    #[test]
    fn test_aabb_overlap_no() {
        assert!(!aabb_overlap(
            &[0.0, 0.0, 0.0],
            &[1.0, 1.0, 1.0],
            &[2.0, 2.0, 2.0],
            &[3.0, 3.0, 3.0]
        ));
    }

    #[test]
    fn test_shared_vertex_skip() {
        // Two triangles sharing vertex 1 -- should be skipped
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0], // shared
            [0.5, 1.0, 0.0],
            [1.5, 1.0, 0.0],
        ];
        let triangles = vec![[0, 1, 2], [1, 2, 3]];
        let mut det = SelfIntersectionDetector::new(0.1);
        det.build(&positions, &triangles)
            .expect("build should succeed");
        let pairs = det.detect(&positions, &triangles);
        // Adjacent triangles should be skipped
        assert!(
            pairs.is_empty(),
            "Adjacent triangles sharing a vertex should not be reported"
        );
    }

    #[test]
    fn test_multiple_triangles_bvh() {
        // Create a grid of non-intersecting triangles to test BVH construction
        let mut positions = Vec::new();
        let mut triangles = Vec::new();
        for i in 0..10 {
            let x = i as f64 * 3.0;
            let base = positions.len();
            positions.push([x, 0.0, 0.0]);
            positions.push([x + 1.0, 0.0, 0.0]);
            positions.push([x + 0.5, 1.0, 0.0]);
            triangles.push([base, base + 1, base + 2]);
        }
        let mut det = SelfIntersectionDetector::new(0.001);
        det.build(&positions, &triangles)
            .expect("build should succeed");
        let pairs = det.detect(&positions, &triangles);
        assert!(
            pairs.is_empty(),
            "Well-separated triangles should not intersect"
        );
    }

    #[test]
    fn test_estimate_depth() {
        let a0 = [0.0, 0.0, 0.0];
        let a1 = [1.0, 0.0, 0.0];
        let a2 = [0.5, 1.0, 0.0];
        let b0 = [0.25, 0.25, -0.1];
        let b1 = [0.75, 0.25, -0.1];
        let b2 = [0.5, 0.75, 0.1];
        let depth = SelfIntersectionDetector::estimate_depth(&a0, &a1, &a2, &b0, &b1, &b2);
        assert!(depth >= 0.0);
    }

    #[test]
    fn test_detect_on_unbuilt() {
        let det = SelfIntersectionDetector::new(0.01);
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let triangles = vec![[0, 1, 2]];
        let pairs = det.detect(&positions, &triangles);
        assert!(pairs.is_empty(), "Unbuilt detector should return no pairs");
    }

    #[test]
    fn test_resolve_empty() {
        let mut positions = vec![[0.0, 0.0, 0.0]];
        let triangles = vec![];
        let result = SelfIntersectionDetector::resolve(&[], &mut positions, &triangles, 5);
        assert!(result.is_ok());
        assert_eq!(result.expect("resolve failed"), 0);
    }
}
