// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Closest-point-on-mesh query stub.

/// Result of a closest-point query.
#[derive(Debug, Clone, Copy)]
pub struct ClosestPointResult {
    pub point: [f32; 3],
    pub distance: f32,
    pub triangle_index: u32,
    pub barycentric: [f32; 3],
}

fn dist3sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Closest point on a triangle to a query point (using barycentric coords).
pub fn closest_point_on_triangle(
    query: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    /* Returns (closest_point, barycentric) */
    let ab = sub3(v1, v0);
    let ac = sub3(v2, v0);
    let ap = sub3(query, v0);
    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (v0, [1.0, 0.0, 0.0]);
    }
    let bp = sub3(query, v1);
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (v1, [0.0, 1.0, 0.0]);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return (add3(v0, scale3(ab, v)), [1.0 - v, v, 0.0]);
    }
    let cp = sub3(query, v2);
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (v2, [0.0, 0.0, 1.0]);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return (add3(v0, scale3(ac, w)), [1.0 - w, 0.0, w]);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (add3(v1, scale3(sub3(v2, v1), w)), [0.0, 1.0 - w, w]);
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    (
        add3(add3(scale3(v0, u), scale3(v1, v)), scale3(v2, w)),
        [u, v, w],
    )
}

/// Find the closest point on a mesh to a query point.
pub fn query_closest_point(
    query: [f32; 3],
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Option<ClosestPointResult> {
    /* Brute-force search over all triangles */
    if tris.is_empty() || verts.is_empty() {
        return None;
    }
    let mut best_dist_sq = f32::MAX;
    let mut best = None;

    for (i, tri) in tris.iter().enumerate() {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        let (pt, bary) = closest_point_on_triangle(query, verts[i0], verts[i1], verts[i2]);
        let dsq = dist3sq(query, pt);
        if dsq < best_dist_sq {
            best_dist_sq = dsq;
            best = Some(ClosestPointResult {
                point: pt,
                distance: best_dist_sq.sqrt(),
                triangle_index: i as u32,
                barycentric: bary,
            });
        }
    }
    best
}

/// Find all closest points within a radius.
pub fn query_closest_within_radius(
    query: [f32; 3],
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    radius: f32,
) -> Vec<ClosestPointResult> {
    /* Return results sorted by distance ascending */
    let r2 = radius * radius;
    let mut results = Vec::new();
    for (i, tri) in tris.iter().enumerate() {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        let (pt, bary) = closest_point_on_triangle(query, verts[i0], verts[i1], verts[i2]);
        let dsq = dist3sq(query, pt);
        if dsq <= r2 {
            results.push(ClosestPointResult {
                point: pt,
                distance: dsq.sqrt(),
                triangle_index: i as u32,
                barycentric: bary,
            });
        }
    }
    results.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results
}

/// Check if a point is within a given distance of the mesh surface.
pub fn is_near_surface(
    query: [f32; 3],
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    threshold: f32,
) -> bool {
    query_closest_point(query, verts, tris).is_some_and(|r| r.distance <= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        (
            vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0u32, 1, 2]],
        )
    }

    #[test]
    fn test_closest_point_on_triangle_interior() {
        let (pt, bary) = closest_point_on_triangle(
            [0.25, 0.25, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!(bary.iter().all(|&b| b >= 0.0) /* valid barycentric */);
        assert!((pt[2]).abs() < 1e-5 /* projected to z=0 plane */);
    }

    #[test]
    fn test_closest_point_on_triangle_vertex() {
        let (pt, _) = closest_point_on_triangle(
            [-5.0, -5.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        /* Should clamp to v0 = origin */
        assert!(pt[0].abs() < 1e-5);
        assert!(pt[1].abs() < 1e-5);
    }

    #[test]
    fn test_query_closest_point_some() {
        let (verts, tris) = unit_triangle();
        let result = query_closest_point([0.25, 0.25, 1.0], &verts, &tris);
        assert!(result.is_some() /* hit found */);
    }

    #[test]
    fn test_query_closest_point_none_empty() {
        let result = query_closest_point([0.0; 3], &[], &[]);
        assert!(result.is_none() /* empty mesh */);
    }

    #[test]
    fn test_query_closest_distance_positive() {
        let (verts, tris) = unit_triangle();
        let result = query_closest_point([0.25, 0.25, 2.0], &verts, &tris).unwrap();
        assert!(result.distance > 0.0 /* point is above mesh */);
    }

    #[test]
    fn test_query_within_radius_finds_hit() {
        let (verts, tris) = unit_triangle();
        let hits = query_closest_within_radius([0.25, 0.25, 0.5], &verts, &tris, 2.0);
        assert!(!hits.is_empty() /* within radius */);
    }

    #[test]
    fn test_query_within_radius_misses() {
        let (verts, tris) = unit_triangle();
        let hits = query_closest_within_radius([100.0, 100.0, 100.0], &verts, &tris, 1.0);
        assert!(hits.is_empty() /* out of radius */);
    }

    #[test]
    fn test_is_near_surface_true() {
        let (verts, tris) = unit_triangle();
        assert!(is_near_surface([0.25, 0.25, 0.01], &verts, &tris, 0.1) /* near surface */);
    }

    #[test]
    fn test_is_near_surface_false() {
        let (verts, tris) = unit_triangle();
        assert!(!is_near_surface([10.0, 10.0, 10.0], &verts, &tris, 0.1) /* far away */);
    }
}
