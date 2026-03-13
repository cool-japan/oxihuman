// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Convex hull computation for 3D point sets using an incremental algorithm.
//!
//! Provides functions to compute convex hulls, query geometric properties
//! (volume, surface area, centroid), and test point containment.

#[allow(dead_code)]
/// Result of a convex hull computation.
pub struct ConvexHullResult {
    /// Positions of the hull vertices.
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices (triples) forming the hull faces.
    pub indices: Vec<u32>,
}

#[allow(dead_code)]
/// Configuration for convex hull computation.
pub struct HullConfig {
    /// Merge tolerance for coplanar faces.
    pub merge_tolerance: f32,
    /// Whether to normalize output winding order.
    pub normalize_winding: bool,
}

// ── math helpers ─────────────────────────────────────────────────────────────

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn normalize3(a: [f32; 3]) -> [f32; 3] {
    let l = len3(a);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        scale3(a, 1.0 / l)
    }
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(a, b))
}

// ── Face representation for incremental algorithm ────────────────────────────

struct HullFace {
    verts: [usize; 3],
    normal: [f32; 3],
    dist: f32,     // signed distance from origin along normal
    visible: bool, // marked for removal
}

impl HullFace {
    fn new(positions: &[[f32; 3]], a: usize, b: usize, c: usize) -> Self {
        let ab = sub3(positions[b], positions[a]);
        let ac = sub3(positions[c], positions[a]);
        let n = normalize3(cross3(ab, ac));
        let d = dot3(n, positions[a]);
        HullFace {
            verts: [a, b, c],
            normal: n,
            dist: d,
            visible: false,
        }
    }

    fn signed_distance(&self, p: [f32; 3]) -> f32 {
        dot3(self.normal, p) - self.dist
    }
}

// ── public API ───────────────────────────────────────────────────────────────

/// Returns a default hull configuration.
#[allow(dead_code)]
pub fn default_hull_config() -> HullConfig {
    HullConfig {
        merge_tolerance: 1e-6,
        normalize_winding: true,
    }
}

/// Compute the convex hull of a set of 3D points using an incremental algorithm.
///
/// Returns `None` if the point set is degenerate (fewer than 4 non-coplanar points).
#[allow(dead_code)]
pub fn compute_convex_hull(points: &[[f32; 3]], _config: &HullConfig) -> Option<ConvexHullResult> {
    if points.len() < 4 {
        return None;
    }

    // Find 4 non-coplanar seed points
    let (seed, rest) = find_initial_tetrahedron(points)?;

    // Build initial tetrahedron faces (4 triangles)
    let mut faces: Vec<HullFace> = Vec::new();
    let [a, b, c, d] = seed;

    // Determine winding: ensure face normals point outward
    let centroid = scale3(
        add3(add3(points[a], points[b]), add3(points[c], points[d])),
        0.25,
    );

    let mut add_face = |i: usize, j: usize, k: usize| {
        let mut f = HullFace::new(points, i, j, k);
        // Ensure normal points away from centroid
        if f.signed_distance(centroid) > 0.0 {
            f = HullFace::new(points, i, k, j);
        }
        faces.push(f);
    };

    add_face(a, b, c);
    add_face(a, b, d);
    add_face(a, c, d);
    add_face(b, c, d);

    // Incrementally add remaining points
    for &pi in &rest {
        // Find faces visible from this point
        let mut any_visible = false;
        for face in &mut faces {
            if !face.visible && face.signed_distance(points[pi]) > 1e-8 {
                face.visible = true;
                any_visible = true;
            }
        }

        if !any_visible {
            continue; // Point is inside current hull
        }

        // Find horizon edges (edges between visible and non-visible faces)
        let horizon = find_horizon_edges(&faces);

        // Remove visible faces
        faces.retain(|f| !f.visible);

        // Create new faces from horizon edges to the new point
        for (e0, e1) in &horizon {
            faces.push(HullFace::new(points, *e0, *e1, pi));
        }

        // Fix winding for new faces
        let new_centroid = hull_centroid_from_faces(points, &faces);
        for face in &mut faces {
            if face.signed_distance(new_centroid) > 0.0 {
                face.verts.swap(1, 2);
                *face = HullFace::new(points, face.verts[0], face.verts[1], face.verts[2]);
            }
        }
    }

    // Collect unique vertex indices and remap
    let mut used: Vec<usize> = Vec::new();
    for f in &faces {
        for &v in &f.verts {
            if !used.contains(&v) {
                used.push(v);
            }
        }
    }
    used.sort_unstable();

    let positions: Vec<[f32; 3]> = used.iter().map(|&i| points[i]).collect();
    let indices: Vec<u32> = faces
        .iter()
        .flat_map(|f| {
            f.verts
                .iter()
                .map(|v| used.iter().position(|&u| u == *v).unwrap_or(0) as u32)
        })
        .collect();

    Some(ConvexHullResult { positions, indices })
}

