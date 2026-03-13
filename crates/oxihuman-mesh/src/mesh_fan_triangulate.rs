// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Fan triangulation of convex polygons.

/* ── legacy API (keep for existing lib.rs exports) ── */

pub fn fan_triangulate(polygon: &[u32]) -> Vec<[u32; 3]> {
    if polygon.len() < 3 {
        return Vec::new();
    }
    let anchor = polygon[0];
    (1..polygon.len() - 1)
        .map(|i| [anchor, polygon[i], polygon[i + 1]])
        .collect()
}

pub fn fan_triangulate_all(polygons: &[Vec<u32>]) -> Vec<[u32; 3]> {
    polygons.iter().flat_map(|p| fan_triangulate(p)).collect()
}

pub fn polygon_area_2d(poly: &[[f32; 2]]) -> f32 {
    let n = poly.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for i in 0..n {
        let j = (i + 1) % n;
        area += poly[i][0] * poly[j][1];
        area -= poly[j][0] * poly[i][1];
    }
    (area * 0.5).abs()
}

/// Rough convexity check: all cross products same sign.
pub fn is_convex_polygon(verts: &[[f32; 3]], poly: &[u32]) -> bool {
    let n = poly.len();
    if n < 3 {
        return false;
    }
    let mut sign = 0.0f32;
    for i in 0..n {
        let a = verts[poly[i] as usize];
        let b = verts[poly[(i + 1) % n] as usize];
        let c = verts[poly[(i + 2) % n] as usize];
        let ex = b[0] - a[0];
        let ey = b[1] - a[1];
        let fx = c[0] - b[0];
        let fy = c[1] - b[1];
        let cross = ex * fy - ey * fx;
        if i == 0 {
            sign = cross;
        } else if sign * cross < 0.0 {
            return false;
        }
    }
    true
}

/* ── spec functions (wave 150B) ── */

/// Fan-triangulate a polygon from vertex 0. Returns index triples into `polygon`.
pub fn fan_triangulate_3d(polygon: &[[f32; 3]]) -> Vec<[usize; 3]> {
    let n = polygon.len();
    if n < 3 {
        return vec![];
    }
    (1..n - 1).map(|i| [0, i, i + 1]).collect()
}

/// Number of triangles in a fan triangulation of an n-gon (n-2).
pub fn fan_triangle_count(polygon_len: usize) -> usize {
    if polygon_len < 3 {
        0
    } else {
        polygon_len - 2
    }
}

/// Centroid of a 3D polygon.
pub fn fan_centroid(polygon: &[[f32; 3]]) -> [f32; 3] {
    let n = polygon.len();
    if n == 0 {
        return [0.0; 3];
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for p in polygon {
        cx += p[0];
        cy += p[1];
        cz += p[2];
    }
    let n = n as f32;
    [cx / n, cy / n, cz / n]
}

/// Fan-triangulate from a given center index in a ring.
pub fn fan_triangulate_ngon(polygon: &[[f32; 3]], center_idx: usize) -> Vec<[usize; 3]> {
    let n = polygon.len();
    if n < 3 || center_idx >= n {
        return vec![];
    }
    let mut tris = Vec::new();
    for i in 0..n {
        if i == center_idx {
            continue;
        }
        let j = (i + 1) % n;
        if j == center_idx {
            continue;
        }
        tris.push([center_idx, i, j]);
    }
    tris
}

/// Sum of triangle areas for a fan-triangulated polygon.
pub fn fan_area(polygon: &[[f32; 3]]) -> f32 {
    let tris = fan_triangulate_3d(polygon);
    tris.iter()
        .map(|&[a, b, c]| {
            let pa = polygon[a];
            let pb = polygon[b];
            let pc = polygon[c];
            let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_triangulate_quad() {
        /* quad produces 2 triangles */
        let poly = vec![0u32, 1, 2, 3];
        let tris = fan_triangulate(&poly);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn fan_triangulate_triangle() {
        let poly = vec![0u32, 1, 2];
        let tris = fan_triangulate(&poly);
        assert_eq!(tris.len(), 1);
        assert_eq!(tris[0], [0, 1, 2]);
    }

    #[test]
    fn fan_triangulate_degenerate() {
        let poly = vec![0u32, 1];
        let tris = fan_triangulate(&poly);
        assert!(tris.is_empty());
    }

    #[test]
    fn fan_triangulate_pentagon() {
        let poly = vec![0u32, 1, 2, 3, 4];
        let tris = fan_triangulate(&poly);
        assert_eq!(tris.len(), 3);
    }

    #[test]
    fn fan_triangulate_all_two_polygons() {
        let polys = vec![vec![0u32, 1, 2, 3], vec![0u32, 1, 2]];
        let tris = fan_triangulate_all(&polys);
        assert_eq!(tris.len(), 3);
    }

    #[test]
    fn polygon_area_2d_unit_square() {
        let poly = [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let area = polygon_area_2d(&poly);
        assert!((area - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fan_triangle_count() {
        /* n-gon yields n-2 triangles */
        assert_eq!(fan_triangle_count(4), 2);
        assert_eq!(fan_triangle_count(3), 1);
        assert_eq!(fan_triangle_count(2), 0);
    }

    #[test]
    fn test_fan_centroid_square() {
        let poly = [
            [0.0f32, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];
        let c = fan_centroid(&poly);
        assert!((c[0] - 1.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fan_area_unit_square() {
        /* unit square in XY plane → area 1.0 */
        let poly = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let a = fan_area(&poly);
        assert!((a - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_fan_triangulate_3d_quad() {
        let poly = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let tris = fan_triangulate_3d(&poly);
        assert_eq!(tris.len(), 2);
    }
}
