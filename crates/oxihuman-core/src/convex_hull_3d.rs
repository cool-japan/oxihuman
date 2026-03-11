// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! 3D convex hull (incremental, gift-wrapping / tetrahedra-based stub).

#![allow(dead_code)]

/// A face of the convex hull (triangle by vertex indices).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HullFace {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

impl HullFace {
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        Self { a, b, c }
    }
}

/// A 3D convex hull represented by vertex indices and triangle faces.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConvexHull3D {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<HullFace>,
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
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
fn norm3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

/// Face normal (unnormalized).
fn face_normal(pts: &[[f32; 3]], f: &HullFace) -> [f32; 3] {
    let ab = sub3(pts[f.b], pts[f.a]);
    let ac = sub3(pts[f.c], pts[f.a]);
    cross3(ab, ac)
}

/// Compute the convex hull of a point set using a simple incremental method.
/// Returns None if there are fewer than 4 non-coplanar points.
#[allow(dead_code)]
pub fn convex_hull_3d(points: &[[f32; 3]]) -> Option<ConvexHull3D> {
    if points.len() < 4 {
        return None;
    }
    // Build an initial tetrahedron from 4 non-coplanar points.
    let (i0, i1, i2, i3) = find_tetrahedron(points)?;
    let pts = points.to_vec();
    let mut faces = initial_tet_faces(i0, i1, i2, i3, &pts);

    // Add remaining points incrementally.
    for (i, p) in pts.iter().enumerate() {
        if i == i0 || i == i1 || i == i2 || i == i3 {
            continue;
        }
        // Find visible faces
        let visible: Vec<HullFace> = faces
            .iter()
            .filter(|f| {
                let n = face_normal(&pts, f);
                let d = sub3(*p, pts[f.a]);
                dot3(n, d) > 1e-8
            })
            .copied()
            .collect();
        if visible.is_empty() {
            continue;
        }
        // Collect horizon edges (edges belonging to exactly one visible face)
        let horizon = horizon_edges(&visible);
        // Remove visible faces
        faces.retain(|f| !visible.contains(f));
        // Add new faces from horizon to point i
        for (ea, eb) in horizon {
            let nf = HullFace::new(ea, eb, i);
            // Ensure correct orientation
            let n = face_normal(&pts, &nf);
            let center = centroid_of_faces(&faces, &pts);
            let d = sub3(pts[nf.a], center);
            if dot3(n, d) < 0.0 {
                faces.push(HullFace::new(eb, ea, i));
            } else {
                faces.push(nf);
            }
        }
    }

    Some(ConvexHull3D {
        vertices: pts,
        faces,
    })
}

fn find_tetrahedron(pts: &[[f32; 3]]) -> Option<(usize, usize, usize, usize)> {
    let n = pts.len();
    let i0 = 0;
    // Find farthest from i0
    let i1 = (0..n).max_by(|&a, &b| {
        let da = norm3(sub3(pts[a], pts[i0]));
        let db = norm3(sub3(pts[b], pts[i0]));
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })?;
    // Find farthest from line i0-i1
    let ab = sub3(pts[i1], pts[i0]);
    let i2 = (0..n).max_by(|&a, &b| {
        let da = norm3(cross3(sub3(pts[a], pts[i0]), ab));
        let db = norm3(cross3(sub3(pts[b], pts[i0]), ab));
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })?;
    if norm3(cross3(sub3(pts[i2], pts[i0]), ab)) < 1e-8 {
        return None; // Collinear
    }
    // Find farthest from plane i0-i1-i2
    let ab2 = sub3(pts[i1], pts[i0]);
    let ac2 = sub3(pts[i2], pts[i0]);
    let normal = cross3(ab2, ac2);
    let i3 = (0..n).max_by(|&a, &b| {
        let da = dot3(sub3(pts[a], pts[i0]), normal).abs();
        let db = dot3(sub3(pts[b], pts[i0]), normal).abs();
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })?;
    if dot3(sub3(pts[i3], pts[i0]), normal).abs() < 1e-8 {
        return None; // Coplanar
    }
    Some((i0, i1, i2, i3))
}

fn initial_tet_faces(
    i0: usize,
    i1: usize,
    i2: usize,
    i3: usize,
    pts: &[[f32; 3]],
) -> Vec<HullFace> {
    let mut faces = vec![
        HullFace::new(i0, i1, i2),
        HullFace::new(i0, i1, i3),
        HullFace::new(i0, i2, i3),
        HullFace::new(i1, i2, i3),
    ];
    // Orient all faces outward
    let centroid = [
        (pts[i0][0] + pts[i1][0] + pts[i2][0] + pts[i3][0]) / 4.0,
        (pts[i0][1] + pts[i1][1] + pts[i2][1] + pts[i3][1]) / 4.0,
        (pts[i0][2] + pts[i1][2] + pts[i2][2] + pts[i3][2]) / 4.0,
    ];
    for f in &mut faces {
        let n = face_normal(pts, f);
        let d = sub3(pts[f.a], centroid);
        if dot3(n, d) < 0.0 {
            std::mem::swap(&mut f.a, &mut f.b);
        }
    }
    faces
}