fn find_initial_tetrahedron(points: &[[f32; 3]]) -> Option<([usize; 4], Vec<usize>)> {
    let n = points.len();
    if n < 4 {
        return None;
    }

    // Find two points farthest apart
    let mut a = 0;
    let mut b = 1;
    let mut best_dist = 0.0f32;
    for i in 0..n {
        for j in (i + 1)..n {
            let d = dist3(points[i], points[j]);
            if d > best_dist {
                best_dist = d;
                a = i;
                b = j;
            }
        }
    }

    // Find point farthest from line ab
    let ab = sub3(points[b], points[a]);
    let mut c = 0;
    let mut best_area = 0.0f32;
    for i in 0..n {
        if i == a || i == b {
            continue;
        }
        let ai = sub3(points[i], points[a]);
        let area = len3(cross3(ab, ai));
        if area > best_area {
            best_area = area;
            c = i;
        }
    }
    if best_area < 1e-10 {
        return None; // Collinear
    }

    // Find point farthest from plane abc
    let ac = sub3(points[c], points[a]);
    let normal = normalize3(cross3(ab, ac));
    let mut d = 0;
    let mut best_vol = 0.0f32;
    for i in 0..n {
        if i == a || i == b || i == c {
            continue;
        }
        let ai = sub3(points[i], points[a]);
        let vol = dot3(normal, ai).abs();
        if vol > best_vol {
            best_vol = vol;
            d = i;
        }
    }
    if best_vol < 1e-10 {
        return None; // Coplanar
    }

    let rest: Vec<usize> = (0..n)
        .filter(|&i| i != a && i != b && i != c && i != d)
        .collect();
    Some(([a, b, c, d], rest))
}

fn find_horizon_edges(faces: &[HullFace]) -> Vec<(usize, usize)> {
    let mut horizon = Vec::new();
    for face in faces {
        if !face.visible {
            continue;
        }
        let edges = [
            (face.verts[0], face.verts[1]),
            (face.verts[1], face.verts[2]),
            (face.verts[2], face.verts[0]),
        ];
        for (e0, e1) in edges {
            // Check if the adjacent face sharing this edge is not visible
            let has_non_visible_neighbor = faces
                .iter()
                .any(|other| !other.visible && shares_edge(other, e0, e1));
            if has_non_visible_neighbor {
                horizon.push((e1, e0)); // Reverse for correct winding
            }
        }
    }
    horizon
}

fn shares_edge(face: &HullFace, e0: usize, e1: usize) -> bool {
    let v = &face.verts;
    let edges = [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])];
    edges
        .iter()
        .any(|&(a, b)| (a == e0 && b == e1) || (a == e1 && b == e0))
}

fn hull_centroid_from_faces(points: &[[f32; 3]], faces: &[HullFace]) -> [f32; 3] {
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let mut count = 0u32;
    for f in faces {
        for &v in &f.verts {
            cx += points[v][0];
            cy += points[v][1];
            cz += points[v][2];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0, 0.0, 0.0];
    }
    let inv = 1.0 / count as f32;
    [cx * inv, cy * inv, cz * inv]
}

/// Compute the volume of a convex hull using the divergence theorem.
#[allow(dead_code)]
pub fn convex_hull_volume(hull: &ConvexHullResult) -> f32 {
    let mut vol = 0.0f32;
    let nf = hull.indices.len() / 3;
    for i in 0..nf {
        let a = hull.positions[hull.indices[i * 3] as usize];
        let b = hull.positions[hull.indices[i * 3 + 1] as usize];
        let c = hull.positions[hull.indices[i * 3 + 2] as usize];
        // Signed volume of tetrahedron with origin
        vol += dot3(a, cross3(b, c));
    }
    (vol / 6.0).abs()
}

