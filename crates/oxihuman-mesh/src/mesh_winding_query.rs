// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Winding number inside/outside test for meshes stub.

use std::f32::consts::PI;

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Solid angle subtended by a triangle at a query point.
pub fn solid_angle_triangle(query: [f32; 3], v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    /* Oosterom–Strackee formula */
    let a = sub3(v0, query);
    let b = sub3(v1, query);
    let c = sub3(v2, query);
    let la = len3(a);
    let lb = len3(b);
    let lc = len3(c);
    if la < 1e-12 || lb < 1e-12 || lc < 1e-12 {
        return 0.0;
    }
    let num = dot3(a, cross3(b, c));
    let denom = la * lb * lc + dot3(a, b) * lc + dot3(b, c) * la + dot3(a, c) * lb;
    2.0 * num.atan2(denom)
}

/// Compute the winding number of a mesh around a query point.
pub fn winding_number(query: [f32; 3], verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    /* Sum solid angles over all triangles, divide by 4π */
    let mut total = 0.0f32;
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        total += solid_angle_triangle(query, verts[i0], verts[i1], verts[i2]);
    }
    total / (4.0 * PI)
}

/// Classify a point as inside (winding number near ±1) or outside (near 0).
pub fn is_inside_winding(query: [f32; 3], verts: &[[f32; 3]], tris: &[[u32; 3]]) -> bool {
    (winding_number(query, verts, tris).abs() - 1.0).abs() < 0.5
}

/// Batch winding-number test for many query points.
pub fn winding_number_batch(
    queries: &[[f32; 3]],
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Vec<f32> {
    queries
        .iter()
        .map(|&q| winding_number(q, verts, tris))
        .collect()
}

/// Count how many of the query points are inside the mesh.
pub fn count_inside_winding(queries: &[[f32; 3]], verts: &[[f32; 3]], tris: &[[u32; 3]]) -> usize {
    queries
        .iter()
        .filter(|&&q| is_inside_winding(q, verts, tris))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_angle_zero_empty() {
        /* Query at origin with degenerate triangle */
        let sa = solid_angle_triangle([0.0; 3], [0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(sa.abs() < 0.1 /* near-zero when query is on vertex */);
    }

    #[test]
    fn test_winding_number_empty_mesh() {
        let w = winding_number([0.5; 3], &[], &[]);
        assert_eq!(w, 0.0 /* no triangles, winding is zero */);
    }

    #[test]
    fn test_winding_batch_same_length() {
        let queries = vec![[0.0f32; 3], [1.0, 1.0, 1.0]];
        let verts = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let wns = winding_number_batch(&queries, &verts, &tris);
        assert_eq!(wns.len(), 2 /* one per query */);
    }

    #[test]
    fn test_winding_outside_near_zero() {
        /* A far-away point should have near-zero winding number */
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let w = winding_number([100.0, 100.0, 100.0], &verts, &tris);
        assert!(w.abs() < 0.1 /* far away → near zero */);
    }

    #[test]
    fn test_is_inside_outside_mesh() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        /* Far away point should not be classified inside */
        assert!(!is_inside_winding([100.0, 100.0, 100.0], &verts, &tris));
    }

    #[test]
    fn test_count_inside_all_outside() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let queries = vec![[100.0f32; 3], [200.0; 3]];
        assert_eq!(
            count_inside_winding(&queries, &verts, &tris),
            0 /* all outside */
        );
    }

    #[test]
    fn test_solid_angle_finite() {
        let sa = solid_angle_triangle(
            [2.0, 2.0, 2.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!(sa.is_finite() /* result is finite */);
    }

    #[test]
    fn test_winding_batch_empty_queries() {
        let wns = winding_number_batch(&[], &[], &[]);
        assert!(wns.is_empty() /* no queries, no results */);
    }
}
