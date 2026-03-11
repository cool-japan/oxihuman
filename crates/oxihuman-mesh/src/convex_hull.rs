// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ─── math helpers ─────────────────────────────────────────────────────────────

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize(v: [f32; 3]) -> Option<[f32; 3]> {
    let len = length(v);
    if len < 1e-10 {
        None
    } else {
        Some([v[0] / len, v[1] / len, v[2] / len])
    }
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

// ─── internal hull face representation ───────────────────────────────────────

/// A triangular face with outward-facing normal.
#[derive(Clone, Debug)]
struct HullFace {
    /// Indices into the points array (original input indices).
    verts: [usize; 3],
    /// Outward-facing normal.
    normal: [f32; 3],
    /// A point on the plane (used for distance tests).
    center: [f32; 3],
}

impl HullFace {
    fn new(pts: &[[f32; 3]], a: usize, b: usize, c: usize) -> Option<Self> {
        let pa = pts[a];
        let pb = pts[b];
        let pc = pts[c];
        let e1 = sub(pb, pa);
        let e2 = sub(pc, pa);
        let n = cross(e1, e2);
        let normal = normalize(n)?;
        let center = [
            (pa[0] + pb[0] + pc[0]) / 3.0,
            (pa[1] + pb[1] + pc[1]) / 3.0,
            (pa[2] + pb[2] + pc[2]) / 3.0,
        ];
        Some(HullFace {
            verts: [a, b, c],
            normal,
            center,
        })
    }

    /// Signed distance of point p from this face plane.
    /// Positive means the point is on the outward side (visible).
    fn signed_distance(&self, p: [f32; 3]) -> f32 {
        dot(self.normal, sub(p, self.center))
    }

    /// True if point p is "above" (in front of) this face with some epsilon.
    fn is_visible_from(&self, p: [f32; 3]) -> bool {
        self.signed_distance(p) > 1e-7
    }
}

// ─── horizon edge helpers ─────────────────────────────────────────────────────

/// An oriented edge: (from, to) with index of the face it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Edge {
    from: usize,
    to: usize,
}

impl Edge {
    fn reversed(self) -> Self {
        Edge {
            from: self.to,
            to: self.from,
        }
    }
}

/// Given the set of visible face indices, find horizon edges:
/// edges that are adjacent to exactly one visible face
/// (i.e., shared with a non-visible face, forming the boundary).
fn horizon_edges(faces: &[HullFace], visible: &[bool]) -> Vec<Edge> {
    // Collect all edges of visible faces. An edge is on the horizon
    // if its reverse doesn't appear in any visible face's edges.
    let mut visible_edges: Vec<Edge> = Vec::new();
    for (fi, face) in faces.iter().enumerate() {
        if !visible[fi] {
            continue;
        }
        let [a, b, c] = face.verts;
        visible_edges.push(Edge { from: a, to: b });
        visible_edges.push(Edge { from: b, to: c });
        visible_edges.push(Edge { from: c, to: a });
    }

    // An edge is on the horizon if its reverse is NOT in the visible edges set.
    visible_edges
        .iter()
        .filter(|e| !visible_edges.contains(&e.reversed()))
        .copied()
        .collect()
}

// ─── initial tetrahedron ─────────────────────────────────────────────────────