/// Compute the surface area of the convex hull.
#[allow(dead_code)]
pub fn convex_hull_surface_area(hull: &ConvexHullResult) -> f32 {
    let mut area = 0.0f32;
    let nf = hull.indices.len() / 3;
    for i in 0..nf {
        let a = hull.positions[hull.indices[i * 3] as usize];
        let b = hull.positions[hull.indices[i * 3 + 1] as usize];
        let c = hull.positions[hull.indices[i * 3 + 2] as usize];
        let ab = sub3(b, a);
        let ac = sub3(c, a);
        area += len3(cross3(ab, ac)) * 0.5;
    }
    area
}

/// Test whether a point lies inside the convex hull.
#[allow(dead_code)]
pub fn is_point_inside_hull(hull: &ConvexHullResult, point: [f32; 3]) -> bool {
    let nf = hull.indices.len() / 3;
    for i in 0..nf {
        let a = hull.positions[hull.indices[i * 3] as usize];
        let b = hull.positions[hull.indices[i * 3 + 1] as usize];
        let c = hull.positions[hull.indices[i * 3 + 2] as usize];
        let ab = sub3(b, a);
        let ac = sub3(c, a);
        let n = cross3(ab, ac);
        let ap = sub3(point, a);
        if dot3(n, ap) > 1e-6 {
            return false;
        }
    }
    true
}

/// Compute the centroid of the convex hull (average of hull vertices).
#[allow(dead_code)]
pub fn hull_centroid(hull: &ConvexHullResult) -> [f32; 3] {
    if hull.positions.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut c = [0.0f32; 3];
    for p in &hull.positions {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let inv = 1.0 / hull.positions.len() as f32;
    [c[0] * inv, c[1] * inv, c[2] * inv]
}

/// Return the number of vertices in the hull.
#[allow(dead_code)]
pub fn hull_vertex_count(hull: &ConvexHullResult) -> usize {
    hull.positions.len()
}

/// Return the number of triangular faces in the hull.
#[allow(dead_code)]
pub fn hull_face_count(hull: &ConvexHullResult) -> usize {
    hull.indices.len() / 3
}

/// Find the support point (farthest point along a given direction).
#[allow(dead_code)]
pub fn support_point(hull: &ConvexHullResult, direction: [f32; 3]) -> [f32; 3] {
    let mut best = 0;
    let mut best_dot = f32::NEG_INFINITY;
    for (i, p) in hull.positions.iter().enumerate() {
        let d = dot3(*p, direction);
        if d > best_dot {
            best_dot = d;
            best = i;
        }
    }
    hull.positions[best]
}

/// Project a point onto the nearest point on the hull surface.
#[allow(dead_code)]
pub fn project_to_hull_surface(hull: &ConvexHullResult, point: [f32; 3]) -> [f32; 3] {
    let nf = hull.indices.len() / 3;
    let mut best_dist = f32::INFINITY;
    let mut best_proj = point;

    for i in 0..nf {
        let a = hull.positions[hull.indices[i * 3] as usize];
        let b = hull.positions[hull.indices[i * 3 + 1] as usize];
        let c = hull.positions[hull.indices[i * 3 + 2] as usize];
        let proj = closest_point_on_triangle(point, a, b, c);
        let d = dist3(point, proj);
        if d < best_dist {
            best_dist = d;
            best_proj = proj;
        }
    }
    best_proj
}

fn closest_point_on_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ap = sub3(p, a);

    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }

    let bp = sub3(p, b);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return add3(a, scale3(ab, v));
    }

    let cp = sub3(p, c);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return add3(a, scale3(ac, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return add3(b, scale3(sub3(c, b), w));
    }

    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    add3(a, add3(scale3(ab, v), scale3(ac, w)))
}

/// Compute the axis-aligned bounding box of the hull.
/// Returns `(min, max)`.
#[allow(dead_code)]
pub fn hull_bounding_box(hull: &ConvexHullResult) -> ([f32; 3], [f32; 3]) {
    if hull.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in &hull.positions {
        for k in 0..3 {
            if p[k] < min[k] {
                min[k] = p[k];
            }
            if p[k] > max[k] {
                max[k] = p[k];
            }
        }
    }
    (min, max)
}

