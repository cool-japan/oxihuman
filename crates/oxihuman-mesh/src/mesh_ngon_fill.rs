// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ngon fan/ear fill: triangulate an N-gon polygon by fan or ear-clip.

/// Fill method for N-gon triangulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NgonFillMethod {
    /// Fan triangulation from vertex 0.
    Fan,
    /// Ear-clip triangulation.
    EarClip,
}

/// Result of N-gon fill.
#[derive(Debug, Clone)]
pub struct NgonFillResult {
    pub triangles: Vec<[usize; 3]>,
    pub method_used: NgonFillMethod,
}

/// Fan-triangulate a polygon given its vertex indices (in order).
pub fn fan_fill(polygon: &[usize]) -> Vec<[usize; 3]> {
    if polygon.len() < 3 {
        return vec![];
    }
    let pivot = polygon[0];
    (1..polygon.len().saturating_sub(1))
        .map(|i| [pivot, polygon[i], polygon[i + 1]])
        .collect()
}

fn signed_area_2d(pts: &[[f32; 2]]) -> f32 {
    let n = pts.len();
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i][0] * pts[j][1];
        area -= pts[j][0] * pts[i][1];
    }
    area * 0.5
}

fn is_ear_2d(pts: &[[f32; 2]], indices: &[usize], i: usize) -> bool {
    let n = indices.len();
    let prev = indices[(i + n - 1) % n];
    let curr = indices[i];
    let next = indices[(i + 1) % n];
    let a = pts[prev];
    let b = pts[curr];
    let c = pts[next];
    /* Convexity check (CCW winding) */
    let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
    if cross <= 0.0 {
        return false;
    }
    /* Point-in-triangle check for all other vertices */
    #[allow(clippy::needless_range_loop)]
    for j in 0..n {
        let vi = indices[j];
        if vi == prev || vi == curr || vi == next {
            continue;
        }
        let p = pts[vi];
        let d0 = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        let d1 = (c[0] - b[0]) * (p[1] - b[1]) - (c[1] - b[1]) * (p[0] - b[0]);
        let d2 = (a[0] - c[0]) * (p[1] - c[1]) - (a[1] - c[1]) * (p[0] - c[0]);
        if d0 > 0.0 && d1 > 0.0 && d2 > 0.0 {
            return false;
        }
    }
    true
}

/// Ear-clip triangulation of a 2D polygon.
pub fn ear_clip_fill_2d(pts_2d: &[[f32; 2]], polygon: &[usize]) -> Vec<[usize; 3]> {
    if polygon.len() < 3 {
        return vec![];
    }
    let mut remaining: Vec<usize> = polygon.to_vec();
    /* Ensure CCW winding */
    let pts_ordered: Vec<[f32; 2]> = remaining.iter().map(|&i| pts_2d[i]).collect();
    if signed_area_2d(&pts_ordered) < 0.0 {
        remaining.reverse();
    }
    let mut tris: Vec<[usize; 3]> = Vec::new();
    let mut iters = 0;
    while remaining.len() > 3 {
        let n = remaining.len();
        let mut clipped = false;
        for i in 0..n {
            if is_ear_2d(pts_2d, &remaining, i) {
                let prev = remaining[(i + n - 1) % n];
                let curr = remaining[i];
                let next = remaining[(i + 1) % n];
                tris.push([prev, curr, next]);
                remaining.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            break;
        } /* Degenerate polygon guard */
        iters += 1;
        if iters > 10_000 {
            break;
        }
    }
    if remaining.len() == 3 {
        tris.push([remaining[0], remaining[1], remaining[2]]);
    }
    tris
}

/// Fill an N-gon polygon using the specified method.
pub fn ngon_fill(
    positions_2d: &[[f32; 2]],
    polygon: &[usize],
    method: NgonFillMethod,
) -> NgonFillResult {
    let triangles = match method {
        NgonFillMethod::Fan => fan_fill(polygon),
        NgonFillMethod::EarClip => ear_clip_fill_2d(positions_2d, polygon),
    };
    NgonFillResult {
        triangles,
        method_used: method,
    }
}

/// Count triangles in the result.
pub fn triangle_count(result: &NgonFillResult) -> usize {
    result.triangles.len()
}

/// Expected triangle count for an N-gon.
pub fn expected_tri_count(n_verts: usize) -> usize {
    if n_verts < 3 {
        0
    } else {
        n_verts - 2
    }
}

/// Check if all triangles have valid (distinct) vertex indices.
pub fn all_valid(result: &NgonFillResult) -> bool {
    result
        .triangles
        .iter()
        .all(|&[a, b, c]| a != b && b != c && a != c)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pentagon_2d() -> (Vec<[f32; 2]>, Vec<usize>) {
        /* Regular pentagon */
        let pts: Vec<[f32; 2]> = (0..5)
            .map(|i| {
                let angle = 2.0 * std::f32::consts::PI * i as f32 / 5.0;
                [angle.cos(), angle.sin()]
            })
            .collect();
        let poly: Vec<usize> = (0..5).collect();
        (pts, poly)
    }

    #[test]
    fn test_fan_fill_triangle() {
        let tris = fan_fill(&[0, 1, 2]);
        assert_eq!(tris.len(), 1);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn test_fan_fill_quad() {
        let tris = fan_fill(&[0, 1, 2, 3]);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn test_fan_fill_pentagon() {
        let tris = fan_fill(&[0, 1, 2, 3, 4]);
        assert_eq!(tris.len(), 3);
    }

    #[test]
    fn test_expected_tri_count() {
        assert_eq!(expected_tri_count(5), 3);
        assert_eq!(expected_tri_count(3), 1);
    }

    #[test]
    fn test_ngon_fill_fan() {
        let (pts, poly) = pentagon_2d();
        let result = ngon_fill(&pts, &poly, NgonFillMethod::Fan);
        assert_eq!(result.triangles.len(), 3);
        assert!(all_valid(&result));
    }

    #[test]
    fn test_ngon_fill_ear_clip() {
        let (pts, poly) = pentagon_2d();
        let result = ngon_fill(&pts, &poly, NgonFillMethod::EarClip);
        assert_eq!(result.triangles.len(), 3);
        assert!(all_valid(&result));
    }

    #[test]
    fn test_triangle_count() {
        let result = NgonFillResult {
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            method_used: NgonFillMethod::Fan,
        };
        assert_eq!(triangle_count(&result), 2);
    }

    #[test]
    fn test_fan_fill_less_than_3() {
        let tris = fan_fill(&[0, 1]);
        assert!(tris.is_empty());
    }

    #[test]
    fn test_ear_clip_triangle() {
        let pts = vec![[0.0f32, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let poly = vec![0usize, 1, 2];
        let tris = ear_clip_fill_2d(&pts, &poly);
        assert_eq!(tris.len(), 1);
    }
}
