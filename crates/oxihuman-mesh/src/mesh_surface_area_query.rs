// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Mesh surface area query stub.

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

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Area of a single triangle given three vertex positions.
pub fn triangle_area(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> f32 {
    let e1 = sub3(v1, v0);
    let e2 = sub3(v2, v0);
    len3(cross3(e1, e2)) * 0.5
}

/// Total surface area of a triangle mesh.
pub fn mesh_surface_area(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    let mut total = 0.0f32;
    for tri in tris {
        let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
            continue;
        }
        total += triangle_area(verts[i0], verts[i1], verts[i2]);
    }
    total
}

/// Compute area of each triangle and return per-triangle areas.
pub fn per_triangle_areas(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<f32> {
    tris.iter()
        .map(|tri| {
            let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
                return 0.0;
            }
            triangle_area(verts[i0], verts[i1], verts[i2])
        })
        .collect()
}

/// Find the largest triangle by area.
pub fn largest_triangle(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Option<(u32, f32)> {
    /* Returns (triangle_index, area) */
    let areas = per_triangle_areas(verts, tris);
    areas
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, &a)| (i as u32, a))
}

/// Compute the average triangle area.
pub fn average_triangle_area(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    if tris.is_empty() {
        return 0.0;
    }
    mesh_surface_area(verts, tris) / tris.len() as f32
}

/// Compute the ratio of mesh surface area to its AABB surface area.
pub fn surface_compactness(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> f32 {
    /* Ratio of mesh area to AABB area; 1.0 for a box */
    if verts.is_empty() {
        return 0.0;
    }
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for v in verts {
        for k in 0..3 {
            if v[k] < mn[k] {
                mn[k] = v[k];
            }
            if v[k] > mx[k] {
                mx[k] = v[k];
            }
        }
    }
    let dx = (mx[0] - mn[0]).max(0.0);
    let dy = (mx[1] - mn[1]).max(0.0);
    let dz = (mx[2] - mn[2]).max(0.0);
    let aabb_area = 2.0 * (dx * dy + dy * dz + dz * dx);
    if aabb_area < 1e-12 {
        return 0.0;
    }
    mesh_surface_area(verts, tris) / aabb_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area_right_triangle() {
        let area = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((area - 0.5).abs() < 1e-5 /* unit right triangle area = 0.5 */);
    }

    #[test]
    fn test_triangle_area_degenerate() {
        let area = triangle_area([0.0; 3], [0.0; 3], [0.0; 3]);
        assert_eq!(area, 0.0 /* degenerate triangle */);
    }

    #[test]
    fn test_mesh_surface_area_empty() {
        assert_eq!(mesh_surface_area(&[], &[]), 0.0 /* empty mesh */);
    }

    #[test]
    fn test_mesh_surface_area_single_triangle() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let area = mesh_surface_area(&verts, &tris);
        assert!((area - 0.5).abs() < 1e-5 /* single triangle area */);
    }

    #[test]
    fn test_per_triangle_areas_length() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2], [0, 1, 2]];
        let areas = per_triangle_areas(&verts, &tris);
        assert_eq!(areas.len(), 2 /* one per triangle */);
    }

    #[test]
    fn test_largest_triangle_index() {
        let verts = vec![
            [0.0f32, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0], /* large: area 2 */
            [0.0, 0.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.0, 0.5, 0.0], /* small: area 0.125 */
        ];
        let tris = vec![[0u32, 1, 2], [3, 4, 5]];
        let (idx, area) = largest_triangle(&verts, &tris).unwrap();
        assert_eq!(idx, 0 /* first triangle is largest */);
        assert!(area > 1.0 /* area > 1 */);
    }

    #[test]
    fn test_average_area_empty() {
        assert_eq!(average_triangle_area(&[], &[]), 0.0 /* empty */);
    }

    #[test]
    fn test_surface_compactness_empty() {
        assert_eq!(surface_compactness(&[], &[]), 0.0 /* empty */);
    }

    #[test]
    fn test_surface_compactness_positive() {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0u32, 1, 2]];
        let c = surface_compactness(&verts, &tris);
        assert!(c > 0.0 /* positive ratio */);
    }
}