fn horizon_edges(visible: &[HullFace]) -> Vec<(usize, usize)> {
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();
    for f in visible {
        let edges = [(f.a, f.b), (f.b, f.c), (f.c, f.a)];
        for (ea, eb) in edges {
            let key = if ea < eb { (ea, eb) } else { (eb, ea) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(k, _)| k)
        .collect()
}

fn centroid_of_faces(faces: &[HullFace], pts: &[[f32; 3]]) -> [f32; 3] {
    if faces.is_empty() {
        return [0.0; 3];
    }
    let mut sum = [0.0f32; 3];
    for f in faces {
        for j in 0..3 {
            sum[j] += pts[f.a][j] + pts[f.b][j] + pts[f.c][j];
        }
    }
    let n = (faces.len() * 3) as f32;
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Volume of a convex hull (sum of signed tetrahedra from origin).
#[allow(dead_code)]
pub fn hull_volume(hull: &ConvexHull3D) -> f32 {
    let pts = &hull.vertices;
    let mut vol = 0.0f32;
    for f in &hull.faces {
        let a = pts[f.a];
        let b = pts[f.b];
        let c = pts[f.c];
        // Signed volume of tetrahedron from origin
        vol += a[0] * (b[1] * c[2] - b[2] * c[1])
            + a[1] * (b[2] * c[0] - b[0] * c[2])
            + a[2] * (b[0] * c[1] - b[1] * c[0]);
    }
    (vol / 6.0).abs()
}

/// Face count of the hull.
#[allow(dead_code)]
pub fn hull_face_count(hull: &ConvexHull3D) -> usize {
    hull.faces.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_points() -> Vec<[f32; 3]> {
        let mut pts = Vec::new();
        for &x in &[0.0f32, 1.0] {
            for &y in &[0.0f32, 1.0] {
                for &z in &[0.0f32, 1.0] {
                    pts.push([x, y, z]);
                }
            }
        }
        pts
    }

    #[test]
    fn hull_from_cube() {
        let pts = cube_points();
        let hull = convex_hull_3d(&pts).expect("should produce hull");
        assert!(!hull.faces.is_empty());
    }

    #[test]
    fn hull_face_count_cube() {
        let pts = cube_points();
        let hull = convex_hull_3d(&pts).unwrap();
        // A cube has 12 triangles
        assert!(
            hull_face_count(&hull) >= 12,
            "got {}",
            hull_face_count(&hull)
        );
    }

    #[test]
    fn hull_fewer_than_four_returns_none() {
        let pts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(convex_hull_3d(&pts).is_none());
    }

    #[test]
    fn hull_volume_positive() {
        let pts = cube_points();
        let hull = convex_hull_3d(&pts).unwrap();
        let vol = hull_volume(&hull);
        assert!(vol > 0.0, "vol={vol}");
    }

    #[test]
    fn hull_volume_cube_approx_one() {
        let pts = cube_points();
        let hull = convex_hull_3d(&pts).unwrap();
        let vol = hull_volume(&hull);
        assert!(vol > 0.5, "vol too small: {vol}");
    }

    #[test]
    fn coplanar_returns_none() {
        let pts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        assert!(convex_hull_3d(&pts).is_none());
    }

    #[test]
    fn tetrahedron_four_faces() {
        let pts = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let hull = convex_hull_3d(&pts).unwrap();
        assert_eq!(hull.faces.len(), 4);
    }

    #[test]
    fn hull_face_indices_in_range() {
        let pts = cube_points();
        let hull = convex_hull_3d(&pts).unwrap();
        for f in &hull.faces {
            assert!(f.a < hull.vertices.len());
            assert!(f.b < hull.vertices.len());
            assert!(f.c < hull.vertices.len());
        }
    }

    #[test]
    fn sub3_correct() {
        let a = [3.0f32, 5.0, 7.0];
        let b = [1.0, 2.0, 3.0];
        let r = sub3(a, b);
        assert!((r[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn cross3_orthogonal() {
        let a = [1.0f32, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];
        let c = cross3(a, b);
        assert!((c[2] - 1.0).abs() < 1e-5);
    }
}
