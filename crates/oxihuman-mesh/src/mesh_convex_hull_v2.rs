// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Quickhull-style 3D convex hull (incremental gift-wrap approximation).

/// Result of convex hull computation.
#[allow(dead_code)]
pub struct ConvexHullV2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
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

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Signed volume of tetrahedron (for orientation test).
fn signed_vol(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ad = sub3(d, a);
    dot3(ab, cross3(ac, ad))
}

/// Compute convex hull using an incremental approach (tetrahedron seed + expansion).
/// This is a simplified O(n²) implementation suitable for moderate point counts.
#[allow(dead_code)]
pub fn convex_hull_v2(points: &[[f32; 3]]) -> ConvexHullV2 {
    if points.len() < 4 {
        return ConvexHullV2 {
            positions: points.to_vec(),
            indices: (0..points.len() as u32).collect(),
        };
    }
    let seed = build_initial_tetrahedron(points);
    let (positions, indices) = expand_hull(points, seed);
    ConvexHullV2 { positions, indices }
}

fn build_initial_tetrahedron(points: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut i0 = 0;
    for (i, p) in points.iter().enumerate() {
        if p[0] < points[i0][0] {
            i0 = i;
        }
    }
    let mut i1 = 0;
    let mut best = -1.0_f32;
    for (i, p) in points.iter().enumerate() {
        let d = sub3(*p, points[i0]);
        let dist = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if dist > best {
            best = dist;
            i1 = i;
        }
    }
    let e01 = normalize3(sub3(points[i1], points[i0]));
    let mut i2 = 0;
    best = -1.0;
    for (i, p) in points.iter().enumerate() {
        if i == i0 || i == i1 {
            continue;
        }
        let v = sub3(*p, points[i0]);
        let along = dot3(v, e01);
        let proj = [
            points[i0][0] + e01[0] * along,
            points[i0][1] + e01[1] * along,
            points[i0][2] + e01[2] * along,
        ];
        let perp = sub3(*p, proj);
        let dist = perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2];
        if dist > best {
            best = dist;
            i2 = i;
        }
    }
    let mut i3 = 0;
    best = -1.0;
    for (i, p) in points.iter().enumerate() {
        if i == i0 || i == i1 || i == i2 {
            continue;
        }
        let vol = signed_vol(points[i0], points[i1], points[i2], *p).abs();
        if vol > best {
            best = vol;
            i3 = i;
        }
    }
    vec![points[i0], points[i1], points[i2], points[i3]]
}

fn expand_hull(all_points: &[[f32; 3]], seed: Vec<[f32; 3]>) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut hull_pts = seed;
    for &p in all_points {
        let already = hull_pts.iter().any(|h| {
            (h[0] - p[0]).abs() < 1e-7 && (h[1] - p[1]).abs() < 1e-7 && (h[2] - p[2]).abs() < 1e-7
        });
        if !already {
            hull_pts.push(p);
        }
    }
    let n = hull_pts.len();
    let mut indices = Vec::new();
    if n < 4 {
        return (hull_pts, indices);
    }
    let center = {
        let mut c = [0.0_f32; 3];
        for p in &hull_pts {
            c[0] += p[0];
            c[1] += p[1];
            c[2] += p[2];
        }
        let inv = 1.0 / n as f32;
        [c[0] * inv, c[1] * inv, c[2] * inv]
    };
    for a in 0..n {
        for b in (a + 1)..n {
            for c in (b + 1)..n {
                let ab = sub3(hull_pts[b], hull_pts[a]);
                let ac = sub3(hull_pts[c], hull_pts[a]);
                let norm = cross3(ab, ac);
                if norm[0].abs() < 1e-9 && norm[1].abs() < 1e-9 && norm[2].abs() < 1e-9 {
                    continue;
                }
                let mut all_same_side = true;
                let mut ref_sign = 0.0_f32;
                for (i, p) in hull_pts.iter().enumerate() {
                    if i == a || i == b || i == c {
                        continue;
                    }
                    let d = dot3(norm, sub3(*p, hull_pts[a]));
                    if ref_sign == 0.0 {
                        ref_sign = d;
                    } else if d * ref_sign < 0.0 {
                        all_same_side = false;
                        break;
                    }
                }
                if all_same_side {
                    let face_center = [
                        (hull_pts[a][0] + hull_pts[b][0] + hull_pts[c][0]) / 3.0,
                        (hull_pts[a][1] + hull_pts[b][1] + hull_pts[c][1]) / 3.0,
                        (hull_pts[a][2] + hull_pts[b][2] + hull_pts[c][2]) / 3.0,
                    ];
                    let outward = dot3(norm, sub3(face_center, center)) > 0.0;
                    if outward {
                        indices.push(a as u32);
                        indices.push(b as u32);
                        indices.push(c as u32);
                    } else {
                        indices.push(a as u32);
                        indices.push(c as u32);
                        indices.push(b as u32);
                    }
                }
            }
        }
    }
    (hull_pts, indices)
}