/// Find 4 non-coplanar points to form the initial tetrahedron.
/// Returns (i0, i1, i2, i3) or None if fewer than 4 non-coplanar points exist.
fn find_initial_tetrahedron(pts: &[[f32; 3]]) -> Option<(usize, usize, usize, usize)> {
    let n = pts.len();
    if n < 4 {
        return None;
    }

    // i0: point with lowest y (or x if ties)
    let i0 = pts
        .iter()
        .enumerate()
        .min_by(|a, b| {
            a.1[1]
                .partial_cmp(&b.1[1])
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.1[0]
                        .partial_cmp(&b.1[0])
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
        })
        .map(|(i, _)| i)?;

    // i1: farthest point from i0
    let i1 = pts
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != i0)
        .max_by(|a, b| {
            let da = length(sub(*a.1, pts[i0]));
            let db = length(sub(*b.1, pts[i0]));
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)?;

    // i2: farthest point from line i0–i1
    let axis = sub(pts[i1], pts[i0]);
    let i2 = pts
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != i0 && *i != i1)
        .max_by(|a, b| {
            let da = length(cross(sub(*a.1, pts[i0]), axis));
            let db = length(cross(sub(*b.1, pts[i0]), axis));
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)?;

    // Verify i2 is not collinear
    let axis2 = sub(pts[i1], pts[i0]);
    let cross2 = cross(sub(pts[i2], pts[i0]), axis2);
    if length(cross2) < 1e-7 {
        return None; // all points collinear
    }

    // i3: farthest point from plane i0–i1–i2
    let plane_n = cross(sub(pts[i1], pts[i0]), sub(pts[i2], pts[i0]));
    let plane_n_len = length(plane_n);
    if plane_n_len < 1e-10 {
        return None;
    }
    let plane_n_unit = scale(plane_n, 1.0 / plane_n_len);

    let i3 = pts
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != i0 && *i != i1 && *i != i2)
        .max_by(|a, b| {
            let da = dot(plane_n_unit, sub(*a.1, pts[i0])).abs();
            let db = dot(plane_n_unit, sub(*b.1, pts[i0])).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)?;

    // Verify i3 is not coplanar
    let dist = dot(plane_n_unit, sub(pts[i3], pts[i0])).abs();
    if dist < 1e-7 {
        return None; // all points coplanar
    }

    Some((i0, i1, i2, i3))
}

/// Build the initial 4-face tetrahedron, ensuring consistent outward normals.
fn build_tetrahedron(
    pts: &[[f32; 3]],
    i0: usize,
    i1: usize,
    i2: usize,
    i3: usize,
) -> Option<Vec<HullFace>> {
    // Compute centroid of the 4 points
    let centroid = [
        (pts[i0][0] + pts[i1][0] + pts[i2][0] + pts[i3][0]) / 4.0,
        (pts[i0][1] + pts[i1][1] + pts[i2][1] + pts[i3][1]) / 4.0,
        (pts[i0][2] + pts[i1][2] + pts[i2][2] + pts[i3][2]) / 4.0,
    ];

    let mut faces = Vec::new();
    // 4 faces of the tetrahedron: each face omits one vertex
    let face_verts = [[i0, i1, i2], [i0, i1, i3], [i0, i2, i3], [i1, i2, i3]];

    for [a, b, c] in face_verts {
        let mut face = HullFace::new(pts, a, b, c)?;
        // Flip normal if it points toward centroid (should point away)
        if dot(face.normal, sub(centroid, face.center)) > 0.0 {
            face.normal = scale(face.normal, -1.0);
            // Also flip winding
            face.verts = [face.verts[0], face.verts[2], face.verts[1]];
        }
        faces.push(face);
    }

    Some(faces)
}

// ─── main hull algorithm ──────────────────────────────────────────────────────

/// Result of convex hull computation.
pub struct ConvexHull {
    /// Hull vertex positions (subset of input points).
    pub vertices: Vec<[f32; 3]>,
    /// Triangle indices into `hull.vertices`.
    pub indices: Vec<u32>,
    /// Original indices of hull vertices in the input point cloud.
    pub vertex_indices: Vec<usize>,
}

impl ConvexHull {
    /// Convert to `MeshBuffers` with computed normals and dummy UVs.
    pub fn to_mesh_buffers(&self) -> MeshBuffers {
        let n = self.vertices.len();
        let mut m = MeshBuffers {
            positions: self.vertices.clone(),
            normals: vec![[0.0, 1.0, 0.0]; n],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
            uvs: vec![[0.0, 0.0]; n],
            indices: self.indices.clone(),
            colors: None,
            has_suit: false,
        };
        compute_normals(&mut m);
        m
    }

