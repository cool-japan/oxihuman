// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ear-clipping polygon triangulator.

/* ── legacy types (keep for existing lib.rs exports) ── */

#[derive(Debug, Clone)]
pub struct EarClipResult {
    pub triangles: Vec<[usize; 3]>,
    pub ear_count: usize,
}

fn signed_area_2d_idx(pts: &[[f32; 2]], indices: &[usize]) -> f32 {
    let n = indices.len();
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        let p = pts[indices[i]];
        let q = pts[indices[j]];
        area += p[0] * q[1] - q[0] * p[1];
    }
    area * 0.5
}

fn cross2(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

fn point_in_triangle_2d(p: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let d0 = cross2(a, b, p);
    let d1 = cross2(b, c, p);
    let d2 = cross2(c, a, p);
    let has_neg = d0 < 0.0 || d1 < 0.0 || d2 < 0.0;
    let has_pos = d0 > 0.0 || d1 > 0.0 || d2 > 0.0;
    !(has_neg && has_pos)
}

fn is_ear_idx(pts: &[[f32; 2]], remaining: &[usize], i: usize, ccw: bool) -> bool {
    let n = remaining.len();
    let prev = remaining[(i + n - 1) % n];
    let curr = remaining[i];
    let next = remaining[(i + 1) % n];
    let a = pts[prev];
    let b = pts[curr];
    let c = pts[next];
    let cr = cross2(a, b, c);
    if ccw && cr <= 0.0 {
        return false;
    }
    if !ccw && cr >= 0.0 {
        return false;
    }
    #[allow(clippy::needless_range_loop)]
    for j in 0..n {
        let vi = remaining[j];
        if vi == prev || vi == curr || vi == next {
            continue;
        }
        if point_in_triangle_2d(pts[vi], a, b, c) {
            return false;
        }
    }
    true
}

/// Legacy: triangulate using index list into a pts array.
pub fn ear_clip(pts_2d: &[[f32; 2]], polygon: &[usize]) -> EarClipResult {
    if polygon.len() < 3 {
        return EarClipResult {
            triangles: vec![],
            ear_count: 0,
        };
    }
    let mut remaining: Vec<usize> = polygon.to_vec();
    let area = signed_area_2d_idx(pts_2d, &remaining);
    let ccw = area > 0.0;
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    let mut ear_count = 0;
    let mut iters = 0;
    while remaining.len() > 3 {
        let n = remaining.len();
        let mut clipped = false;
        for i in 0..n {
            if is_ear_idx(pts_2d, &remaining, i, ccw) {
                let prev = remaining[(i + n - 1) % n];
                let curr = remaining[i];
                let next = remaining[(i + 1) % n];
                triangles.push([prev, curr, next]);
                remaining.remove(i);
                ear_count += 1;
                clipped = true;
                break;
            }
        }
        if !clipped {
            break;
        }
        iters += 1;
        if iters > 50_000 {
            break;
        }
    }
    if remaining.len() == 3 {
        triangles.push([remaining[0], remaining[1], remaining[2]]);
    }
    EarClipResult {
        triangles,
        ear_count,
    }
}

pub fn triangle_count(result: &EarClipResult) -> usize {
    result.triangles.len()
}
pub fn expected_count(n: usize) -> usize {
    if n < 3 {
        0
    } else {
        n - 2
    }
}
pub fn all_valid(result: &EarClipResult) -> bool {
    result
        .triangles
        .iter()
        .all(|&[a, b, c]| a != b && b != c && a != c)
}
pub fn polygon_area_2d(pts_2d: &[[f32; 2]], polygon: &[usize]) -> f32 {
    signed_area_2d_idx(pts_2d, polygon)
}
pub fn is_ccw(pts_2d: &[[f32; 2]], polygon: &[usize]) -> bool {
    signed_area_2d_idx(pts_2d, polygon) > 0.0
}

/* ── spec functions (wave 150B) ── */

/// Signed area of a 2D triangle (positive if CCW).
pub fn triangle_area_2d(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
    0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]))
}

