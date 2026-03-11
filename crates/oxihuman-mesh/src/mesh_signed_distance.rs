// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Signed distance field query for meshes stub.

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
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

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Compute the triangle face normal.
pub fn triangle_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    let n = cross3(e1, e2);
    let l = len3(n);
    if l < 1e-12 {
        return [0.0, 1.0, 0.0];
    }
    [n[0] / l, n[1] / l, n[2] / l]
}

/// Signed distance from a point to a triangle.
pub fn signed_distance_to_triangle(
    query: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> f32 {
    /* Unsigned distance projected to triangle, sign from face normal */
    let centroid = [
        (v0[0] + v1[0] + v2[0]) / 3.0,
        (v0[1] + v1[1] + v2[1]) / 3.0,
        (v0[2] + v1[2] + v2[2]) / 3.0,
    ];
    let normal = triangle_normal(v0, v1, v2);
    let d_vec = sub3(query, centroid);
    let sign = if dot3(d_vec, normal) >= 0.0 {
        1.0f32
    } else {
        -1.0
    };
    sign * dist3(query, centroid)
}

/// Compute the signed distance from a point to the nearest surface of a mesh.
pub fn signed_distance_to_mesh(
    query: [f32; 3],
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Option<f32> {
    /* Return None if mesh is empty; uses nearest triangle centroid as approx */
    if tris.is_empty() || verts.is_empty() {
        return None;
    }
    let mut best = f32::MAX;
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        let sd = signed_distance_to_triangle(query, verts[i0], verts[i1], verts[i2]);
        if sd.abs() < best.abs() {
            best = sd;
        }
    }
    Some(best)
}

/// Sample the signed distance field over a regular grid.
pub fn sample_sdf_grid(
    verts: &[[f32; 3]],
    tris: &[[u32; 3]],
    min: [f32; 3],
    max: [f32; 3],
    resolution: usize,
) -> Vec<f32> {
    /* Returns a flat array of SDF values sampled on a resolution^3 grid */
    let mut values = Vec::with_capacity(resolution * resolution * resolution);
    for iz in 0..resolution {
        for iy in 0..resolution {
            for ix in 0..resolution {
                let t = |i: usize, lo: f32, hi: f32| {
                    lo + (i as f32 / (resolution - 1).max(1) as f32) * (hi - lo)
                };
                let query = [
                    t(ix, min[0], max[0]),
                    t(iy, min[1], max[1]),
                    t(iz, min[2], max[2]),
                ];
                let sd = signed_distance_to_mesh(query, verts, tris).unwrap_or(f32::MAX);
                values.push(sd);
            }
        }
    }
    values
}

/// Check if a point is inside the mesh (negative signed distance).
pub fn is_inside_mesh(query: [f32; 3], verts: &[[f32; 3]], tris: &[[u32; 3]]) -> bool {
    signed_distance_to_mesh(query, verts, tris).is_some_and(|sd| sd < 0.0)
}

/// Gradient of the SDF at a query point (finite differences, stub).
pub fn sdf_gradient(query: [f32; 3], verts: &[[f32; 3]], tris: &[[u32; 3]], eps: f32) -> [f32; 3] {
    /* Central finite differences */
    let sdf = |q: [f32; 3]| signed_distance_to_mesh(q, verts, tris).unwrap_or(0.0);
    let dx = (sdf([query[0] + eps, query[1], query[2]])
        - sdf([query[0] - eps, query[1], query[2]]))
        / (2.0 * eps);
    let dy = (sdf([query[0], query[1] + eps, query[2]])
        - sdf([query[0], query[1] - eps, query[2]]))
        / (2.0 * eps);
    let dz = (sdf([query[0], query[1], query[2] + eps])
        - sdf([query[0], query[1], query[2] - eps]))
        / (2.0 * eps);
    [dx, dy, dz]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_normal_unit() {
        let n = triangle_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((len3(n) - 1.0).abs() < 1e-5 /* unit normal */);
    }

    #[test]
    fn test_signed_distance_above_positive() {
        let sd = signed_distance_to_triangle(
            [0.33, 0.33, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!(sd > 0.0 /* above triangle */);
    }

    #[test]
    fn test_signed_distance_below_negative() {
        let sd = signed_distance_to_triangle(
            [0.33, 0.33, -1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        assert!(sd < 0.0 /* below triangle */);
    }

    #[test]
    fn test_signed_distance_to_mesh_some() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let sd = signed_distance_to_mesh([0.33, 0.33, 0.5], &verts, &tris);
        assert!(sd.is_some() /* valid result */);
    }

    #[test]
    fn test_signed_distance_to_mesh_empty() {
        let sd = signed_distance_to_mesh([0.0; 3], &[], &[]);
        assert!(sd.is_none() /* empty mesh */);
    }

    #[test]
    fn test_sample_sdf_grid_size() {
        let verts = vec![[0.0f32; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let grid = sample_sdf_grid(&verts, &tris, [0.0; 3], [1.0; 3], 2);
        assert_eq!(grid.len(), 8 /* 2^3 samples */);
    }

    #[test]
    fn test_is_inside_returns_false_for_above() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        /* A flat triangle has no volume; test that above returns false */
        let inside = is_inside_mesh([0.33, 0.33, 5.0], &verts, &tris);
        assert!(!inside /* outside above */);
    }

    #[test]
    fn test_sdf_gradient_nonzero() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let grad = sdf_gradient([0.33, 0.33, 1.0], &verts, &tris, 0.01);
        let mag = (grad[0] * grad[0] + grad[1] * grad[1] + grad[2] * grad[2]).sqrt();
        assert!(mag > 0.0 /* gradient has magnitude */);
    }
}