    /// Number of triangular faces in the hull.
    pub fn face_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Approximate volume using the divergence theorem (signed tetrahedral decomposition).
    pub fn volume(&self) -> f32 {
        let mut vol = 0.0f32;
        for tri in self.indices.chunks_exact(3) {
            let a = self.vertices[tri[0] as usize];
            let b = self.vertices[tri[1] as usize];
            let c = self.vertices[tri[2] as usize];
            // Signed volume of tetrahedron with origin
            vol += (a[0] * (b[1] * c[2] - b[2] * c[1])
                + a[1] * (b[2] * c[0] - b[0] * c[2])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0;
        }
        vol.abs()
    }

    /// Surface area: sum of triangle areas.
    pub fn surface_area(&self) -> f32 {
        let mut area = 0.0f32;
        for tri in self.indices.chunks_exact(3) {
            let a = self.vertices[tri[0] as usize];
            let b = self.vertices[tri[1] as usize];
            let c = self.vertices[tri[2] as usize];
            let e1 = sub(b, a);
            let e2 = sub(c, a);
            area += length(cross(e1, e2)) * 0.5;
        }
        area
    }
}

/// Compute the 3D convex hull of a point cloud using an incremental algorithm.
///
/// Returns `None` if fewer than 4 non-coplanar points are provided.
pub fn convex_hull(points: &[[f32; 3]]) -> Option<ConvexHull> {
    if points.len() < 4 {
        return None;
    }

    // Find initial tetrahedron
    let (i0, i1, i2, i3) = find_initial_tetrahedron(points)?;
    let mut faces = build_tetrahedron(points, i0, i1, i2, i3)?;

    // Process each remaining point
    for (pi, &p) in points.iter().enumerate() {
        if pi == i0 || pi == i1 || pi == i2 || pi == i3 {
            continue;
        }

        // Determine which faces are visible from p
        let visible: Vec<bool> = faces.iter().map(|f| f.is_visible_from(p)).collect();

        // If no face is visible, p is inside the hull — skip it
        if !visible.iter().any(|&v| v) {
            continue;
        }

        // Find horizon edges
        let horizon = horizon_edges(&faces, &visible);
        if horizon.is_empty() {
            continue;
        }

        // Remove visible faces
        let mut new_faces: Vec<HullFace> = faces
            .into_iter()
            .zip(visible.iter())
            .filter_map(|(f, &vis)| if vis { None } else { Some(f) })
            .collect();

        // Compute the centroid of the remaining hull to orient new faces
        // We use the centroid of i0, i1, i2, i3 as a stable interior point
        let interior = [
            (points[i0][0] + points[i1][0] + points[i2][0] + points[i3][0]) / 4.0,
            (points[i0][1] + points[i1][1] + points[i2][1] + points[i3][1]) / 4.0,
            (points[i0][2] + points[i1][2] + points[i2][2] + points[i3][2]) / 4.0,
        ];

        // Add new faces: connect p to each horizon edge
        for edge in horizon {
            if let Some(mut face) = HullFace::new(points, edge.from, edge.to, pi) {
                // Ensure outward normal (away from interior)
                if dot(face.normal, sub(interior, face.center)) > 0.0 {
                    face.normal = scale(face.normal, -1.0);
                    face.verts = [face.verts[0], face.verts[2], face.verts[1]];
                }
                new_faces.push(face);
            }
        }

        faces = new_faces;
    }

    if faces.is_empty() {
        return None;
    }

    // Collect unique hull vertex indices
    let mut hull_vert_set: Vec<usize> = faces.iter().flat_map(|f| f.verts).collect();
    hull_vert_set.sort_unstable();
    hull_vert_set.dedup();

    // Build output: remap original indices to hull-local indices
    let vertex_indices = hull_vert_set.clone();
    let vertices: Vec<[f32; 3]> = vertex_indices.iter().map(|&i| points[i]).collect();

    // Build index buffer
    let orig_to_hull: std::collections::HashMap<usize, u32> = vertex_indices
        .iter()
        .enumerate()
        .map(|(hi, &oi)| (oi, hi as u32))
        .collect();

    let indices: Vec<u32> = faces
        .iter()
        .flat_map(|f| {
            [
                orig_to_hull[&f.verts[0]],
                orig_to_hull[&f.verts[1]],
                orig_to_hull[&f.verts[2]],
            ]
        })
        .collect();

    Some(ConvexHull {
        vertices,
        indices,
        vertex_indices,
    })
}

/// Compute the convex hull of the vertex positions in a `MeshBuffers`.
pub fn mesh_convex_hull(mesh: &MeshBuffers) -> Option<ConvexHull> {
    convex_hull(&mesh.positions)
}

/// Check whether a point is inside (or on the surface of) the convex hull.
///
/// A point is inside if it is not in front of any face (all signed distances ≤ ε).
pub fn point_in_hull(hull: &ConvexHull, point: [f32; 3]) -> bool {
    const EPSILON: f32 = 1e-4;
    for tri in hull.indices.chunks_exact(3) {
        let a = hull.vertices[tri[0] as usize];
        let b = hull.vertices[tri[1] as usize];
        let c = hull.vertices[tri[2] as usize];
        let e1 = sub(b, a);
        let e2 = sub(c, a);
        let n_raw = cross(e1, e2);
        if let Some(n) = normalize(n_raw) {
            let d = dot(n, sub(point, a));
            if d > EPSILON {
                return false;
            }
        }
    }
    true
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::sphere;

    fn cube_pts() -> Vec<[f32; 3]> {
        vec![
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ]
    }

    fn tetra_pts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn convex_hull_of_cube_vertices() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("cube hull should succeed");
        // A cube has 6 quad faces = 12 triangles
        assert_eq!(
            hull.face_count(),
            12,
            "cube hull should have 12 triangular faces"
        );
    }

