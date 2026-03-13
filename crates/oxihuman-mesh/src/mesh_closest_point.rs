// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Brute-force closest point on mesh surface query.

/// Result of a closest-point query.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClosestPointResult {
    pub point: [f32; 3],
    pub distance: f32,
    pub face_index: usize,
    pub barycentric: [f32; 3],
}

/// Find the closest point on the mesh surface to `query`.
/// Returns `None` if the mesh has no faces.
#[allow(dead_code)]
pub fn closest_point_on_mesh(
    query: [f32; 3],
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Option<ClosestPointResult> {
    let face_count = indices.len() / 3;
    if face_count == 0 {
        return None;
    }
    let mut best_dist = f32::INFINITY;
    let mut best = ClosestPointResult {
        point: [0.0; 3],
        distance: 0.0,
        face_index: 0,
        barycentric: [0.0; 3],
    };
    for fi in 0..face_count {
        let a = positions[indices[fi * 3] as usize];
        let b = positions[indices[fi * 3 + 1] as usize];
        let c = positions[indices[fi * 3 + 2] as usize];
        let (cp, bary) = closest_point_on_triangle(query, a, b, c);
        let d = dist3(query, cp);
        if d < best_dist {
            best_dist = d;
            best = ClosestPointResult {
                point: cp,
                distance: d,
                face_index: fi,
                barycentric: bary,
            };
        }
    }
    Some(best)
}

/// Closest point on a triangle to a query point (brute force clamped projection).
#[allow(dead_code)]
pub fn closest_point_on_triangle(
    p: [f32; 3],
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> ([f32; 3], [f32; 3]) {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (a, [1.0, 0.0, 0.0]);
    }
    let bp = [p[0] - b[0], p[1] - b[1], p[2] - b[2]];
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (b, [0.0, 1.0, 0.0]);
    }
    let cp_pt = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let d5 = dot3(ab, cp_pt);
    let d6 = dot3(ac, cp_pt);
    if d6 >= 0.0 && d5 <= d6 {
        return (c, [0.0, 0.0, 1.0]);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let pt = [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
        return (pt, [1.0 - v, v, 0.0]);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let pt = [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
        return (pt, [1.0 - w, 0.0, w]);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];
        let pt = [b[0] + w * bc[0], b[1] + w * bc[1], b[2] + w * bc[2]];
        return (pt, [0.0, 1.0 - w, w]);
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    let pt = [
        a[0] + v * ab[0] + w * ac[0],
        a[1] + v * ab[1] + w * ac[1],
        a[2] + v * ab[2] + w * ac[2],
    ];
    (pt, [u, v, w])
}

/// Euclidean distance between two 3-D points.
#[allow(dead_code)]
pub fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn closest_point_on_surface() {
        let (pos, idx) = unit_quad();
        let q = [0.5, 1.0, 0.5];
        let res = closest_point_on_mesh(q, &pos, &idx).expect("should succeed");
        assert!((res.point[0] - 0.5).abs() < 0.01);
        assert!((res.point[2] - 0.5).abs() < 0.01);
    }

    #[test]
    fn closest_distance_correct() {
        let (pos, idx) = unit_quad();
        let q = [0.5, 2.0, 0.5];
        let res = closest_point_on_mesh(q, &pos, &idx).expect("should succeed");
        assert!((res.distance - 2.0).abs() < 0.01);
    }

    #[test]
    fn closest_empty_mesh_returns_none() {
        let result = closest_point_on_mesh([0.0, 0.0, 0.0], &[], &[]);
        assert!(result.is_none());
    }

    #[test]
    fn closest_point_on_triangle_centroid() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let query = [1.0 / 3.0, 1.0 / 3.0, 0.0];
        let (cp, bary) = closest_point_on_triangle(query, a, b, c);
        assert!((cp[0] - 1.0 / 3.0).abs() < 0.01);
        let bary_sum = bary[0] + bary[1] + bary[2];
        assert!((bary_sum - 1.0).abs() < 0.01);
    }

    #[test]
    fn closest_point_corner_a() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let (cp, bary) = closest_point_on_triangle([-1.0, -1.0, 0.0], a, b, c);
        assert!((cp[0] - a[0]).abs() < 0.01);
        assert!((bary[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn dist3_zero_same_point() {
        assert!((dist3([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn dist3_unit() {
        assert!((dist3([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn face_index_valid() {
        let (pos, idx) = unit_quad();
        let res = closest_point_on_mesh([0.5, 0.5, 0.5], &pos, &idx).expect("should succeed");
        assert!(res.face_index < idx.len() / 3);
    }

    #[test]
    fn barycentric_sum_one() {
        let (pos, idx) = unit_quad();
        let res = closest_point_on_mesh([0.5, 0.0, 0.5], &pos, &idx).expect("should succeed");
        let s = res.barycentric[0] + res.barycentric[1] + res.barycentric[2];
        assert!((s - 1.0).abs() < 0.01);
    }

    #[test]
    fn closest_to_corner() {
        let (pos, idx) = unit_quad();
        let res = closest_point_on_mesh([0.0, 0.0, 0.0], &pos, &idx).expect("should succeed");
        assert!(res.distance < 0.01);
    }
}