/// Compute the number of unique edges in the hull (Euler: E = V + F - 2).
#[allow(dead_code)]
pub fn hull_edge_count(hull: &ConvexHullResult) -> usize {
    let v = hull_vertex_count(hull);
    let f = hull_face_count(hull);
    // For a convex polyhedron: V - E + F = 2 => E = V + F - 2
    if v + f < 2 {
        return 0;
    }
    v + f - 2
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_points() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ]
    }

    fn tetra_points() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ]
    }

    #[test]
    fn test_default_hull_config() {
        let cfg = default_hull_config();
        assert!(cfg.merge_tolerance > 0.0);
        assert!(cfg.normalize_winding);
    }

    #[test]
    fn test_compute_hull_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        assert_eq!(hull_vertex_count(&hull), 8);
        // Cube has 12 triangular faces (6 quads / 2 triangles each)
        assert_eq!(hull_face_count(&hull), 12);
    }

    #[test]
    fn test_compute_hull_tetrahedron() {
        let pts = tetra_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        assert_eq!(hull_vertex_count(&hull), 4);
        assert_eq!(hull_face_count(&hull), 4);
    }

    #[test]
    fn test_too_few_points() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let cfg = default_hull_config();
        assert!(compute_convex_hull(&pts, &cfg).is_none());
    }

    #[test]
    fn test_coplanar_points_fail() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let cfg = default_hull_config();
        assert!(compute_convex_hull(&pts, &cfg).is_none());
    }

    #[test]
    fn test_hull_volume_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let vol = convex_hull_volume(&hull);
        assert!((vol - 1.0).abs() < 0.1, "Cube volume ~ 1.0, got {vol}");
    }

    #[test]
    fn test_hull_surface_area_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let area = convex_hull_surface_area(&hull);
        assert!(
            (area - 6.0).abs() < 0.1,
            "Cube surface area ~ 6.0, got {area}"
        );
    }

    #[test]
    fn test_point_inside_hull() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        assert!(is_point_inside_hull(&hull, [0.5, 0.5, 0.5]));
    }

    #[test]
    fn test_point_outside_hull() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        assert!(!is_point_inside_hull(&hull, [2.0, 2.0, 2.0]));
    }

    #[test]
    fn test_hull_centroid_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let c = hull_centroid(&hull);
        assert!((c[0] - 0.5).abs() < 0.01);
        assert!((c[1] - 0.5).abs() < 0.01);
        assert!((c[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_support_point_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let sp = support_point(&hull, [1.0, 0.0, 0.0]);
        assert!((sp[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_hull_bounding_box_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let (mn, mx) = hull_bounding_box(&hull);
        for k in 0..3 {
            assert!(mn[k].abs() < 0.01);
            assert!((mx[k] - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_hull_edge_count_tetrahedron() {
        let pts = tetra_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        // Tetrahedron: V=4, F=4, E=6
        assert_eq!(hull_edge_count(&hull), 6);
    }

    #[test]
    fn test_hull_edge_count_cube() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        // Triangulated cube: V=8, F=12, E = 8+12-2 = 18
        assert_eq!(hull_edge_count(&hull), 18);
    }

    #[test]
    fn test_project_to_hull_surface() {
        let pts = cube_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let proj = project_to_hull_surface(&hull, [0.5, 0.5, 2.0]);
        // Should project onto the z=1 face
        assert!((proj[2] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_interior_points_ignored() {
        // Add interior points to cube - hull should still have 8 verts
        let mut pts = cube_points();
        pts.push([0.5, 0.5, 0.5]);
        pts.push([0.3, 0.3, 0.3]);
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        assert_eq!(hull_vertex_count(&hull), 8);
    }

    #[test]
    fn test_hull_volume_tetrahedron() {
        let pts = tetra_points();
        let cfg = default_hull_config();
        let hull = compute_convex_hull(&pts, &cfg).expect("should succeed");
        let vol = convex_hull_volume(&hull);
        // Tetrahedron volume = |det([b-a, c-a, d-a])| / 6
        assert!(vol > 0.0);
    }
}