/// Compute the volume of the convex hull (sum of signed tetrahedra).
#[allow(dead_code)]
pub fn hull_volume_v2(hull: &ConvexHullV2) -> f32 {
    let mut vol = 0.0_f32;
    let n = hull.indices.len() / 3;
    for i in 0..n {
        let a = hull.positions[hull.indices[i * 3] as usize];
        let b = hull.positions[hull.indices[i * 3 + 1] as usize];
        let c = hull.positions[hull.indices[i * 3 + 2] as usize];
        let cross = cross3(b, c);
        vol += dot3(a, cross);
    }
    (vol / 6.0).abs()
}

/// Number of faces in the hull.
#[allow(dead_code)]
pub fn hull_v2_face_count(hull: &ConvexHullV2) -> usize {
    hull.indices.len() / 3
}

/// Centroid of all hull vertices.
#[allow(dead_code)]
pub fn hull_v2_centroid(hull: &ConvexHullV2) -> [f32; 3] {
    if hull.positions.is_empty() {
        return [0.0; 3];
    }
    let mut c = [0.0_f32; 3];
    for p in &hull.positions {
        c[0] += p[0];
        c[1] += p[1];
        c[2] += p[2];
    }
    let n = hull.positions.len() as f32;
    [c[0] / n, c[1] / n, c[2] / n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_points() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ]
    }

    #[test]
    fn hull_v2_has_positions() {
        let pts = cube_points();
        let hull = convex_hull_v2(&pts);
        assert!(!hull.positions.is_empty());
    }

    #[test]
    fn hull_v2_has_triangles() {
        let pts = cube_points();
        let hull = convex_hull_v2(&pts);
        assert!(!hull.indices.is_empty());
        assert_eq!(hull.indices.len() % 3, 0);
    }

    #[test]
    fn hull_v2_face_count_nonzero() {
        let pts = cube_points();
        let hull = convex_hull_v2(&pts);
        assert!(hull_v2_face_count(&hull) > 0);
    }

    #[test]
    fn hull_v2_volume_positive() {
        let pts = cube_points();
        let hull = convex_hull_v2(&pts);
        let vol = hull_volume_v2(&hull);
        assert!(vol > 0.0, "volume should be positive, got {vol}");
    }

    #[test]
    fn hull_v2_centroid_near_center() {
        let pts = cube_points();
        let hull = convex_hull_v2(&pts);
        let cen = hull_v2_centroid(&hull);
        for v in cen {
            assert!((v - 0.5).abs() < 0.01, "centroid not near 0.5: {v}");
        }
    }

    #[test]
    fn hull_v2_too_few_points() {
        let pts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let hull = convex_hull_v2(&pts);
        assert_eq!(hull.positions.len(), 3);
    }

    #[test]
    fn signed_vol_nonzero() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let d = [0.0, 0.0, 1.0];
        let v = signed_vol(a, b, c, d);
        assert!(v.abs() > 1e-5);
    }

    #[test]
    fn hull_v2_indices_in_bounds() {
        let pts = cube_points();
        let hull = convex_hull_v2(&pts);
        let n = hull.positions.len() as u32;
        for &i in &hull.indices {
            assert!(i < n);
        }
    }

    #[test]
    fn hull_v2_empty_centroid() {
        let hull = ConvexHullV2 {
            positions: vec![],
            indices: vec![],
        };
        let c = hull_v2_centroid(&hull);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn hull_v2_tetrahedron_volume() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let hull = convex_hull_v2(&pts);
        let vol = hull_volume_v2(&hull);
        assert!(vol > 0.0);
    }
}