/// True if `curr` is a convex vertex in a CCW polygon.
pub fn polygon_is_convex_vertex(prev: [f32; 2], curr: [f32; 2], next: [f32; 2]) -> bool {
    triangle_area_2d(prev, curr, next) > 0.0
}

/// True if vertex `i` of `polygon` is an ear (convex and no other point inside).
pub fn ear_is_ear(polygon: &[[f32; 2]], i: usize) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }
    let prev = polygon[(i + n - 1) % n];
    let curr = polygon[i];
    let next = polygon[(i + 1) % n];
    if !polygon_is_convex_vertex(prev, curr, next) {
        return false;
    }
    for (j, &p) in polygon.iter().enumerate() {
        if j == (i + n - 1) % n || j == i || j == (i + 1) % n {
            continue;
        }
        let d0 = triangle_area_2d(prev, curr, p);
        let d1 = triangle_area_2d(curr, next, p);
        let d2 = triangle_area_2d(next, prev, p);
        if d0 >= 0.0 && d1 >= 0.0 && d2 >= 0.0 {
            return false;
        }
    }
    true
}

/// Number of triangles produced by ear-clipping an n-gon (n-2).
pub fn ear_clip_count(n: usize) -> usize {
    if n < 3 {
        0
    } else {
        n - 2
    }
}

/// Ear-clip triangulate a simple polygon (direct positions). Returns index triples into `polygon`.
pub fn ear_clip_triangulate(polygon: &[[f32; 2]]) -> Vec<[usize; 3]> {
    let n = polygon.len();
    if n < 3 {
        return vec![];
    }
    let mut indices: Vec<usize> = (0..n).collect();
    let mut triangles = Vec::with_capacity(n - 2);
    let mut attempts = 0usize;
    while indices.len() > 3 {
        let len = indices.len();
        let mut found = false;
        let sub: Vec<[f32; 2]> = indices.iter().map(|&k| polygon[k]).collect();
        for i in 0..len {
            if ear_is_ear(&sub, i) {
                let prev = indices[(i + len - 1) % len];
                let curr = indices[i];
                let next = indices[(i + 1) % len];
                triangles.push([prev, curr, next]);
                indices.remove(i);
                found = true;
                break;
            }
        }
        if !found {
            attempts += 1;
            if attempts > len {
                break;
            }
        } else {
            attempts = 0;
        }
    }
    if indices.len() == 3 {
        triangles.push([indices[0], indices[1], indices[2]]);
    }
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area_ccw() {
        /* CCW triangle has positive area */
        let a = triangle_area_2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]);
        assert!(a > 0.0, "CCW area must be positive");
    }

    #[test]
    fn test_triangle_area_cw() {
        /* CW triangle has negative area */
        let a = triangle_area_2d([0.0, 0.0], [0.0, 1.0], [1.0, 0.0]);
        assert!(a < 0.0, "CW area must be negative");
    }

    #[test]
    fn test_convex_vertex() {
        assert!(polygon_is_convex_vertex([0.0, 0.0], [1.0, 0.0], [1.0, 1.0]));
    }

    #[test]
    fn test_ear_clip_count() {
        assert_eq!(ear_clip_count(4), 2);
        assert_eq!(ear_clip_count(3), 1);
        assert_eq!(ear_clip_count(2), 0);
    }

    #[test]
    fn test_ear_is_ear_triangle() {
        /* every vertex of a CCW triangle is an ear */
        let poly = [[0.0f32, 0.0], [1.0, 0.0], [0.0, 1.0]];
        assert!(ear_is_ear(&poly, 0));
    }

    #[test]
    fn test_ear_clip_square() {
        /* unit square → 2 triangles */
        let poly = [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let tris = ear_clip_triangulate(&poly);
        assert_eq!(tris.len(), 2, "Square must produce 2 triangles");
    }

    #[test]
    fn test_legacy_ear_clip_square() {
        /* legacy API backward-compat */
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let r = ear_clip(&pts, &[0, 1, 2, 3]);
        assert_eq!(triangle_count(&r), 2);
        assert!(all_valid(&r));
    }
}