    #[test]
    fn convex_hull_vertex_count_lte_input() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        assert!(
            hull.vertices.len() <= pts.len(),
            "hull vertices ({}) must not exceed input ({})",
            hull.vertices.len(),
            pts.len()
        );
    }

    #[test]
    fn convex_hull_all_input_inside_or_on_hull() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        for &p in &pts {
            assert!(
                point_in_hull(&hull, p),
                "input point {:?} should be inside or on hull",
                p
            );
        }
    }

    #[test]
    fn convex_hull_tetrahedron_has_4_faces() {
        let pts = tetra_pts();
        let hull = convex_hull(&pts).expect("tetrahedron hull should succeed");
        assert_eq!(
            hull.face_count(),
            4,
            "tetrahedron hull must have exactly 4 faces"
        );
    }

    #[test]
    fn convex_hull_returns_none_for_fewer_than_4_points() {
        assert!(convex_hull(&[[0.0, 0.0, 0.0]]).is_none());
        assert!(convex_hull(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).is_none());
    }

    #[test]
    fn convex_hull_to_mesh_buffers_has_valid_indices() {
        let pts = tetra_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        let mesh = hull.to_mesh_buffers();
        let n = mesh.positions.len() as u32;
        for &idx in &mesh.indices {
            assert!(idx < n, "index {} out of bounds (n={})", idx, n);
        }
    }

    #[test]
    fn convex_hull_face_count_positive() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        assert!(hull.face_count() > 0, "hull must have at least one face");
    }

    #[test]
    fn convex_hull_volume_positive() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        let vol = hull.volume();
        assert!(vol > 0.0, "hull volume must be positive, got {}", vol);
        // Cube volume is 2*2*2 = 8
        assert!(
            (vol - 8.0).abs() < 0.1,
            "cube hull volume should be ~8, got {}",
            vol
        );
    }

    #[test]
    fn convex_hull_surface_area_positive() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        let area = hull.surface_area();
        assert!(
            area > 0.0,
            "hull surface area must be positive, got {}",
            area
        );
        // Cube surface area is 6 * (2*2) = 24
        assert!(
            (area - 24.0).abs() < 0.1,
            "cube hull surface area should be ~24, got {}",
            area
        );
    }

    #[test]
    fn point_in_hull_center_is_inside() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        assert!(
            point_in_hull(&hull, [0.0, 0.0, 0.0]),
            "center of cube hull should be inside"
        );
    }

    #[test]
    fn point_in_hull_far_point_is_outside() {
        let pts = cube_pts();
        let hull = convex_hull(&pts).expect("hull should succeed");
        assert!(
            !point_in_hull(&hull, [10.0, 10.0, 10.0]),
            "far-away point should be outside hull"
        );
    }

    #[test]
    fn mesh_convex_hull_works_on_sphere_mesh() {
        let s = sphere(1.0, 6, 6);
        let hull = mesh_convex_hull(&s).expect("sphere mesh hull should succeed");
        assert!(hull.face_count() > 0, "sphere hull must have faces");
        // Center of the sphere must be strictly inside the hull
        assert!(
            point_in_hull(&hull, [0.0, 0.0, 0.0]),
            "sphere center should be inside hull"
        );
        // The hull vertices are a subset of sphere positions
        assert!(
            hull.vertices.len() <= s.positions.len(),
            "hull vertices ({}) must not exceed sphere vertex count ({})",
            hull.vertices.len(),
            s.positions.len()
        );
        // A point far outside must not be inside
        assert!(
            !point_in_hull(&hull, [0.0, 5.0, 0.0]),
            "far point should be outside sphere hull"
        );
    }

    #[test]
    fn convex_hull_coplanar_returns_none() {
        // All points in the Z=0 plane
        let pts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ];
        // Should return None since all points are coplanar
        assert!(
            convex_hull(&pts).is_none(),
            "coplanar points should return None"
        );
    }
}
